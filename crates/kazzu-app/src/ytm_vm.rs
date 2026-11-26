//! Do catálogo remoto (modelos do `kazzu-ytm`) para view-models. Funções puras: sem rede, sem Slint.
//!
//! Ids de itens remotos na UI levam o prefixo `yt:` (`yt:<video id>`, `yt:<browse id>`): é assim que o controller
//! distingue um item do YouTube Music de um arquivo local (ids numéricos).

use std::collections::HashSet;

use kazzu_core::{Album, Artist, Codec, CoverRef, Playlist, ProviderKind, Track, TrackSource};
use kazzu_ytm::{AlbumPage, ArtistPage, Category, Item, PlaylistPage, Shelf};

use crate::catalog::{ErrorKind, ExploreFeed};
use crate::format;
use crate::vm::*;

pub const SOURCE_NAME: &str = "YouTube Music";
/// Tag das seções (`SourceTag`): ícone genérico de nuvem + este texto, nunca a marca do YouTube.
pub const SOURCE_LABEL: &str = "Do YouTube Music";

const HERO_MAX: usize = 6;
const QUICK_PICKS_MAX: usize = 8;
const MIXES_MAX: usize = 12;
const ALBUMS_MAX: usize = 12;
const TRENDING_MAX: usize = 20;

pub const REMOTE_PREFIX: &str = "yt:";

pub fn remote_id(id: &str) -> String {
    format!("{REMOTE_PREFIX}{id}")
}

/// O id sem o prefixo, se for remoto.
pub fn strip_remote(id: &str) -> Option<&str> {
    id.strip_prefix(REMOTE_PREFIX)
}

pub fn is_remote_id(id: &str) -> bool {
    id.starts_with(REMOTE_PREFIX)
}

fn cover_url(c: &Option<CoverRef>) -> Option<String> {
    match c {
        Some(CoverRef::Url { url }) => Some(url.clone()),
        _ => None,
    }
}

/// "13 músicas • 49 minutos" -> "13 músicas, 49 minutos" (sem ponto-médio decorativo).
pub fn tidy_summary(s: &str) -> String {
    s.replace(" • ", ", ").replace('•', ",")
}

fn spotify(track: &Track) -> bool {
    matches!(track.source, TrackSource::Remote { provider: ProviderKind::Spotify, .. })
}
pub fn track_ui_id(track: &Track) -> String {
    if spotify(track) { format!("sp:{}", track.id.as_str()) } else { remote_id(track.id.as_str()) }
}

pub fn track_vm(t: &Track, liked: bool, why: &str) -> TrackVm {
    let b = match &t.format {
        Some(f) => format::badge(&f.codec.to_string(), f.codec.is_lossless(), f.bit_depth, f.sample_rate, f.bitrate_kbps),
        None => format::Badge::default(),
    };
    TrackVm {
        id: track_ui_id(t),
        title: t.title.clone(),
        artist: t.artist_line(),
        album: t.album.as_ref().map(|a| a.title.clone()).unwrap_or_default(),
        cover: None,
        cover_url: cover_url(&t.cover),
        duration_label: t.duration.map(format::duration_label).unwrap_or_default(),
        codec: b.codec,
        detail: b.detail,
        detail_long: b.detail_long,
        hires: b.hires,
        source: if spotify(t) { "Spotify · Metadados".into() } else { SOURCE_NAME.to_owned() },
        liked,
        playing: false,
        why: why.to_owned(),
    }
}

pub fn album_vm(a: &Album) -> AlbumVm {
    AlbumVm { id: remote_id(a.id.as_str()), title: a.title.clone(), artist: artist_names(&a.artists), cover: None, cover_url: cover_url(&a.cover) }
}

fn artist_names(artists: &[kazzu_core::ArtistRef]) -> String {
    artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
}

fn artist_vm(a: &Artist) -> ArtistVm {
    ArtistVm { id: remote_id(a.id.as_str()), name: a.name.clone(), subtitle: "Artista".to_owned(), cover: None, cover_url: cover_url(&a.image) }
}

