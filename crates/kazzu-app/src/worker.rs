//! Trabalho pesado fora da thread de UI: consultas à biblioteca, scan, decodificação de capas.
//!
//! `Store` é síncrono e testável (`Store::run`). O app usa `ThreadedRunner`, que roda um `Store` numa thread e o
//! scan numa thread própria (com outra conexão SQLite; o banco está em modo WAL).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use std::time::Instant;
use kazzu_core::QueueItem;
use kazzu_library::{Album, AlbumId, Artwork, Library, PlaylistId, ScanProgress, ScanReport, Track, TrackId};

use crate::accent;
use crate::art::{self, Pixels};
use crate::format;
use crate::lru::Lru;
use crate::vm::*;

/// Espera após a última digitação antes de consultar (debounce da busca).
pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(200);
const SEARCH_LIMIT: usize = 50;
const RECENTS: usize = 20;
const HOME_ALBUMS: usize = 24;
const MOODS: usize = 8;
const THUMB_CACHE_MAX: usize = 400;

/// O que tocar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Album(String),
    Liked,
    Playlist(String),
    Genre(String),
    /// Biblioteca inteira.
    All,
    /// Lista exata de faixas (recentes, resultados de busca, detalhe aberto).
    Ids(Vec<String>),
}

/// O que mostrar na tela de detalhe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailRef {
    Album(String),
    Liked,
    Downloads,
    Albums,
    Playlist(String),
    Genre(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueMode {
    PlayNext,
    Enqueue,
}

#[derive(Debug, Clone)]
pub enum Job {
    /// Recarrega tudo que a UI mostra (início, biblioteca, configurações).
    Refresh,
    Scan,
    AddFolder(PathBuf),
    RemoveFolder(PathBuf),
    Search { seq: u64, query: String },
    Play { scope: Scope, start_id: Option<String>, shuffle: bool, label: String },
    QueueItems { id: String, mode: QueueMode },
    OpenDetail { seq: u64, what: DetailRef },
    NowArt { seq: u64, cover: Option<PathBuf> },
    /// View-models (com miniaturas) de faixas já conhecidas, para a fila.
    TrackVms { seq: u64, ids: Vec<String> },
    SetLiked { id: String, liked: bool },
    RecordPlay { id: String, listened_ms: u64 },
    CreatePlaylist(String),
}

pub struct Snapshot {
    pub home: HomeVm,
    pub library: LibraryVm,
    pub settings: SettingsVm,
    pub ambient: Option<Pixels>,
    pub liked: HashSet<String>,
}

pub enum JobResult {
    Refreshed(Box<Snapshot>),
    ScanProgress(ScanVm),
    ScanDone(Result<ScanReport, String>),
    Search { seq: u64, songs: Vec<TrackVm>, albums: Vec<AlbumVm>, ids: Vec<String> },
    Queue { items: Vec<QueueItem>, start: usize, shuffle: bool, label: String },
    QueueItems { items: Vec<QueueItem>, mode: QueueMode },
    Detail { seq: u64, detail: Box<DetailVm>, ids: Vec<String> },
    NowArt { seq: u64, art: Box<NowArtVm> },
    TrackVms { seq: u64, tracks: Vec<TrackVm> },
    /// Lista "Recentes" atualizada (depois de registrar uma escuta).
    Recents(Vec<TrackVm>),
    /// Algo falhou: o texto já é para o usuário.
    Failed(String),
    /// Curtida gravada.
    LikedSaved { id: String, liked: bool },
    FoldersChanged,
}

pub type Emit<'a> = &'a (dyn Fn(JobResult) + Sync);

/// Quem executa jobs. O app usa threads; os testes, execução na hora.
pub trait JobRunner {
    fn submit(&self, job: Job);
}

fn parse_id(id: &str) -> Option<i64> {
    id.parse().ok()
}

/// Miniaturas decodificadas (chave = caminho do JPEG), com limite: descarta as menos usadas, uma a uma.
struct ThumbCache {
    map: Lru<PathBuf, Option<Pixels>>,
}

impl Default for ThumbCache {
    fn default() -> Self {
        Self { map: Lru::new(THUMB_CACHE_MAX) }
    }
}

impl ThumbCache {
    fn get(&mut self, path: &Path) -> Option<Pixels> {
        let key = path.to_path_buf();
        if let Some(hit) = self.map.get(&key) {
            return hit.clone();
        }
        let px = art::load(path).map(|img| art::to_pixels(&img));
        self.map.insert(key, px.clone());
        px
    }
}

pub struct Store {
    lib: Library,
    thumbs: ThumbCache,
    albums: HashMap<i64, Album>,
    liked: HashSet<i64>,
}

impl Store {
    pub fn open(data_dir: &Path) -> Result<Self, kazzu_library::Error> {
        Ok(Self::new(Library::open(data_dir)?))
    }

    pub fn new(lib: Library) -> Self {
        Self { lib, thumbs: ThumbCache::default(), albums: HashMap::new(), liked: HashSet::new() }
    }

    pub fn library(&self) -> &Library {
        &self.lib
    }

    fn reload_caches(&mut self) -> Result<(), kazzu_library::Error> {
        self.albums = self.lib.albums()?.into_iter().map(|a| (a.id.0, a)).collect();
        self.liked = self.lib.liked()?.iter().map(|t| t.id.0).collect();
        Ok(())
    }

    fn artwork(&self, album: Option<AlbumId>) -> Option<&Artwork> {
        self.albums.get(&album?.0)?.artwork.as_ref()
    }

    fn thumb(&mut self, album: Option<AlbumId>) -> Option<Pixels> {
        let path = self.artwork(album)?.thumb.clone();
        self.thumbs.get(&path)
    }

    fn track_vm(&mut self, t: &Track) -> TrackVm {
        let b = format::badge(&t.codec, format::is_lossless_name(&t.codec), t.bit_depth, t.sample_rate, t.bitrate);
        TrackVm {
            id: t.id.0.to_string(),
            title: t.title.clone(),
            artist: t.artist_display(),
            album: t.album.clone().unwrap_or_default(),
            cover: self.thumb(t.album_id),
            duration_label: format::duration_label_ms(t.duration_ms),
            codec: b.codec,
            detail: b.detail,
            detail_long: b.detail_long,
            hires: b.hires,
            source: String::new(),
            liked: self.liked.contains(&t.id.0),
            playing: false,
        }
    }

    fn album_vm(&mut self, a: &Album) -> AlbumVm {
        let cover = a.artwork.as_ref().and_then(|w| self.thumbs.get(&w.thumb.clone()));
        AlbumVm { id: a.id.0.to_string(), title: a.title.clone(), artist: a.artist.clone(), cover }
    }

    /// Capa grande (≤ 640 px) do álbum, para cartões de destaque e gêneros.
    fn big_cover(&mut self, a: &Album) -> Option<Pixels> {
        let path = a.artwork.as_ref()?.cover.clone();
        self.thumbs.get(&path)
    }

    /// Nome para "Tocando de …" quando quem pediu não deu um.
    fn scope_label(&self, scope: &Scope) -> String {
        match scope {
            Scope::Album(id) => parse_id(id).and_then(|i| self.albums.get(&i)).map_or_else(|| "Álbum".to_owned(), |a| a.title.clone()),
            Scope::Playlist(id) => parse_id(id)
                .and_then(|i| self.lib.playlists().ok()?.into_iter().find(|p| p.id.0 == i))
                .map_or_else(|| "Playlist".to_owned(), |p| p.name),
            Scope::Liked => "Curtidos".to_owned(),
            Scope::Genre(g) => g.clone(),
            Scope::All | Scope::Ids(_) => "Biblioteca".to_owned(),
        }
    }

    /// Resolve uma `Scope` em faixas (na ordem de reprodução natural).
    fn tracks_for(&mut self, scope: &Scope) -> Result<Vec<Track>, kazzu_library::Error> {
        Ok(match scope {
            Scope::Album(id) => parse_id(id).map(|i| self.lib.album_tracks(AlbumId(i))).transpose()?.unwrap_or_default(),
            Scope::Liked => self.lib.liked()?,
            Scope::Playlist(id) => parse_id(id).map(|i| self.lib.playlist_tracks(PlaylistId(i))).transpose()?.unwrap_or_default(),
            Scope::Genre(g) => self.lib.tracks()?.into_iter().filter(|t| t.genre.as_deref() == Some(g.as_str())).collect(),
            Scope::All => self.lib.tracks()?,
            Scope::Ids(ids) => {
                let mut out = Vec::with_capacity(ids.len());
                for id in ids {
                    if let Some(t) = parse_id(id).map(|i| self.lib.track(TrackId(i))).transpose()?.flatten() {
                        out.push(t);
                    }
                }
                out
            }
        })
    }

    fn queue_item(&self, t: &Track) -> Option<QueueItem> {
        QueueItem::local(t.to_core(self.artwork(t.album_id)))
    }

    /// Executa um job. `emit` entrega resultados (pode ser chamado mais de uma vez, ex.: progresso do scan).
    pub fn run(&mut self, job: Job, emit: Emit<'_>) {
        if let Err(e) = self.run_inner(job, emit) {
            log::warn!("job falhou: {e}");
            emit(JobResult::Failed(format!("Não foi possível ler a biblioteca. {e}")));
        }
    }

    fn run_inner(&mut self, job: Job, emit: Emit<'_>) -> Result<(), kazzu_library::Error> {
        match job {
            Job::Refresh => {
                let snap = self.snapshot()?;
                emit(JobResult::Refreshed(Box::new(snap)));
            }
            Job::Scan => {
                let result = scan(&mut self.lib, emit);
                emit(JobResult::ScanDone(result));
            }
            Job::AddFolder(path) => {
                if !path.is_dir() {
                    emit(JobResult::Failed(format!("A pasta {} não existe mais.", path.display())));
                } else {
                    self.lib.add_folder(&path)?;
                    emit(JobResult::FoldersChanged);
                }
            }
            Job::RemoveFolder(path) => {
                self.lib.remove_folder(&path)?;
                emit(JobResult::FoldersChanged);
            }
            Job::Search { seq, query } => {
                self.reload_caches_if_empty()?;
                let found = self.lib.search(&query, SEARCH_LIMIT)?;
                let songs: Vec<TrackVm> = found.tracks.iter().map(|t| self.track_vm(t)).collect();
                let ids = songs.iter().map(|s| s.id.clone()).collect();
                let albums = found.albums.iter().map(|a| self.album_vm(a)).collect();
                emit(JobResult::Search { seq, songs, albums, ids });
            }
            Job::Play { scope, start_id, shuffle, label } => {
                self.reload_caches_if_empty()?;
                let label = if label.is_empty() { self.scope_label(&scope) } else { label };
                let tracks = self.tracks_for(&scope)?;
                // faixas que não viram item (ex.: remotas sem stream) saem da lista; o início é recalculado sobre ela
                let playable: Vec<(&Track, QueueItem)> = tracks.iter().filter_map(|t| self.queue_item(t).map(|i| (t, i))).collect();
                if playable.is_empty() {
                    emit(JobResult::Failed("Não há músicas para tocar aqui.".into()));
                } else {
                    let start = start_id.and_then(|id| playable.iter().position(|(t, _)| t.id.0.to_string() == id)).unwrap_or(0);
                    let items: Vec<QueueItem> = playable.into_iter().map(|(_, i)| i).collect();
                    emit(JobResult::Queue { items, start, shuffle, label });
                }
            }
            Job::QueueItems { id, mode } => {
                self.reload_caches_if_empty()?;
                let tracks = self.tracks_for(&Scope::Ids(vec![id]))?;
                let items: Vec<QueueItem> = tracks.iter().filter_map(|t| self.queue_item(t)).collect();
                if !items.is_empty() {
                    emit(JobResult::QueueItems { items, mode });
                }
            }
            Job::OpenDetail { seq, what } => {
                self.reload_caches_if_empty()?;
                let (detail, ids) = self.detail(&what)?;
                emit(JobResult::Detail { seq, detail: Box::new(detail), ids });
            }
            Job::NowArt { seq, cover } => {
                let art = match cover.as_deref().and_then(art::load) {
                    Some(img) => NowArtVm {
                        accent: accent::derive(&img),
                        backdrop: Some(art::to_pixels(&art::backdrop(&img))),
                        thumb: Some(art::to_pixels(&art::fit_within(img.clone(), 192))),
                        cover: Some(art::to_pixels(&art::fit_within(img, 640))),
                    },
                    None => NowArtVm::default(),
                };
                emit(JobResult::NowArt { seq, art: Box::new(art) });
            }
            Job::TrackVms { seq, ids } => {
                self.reload_caches_if_empty()?;
                let tracks = self.tracks_for(&Scope::Ids(ids))?;
                let vms = tracks.iter().map(|t| self.track_vm(t)).collect();
                emit(JobResult::TrackVms { seq, tracks: vms });
            }
            Job::SetLiked { id, liked } => {
                if let Some(tid) = parse_id(&id) {
                    if liked { self.lib.like(TrackId(tid))? } else { self.lib.unlike(TrackId(tid))? }
                    if liked { self.liked.insert(tid); } else { self.liked.remove(&tid); }
                    emit(JobResult::LikedSaved { id, liked });
                }
            }
            Job::RecordPlay { id, listened_ms } => {
                if let Some(tid) = parse_id(&id) {
                    self.lib.record_play(TrackId(tid), listened_ms)?;
                    self.reload_caches_if_empty()?;
                    let recents = self.recent_tracks()?.iter().map(|t| self.track_vm(t)).collect();
                    emit(JobResult::Recents(recents));
                }
            }
            Job::CreatePlaylist(name) => {
                let name = name.trim();
                if name.is_empty() {
                    emit(JobResult::Failed("Dê um nome à playlist.".into()));
                } else {
                    self.lib.create_playlist(name)?;
                    let snap = self.snapshot()?;
                    emit(JobResult::Refreshed(Box::new(snap)));
                }
            }
        }
        Ok(())
    }

    fn reload_caches_if_empty(&mut self) -> Result<(), kazzu_library::Error> {
        if self.albums.is_empty() {
            self.reload_caches()?;
        }
        Ok(())
    }

    /// Tudo que Início, Biblioteca e Configurações mostram, de uma vez.
    fn snapshot(&mut self) -> Result<Snapshot, kazzu_library::Error> {
        self.reload_caches()?;
        let folders = self.lib.folders()?;
        let track_count = self.lib.track_count()?;
        let all_albums = self.lib.albums()?;

        let recent = self.recent_tracks()?;
        let recents: Vec<TrackVm> = recent.iter().map(|t| self.track_vm(t)).collect();

        // Álbuns: primeiro os dos recentes, depois os demais.
        let mut order: Vec<&Album> = Vec::new();
        let mut seen = HashSet::new();
        for t in &recent {
            if let Some(a) = t.album_id.and_then(|id| self.albums.get(&id.0))
                && seen.insert(a.id.0)
            {
                order.push(a);
            }
        }
        for a in &all_albums {
            if seen.insert(a.id.0) {
                order.push(a);
            }
        }
        let order: Vec<Album> = order.into_iter().take(HOME_ALBUMS).cloned().collect();
        let albums: Vec<AlbumVm> = order.iter().map(|a| self.album_vm(a)).collect();

        let liked_tracks = self.lib.liked()?;
        let highlights = self.highlights(&all_albums, &liked_tracks);
        let moods = self.moods()?;

        let playlists = self.lib.playlists()?;
        let mut playlist_vms = Vec::with_capacity(playlists.len());
        for p in &playlists {
            let tracks = self.lib.playlist_tracks(p.id)?;
            playlist_vms.push(PlaylistVm {
                id: p.id.0.to_string(),
                name: p.name.clone(),
                count_label: format::songs_label(u64::from(p.track_count)),
                covers: self.distinct_covers(&tracks, 4),
            });
        }

        let covers_of_albums: Vec<Pixels> = all_albums.iter().filter_map(|a| self.album_vm(a).cover).take(3).collect();
        let liked_covers = self.distinct_covers(&liked_tracks, 1);
        let collections = vec![
            CollectionVm { id: "downloads".into(), title: "Baixados".into(), count_label: "Em breve".into(), covers: vec![] },
            CollectionVm { id: "liked".into(), title: "Curtidos".into(), count_label: format::songs_label(liked_tracks.len() as u64), covers: liked_covers },
            CollectionVm { id: "albums".into(), title: "Álbuns".into(), count_label: format::albums_label(all_albums.len() as u64), covers: covers_of_albums },
            CollectionVm {
                id: "playlists".into(),
                title: "Playlists".into(),
                count_label: format::playlists_label(playlists.len() as u64),
                covers: playlist_vms.iter().flat_map(|p| p.covers.iter().cloned()).take(4).collect(),
            },
        ];

        // Fundo ambiente enquanto nada toca: o primeiro álbum.
        let ambient = all_albums
            .first()
            .and_then(|a| a.artwork.as_ref())
            .and_then(|w| art::load(&w.cover))
            .map(|img| art::to_pixels(&art::backdrop(&img)));

        Ok(Snapshot {
            home: HomeVm { highlights, recents, albums, moods },
            library: LibraryVm { collections, playlists: playlist_vms },
            settings: SettingsVm {
                folders: folders.iter().map(|p| p.display().to_string()).collect(),
                summary: format::library_summary(folders.len(), track_count),
                has_library: track_count > 0,
                track_count,
            },
            ambient,
            liked: self.liked.iter().map(|i| i.to_string()).collect(),
        })
    }

    /// Recentes: o que foi ouvido por último; o que falta para completar a lista vem do que entrou há pouco.
    fn recent_tracks(&self) -> Result<Vec<Track>, kazzu_library::Error> {
        let mut out = self.lib.recent(RECENTS)?;
        if out.len() < RECENTS {
            let seen: HashSet<i64> = out.iter().map(|t| t.id.0).collect();
            out.extend(self.lib.recently_added(RECENTS)?.into_iter().filter(|t| !seen.contains(&t.id.0)));
            out.truncate(RECENTS);
        }
        Ok(out)
    }

    fn distinct_covers(&mut self, tracks: &[Track], max: usize) -> Vec<Pixels> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for t in tracks {
            let Some(aid) = t.album_id else { continue };
            if seen.insert(aid.0)
                && let Some(px) = self.thumb(Some(aid))
            {
                out.push(px);
                if out.len() == max {
                    break;
                }
            }
        }
        out
    }

    /// Até três cartões: álbum mais novo, curtidos, tudo em ordem aleatória.
    fn highlights(&mut self, albums: &[Album], liked: &[Track]) -> Vec<HighlightVm> {
        let mut out = Vec::new();
        let newest = albums.iter().max_by_key(|a| a.id.0);
        if let Some(a) = newest {
            out.push(HighlightVm {
                id: format!("album:{}", a.id.0),
                caption: "Adicionado há pouco".into(),
                title: a.title.clone(),
                subtitle: format!("{}, {}", a.artist, format::count_label(u64::from(a.track_count), "faixa", "faixas")),
                cover: self.big_cover(a),
            });
        }
        if !liked.is_empty() {
            let cover = liked.iter().find_map(|t| t.album_id).and_then(|id| self.albums.get(&id.0).cloned()).and_then(|a| self.big_cover(&a));
            out.push(HighlightVm {
                id: "liked".into(),
                caption: "Dos seus curtidos".into(),
                title: "Curtidos".into(),
                subtitle: format::songs_label(liked.len() as u64),
                cover,
            });
        }
        if let Some(a) = albums.get(albums.len() / 2).filter(|_| albums.len() > 1 || out.is_empty()) {
            let total: u64 = albums.iter().map(|a| u64::from(a.track_count)).sum();
            out.push(HighlightVm {
                id: "shuffle-all".into(),
                caption: "Do seu acervo".into(),
                title: "Tudo em ordem aleatória".into(),
                subtitle: format::songs_label(total),
                cover: self.big_cover(a),
            });
        }
        out
    }

    /// Gêneros do acervo (tag de gênero), do maior para o menor.
    fn moods(&mut self) -> Result<Vec<HighlightVm>, kazzu_library::Error> {
        let mut by_genre: HashMap<String, (u64, Option<AlbumId>)> = HashMap::new();
        for t in self.lib.tracks()? {
            let Some(g) = t.genre.as_deref().map(str::trim).filter(|g| !g.is_empty()) else { continue };
            let e = by_genre.entry(g.to_owned()).or_insert((0, None));
            e.0 += 1;
            if e.1.is_none() {
                e.1 = t.album_id;
            }
        }
        let mut genres: Vec<_> = by_genre.into_iter().collect();
        genres.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| a.0.cmp(&b.0)));
        genres.truncate(MOODS);
        let mut out = Vec::new();
        for (g, (n, album)) in genres {
            let cover = album.and_then(|id| self.albums.get(&id.0).cloned()).and_then(|a| self.big_cover(&a));
            out.push(HighlightVm { id: g.clone(), caption: String::new(), title: g, subtitle: format::songs_label(n), cover });
        }
        Ok(out)
    }

    /// Detalhe + ids das faixas (para tocar na ordem exibida).
    fn detail(&mut self, r: &DetailRef) -> Result<(DetailVm, Vec<String>), kazzu_library::Error> {
        let (title, subtitle, empty, scope) = match r {
            DetailRef::Album(id) => {
                let album = parse_id(id).and_then(|i| self.albums.get(&i).cloned());
                match album {
                    Some(a) => {
                        let sub = format!("{}{}", a.artist, a.year.map(|y| format!(", {y}")).unwrap_or_default());
                        (a.title, sub, "Este álbum não tem faixas indexadas.".to_owned(), Scope::Album(id.clone()))
                    }
                    None => ("Álbum".into(), String::new(), "Não encontramos este álbum. Ele pode ter sido removido da pasta.".into(), Scope::Album(id.clone())),
                }
            }
            DetailRef::Liked => ("Curtidos".into(), String::new(), "Toque no coração de uma música para ela aparecer aqui.".into(), Scope::Liked),
            DetailRef::Downloads => ("Baixados".into(), String::new(), "O modo offline com downloads chega em uma próxima etapa.".into(), Scope::Ids(vec![])),
            DetailRef::Albums => {
                let mut albums: Vec<Album> = self.albums.values().cloned().collect();
                albums.sort_by_key(|a| a.title.to_lowercase());
                let vms = albums.iter().map(|a| self.album_vm(a)).collect::<Vec<_>>();
                let sub = format::albums_label(vms.len() as u64);
                return Ok((
                    DetailVm { kind: DetailKindVm::Albums, title: "Álbuns".into(), subtitle: sub, empty_text: "Adicione uma pasta de músicas para ver seus álbuns.".into(), albums: vms, ..Default::default() },
                    vec![],
                ));
            }
            DetailRef::Playlist(id) => {
                let name = parse_id(id)
                    .and_then(|i| self.lib.playlists().ok()?.into_iter().find(|p| p.id.0 == i))
                    .map(|p| p.name)
                    .unwrap_or_else(|| "Playlist".into());
                (name, String::new(), "Esta playlist ainda está vazia.".into(), Scope::Playlist(id.clone()))
            }
            DetailRef::Genre(g) => (g.clone(), String::new(), "Nenhuma música com este gênero.".into(), Scope::Genre(g.clone())),
        };
        let tracks = self.tracks_for(&scope)?;
        let total_ms: u64 = tracks.iter().map(|t| t.duration_ms).sum();
        let vms: Vec<TrackVm> = tracks.iter().map(|t| self.track_vm(t)).collect();
        let ids = vms.iter().map(|t| t.id.clone()).collect();
        let subtitle = if subtitle.is_empty() && !vms.is_empty() {
            format!("{}, {} min", format::songs_label(vms.len() as u64), format::thousands(total_ms.div_ceil(60_000)))
        } else {
            subtitle
        };
        Ok((DetailVm { kind: DetailKindVm::Tracks, title, subtitle, empty_text: empty, tracks: vms, albums: vec![] }, ids))
    }
}

