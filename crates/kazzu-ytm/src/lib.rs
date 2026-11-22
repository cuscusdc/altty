//! kazzu-ytm: YouTube Music provider for Kazzu — no account, no setup.
//!
//! - Metadata (home, charts, new releases, moods, search, suggestions,
//!   album/artist/playlist pages, radio, lyrics) through the anonymous
//!   `WEB_REMIX` innertube client of music.youtube.com.
//! - Audio streams through innertube player clients that return direct
//!   googlevideo URLs without signature cipher or PO token (`VISIONOS`, then
//!   `IOS_PRODUCER`), best format first (Opus/WebM, else AAC/M4A).
//! - Disk cache of metadata with TTL and stale fallback (offline mode).
//! - Local library (likes, saved albums/artists/playlists) on this device.
//!
//! Pure Rust, no yt-dlp, no JS engine, no tokio: blocking HTTP (ureq +
//! rustls) on a small thread pool behind async methods, usable from any
//! executor (`pollster`, tokio, smol...).
//!
//! No network call happens unless a method is called (no telemetry, no
//! prefetch on construction). See `docs/research/ytm-notes.md`.

pub mod cache;
mod client;
pub mod error;
mod innertube;
mod json;
pub mod library;
pub mod locale;
pub mod model;
mod parse;
mod stream;
pub mod transport;

use std::sync::Arc;

use async_trait::async_trait;
use kazzu_core::{
    AlbumId, LibrarySnapshot, PlaylistId, Provider, ProviderCapabilities, ProviderKind,
    SearchQuery, SearchResults, StreamQuality, StreamSource, Track, TrackId, TrackSource,
};

pub use cache::{DiskCache, Ttl};
pub use client::{Config, SearchFilter, YtClient};
pub use error::{ErrorKind, Result, YtmError};
pub use innertube::{DEFAULT_PLAYER_CLIENTS, PlayerClient};
pub use library::LocalLibrary;
pub use locale::Locale;
pub use model::{
    AlbumPage, ArtistPage, BrowseRef, Category, Discover, Item, Lyrics, PlaylistPage, Radio,
    ResolvedStream, Shelf, ShelfPage, Suggestions,
};
pub use transport::{HttpRequest, HttpResponse, Method, Transport, UreqTransport};

/// Upper bound of tracks fetched by `Provider::playlist_tracks`.
pub const MAX_PLAYLIST_TRACKS: usize = 2000;

/// Async YouTube Music provider. Cheap to clone; implements
/// [`kazzu_core::Provider`] and offers the richer YouTube-specific API.
#[derive(Clone)]
pub struct YtMusic {
    client: YtClient,
}

macro_rules! unblock {
    ($self:ident, |$c:ident| $body:expr) => {{
        let $c = $self.client.clone();
        blocking::unblock(move || $body).await
    }};
}