fn playlist_subtitle(p: &Playlist) -> String {
    p.description.clone().filter(|d| !d.trim().is_empty()).unwrap_or_else(|| "Playlist".to_owned())
}

/// Item de um carrossel como cartão (destaque, mix, melhor resultado).
pub fn highlight(item: &Item, caption: &str) -> HighlightVm {
    let base = HighlightVm { caption: caption.to_owned(), ..Default::default() };
    match item {
        Item::Album(a) => HighlightVm {
            id: remote_id(a.id.as_str()),
            kind: "album".into(),
            title: a.title.clone(),
            subtitle: artist_names(&a.artists),
            cover_url: cover_url(&a.cover),
            ..base
        },
        Item::Playlist(p) => HighlightVm {
            id: remote_id(p.id.as_str()),
            kind: "playlist".into(),
            title: p.name.clone(),
            subtitle: playlist_subtitle(p),
            cover_url: cover_url(&p.cover),
            ..base
        },
        Item::Artist(a) => HighlightVm {
            id: remote_id(a.id.as_str()),
            kind: "artist".into(),
            title: a.name.clone(),
            subtitle: "Artista".into(),
            cover_url: cover_url(&a.image),
            ..base
        },
        Item::Track(t) => HighlightVm {
            id: remote_id(t.id.as_str()),
            kind: "track".into(),
            title: t.title.clone(),
            subtitle: t.artist_line(),
            cover_url: cover_url(&t.cover),
            ..base
        },
    }
}

fn shelf_tracks(shelf: &Shelf) -> impl Iterator<Item = &Track> {
    shelf.items.iter().filter_map(|i| if let Item::Track(t) = i { Some(t) } else { None })
}

/// Maioria de faixas (a estante vira "Escolhas rápidas").
fn is_track_shelf(shelf: &Shelf) -> bool {
    let tracks = shelf_tracks(shelf).count();
    tracks >= 3 && tracks * 2 >= shelf.items.len()
}

/// Todas as faixas remotas mostradas, para o controller poder tocá-las depois pelo id da linha.
pub type TrackSink = Vec<Track>;

/// Monta o Início a partir das estantes do feed. `section` filtra o que aparece (chips do Início).
pub fn build_home(shelves: &[Shelf], section: HomeSectionVm, liked: &HashSet<String>, sink: &mut TrackSink) -> HomeVm {
    let mut vm = HomeVm { state: LoadStateVm::Ready, section, source_label: SOURCE_LABEL.to_owned(), ..Default::default() };
    let mut seen: HashSet<String> = HashSet::new();

    // destaques: álbuns e playlists das primeiras estantes (nunca faixas soltas, salvo se não houver mais nada)
    for shelf in shelves {
        for item in &shelf.items {
            if vm.highlights.len() >= HERO_MAX {
                break;
            }
            if matches!(item, Item::Album(_) | Item::Playlist(_)) && seen.insert(item.id().to_owned()) {
                vm.highlights.push(highlight(item, &shelf.title));
            }
        }
    }
    // escolhas rápidas: primeira estante majoritariamente de faixas
    if let Some(shelf) = shelves.iter().find(|s| is_track_shelf(s)) {
        for t in shelf_tracks(shelf).take(QUICK_PICKS_MAX) {
            vm.quick_picks.push(track_vm(t, liked.contains(&remote_id(t.id.as_str())), &shelf.title));
            sink.push(t.clone());
        }
    }
    // mixes: playlists que não viraram destaque
    for shelf in shelves {
        for item in &shelf.items {
            if let Item::Playlist(_) = item
                && vm.mixes.len() < MIXES_MAX
                && !vm.highlights.iter().any(|h| h.id == remote_id(item.id()))
            {
                vm.mixes.push(highlight(item, ""));
            }
        }
    }
    // álbuns em destaque
    for shelf in shelves {
        for item in &shelf.items {
            if let Item::Album(a) = item
                && vm.albums.len() < ALBUMS_MAX
                && !vm.highlights.iter().any(|h| h.id == remote_id(item.id()))
            {
                vm.albums.push(album_vm(a));
            }
        }
    }
    if vm.highlights.is_empty() && !vm.quick_picks.is_empty() {
        // feed só de faixas: as primeiras viram destaque (toque = toca)
        for t in vm.quick_picks.iter().take(4).cloned().collect::<Vec<_>>() {
            vm.highlights.push(HighlightVm { id: t.id, kind: "track".into(), caption: t.why, title: t.title, subtitle: t.artist, cover_url: t.cover_url, ..Default::default() });
        }
    }
    match section {
        HomeSectionVm::ForYou => {}
        HomeSectionVm::Discover => {
            vm.mixes.clear();
            vm.highlights.retain(|h| h.kind == "album");
        }
        HomeSectionVm::Mixes => {
            vm.highlights.retain(|h| h.kind == "playlist");
            vm.quick_picks.clear();
            vm.albums.clear();
        }
        HomeSectionVm::Focus | HomeSectionVm::Workout => {}
    }
    if vm.highlights.is_empty() && vm.quick_picks.is_empty() && vm.mixes.is_empty() && vm.albums.is_empty() {
        vm.state = LoadStateVm::Empty;
        vm.error_title = "Nada por aqui ainda".into();
        vm.error_text = "O YouTube Music não trouxe sugestões agora. Tente de novo em instantes.".into();
    }
    vm
}