/// Scan com progresso. Roda na thread que chamar (o app usa uma thread dedicada).
pub fn scan(lib: &mut Library, emit: Emit<'_>) -> Result<ScanReport, String> {
    let last_pct = AtomicU32::new(u32::MAX);
    let report = lib.scan(&|p| {
        let vm = match p {
            ScanProgress::Discovering { files_found } => ScanVm { progress: -1.0, label: format!("Procurando músicas, {} arquivos", format::thousands(files_found as u64)) },
            ScanProgress::Reading { done, total } => ScanVm {
                progress: if total == 0 { 0.7 } else { 0.7 * done as f32 / total as f32 },
                label: format!("Lendo tags, {} de {}", format::thousands(done as u64), format::thousands(total as u64)),
            },
            ScanProgress::Artwork { done, total } => ScanVm {
                progress: if total == 0 { 1.0 } else { 0.7 + 0.3 * done as f32 / total as f32 },
                label: format!("Preparando capas, {} de {}", format::thousands(done as u64), format::thousands(total as u64)),
            },
            ScanProgress::Finished(_) => return,
        };
        // no máximo uma atualização por ponto percentual (e uma por etapa indeterminada)
        let pct = if vm.progress < 0.0 { 1000 } else { (vm.progress * 100.0) as u32 };
        if vm.progress >= 0.0 && last_pct.swap(pct, Ordering::Relaxed) == pct {
            return;
        }
        emit(JobResult::ScanProgress(vm));
    });
    report.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------------
// Execução em threads
// ---------------------------------------------------------------------------------------------

/// Thread do `Store` + thread de scan dedicada.
pub struct ThreadedRunner {
    tx: Sender<Job>,
}

impl ThreadedRunner {
    /// `emit` é chamado das threads de trabalho; deve só empurrar para a UI (ex.: `invoke_from_event_loop`).
    pub fn spawn(data_dir: PathBuf, emit: Arc<dyn Fn(JobResult) + Send + Sync>) -> std::io::Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded::<Job>();
        let scanning = Arc::new(AtomicBool::new(false));
        // Scan pedido durante outro scan (pasta adicionada/removida no meio): roda de novo quando o atual termina.
        let rescan_pending = Arc::new(AtomicBool::new(false));
        let own_tx = tx.clone();
        std::thread::Builder::new().name("kazzu-library".into()).spawn(move || {
            let mut store = match Store::open(&data_dir) {
                Ok(s) => s,
                Err(e) => {
                    emit(JobResult::Failed(format!("Não foi possível abrir a biblioteca. {e}")));
                    return;
                }
            };
            let mut queue = JobQueue::new(rx, SEARCH_DEBOUNCE);
            while let Some(job) = queue.next_job() {
                match job {
                    Job::Scan => {
                        if scanning.swap(true, Ordering::SeqCst) {
                            rescan_pending.store(true, Ordering::SeqCst);
                            continue;
                        }
                        let (scan_emit, dir, flag, pending, again) = (emit.clone(), data_dir.clone(), scanning.clone(), rescan_pending.clone(), own_tx.clone());
                        let spawned = std::thread::Builder::new().name("kazzu-scan".into()).spawn(move || {
                            let result = match Library::open(&dir) {
                                Ok(mut lib) => scan(&mut lib, &*scan_emit),
                                Err(e) => Err(e.to_string()),
                            };
                            flag.store(false, Ordering::SeqCst);
                            scan_emit(JobResult::ScanDone(result));
                            if pending.swap(false, Ordering::SeqCst) {
                                let _ = again.send(Job::Scan);
                            }
                        });
                        if let Err(e) = spawned {
                            scanning.store(false, Ordering::SeqCst);
                            emit(JobResult::Failed(format!("Não foi possível iniciar a leitura da biblioteca. {e}")));
                        }
                    }
                    other => store.run(other, &*emit),
                }
            }
        })?;
        Ok(Self { tx })
    }
}

impl JobRunner for ThreadedRunner {
    fn submit(&self, job: Job) {
        let _ = self.tx.send(job);
    }
}

/// Fila de jobs da thread de trabalho, com debounce só para buscas: a última busca de uma rajada roda `debounce`
/// depois da última digitação; qualquer outro job (tocar, capa, curtir) é atendido na hora, sem esperar a busca.
pub struct JobQueue {
    rx: Receiver<Job>,
    debounce: Duration,
    pending_search: Option<(Job, Instant)>,
}

impl JobQueue {
    pub fn new(rx: Receiver<Job>, debounce: Duration) -> Self {
        Self { rx, debounce, pending_search: None }
    }

    /// Próximo job a executar; `None` quando o canal fecha e não há busca pendente.
    pub fn next_job(&mut self) -> Option<Job> {
        loop {
            let received = match &self.pending_search {
                Some((_, deadline)) => match self.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(j) => j,
                    Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return self.pending_search.take().map(|p| p.0),
                },
                None => self.rx.recv().ok()?,
            };
            match received {
                j @ Job::Search { .. } => self.pending_search = Some((j, Instant::now() + self.debounce)),
                other => return Some(other),
            }
        }
    }
}
