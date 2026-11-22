//! Controller: liga motor de áudio, biblioteca e tela. Não conhece o Slint (fala com a tela pelo trait `View`),
//! então é testável sem janela. Tudo aqui roda na thread de UI e **nunca bloqueia**: o trabalho pesado vai
//! para `JobRunner`; os resultados voltam como `Msg`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use kazzu_audio::EngineHandle;
use kazzu_core::{CoverRef, OutputInfo, PlaybackEvent, PlaybackStatus, QueueItem, RepeatMode};

use crate::accent::Accent;
use crate::config::{AccentPref, Config, ThemePref};
use crate::format;
use crate::lru::Lru;
use crate::view::View;
use crate::vm::*;
use crate::worker::{DetailRef, Job, JobResult, JobRunner, QueueMode, Scope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Home,
    Explore,
    Library,
    Search,
    Settings,
}

/// Preferências lidas do sistema operacional na inicialização.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemPrefs {
    pub reduce_motion: Option<bool>,
}

/// Tudo que chega ao controller vindo de outras threads.
pub enum Msg {
    Job(JobResult),
    Engine(PlaybackEvent),
    FolderPicked(Option<PathBuf>),
    System(SystemPrefs),
}

pub type MsgSink = Arc<dyn Fn(Msg) + Send + Sync>;

/// Escuta de pelo menos 30 s (ou metade da faixa) conta como "ouvida" no histórico.
const LISTEN_MIN: Duration = Duration::from_secs(30);
const SEEK_STEP_SECS: i64 = 5;
const VOLUME_STEP: f32 = 0.05;
/// Quantas próximas faixas a fila da UI mostra (o resto vira "e mais N"): fila de milhares não reconstrói milhares de linhas.
pub const QUEUE_VIEW_MAX: usize = 100;
/// View-models de faixas da fila guardados (LRU).
const VM_CACHE_MAX: usize = 600;

/// Número de ponto flutuante utilizável como fração/volume: finito, preso a 0..=1; NaN e infinitos viram `fallback`.
pub fn sane_unit(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { fallback }
}

/// Texto curto para o usuário; o detalhe técnico vai para o log.
fn friendly_error(message: &str) -> String {
    let m = message.to_ascii_lowercase();
    if m.contains("audio output") || m.contains("alsa") || m.contains("device") || m.contains("stream") {
        "A saída de áudio não respondeu. Confira o dispositivo nas configurações do sistema.".to_owned()
    } else if m.contains("unsupported") || m.contains("codec") || m.contains("format") {
        "O formato deste arquivo não é suportado.".to_owned()
    } else if m.contains("no such file") || m.contains("not found") || m.contains("os error 2") {
        "O arquivo não está mais na pasta.".to_owned()
    } else {
        message.to_owned()
    }
}

pub struct Controller<V: View> {
    view: V,
    engine: EngineHandle,
    jobs: Box<dyn JobRunner>,
    sink: MsgSink,
    cfg: Config,
    cfg_path: Option<PathBuf>,

    tab: NavTab,
    detail_open: bool,
    detail_ids: Vec<String>,
    detail_title: String,
    recents_ids: Vec<String>,

    pb: PlaybackVm,
    queue_items: Vec<QueueItem>,
    current: Option<usize>,
    position: Duration,
    duration: Option<Duration>,
    queue_seq: u64,
    vm_cache: Lru<String, TrackVm>,
    detail_seq: u64,
    cfg_dirty: bool,
    art_seq: u64,
    now_art: NowArtVm,
    recorded_current: bool,
    source_label: String,

    liked: HashSet<String>,
    /// Curtidas alteradas aqui e ainda não confirmadas pela biblioteca (uma atualização atrasada não pode desfazê-las).
    pending_likes: HashMap<String, bool>,
    home: HomeVm,
    library: LibraryVm,
    settings: SettingsVm,
    scanning: bool,
    auto_scanned: bool,

    search_query: String,
    search_seq: u64,
    search_songs: Vec<TrackVm>,
    search_albums: Vec<AlbumVm>,
    search_ids: Vec<String>,
    search_local: bool,
    search_kind: i32,
}