/// Início dos chips Foco e Treino: playlists e músicas de uma busca temática.
pub fn build_home_from_items(items: &[Item], section: HomeSectionVm, liked: &HashSet<String>, sink: &mut TrackSink) -> HomeVm {
    let mut vm = HomeVm { state: LoadStateVm::Ready, section, source_label: SOURCE_LABEL.to_owned(), ..Default::default() };
    for item in items {
        match item {
            Item::Playlist(_) if vm.mixes.len() < MIXES_MAX => vm.mixes.push(highlight(item, "")),
            Item::Album(a) if vm.albums.len() < ALBUMS_MAX => vm.albums.push(album_vm(a)),
            Item::Track(t) if vm.quick_picks.len() < QUICK_PICKS_MAX => {
                vm.quick_picks.push(track_vm(t, liked.contains(&remote_id(t.id.as_str())), ""));
                sink.push(t.clone());
            }
            _ => {}
        }
    }
    vm.highlights = vm.mixes.iter().take(4).cloned().collect();
    vm.mixes.drain(..vm.highlights.len().min(vm.mixes.len()));
    if vm.highlights.is_empty() && vm.quick_picks.is_empty() && vm.albums.is_empty() {
        vm.state = LoadStateVm::Empty;
        vm.error_title = "Nada por aqui ainda".into();
        vm.error_text = "Não achei sugestões para este tema agora.".into();
    }
    vm
}

pub fn build_explore(feed: &ExploreFeed, liked: &HashSet<String>, sink: &mut TrackSink) -> ExploreVm {
    let mut vm = ExploreVm { state: LoadStateVm::Ready, ..Default::default() };
    for shelf in &feed.new_releases {
        for item in &shelf.items {
            if let Item::Album(a) = item
                && vm.new_releases.len() < ALBUMS_MAX
            {
                vm.new_releases.push(album_vm(a));
            }
        }
    }
    for (i, c) in feed.categories.iter().enumerate() {
        vm.moods.push(mood_highlight(i, c));
    }
    if let Some(shelf) = feed.charts.iter().find(|s| is_track_shelf(s)) {
        for t in shelf_tracks(shelf).take(TRENDING_MAX) {
            vm.trending.push(track_vm(t, liked.contains(&remote_id(t.id.as_str())), ""));
            sink.push(t.clone());
        }
    }
    if vm.new_releases.is_empty() && vm.moods.is_empty() && vm.trending.is_empty() {
        vm.state = LoadStateVm::Empty;
        vm.error_text = "O YouTube Music não trouxe novidades agora.".into();
    }
    vm
}

