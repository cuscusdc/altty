//! View-models: dados prontos para a UI, sem tipos do Slint (podem cruzar threads).
//! A conversão para `slint::Image`/structs do Slint acontece só na thread de UI (`slint_view`).

use crate::accent::Accent;
use crate::art::Pixels;
use crate::config::{AccentPref, ThemePref};

#[derive(Clone, Default)]
pub struct TrackVm {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: Option<Pixels>,
    pub duration_label: String,
    pub codec: String,
    pub detail: String,
    pub detail_long: String,
    pub hires: bool,
    /// "" = local; "YT Music", "Spotify"...
    pub source: String,
    pub liked: bool,
    pub playing: bool,
}

#[derive(Clone, Default)]
pub struct AlbumVm {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover: Option<Pixels>,
}

#[derive(Clone, Default)]
pub struct HighlightVm {
    pub id: String,
    pub caption: String,
    pub title: String,
    pub subtitle: String,
    pub cover: Option<Pixels>,
}

#[derive(Clone, Default)]
pub struct PlaylistVm {
    pub id: String,
    pub name: String,
    pub count_label: String,
    pub covers: Vec<Pixels>,
}

#[derive(Clone, Default)]
pub struct CollectionVm {
    /// "downloads" | "liked" | "albums" | "playlists"
    pub id: String,
    pub title: String,
    pub count_label: String,
    pub covers: Vec<Pixels>,
}

#[derive(Clone, Default)]
pub struct HomeVm {
    pub highlights: Vec<HighlightVm>,
    pub recents: Vec<TrackVm>,
    pub albums: Vec<AlbumVm>,
    pub moods: Vec<HighlightVm>,
}

#[derive(Clone, Default)]
pub struct LibraryVm {
    pub collections: Vec<CollectionVm>,
    pub playlists: Vec<PlaylistVm>,
}

#[derive(Clone, Default)]
pub struct SettingsVm {
    pub folders: Vec<String>,
    pub summary: String,
    pub has_library: bool,
    pub track_count: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailKindVm {
    #[default]
    Tracks,
    Albums,
}

#[derive(Clone, Default)]
pub struct DetailVm {
    pub kind: DetailKindVm,
    pub title: String,
    pub subtitle: String,
    pub empty_text: String,
    pub tracks: Vec<TrackVm>,
    pub albums: Vec<AlbumVm>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum RepeatVm {
    #[default]
    Off,
    All,
    One,
}

#[derive(Clone, Default)]
pub struct PlaybackVm {
    pub has_now: bool,
    pub now: Option<TrackVm>,
    pub playing: bool,
    pub loading: bool,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: RepeatVm,
    pub output_name: String,
    pub output_note: String,
    pub bit_perfect: bool,
    pub source_label: String,
    pub error: String,
}

#[derive(Clone)]
pub struct NowArtVm {
    /// Capa grande (≤ 640 px) do player.
    pub cover: Option<Pixels>,
    /// Miniatura (≤ 192 px) do mini player, da NowBar e da fila.
    pub thumb: Option<Pixels>,
    pub backdrop: Option<Pixels>,
    pub accent: Accent,
}

impl Default for NowArtVm {
    fn default() -> Self {
        Self { cover: None, thumb: None, backdrop: None, accent: Accent::BRAND }
    }
}

#[derive(Clone, Default)]
pub struct QueueVm {
    pub tracks: Vec<TrackVm>,
    pub summary: String,
    /// "e mais 1.900 músicas" quando a fila é maior que a parte mostrada.
    pub more_label: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SearchStateVm {
    #[default]
    Idle,
    Results,
    Empty,
}

#[derive(Clone, Default)]
pub struct SearchVm {
    pub state: SearchStateVm,
    pub songs: Vec<TrackVm>,
    pub albums: Vec<AlbumVm>,
    pub recent: Vec<String>,
}

#[derive(Clone, Default)]
pub struct ScanVm {
    /// 0..1, ou negativo = indeterminado.
    pub progress: f32,
    pub label: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PrefsVm {
    pub theme: ThemePref,
    pub accent: AccentPref,
    pub reduce_motion: bool,
    pub reduce_transparency: bool,
}

#[derive(Clone, Default)]
pub struct ProviderVm {
    pub id: String,
    pub name: String,
    pub status_label: String,
    pub connected: bool,
    pub available: bool,
}