impl<V: View> Controller<V> {
    pub fn new(view: V, engine: EngineHandle, jobs: Box<dyn JobRunner>, sink: MsgSink, cfg: Config, cfg_path: Option<PathBuf>) -> Self {
        let volume = sane_unit(cfg.volume, 1.0);
        Self {
            view,
            engine,
            jobs,
            sink,
            cfg,
            cfg_path,
            tab: NavTab::Home,
            detail_open: false,
            detail_ids: vec![],
            detail_title: String::new(),
            recents_ids: vec![],
            pb: PlaybackVm { volume, output_name: "Padrão do sistema".into(), bit_perfect: true, ..Default::default() },
            queue_items: vec![],
            current: None,
            position: Duration::ZERO,
            duration: None,
            queue_seq: 0,
            vm_cache: Lru::new(VM_CACHE_MAX),
            detail_seq: 0,
            cfg_dirty: false,
            art_seq: 0,
            now_art: NowArtVm::default(),
            recorded_current: false,
            source_label: String::new(),
            liked: HashSet::new(),
            pending_likes: HashMap::new(),
            home: HomeVm::default(),
            library: LibraryVm::default(),
            settings: SettingsVm::default(),
            scanning: false,
            auto_scanned: false,
            search_query: String::new(),
            search_seq: 0,
            search_songs: vec![],
            search_albums: vec![],
            search_ids: vec![],
            search_local: true,
            search_kind: 0,
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn playback(&self) -> &PlaybackVm {
        &self.pb
    }

    pub fn engine(&self) -> &EngineHandle {
        &self.engine
    }

    /// Primeiro quadro: aplica preferências e pede os dados à biblioteca.
    pub fn init(&mut self, system: SystemPrefs) {
        if self.cfg.reduce_motion.is_none() {
            self.cfg.reduce_motion = system.reduce_motion;
        }
        self.push_prefs();
        self.push_accent();
        self.view.set_playback(&self.pb);
        self.view.set_volume(self.pb.volume);
        self.view.set_loading(true);
        self.publish_search();
        self.jobs.submit(Job::Refresh);
    }

    // ----------------------------------------------------------------------------------------
    // Mensagens de outras threads
    // ----------------------------------------------------------------------------------------

    pub fn handle(&mut self, msg: Msg) {
        match msg {
            Msg::Engine(ev) => self.on_engine(ev),
            Msg::Job(r) => self.on_job(r),
            Msg::FolderPicked(Some(path)) => self.jobs.submit(Job::AddFolder(path)),
            Msg::FolderPicked(None) => {}
            Msg::System(sys) => {
                if self.cfg.reduce_motion.is_none() {
                    self.cfg.reduce_motion = sys.reduce_motion;
                    self.push_prefs();
                }
            }
        }
    }

    fn on_job(&mut self, r: JobResult) {
        match r {
            JobResult::Refreshed(snap) => {
                let mut snap = *snap;
                // abertura: reindexa (incremental) as pastas já configuradas
                if !self.auto_scanned && !snap.settings.folders.is_empty() {
                    self.auto_scanned = true;
                    self.jobs.submit(Job::Scan);
                }
                self.liked = snap.liked;
                for (id, liked) in &self.pending_likes {
                    if *liked { self.liked.insert(id.clone()); } else { self.liked.remove(id); }
                }
                self.flag_tracks(&mut snap.home.recents);
                self.recents_ids = snap.home.recents.iter().map(|t| t.id.clone()).collect();
                self.home = snap.home;
                self.library = snap.library;
                self.settings = snap.settings;
                self.view.set_loading(false);
                self.view.set_home(&self.home);
                self.view.set_library(&self.library);
                self.publish_settings();
                if self.pb.now.is_none() {
                    self.view.set_ambient(snap.ambient.as_ref());
                }
            }
            JobResult::ScanProgress(p) => {
                self.scanning = true;
                self.view.set_scan(Some(&p));
            }
            JobResult::ScanDone(res) => {
                self.scanning = false;
                self.view.set_scan(None);
                match res {
                    Ok(report) => {
                        if !report.failed.is_empty() {
                            let n = report.failed.len();
                            let text = if n == 1 { "1 arquivo não pôde ser lido.".to_owned() } else { format!("{} arquivos não puderam ser lidos.", format::thousands(n as u64)) };
                            self.view.toast(&text, true);
                        }
                        self.jobs.submit(Job::Refresh);
                    }
                    Err(e) => self.view.toast(&format!("A leitura da biblioteca falhou. {e}"), true),
                }
            }
            JobResult::FoldersChanged => {
                self.jobs.submit(Job::Scan);
                self.jobs.submit(Job::Refresh);
            }
            JobResult::Search { seq, songs, albums, ids } => {
                if seq == self.search_seq {
                    self.search_songs = songs;
                    self.search_albums = albums;
                    self.search_ids = ids;
                    self.publish_search();
                }
            }
            JobResult::Queue { items, start, shuffle, label } => {
                self.source_label = format!("Tocando de {label}");
                self.pb.error.clear();
                self.pb.source_label = self.source_label.clone();
                // O embaralhar escolhido pelo usuário vale para a nova fila (o motor reembaralha no Load). Só o
                // pedido explícito ("Tudo em ordem aleatória", "Aleatório") liga o modo.
                self.engine.load(items, start);
                if shuffle && !self.pb.shuffle {
                    self.engine.set_shuffle(true);
                }
                self.view.set_playback(&self.pb);
            }
            JobResult::QueueItems { items, mode } => {
                let n = items.len();
                match mode {
                    QueueMode::PlayNext => self.engine.play_next(items),
                    QueueMode::Enqueue => self.engine.enqueue(items),
                }
                let what = if n == 1 { "Música" } else { "Músicas" };
                self.view.toast(&format!("{what} {}.", if mode == QueueMode::PlayNext { "tocará em seguida" } else { "adicionada à fila" }), false);
            }
            JobResult::Detail { seq, detail, ids } => {
                // resposta atrasada (o usuário já trocou de aba ou abriu outra coisa): descarta
                if seq != self.detail_seq {
                    return;
                }
                let mut vm = *detail;
                self.flag_tracks(&mut vm.tracks);
                self.detail_ids = ids;
                self.detail_title = vm.title.clone();
                self.view.set_detail(&vm);
                self.view.open_detail(true);
                self.detail_open = true;
            }
            JobResult::NowArt { seq, art } => {
                if seq == self.art_seq {
                    self.now_art = *art;
                    if let Some(now) = self.pb.now.as_mut() {
                        now.cover = self.now_art.thumb.clone();
                    }
                    self.view.set_playback(&self.pb);
                    self.view.set_now_art(&self.now_art);
                    self.view.set_ambient(self.now_art.backdrop.as_ref());
                    self.push_accent();
                }
            }
            JobResult::TrackVms { seq, tracks } => {
                // o cache aproveita qualquer resposta; só publicar depende de ainda ser a fila atual
                for t in tracks {
                    self.vm_cache.insert(t.id.clone(), t);
                }
                if seq == self.queue_seq {
                    self.publish_queue();
                }
            }
            JobResult::Recents(mut list) => {
                self.flag_tracks(&mut list);
                self.recents_ids = list.iter().map(|t| t.id.clone()).collect();
                self.home.recents = list;
                self.view.set_home(&self.home);
            }
            JobResult::Failed(msg) => self.view.toast(&msg, true),
            JobResult::LikedSaved { id, liked } => {
                if self.pending_likes.get(&id) == Some(&liked) {
                    self.pending_likes.remove(&id);
                }
            }
        }
    }

    // ----------------------------------------------------------------------------------------
    // Eventos do motor de áudio
    // ----------------------------------------------------------------------------------------

    pub fn on_engine(&mut self, ev: PlaybackEvent) {
        match ev {
            PlaybackEvent::StatusChanged { status } => {
                self.pb.playing = status == PlaybackStatus::Playing;
                self.pb.loading = status == PlaybackStatus::Buffering;
                self.view.set_playback(&self.pb);
            }
            PlaybackEvent::TrackChanged { item, duration } => {
                self.record_listen_if_needed();
                self.position = Duration::ZERO;
                self.duration = duration;
                self.recorded_current = false;
                self.pb.error.clear();
                match item {
                    Some(item) => {
                        let vm = self.vm_from_item(&item);
                        self.pb.has_now = true;
                        self.pb.now = Some(vm);
                        self.art_seq += 1;
                        let cover = match &item.track.cover {
                            Some(CoverRef::File { path }) => Some(path.clone()),
                            _ => None,
                        };
                        self.jobs.submit(Job::NowArt { seq: self.art_seq, cover });
                    }
                    None => {
                        self.pb.now = None;
                        self.pb.has_now = false;
                        self.now_art = NowArtVm::default();
                        self.view.set_now_art(&self.now_art);
                        self.push_accent();
                    }
                }
                self.view.set_playback(&self.pb);
                self.view.mark_playing(self.pb.now.as_ref().map(|t| t.id.as_str()));
                self.push_progress();
            }
            PlaybackEvent::Position { position, duration } => {
                self.position = position;
                if duration.is_some() {
                    self.duration = duration;
                }
                self.push_progress();
            }
            PlaybackEvent::TrackEnded { id } => {
                if let Some(item) = self.queue_items.iter().find(|i| i.id == id) {
                    let ms = item.track.duration.map_or(0, |d| d.as_millis() as u64);
                    self.jobs.submit(Job::RecordPlay { id: item.track.id.to_string(), listened_ms: ms });
                    self.recorded_current = true;
                }
            }
            PlaybackEvent::QueueChanged { items, current } => {
                self.queue_items = items;
                self.current = current;
                self.queue_seq += 1;
                let missing: Vec<String> = self.upcoming().iter().take(QUEUE_VIEW_MAX).map(|i| i.track.id.to_string()).filter(|id| !self.vm_cache.contains(id)).collect();
                if missing.is_empty() {
                    self.publish_queue();
                } else {
                    self.jobs.submit(Job::TrackVms { seq: self.queue_seq, ids: missing });
                    // mostra já o que se sabe (sem capas) para a fila não ficar defasada
                    self.publish_queue();
                }
            }
            PlaybackEvent::VolumeChanged { volume } => {
                let volume = sane_unit(volume, self.pb.volume);
                self.pb.volume = volume;
                self.cfg.volume = volume;
                self.cfg_dirty = true;
                self.view.set_volume(volume);
            }
            PlaybackEvent::RepeatChanged { mode } => {
                self.pb.repeat = match mode {
                    RepeatMode::Off => RepeatVm::Off,
                    RepeatMode::All => RepeatVm::All,
                    RepeatMode::One => RepeatVm::One,
                };
                self.view.set_playback(&self.pb);
            }
            PlaybackEvent::ShuffleChanged { enabled } => {
                self.pb.shuffle = enabled;
                self.view.set_playback(&self.pb);
            }
            PlaybackEvent::OutputChanged { info } => {
                self.apply_output(&info);
                self.view.set_playback(&self.pb);
            }
            PlaybackEvent::QueueFinished => {
                self.pb.playing = false;
                self.view.set_playback(&self.pb);
            }
            PlaybackEvent::Error { id, message } => {
                let title = id
                    .and_then(|id| self.queue_items.iter().find(|i| i.id == id))
                    .map(|i| i.track.title.clone());
                let is_current = id.is_some() && id == self.current.and_then(|c| self.queue_items.get(c)).map(|i| i.id);
                let reason = friendly_error(&message);
                let text = match &title {
                    Some(t) => format!("Não foi possível tocar “{t}”. {reason}"),
                    None => format!("Erro de reprodução. {reason}"),
                };
                log::warn!("reprodução: {message}");
                if is_current {
                    self.pb.error = text.clone();
                    self.view.set_playback(&self.pb);
                }
                self.view.toast(&text, true);
            }
        }
    }

    fn apply_output(&mut self, info: &OutputInfo) {
        self.pb.output_name = match info.device.as_str() {
            "" | "default" => "Padrão do sistema".to_owned(),
            "null" => "Saída simulada".to_owned(),
            other => other.to_owned(),
        };
        self.pb.bit_perfect = info.bit_perfect;
        self.pb.output_note = if info.bit_perfect {
            "Original, sem equalizador".to_owned()
        } else if let Some(from) = info.resampled_from {
            format!("Reamostrado de {} para {} kHz", format::khz(from), format::khz(info.sample_rate))
        } else {
            "Volume ajustado: o sinal não está bit-perfect".to_owned()
        };
        // o selo do player vem do que o motor realmente está tocando
        if let (Some(src), Some(now)) = (&info.source, self.pb.now.as_mut()) {
            let b = format::badge(&src.codec.to_string(), src.codec.is_lossless(), src.bit_depth, src.sample_rate, src.bitrate_kbps);
            if !b.codec.is_empty() {
                // o motor chama WAV/AIFF de "PCM"; o nome do arquivo (WAV) diz mais ao usuário
                if !(b.codec == "PCM" && !now.codec.is_empty()) {
                    now.codec = b.codec;
                }
                now.detail = b.detail;
                now.detail_long = b.detail_long;
                now.hires = b.hires;
            }
        }
    }

    fn vm_from_item(&self, item: &QueueItem) -> TrackVm {
        let t = &item.track;
        let id = t.id.to_string();
        let path = match &t.source {
            kazzu_core::TrackSource::Local { path } => Some(path.as_path()),
            _ => None,
        };
        let b = match &t.format {
            Some(f) => {
                let codec = if f.codec == kazzu_core::Codec::Pcm { format::pcm_label(path).to_owned() } else { f.codec.to_string() };
                format::badge(&codec, f.codec.is_lossless(), f.bit_depth, f.sample_rate, f.bitrate_kbps)
            }
            None => format::Badge::default(),
        };
        TrackVm {
            liked: self.liked.contains(&id),
            id,
            title: t.title.clone(),
            artist: t.artist_line(),
            album: t.album.as_ref().map(|a| a.title.clone()).unwrap_or_default(),
            cover: None,
            duration_label: t.duration.map(format::duration_label).unwrap_or_default(),
            codec: b.codec,
            detail: b.detail,
            detail_long: b.detail_long,
            hires: b.hires,
            source: String::new(),
            playing: true,
        }
    }

    fn upcoming(&self) -> &[QueueItem] {
        match self.current {
            Some(c) if c < self.queue_items.len() => &self.queue_items[c + 1..],
            _ => &[],
        }
    }

    fn publish_queue(&self) {
        let up = self.upcoming();
        let total: Duration = up.iter().filter_map(|i| i.track.duration).sum();
        let tracks: Vec<TrackVm> = up
            .iter()
            .take(QUEUE_VIEW_MAX)
            .map(|i| {
                let id = i.track.id.to_string();
                let mut vm = self.vm_cache.peek(&id).cloned().unwrap_or_else(|| {
                    let mut vm = self.vm_from_item(i);
                    vm.cover = None;
                    vm
                });
                vm.playing = false;
                vm.liked = self.liked.contains(&vm.id);
                vm
            })
            .collect();
        let hidden = up.len().saturating_sub(QUEUE_VIEW_MAX);
        self.view.set_queue(&QueueVm {
            tracks,
            summary: if up.is_empty() { String::new() } else { format::queue_summary(up.len(), total) },
            more_label: if hidden == 0 { String::new() } else { format!("e mais {}", format::songs_label(hidden as u64)) },
        });
    }

    fn push_progress(&self) {
        let progress = match self.duration {
            Some(d) if !d.is_zero() => (self.position.as_secs_f32() / d.as_secs_f32()).clamp(0.0, 1.0),
            _ => 0.0,
        };
        self.view.set_progress(progress, &format::duration_label(self.position), &format::remaining_label(self.position, self.duration));
    }

    fn record_listen_if_needed(&mut self) {
        if self.recorded_current {
            return;
        }
        let Some(now) = &self.pb.now else { return };
        let long_enough = self.position >= LISTEN_MIN || self.duration.is_some_and(|d| self.position * 2 >= d && !d.is_zero());
        if long_enough {
            self.jobs.submit(Job::RecordPlay { id: now.id.clone(), listened_ms: self.position.as_millis() as u64 });
        }
        self.recorded_current = true;
    }

    // ----------------------------------------------------------------------------------------
    // Comandos da UI
    // ----------------------------------------------------------------------------------------

    pub fn navigate(&mut self, tab: NavTab) {
        self.tab = tab;
        self.detail_open = false;
        self.detail_seq += 1; // um detalhe pedido antes desta troca de aba não pode abrir depois dela
    }

    pub fn detail_closed(&mut self) {
        self.detail_open = false;
        self.detail_seq += 1;
    }

    fn request_detail(&mut self, what: DetailRef) {
        self.detail_seq += 1;
        self.jobs.submit(Job::OpenDetail { seq: self.detail_seq, what });
    }

    pub fn toggle_play(&mut self) {
        if self.pb.has_now {
            self.engine.toggle_pause();
        } else if let Some(first) = self.recents_ids.first().cloned() {
            self.play_context(&first);
        } else {
            self.view.toast("Adicione uma pasta de músicas para começar a ouvir.", false);
        }
    }

    pub fn next(&mut self) {
        self.engine.next();
    }

    pub fn previous(&mut self) {
        self.engine.previous();
    }

    /// `progress` 0..1.
    pub fn seek(&mut self, progress: f32) {
        if let Some(d) = self.duration {
            let target = d.mul_f32(sane_unit(progress, 0.0));
            self.engine.seek(target);
            self.position = target;
            self.push_progress();
        }
    }

    pub fn seek_relative(&mut self, secs: i64) {
        let Some(d) = self.duration else { return };
        let step = secs.signum() * SEEK_STEP_SECS.max(secs.abs());
        let target = if step >= 0 { self.position + Duration::from_secs(step as u64) } else { self.position.saturating_sub(Duration::from_secs(step.unsigned_abs())) };
        let target = target.min(d);
        self.engine.seek(target);
        self.position = target;
        self.push_progress();
    }

    pub fn set_volume(&mut self, v: f32) {
        let v = sane_unit(v, self.pb.volume);
        self.pb.volume = v;
        self.cfg.volume = (v * 1000.0).round() / 1000.0;
        self.cfg_dirty = true;
        self.engine.set_volume(v);
    }

    pub fn volume_step(&mut self, dir: i32) {
        let v = ((self.pb.volume / VOLUME_STEP).round() + dir as f32) * VOLUME_STEP;
        self.set_volume(v.clamp(0.0, 1.0));
        self.view.set_volume(self.pb.volume);
    }

    pub fn toggle_shuffle(&mut self) {
        self.engine.set_shuffle(!self.pb.shuffle);
    }

    pub fn cycle_repeat(&mut self) {
        let next = match self.pb.repeat {
            RepeatVm::Off => RepeatMode::All,
            RepeatVm::All => RepeatMode::One,
            RepeatVm::One => RepeatMode::Off,
        };
        self.engine.set_repeat(next);
    }

    pub fn retry_track(&mut self) {
        if let Some(c) = self.current {
            self.pb.error.clear();
            self.view.set_playback(&self.pb);
            self.engine.load(self.queue_items.clone(), c);
        }
    }

    /// Clique numa faixa (linha, destaque ou resultado): a lista de origem vira a fila.
    pub fn play_track(&mut self, id: &str) {
        self.play_context(id);
    }

    fn play_context(&mut self, id: &str) {
        let (scope, label, shuffle, start) = if let Some(album) = id.strip_prefix("album:") {
            (Scope::Album(album.to_owned()), String::new(), false, None)
        } else if id == "liked" {
            (Scope::Liked, "Curtidos".to_owned(), false, None)
        } else if id == "shuffle-all" {
            (Scope::All, "Biblioteca".to_owned(), true, None)
        } else if self.detail_open && self.detail_ids.iter().any(|t| t == id) {
            (Scope::Ids(self.detail_ids.clone()), self.detail_title.clone(), false, Some(id.to_owned()))
        } else if self.tab == NavTab::Search && self.search_ids.iter().any(|t| t == id) {
            self.remember_search();
            (Scope::Ids(self.search_ids.clone()), "Busca".to_owned(), false, Some(id.to_owned()))
        } else if self.recents_ids.iter().any(|t| t == id) {
            (Scope::Ids(self.recents_ids.clone()), "Recentes".to_owned(), false, Some(id.to_owned()))
        } else {
            (Scope::Ids(vec![id.to_owned()]), "Biblioteca".to_owned(), false, None)
        };
        self.jobs.submit(Job::Play { scope, start_id: start, shuffle, label });
    }

    fn remember_search(&mut self) {
        if self.search_query.trim().len() >= 2 {
            self.cfg.push_recent_search(&self.search_query);
            self.save_config();
            self.publish_search();
        }
    }

    pub fn open_album(&mut self, id: &str) {
        self.request_detail(DetailRef::Album(id.to_owned()));
    }

    pub fn open_playlist(&mut self, id: &str) {
        self.request_detail(DetailRef::Playlist(id.to_owned()));
    }

    pub fn open_genre(&mut self, id: &str) {
        self.request_detail(DetailRef::Genre(id.to_owned()));
    }

    pub fn open_collection(&mut self, id: &str) {
        match id {
            "downloads" => self.request_detail(DetailRef::Downloads),
            "liked" => self.request_detail(DetailRef::Liked),
            "albums" => self.request_detail(DetailRef::Albums),
            _ => {
                let msg = if self.library.playlists.is_empty() { "Você ainda não criou playlists. Use Criar playlist." } else { "Suas playlists estão logo abaixo." };
                self.view.toast(msg, false);
            }
        }
    }

    /// Toca o detalhe aberto (ou o embaralha).
    pub fn play_all(&mut self, shuffle: bool) {
        if self.detail_ids.is_empty() {
            return;
        }
        self.jobs.submit(Job::Play { scope: Scope::Ids(self.detail_ids.clone()), start_id: None, shuffle, label: self.detail_title.clone() });
    }

    pub fn toggle_like(&mut self, id: &str) {
        if id.is_empty() {
            return;
        }
        let liked = !self.liked.contains(id);
        if liked { self.liked.insert(id.to_owned()); } else { self.liked.remove(id); }
        self.pending_likes.insert(id.to_owned(), liked);
        if let Some(now) = self.pb.now.as_mut().filter(|t| t.id == id) {
            now.liked = liked;
            self.view.set_playback(&self.pb);
        }
        self.view.mark_liked(id, liked);
        if let Some(c) = self.library.collections.iter_mut().find(|c| c.id == "liked") {
            c.count_label = format::songs_label(self.liked.len() as u64);
            self.view.set_library(&self.library);
        }
        self.jobs.submit(Job::SetLiked { id: id.to_owned(), liked });
    }

    pub fn queue_track(&mut self, id: &str, mode: QueueMode) {
        self.jobs.submit(Job::QueueItems { id: id.to_owned(), mode });
    }

    pub fn queue_remove(&mut self, index: usize) {
        if let Some(item) = self.upcoming().get(index) {
            self.engine.remove(item.id);
        }
    }

    pub fn queue_jump(&mut self, index: usize) {
        if let Some(item) = self.upcoming().get(index) {
            self.engine.jump_to(item.id);
        }
    }

    pub fn queue_clear(&mut self) {
        self.engine.clear();
    }

    pub fn search(&mut self, text: &str) {
        self.search_query = text.trim().to_owned();
        self.search_seq += 1;
        if self.search_query.is_empty() {
            self.search_songs.clear();
            self.search_albums.clear();
            self.search_ids.clear();
            self.publish_search();
        } else {
            self.jobs.submit(Job::Search { seq: self.search_seq, query: self.search_query.clone() });
        }
    }

    pub fn search_filters(&mut self, kind: i32, local: bool) {
        self.search_kind = kind;
        self.search_local = local;
        self.publish_search();
    }

    pub fn clear_recent_searches(&mut self) {
        self.cfg.recent_searches.clear();
        self.save_config();
        self.publish_search();
    }

    fn publish_search(&self) {
        let state = if self.search_query.is_empty() {
            SearchStateVm::Idle
        } else if !self.search_local || (self.search_songs.is_empty() && self.search_albums.is_empty()) {
            SearchStateVm::Empty
        } else {
            SearchStateVm::Results
        };
        let (songs, albums) = if self.search_local { (self.search_songs.clone(), self.search_albums.clone()) } else { (vec![], vec![]) };
        let mut songs = songs;
        self.flag_tracks(&mut songs);
        self.view.set_search(&SearchVm { state, songs, albums, recent: self.cfg.recent_searches.clone() });
    }

    pub fn create_playlist(&mut self, name: &str) {
        self.jobs.submit(Job::CreatePlaylist(name.to_owned()));
    }

    pub fn add_folder(&mut self) {
        let sink = self.sink.clone();
        let spawned = std::thread::Builder::new().name("kazzu-folder-dialog".into()).spawn(move || {
            let picked = pollster::block_on(rfd::AsyncFileDialog::new().set_title("Escolha a pasta de músicas").pick_folder());
            sink(Msg::FolderPicked(picked.map(|h| h.path().to_path_buf())));
        });
        if spawned.is_err() {
            self.view.toast("Não foi possível abrir o seletor de pastas.", true);
        }
    }

    /// Adiciona uma pasta já conhecida (ex.: `--demo-library`).
    pub fn add_folder_path(&mut self, path: PathBuf) {
        self.jobs.submit(Job::AddFolder(path));
    }

    pub fn remove_folder(&mut self, path: &str) {
        self.jobs.submit(Job::RemoveFolder(PathBuf::from(path)));
    }

    pub fn rescan(&mut self) {
        if !self.scanning {
            self.jobs.submit(Job::Scan);
        }
    }

    pub fn coming_soon(&mut self, what: &str) {
        self.view.toast(&format!("{what} chega em uma próxima versão."), false);
    }

    // ----------------------------------------------------------------------------------------
    // Preferências
    // ----------------------------------------------------------------------------------------

    pub fn set_theme(&mut self, theme: ThemePref) {
        self.cfg.theme = theme;
        self.push_prefs();
        self.save_config();
    }

    pub fn set_accent_pref(&mut self, accent: AccentPref) {
        self.cfg.accent = accent;
        self.push_prefs();
        self.push_accent();
        self.save_config();
    }

    pub fn set_reduce_transparency(&mut self, on: bool) {
        self.cfg.reduce_transparency = Some(on);
        self.push_prefs();
        self.save_config();
    }

    pub fn set_reduce_motion(&mut self, on: bool) {
        self.cfg.reduce_motion = Some(on);
        self.push_prefs();
        self.save_config();
    }

    fn push_prefs(&self) {
        let prefs = PrefsVm {
            theme: self.cfg.theme,
            accent: self.cfg.accent,
            reduce_motion: self.cfg.reduce_motion.unwrap_or(false),
            reduce_transparency: self.cfg.reduce_transparency.unwrap_or(false),
        };
        self.view.set_prefs(&prefs);
    }

    fn push_accent(&self) {
        let accent = if self.pb.has_now { self.now_art.accent } else { Accent::BRAND };
        self.view.set_accent(&accent, self.cfg.accent == AccentPref::Cover);
    }

    fn publish_settings(&self) {
        let providers = vec![
            ProviderVm { id: "local".into(), name: "Biblioteca local".into(), status_label: self.settings.summary.clone(), connected: self.settings.has_library, available: true },
            ProviderVm { id: "ytmusic".into(), name: "YT Music".into(), status_label: "Chega em uma próxima versão".into(), connected: false, available: false },
            ProviderVm { id: "spotify".into(), name: "Spotify".into(), status_label: "Chega em uma próxima versão".into(), connected: false, available: false },
            ProviderVm { id: "tidal-qobuz".into(), name: "Tidal e Qobuz".into(), status_label: "Chegam em uma versão futura".into(), connected: false, available: false },
        ];
        self.view.set_settings(&self.settings, &providers);
    }

    fn save_config(&mut self) {
        self.cfg_dirty = false;
        if let Some(path) = &self.cfg_path
            && let Err(e) = self.cfg.save(path)
        {
            log::warn!("{e}");
            self.view.toast("Não foi possível salvar as configurações.", true);
        }
    }

    fn flag_tracks(&self, tracks: &mut [TrackVm]) {
        let now = self.pb.now.as_ref().map(|t| t.id.as_str());
        for t in tracks {
            t.playing = Some(t.id.as_str()) == now;
            t.liked = self.liked.contains(&t.id);
        }
    }

    /// Grava a configuração se algo mudou desde a última gravação (usado pelo temporizador do volume).
    pub fn flush_config(&mut self) {
        if self.cfg_dirty {
            self.save_config();
        }
    }

    /// Mostra um aviso do app (ex.: configuração corrompida na abertura).
    pub fn notice(&self, text: &str, error: bool) {
        self.view.toast(text, error);
    }

    /// Fim do app: grava a configuração e para o motor. Seguro chamar mais de uma vez.
    pub fn shutdown(&mut self) {
        self.cfg_dirty = true;
        self.flush_config();
        self.engine.stop();
    }

    #[cfg(test)]
    pub fn upcoming_ids(&self) -> Vec<String> {
        self.upcoming().iter().map(|i| i.track.id.to_string()).collect()
    }

}