fn mood_highlight(index: usize, c: &Category) -> HighlightVm {
    let tint = c.color.map(|argb| [(argb >> 16) as u8, (argb >> 8) as u8, argb as u8]).unwrap_or([0x2f, 0x1d, 0x10]);
    HighlightVm { id: format!("mood:{index}"), kind: "mood".into(), title: c.title.clone(), subtitle: c.group.clone().unwrap_or_default(), tint: Some(tint), ..Default::default() }
}

/// Resultados de busca por tipo. `kind`: 0 tudo, 1 músicas, 2 álbuns, 3 artistas, 4 playlists.
pub fn build_search(items: &[Item], kind: i32, liked: &HashSet<String>, sink: &mut TrackSink) -> SearchVm {
    let mut vm = SearchVm::default();
    for item in items {
        match item {
            Item::Track(t) => {
                vm.songs.push(track_vm(t, liked.contains(&remote_id(t.id.as_str())), ""));
                sink.push(t.clone());
            }
            Item::Album(a) => vm.albums.push(album_vm(a)),
            Item::Artist(a) => vm.artists.push(artist_vm(a)),
            Item::Playlist(_) => vm.playlists.push(highlight(item, "")),
        }
    }
    if kind == 0 {
        vm.best = items.first().map(|i| highlight(i, "Melhor resultado"));
    }
    vm.state = if items.is_empty() { SearchStateVm::Empty } else { SearchStateVm::Results };
    vm
}

fn year_label(year: Option<i32>) -> String {
    year.map(|y| y.to_string()).unwrap_or_default()
}

fn join_nonempty(parts: &[String]) -> String {
    parts.iter().filter(|p| !p.is_empty()).cloned().collect::<Vec<_>>().join(", ")
}

fn related_items(shelves: &[Shelf]) -> Vec<HighlightVm> {
    shelves.iter().flat_map(|s| s.items.iter()).filter(|i| matches!(i, Item::Album(_) | Item::Playlist(_) | Item::Artist(_))).take(12).map(|i| highlight(i, "Continue explorando")).collect()
}

pub fn build_album(page: &AlbumPage, liked: &HashSet<String>, saved: bool, sink: &mut TrackSink) -> DetailVm {
    let tracks: Vec<TrackVm> = page.tracks.iter().map(|t| track_vm(t, liked.contains(&remote_id(t.id.as_str())), "")).collect();
    sink.extend(page.tracks.iter().cloned());
    let subtitle = join_nonempty(&[artist_names(&page.album.artists), year_label(page.album.year), tidy_summary(page.summary.as_deref().unwrap_or(""))]);
    DetailVm {
        kind: DetailKindVm::Tracks,
        state: if tracks.is_empty() { LoadStateVm::Empty } else { LoadStateVm::Ready },
        title: page.album.title.clone(),
        description: page.description.clone().unwrap_or_default(),
        related: related_items(&page.related),
        subtitle,
        cover_url: cover_url(&page.album.cover),
        source: SOURCE_LABEL.to_owned(),
        empty_text: "Este álbum não tem músicas disponíveis agora.".into(),
        saved,
        can_save: true,
        tracks,
        ..Default::default()
    }
}