impl YtMusic {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            client: YtClient::new(config)?,
        })
    }

    /// With a custom transport (tests, proxies).
    pub fn with_transport(config: Config, transport: Arc<dyn Transport>) -> Result<Self> {
        Ok(Self {
            client: YtClient::with_transport(config, transport)?,
        })
    }

    /// The synchronous client (same state), for code that already runs on
    /// a worker thread.
    pub fn client(&self) -> &YtClient {
        &self.client
    }

    pub fn library_store(&self) -> &LocalLibrary {
        self.client.library()
    }

    pub async fn discover(&self) -> Result<Discover> {
        unblock!(self, |c| c.discover())
    }

    pub async fn home(&self) -> Result<ShelfPage> {
        unblock!(self, |c| c.home())
    }

    pub async fn page_continuation(&self, token: String) -> Result<ShelfPage> {
        unblock!(self, |c| c.page_continuation(&token))
    }

    pub async fn charts(&self, country: Option<String>) -> Result<ShelfPage> {
        unblock!(self, |c| c.charts(country.as_deref()))
    }

    pub async fn new_releases(&self) -> Result<ShelfPage> {
        unblock!(self, |c| c.new_releases())
    }

    pub async fn categories(&self) -> Result<Vec<Category>> {
        unblock!(self, |c| c.categories())
    }

    pub async fn browse(&self, target: BrowseRef) -> Result<ShelfPage> {
        unblock!(self, |c| c.browse(&target))
    }

    pub async fn search_items(&self, text: String, filter: Option<SearchFilter>, limit: usize) -> Result<Vec<Item>> {
        unblock!(self, |c| c.search_items(&text, filter, limit))
    }

    pub async fn suggestions(&self, prefix: String) -> Result<Suggestions> {
        unblock!(self, |c| c.suggestions(&prefix))
    }

    pub async fn album(&self, id: String) -> Result<AlbumPage> {
        unblock!(self, |c| c.album(&id))
    }

    pub async fn artist(&self, id: String) -> Result<ArtistPage> {
        unblock!(self, |c| c.artist(&id))
    }

    pub async fn playlist(&self, id: String) -> Result<PlaylistPage> {
        unblock!(self, |c| c.playlist(&id))
    }

    pub async fn playlist_continuation(&self, token: String) -> Result<(Vec<Track>, Option<String>)> {
        unblock!(self, |c| c.playlist_continuation(&token))
    }

    pub async fn radio(&self, video_id: String) -> Result<Radio> {
        unblock!(self, |c| c.radio(&video_id))
    }

    pub async fn radio_continuation(&self, radio: Radio) -> Result<Radio> {
        unblock!(self, |c| c.radio_continuation(&radio))
    }

    pub async fn lyrics(&self, video_id: String) -> Result<Option<Lyrics>> {
        unblock!(self, |c| c.lyrics(&video_id))
    }

    pub async fn resolve(&self, video_id: String, quality: StreamQuality) -> Result<ResolvedStream> {
        unblock!(self, |c| c.resolve(&video_id, quality))
    }

    pub async fn prefetch(&self, video_id: String, quality: StreamQuality) -> Result<()> {
        unblock!(self, |c| c.prefetch(&video_id, quality))
    }

    pub fn invalidate_stream(&self, video_id: &str) {
        self.client.invalidate_stream(video_id);
    }
}

/// Video id of a YouTube Music track.
pub fn video_id_of(track: &Track) -> kazzu_core::Result<&str> {
    match &track.source {
        TrackSource::Remote {
            provider: ProviderKind::YtMusic,
            id,
        } => Ok(id),
        TrackSource::Remote { provider, .. } => Err(kazzu_core::Error::InvalidArgument(format!(
            "track of {provider} given to the YouTube Music provider"
        ))),
        TrackSource::Local { .. } => Err(kazzu_core::Error::InvalidArgument(
            "local track given to the YouTube Music provider".into(),
        )),
    }
}

#[async_trait]
impl Provider for YtMusic {
    fn kind(&self) -> ProviderKind {
        ProviderKind::YtMusic
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            stream: true,
            download: false,
            // Local to Kazzu: no YouTube account involved.
            library: true,
            edit_playlists: false,
            like: true,
            recommendations: true,
        }
    }

    async fn search(&self, query: &SearchQuery) -> kazzu_core::Result<SearchResults> {
        let query = query.clone();
        Ok(unblock!(self, |c| c.search(&query))?)
    }

    async fn resolve_stream(&self, track: &Track, quality: StreamQuality) -> kazzu_core::Result<StreamSource> {
        let id = video_id_of(track)?.to_owned();
        Ok(self.resolve(id, quality).await?.to_stream_source())
    }

    async fn library(&self) -> kazzu_core::Result<LibrarySnapshot> {
        Ok(self.client.library().snapshot())
    }

    async fn track(&self, id: &TrackId) -> kazzu_core::Result<Track> {
        let id = id.as_str().to_owned();
        Ok(unblock!(self, |c| c.track(&id))?)
    }

    async fn album_tracks(&self, id: &AlbumId) -> kazzu_core::Result<Vec<Track>> {
        Ok(self.album(id.as_str().to_owned()).await?.tracks)
    }

    async fn playlist_tracks(&self, id: &PlaylistId) -> kazzu_core::Result<Vec<Track>> {
        let id = id.as_str().to_owned();
        Ok(unblock!(self, |c| c.playlist_tracks(&id, MAX_PLAYLIST_TRACKS))?)
    }
}