pub fn build_playlist(page: &PlaylistPage, liked: &HashSet<String>, sink: &mut TrackSink) -> DetailVm {
    let tracks: Vec<TrackVm> = page.tracks.iter().map(|t| track_vm(t, liked.contains(&remote_id(t.id.as_str())), "")).collect();
    sink.extend(page.tracks.iter().cloned());
    DetailVm {
        kind: DetailKindVm::Tracks,
        state: if tracks.is_empty() { LoadStateVm::Empty } else { LoadStateVm::Ready },
        title: page.playlist.name.clone(),
        description: page.playlist.description.clone().unwrap_or_default(),
        related: related_items(&page.related),
        subtitle: join_nonempty(&[page.author.clone().unwrap_or_default(), tidy_summary(page.summary.as_deref().unwrap_or(""))]),
        cover_url: cover_url(&page.playlist.cover),
        source: SOURCE_LABEL.to_owned(),
        empty_text: "Esta playlist não tem músicas disponíveis agora.".into(),
        tracks,
        ..Default::default()
    }
}

pub fn build_artist(page: &ArtistPage, liked: &HashSet<String>, sink: &mut TrackSink) -> DetailVm {
    let tracks: Vec<TrackVm> = page.top_tracks.iter().map(|t| track_vm(t, liked.contains(&remote_id(t.id.as_str())), "")).collect();
    sink.extend(page.top_tracks.iter().cloned());
    let albums: Vec<AlbumVm> = page
        .shelves
        .iter()
        .flat_map(|s| s.items.iter())
        .filter_map(|i| if let Item::Album(a) = i { Some(album_vm(a)) } else { None })
        .take(ALBUMS_MAX * 2)
        .collect();
    DetailVm {
        kind: DetailKindVm::Artist,
        state: if tracks.is_empty() && albums.is_empty() { LoadStateVm::Empty } else { LoadStateVm::Ready },
        title: page.artist.name.clone(),
        description: page.description.clone().unwrap_or_default(),
        related: page.shelves.iter().flat_map(|s| s.items.iter()).filter(|i| matches!(i, Item::Artist(_))).take(12).map(|i| highlight(i, "Artistas relacionados")).collect(),
        subtitle: page.audience.clone().unwrap_or_default(),
        cover_url: cover_url(&page.artist.image),
        source: SOURCE_LABEL.to_owned(),
        empty_text: "Não há músicas deste artista disponíveis agora.".into(),
        tracks,
        albums,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------------------------
// Erros
// ---------------------------------------------------------------------------------------------

/// Texto para o usuário e se faz sentido oferecer "Tentar de novo". Tom: calmo, sem culpa (UX_PLAN §7.3).
pub fn error_copy(kind: ErrorKind) -> (&'static str, &'static str, bool) {
    match kind {
        ErrorKind::Network | ErrorKind::Offline => ("Sem conexão", "Não deu para falar com o YouTube Music. Suas músicas do aparelho continuam tocando.", true),
        ErrorKind::Blocked | ErrorKind::RateLimited | ErrorKind::NoStream => {
            ("O YouTube Music não respondeu", "Sua conexão parece ok. O serviço pode estar fora do ar. Tente de novo em instantes.", true)
        }
        ErrorKind::RegionBlocked => ("Indisponível no seu país", "Esta faixa não está disponível na sua região.", false),
        ErrorKind::AgeRestricted => ("Exige conta", "Esta faixa exige uma conta com idade verificada, e o Kazzu não usa conta.", false),
        ErrorKind::Unavailable | ErrorKind::NotFound => ("Indisponível", "Esta faixa não está mais disponível.", false),
        ErrorKind::UpstreamChanged => ("O YouTube Music mudou", "Atualize o Kazzu. As músicas do aparelho seguem funcionando.", false),
        ErrorKind::Storage => ("Não deu para guardar", "O Kazzu não conseguiu gravar no aparelho.", false),
    }
}

/// "network", "blocked"...: o texto pequeno em fonte mono para quem precisa relatar.
pub fn error_code(kind: ErrorKind) -> String {
    format!("ytm:{}", kind.code())
}

/// Erros de rede viram o estado "offline" das telas.
pub fn is_offline(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::Network | ErrorKind::Offline)
}

/// Resumo da faixa para o selo do player, a partir do formato do stream.
pub fn source_of_track(t: &Track) -> &'static str {
    match t.source {
        TrackSource::Local { .. } => "Neste aparelho",
        TrackSource::Remote { provider: ProviderKind::YtMusic, .. } => SOURCE_LABEL,
        TrackSource::Remote { .. } => "De um provedor",
    }
}

/// Codec legível ("OPUS") de uma faixa já resolvida.
pub fn codec_of(t: &Track) -> Option<Codec> {
    t.format.as_ref().map(|f| f.codec.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazzu_core::{AlbumId, AlbumRef, ArtistId, ArtistRef, PlaylistId};

    fn t(id: &str, title: &str) -> Track {
        let mut t = Track::new(id, title, TrackSource::Remote { provider: ProviderKind::YtMusic, id: id.into() });
        t.artists = vec![ArtistRef { id: Some(ArtistId::new("UC1")), name: "Nina Sabiá".into() }];
        t.album = Some(AlbumRef { id: Some(AlbumId::new("MPREb_1")), title: "Calor".into() });
        t.cover = Some(CoverRef::Url { url: format!("https://img/{id}") });
        t.duration = Some(std::time::Duration::from_secs(201));
        t
    }

    fn album(id: &str) -> Album {
        Album { id: AlbumId::new(id), provider: ProviderKind::YtMusic, title: format!("Álbum {id}"), artists: vec![ArtistRef::named("Aurora")], year: Some(2024), track_count: None, cover: Some(CoverRef::Url { url: format!("https://img/{id}") }) }
    }

    fn playlist(id: &str) -> Playlist {
        Playlist { id: PlaylistId::new(id), provider: ProviderKind::YtMusic, name: format!("Mix {id}"), description: None, cover: None, tracks: vec![], editable: false }
    }

    fn feed() -> Vec<Shelf> {
        vec![
            Shelf { title: "Novos lançamentos".into(), subtitle: None, items: vec![Item::Album(album("a1")), Item::Album(album("a2"))], more: None },
            Shelf { title: "Mixes".into(), subtitle: None, items: vec![Item::Playlist(playlist("p1")), Item::Playlist(playlist("p2"))], more: None },
            Shelf { title: "Em alta".into(), subtitle: None, items: (0..5).map(|i| Item::Track(t(&format!("v{i}"), &format!("Faixa {i}")))).collect(), more: None },
        ]
    }

    #[test]
    fn home_is_built_from_shelves_with_the_why_and_prefixed_ids() {
        let mut sink = vec![];
        let vm = build_home(&feed(), HomeSectionVm::ForYou, &HashSet::new(), &mut sink);
        assert_eq!(vm.state, LoadStateVm::Ready);
        assert_eq!(vm.highlights.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(), ["yt:a1", "yt:a2", "yt:p1", "yt:p2"]);
        assert_eq!(vm.quick_picks.len(), 5);
        assert_eq!(vm.quick_picks[0].why, "Em alta");
        assert_eq!(vm.quick_picks[0].id, "yt:v0");
        assert_eq!(vm.quick_picks[0].cover_url.as_deref(), Some("https://img/v0"));
        assert_eq!(vm.source_label, "Do YouTube Music");
        assert_eq!(sink.len(), 5);
        // nada repetido: o que virou destaque não volta em mixes/álbuns
        assert!(vm.mixes.is_empty() && vm.albums.is_empty());
    }

    #[test]
    fn home_sections_filter_the_feed() {
        let mut sink = vec![];
        let d = build_home(&feed(), HomeSectionVm::Discover, &HashSet::new(), &mut sink);
        assert!(d.highlights.iter().all(|h| h.kind == "album"));
        let m = build_home(&feed(), HomeSectionVm::Mixes, &HashSet::new(), &mut sink);
        assert!(m.highlights.iter().all(|h| h.kind == "playlist") && m.quick_picks.is_empty());
    }

    #[test]
    fn empty_feed_is_an_empty_state_not_a_blank_screen() {
        let vm = build_home(&[], HomeSectionVm::ForYou, &HashSet::new(), &mut vec![]);
        assert_eq!(vm.state, LoadStateVm::Empty);
        assert!(!vm.error_text.is_empty());
    }

    #[test]
    fn liked_tracks_are_flagged() {
        let liked: HashSet<String> = ["yt:v1".to_owned()].into();
        let vm = build_home(&feed(), HomeSectionVm::ForYou, &liked, &mut vec![]);
        assert!(vm.quick_picks[1].liked && !vm.quick_picks[0].liked);
    }

    #[test]
    fn search_groups_by_kind_and_picks_a_best_result() {
        let items = vec![
            Item::Artist(Artist { id: ArtistId::new("UC1"), provider: ProviderKind::YtMusic, name: "Nina Sabiá".into(), image: None, genres: Vec::new(), popularity: 0 }),
            Item::Track(t("v1", "Quase Verão")),
            Item::Album(album("a1")),
            Item::Playlist(playlist("p1")),
        ];
        let vm = build_search(&items, 0, &HashSet::new(), &mut vec![]);
        assert_eq!(vm.state, SearchStateVm::Results);
        assert_eq!(vm.best.as_ref().map(|b| b.kind.as_str()), Some("artist"));
        assert_eq!((vm.songs.len(), vm.albums.len(), vm.artists.len(), vm.playlists.len()), (1, 1, 1, 1));
        assert!(build_search(&items, 1, &HashSet::new(), &mut vec![]).best.is_none());
        assert_eq!(build_search(&[], 0, &HashSet::new(), &mut vec![]).state, SearchStateVm::Empty);
    }

    #[test]
    fn album_detail_has_subtitle_without_mid_dots() {
        let page = AlbumPage { album: album("a1"), tracks: vec![t("v1", "A")], description: Some("Sobre este álbum".into()), summary: Some("1 música • 3 minutos".into()), album_type: None, audio_playlist_id: None, related: vec![Shelf { title: "Relacionados".into(), subtitle: None, items: vec![Item::Album(album("a2"))], more: None }] };
        let vm = build_album(&page, &HashSet::new(), false, &mut vec![]);
        assert_eq!(vm.subtitle, "Aurora, 2024, 1 música, 3 minutos");
        assert!(vm.can_save && !vm.subtitle.contains('•'));
        assert_eq!(vm.description, "Sobre este álbum");
        assert_eq!(vm.related[0].id, "yt:a2");
    }

    #[test]
    fn mood_tiles_carry_the_category_color() {
        let c = Category { title: "Foco".into(), group: None, browse: kazzu_ytm::BrowseRef { browse_id: "x".into(), params: None }, color: Some(0xFF11_2233) };
        let h = mood_highlight(3, &c);
        assert_eq!((h.id.as_str(), h.tint), ("mood:3", Some([0x11, 0x22, 0x33])));
    }

    #[test]
    fn every_error_kind_has_calm_copy_and_a_retry_rule() {
        for kind in [ErrorKind::Network, ErrorKind::Blocked, ErrorKind::RateLimited, ErrorKind::RegionBlocked, ErrorKind::AgeRestricted, ErrorKind::Unavailable, ErrorKind::NotFound, ErrorKind::NoStream, ErrorKind::UpstreamChanged, ErrorKind::Offline, ErrorKind::Storage] {
            let (title, text, retry) = error_copy(kind);
            assert!(!title.is_empty() && !text.is_empty() && !text.contains('!'), "{kind:?}");
            assert_eq!(retry, kind.is_retryable() || kind == ErrorKind::Network, "{kind:?}");
        }
        assert!(is_offline(ErrorKind::Network) && !is_offline(ErrorKind::Blocked));
    }

    #[test]
    fn remote_ids_round_trip() {
        assert_eq!(strip_remote(&remote_id("abc")), Some("abc"));
        assert_eq!(strip_remote("12"), None);
        assert!(is_remote_id("yt:x") && !is_remote_id("42"));
    }
}
