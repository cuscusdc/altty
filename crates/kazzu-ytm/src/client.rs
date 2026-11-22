//! [`YtClient`]: the synchronous YouTube Music client (all the logic).
//! [`crate::YtMusic`] wraps it into async methods and the `Provider` trait.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

use kazzu_core::{SearchKind, SearchQuery, SearchResults, StreamQuality, Track};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::cache::{DiskCache, Ttl};
use crate::error::{ErrorKind, Result, YtmError};
use crate::innertube::{
    self, DEFAULT_PLAYER_CLIENTS, MUSIC_API, PLAYER_API, PlayerClient, SW_JS_DATA, WEB_REMIX_ID,
    WEB_REMIX_VERSION, WEB_UA,
};
use crate::library::LocalLibrary;
use crate::locale::Locale;
use crate::model::{
    AlbumPage, ArtistPage, BrowseRef, Category, Discover, Item, Lyrics, PlaylistPage, Radio,
    ResolvedStream, Shelf, ShelfPage, Suggestions,
};
use crate::parse::{self, Ctx};
use crate::stream;
use crate::transport::{HttpRequest, HttpResponse, Transport, UreqTransport};

/// Search filters of YouTube Music (the chips under the search box).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchFilter {
    Songs,
    Videos,
    Albums,
    Artists,
    /// Playlists curated by YouTube Music.
    FeaturedPlaylists,
    /// Playlists made by users.
    CommunityPlaylists,
}

impl SearchFilter {
    /// `params` of the search request (protobuf, base64; stable for years).
    fn params(self) -> &'static str {
        match self {
            SearchFilter::Songs => "EgWKAQIIAWoSEAUQCRADEAQQChAQEBUQDhAR",
            SearchFilter::Videos => "EgWKAQIQAWoSEAUQCRADEAQQChAQEBUQDhAR",
            SearchFilter::Albums => "EgWKAQIYAWoSEAUQCRADEAQQChAQEBUQDhAR",
            SearchFilter::Artists => "EgWKAQIgAWoSEAUQCRADEAQQChAQEBUQDhAR",
            SearchFilter::FeaturedPlaylists => "EgeKAQQoADgBahIQBRAJEAMQBBAKEBAQFRAOEBE%3D",
            SearchFilter::CommunityPlaylists => "EgeKAQQoAEABahIQBRAJEAMQBBAKEBAQFRAOEBE%3D",
        }
    }
}

/// Configuration of the client.
#[derive(Debug, Clone)]
pub struct Config {
    /// Language/country sent to YouTube. Default: the system locale.
    pub locale: Locale,
    /// Metadata cache directory (`None` = no disk cache).
    pub cache_dir: Option<PathBuf>,
    /// Local library file (`None` = in memory only).
    pub library_path: Option<PathBuf>,
    /// Player clients tried in order to resolve streams.
    pub player_clients: Vec<PlayerClient>,
    /// Probe the end of each resolved stream (1 KiB) to detect URLs that
    /// googlevideo would cut after the first megabyte. Recommended.
    pub verify_streams: bool,
    /// Start in offline mode (cache only).
    pub offline: bool,
}

impl Config {
    /// Defaults with an explicit data directory (what Android/iOS pass in):
    /// cache in `data_dir/cache/ytm`, library in `data_dir/ytm-library.json`.
    pub fn with_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        let dir = data_dir.into();
        Self {
            cache_dir: Some(dir.join("cache").join("ytm")),
            library_path: Some(dir.join("ytm-library.json")),
            ..Self::ephemeral()
        }
    }

    /// Defaults for desktop: XDG / Known Folder dirs of `Kazzu`.
    pub fn desktop() -> Self {
        match directories::ProjectDirs::from("app", "kazzu", "Kazzu") {
            Some(dirs) => Self {
                cache_dir: Some(dirs.cache_dir().join("ytm")),
                library_path: Some(dirs.data_dir().join("ytm-library.json")),
                ..Self::ephemeral()
            },
            None => Self::ephemeral(),
        }
    }

    /// No disk cache, in-memory library (tests, previews).
    pub fn ephemeral() -> Self {
        Self {
            locale: Locale::system(),
            cache_dir: None,
            library_path: None,
            player_clients: player_clients_from_env().unwrap_or_else(|| DEFAULT_PLAYER_CLIENTS.to_vec()),
            verify_streams: true,
            offline: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::desktop()
    }
}

/// `KAZZU_YTM_CLIENTS=visionos,ios_producer` overrides the client chain
/// (field mitigation without a rebuild).
fn player_clients_from_env() -> Option<Vec<PlayerClient>> {
    let raw = std::env::var("KAZZU_YTM_CLIENTS").ok()?;
    let list: Vec<PlayerClient> = raw.split(',').filter_map(PlayerClient::parse).collect();
    (!list.is_empty()).then_some(list)
}

#[derive(Debug)]
struct Session {
    visitor_data: Option<String>,
    web_remix_version: String,
    version_refreshed: bool,
    /// Clients whose URLs googlevideo refused during this session.
    gated: HashSet<PlayerClient>,
}

fn quality_key(q: StreamQuality) -> u8 {
    match q {
        StreamQuality::Original => 0,
        StreamQuality::Lossless => 1,
        StreamQuality::High => 2,
        StreamQuality::Low => 3,
    }
}

struct Inner {
    locale: RwLock<Locale>,
    player_clients: Vec<PlayerClient>,
    verify_streams: bool,
    transport: Arc<dyn Transport>,
    cache: DiskCache,
    library: LocalLibrary,
    session: Mutex<Session>,
    prefetched: Mutex<HashMap<(String, u8), ResolvedStream>>,
    offline: AtomicBool,
    last_request: Mutex<Option<Instant>>,
}

/// Synchronous YouTube Music client. Cheap to clone (shared state).
#[derive(Clone)]
pub struct YtClient {
    inner: Arc<Inner>,
}

/// Minimum gap between two requests (politeness; ~10 req/s at most).
const MIN_GAP: Duration = Duration::from_millis(100);
/// Prefetched streams are reused only while they have this much time left.
const STREAM_MARGIN: Duration = Duration::from_secs(20 * 60);

impl YtClient {
    pub fn new(config: Config) -> Result<Self> {
        Self::with_transport(config, Arc::new(UreqTransport::new()))
    }

    pub fn with_transport(config: Config, transport: Arc<dyn Transport>) -> Result<Self> {
        let library = match &config.library_path {
            Some(p) => LocalLibrary::open(p)?,
            None => LocalLibrary::in_memory(),
        };
        let cache = config
            .cache_dir
            .as_ref()
            .map(DiskCache::new)
            .unwrap_or_else(DiskCache::disabled);
        cache.prune(crate::cache::MAX_AGE);
        Ok(Self {
            inner: Arc::new(Inner {
                locale: RwLock::new(config.locale),
                player_clients: config.player_clients,
                verify_streams: config.verify_streams,
                transport,
                cache,
                library,
                session: Mutex::new(Session {
                    visitor_data: None,
                    web_remix_version: WEB_REMIX_VERSION.to_owned(),
                    version_refreshed: false,
                    gated: HashSet::new(),
                }),
                prefetched: Mutex::new(HashMap::new()),
                offline: AtomicBool::new(config.offline),
                last_request: Mutex::new(None),
            }),
        })
    }

    // ------------------------------------------------------------ settings

    pub fn locale(&self) -> Locale {
        self.inner.locale.read().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Changes language/country (cached pages are per locale).
    pub fn set_locale(&self, locale: Locale) {
        *self.inner.locale.write().unwrap_or_else(|p| p.into_inner()) = locale;
    }

    /// Offline mode: no network at all; cached pages of any age are served.
    pub fn set_offline(&self, offline: bool) {
        self.inner.offline.store(offline, Ordering::Relaxed);
    }

    pub fn is_offline(&self) -> bool {
        self.inner.offline.load(Ordering::Relaxed)
    }

    pub fn library(&self) -> &LocalLibrary {
        &self.inner.library
    }

    pub fn cache(&self) -> &DiskCache {
        &self.inner.cache
    }

    fn session(&self) -> std::sync::MutexGuard<'_, Session> {
        self.inner.session.lock().unwrap_or_else(|p| p.into_inner())
    }

    // ------------------------------------------------------------ transport

    fn throttle(&self) {
        let mut last = self.inner.last_request.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(t) = *last {
            let since = t.elapsed();
            if since < MIN_GAP {
                std::thread::sleep(MIN_GAP - since);
            }
        }
        *last = Some(Instant::now());
    }

    fn send(&self, req: &HttpRequest, retry: bool) -> Result<HttpResponse> {
        if self.is_offline() {
            return Err(YtmError::Offline(req.url.clone()));
        }
        self.throttle();
        match self.inner.transport.send(req) {
            Err(e) if retry && e.kind() == ErrorKind::Network => {
                log::debug!("ytm: retrying after network error: {e}");
                std::thread::sleep(Duration::from_millis(400));
                self.throttle();
                self.inner.transport.send(req)
            }
            Ok(r) if retry && r.status >= 500 => {
                std::thread::sleep(Duration::from_millis(400));
                self.throttle();
                self.inner.transport.send(req)
            }
            other => other,
        }
    }

    fn json_of(resp: &HttpResponse, what: &str) -> Result<Value> {
        if !(200..300).contains(&resp.status) {
            return Err(YtmError::from_status(resp.status, what, resp.retry_after));
        }
        serde_json::from_str(&resp.body).map_err(|e| YtmError::UpstreamChanged(format!("{what}: invalid JSON ({e})")))
    }

    /// POST to a music.youtube.com innertube endpoint as WEB_REMIX.
    fn music_call(&self, endpoint: &str, body: Value) -> Result<Value> {
        let locale = self.locale();
        let mut refreshed = false;
        loop {
            let version = self.session().web_remix_version.clone();
            let req = HttpRequest::post_json(
                format!("{MUSIC_API}/{endpoint}?prettyPrint=false"),
                innertube::web_remix_body(&version, &locale, body.clone()).to_string(),
            )
            .header("User-Agent", WEB_UA)
            .header("Origin", "https://music.youtube.com")
            .header("X-Youtube-Client-Name", WEB_REMIX_ID.to_string())
            .header("X-Youtube-Client-Version", version);
            let resp = self.send(&req, true)?;
            match Self::json_of(&resp, endpoint) {
                Err(e) if e.kind() == ErrorKind::UpstreamChanged && !refreshed && self.refresh_version() => {
                    refreshed = true;
                }
                other => return other,
            }
        }
    }

    /// Fetches `sw.js_data` (anonymous visitor token + current WEB_REMIX
    /// version). One small request per session.
    fn bootstrap(&self) -> Result<innertube::Bootstrap> {
        let req = HttpRequest::get(SW_JS_DATA).header("User-Agent", WEB_UA);
        let resp = self.send(&req, true)?;
        if resp.status != 200 {
            return Err(YtmError::from_status(resp.status, "sw.js_data", resp.retry_after));
        }
        Ok(innertube::parse_bootstrap(&resp.body))
    }

    /// Updates the WEB_REMIX version from `sw.js_data` (once per session).
    /// Returns whether a different version was found.
    fn refresh_version(&self) -> bool {
        if std::mem::replace(&mut self.session().version_refreshed, true) {
            return false;
        }
        let Ok(boot) = self.bootstrap() else { return false };
        let mut s = self.session();
        if s.visitor_data.is_none() {
            s.visitor_data = boot.visitor_data;
        }
        match boot.web_remix_version {
            Some(v) if v != s.web_remix_version => {
                log::warn!("ytm: WEB_REMIX version updated {} -> {v}", s.web_remix_version);
                s.web_remix_version = v;
                true
            }
            _ => false,
        }
    }

    /// Anonymous visitor token required by the player endpoint. Kept in
    /// memory only; a new one per app session.
    fn visitor_data(&self) -> Result<String> {
        if let Some(v) = self.session().visitor_data.clone() {
            return Ok(v);
        }
        let boot = self.bootstrap()?;
        let visitor = match boot.visitor_data {
            Some(v) => v,
            None => {
                // Fallback: any WEB_REMIX response carries one.
                let resp = self.music_call("browse", json!({"browseId": "FEmusic_moods_and_genres"}))?;
                crate::json::str_at(&resp, &["responseContext", "visitorData"])
                    .filter(|s| innertube::looks_like_visitor_data(s))
                    .map(str::to_owned)
                    .ok_or_else(|| YtmError::shape("visitorData"))?
            }
        };
        let mut s = self.session();
        if let Some(v) = boot.web_remix_version {
            s.web_remix_version = v;
        }
        s.visitor_data = Some(visitor.clone());
        Ok(visitor)
    }

    /// Cache-aside with stale fallback when the network is the problem.
    fn cached<T, F>(&self, key: &str, ttl: Duration, fetch: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + HasTracks,
        F: FnOnce() -> Result<T>,
    {
        let key = format!("{}|{key}", self.locale().key());
        let hit = self.inner.cache.get::<T>(&key);
        if let Some(h) = &hit
            && (h.age < ttl || self.is_offline())
        {
            return Ok(hit.unwrap().value);
        }
        match fetch().map(|v| self.complete(v)) {
            Ok(v) => {
                self.inner.cache.put(&key, &v);
                Ok(v)
            }
            Err(e) if e.allows_stale() => match hit {
                Some(h) => {
                    log::info!("ytm: serving stale {key} ({}s old): {e}", h.age.as_secs());
                    Ok(h.value)
                }
                None => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    /// Fills `Track.duration` (and missing artists/album) of every track that
    /// lacks it — songs of the mixed search, home cards — with one
    /// `music/get_queue` request per 50 tracks. Best effort: on failure the
    /// tracks stay as they are.
    fn complete<T: HasTracks>(&self, mut value: T) -> T {
        let mut missing: Vec<String> = Vec::new();
        for t in value.tracks_mut() {
            if t.duration.is_none() && !missing.iter().any(|m| m == t.id.as_str()) {
                missing.push(t.id.as_str().to_owned());
            }
        }
        if missing.is_empty() {
            return value;
        }
        let mut found: HashMap<String, Track> = HashMap::new();
        for chunk in missing.chunks(50) {
            match self.music_call("music/get_queue", json!({ "videoIds": chunk })) {
                Ok(resp) => {
                    for r in crate::json::find_all(&resp, "playlistPanelVideoRenderer") {
                        if let Some(t) = parse::panel_video(r) {
                            found.insert(t.id.as_str().to_owned(), t);
                        }
                    }
                }
                Err(e) => {
                    log::info!("ytm: could not fill durations: {e}");
                    break;
                }
            }
        }
        for t in value.tracks_mut() {
            if t.duration.is_none()
                && let Some(f) = found.get(t.id.as_str())
            {
                t.duration = f.duration;
                if t.artists.is_empty() {
                    t.artists = f.artists.clone();
                }
                if t.album.is_none() {
                    t.album = f.album.clone();
                }
            }
        }
        value
    }

    /// Cached copy only (no network), for an instant first paint.
    pub fn peek_home(&self) -> Option<ShelfPage> {
        let key = format!("{}|home", self.locale().key());
        self.inner.cache.get(&key).map(|h| h.value)
    }

    // ------------------------------------------------------------ discover

    /// Home feed of an anonymous user in the user's country.
    pub fn home(&self) -> Result<ShelfPage> {
        self.cached("home", Ttl::HOME, || {
            parse::shelf_page(&self.music_call("browse", json!({"browseId": "FEmusic_home"}))?)
        })
    }

    /// Next page of a shelf page (home, mood...).
    pub fn page_continuation(&self, token: &str) -> Result<ShelfPage> {
        Ok(self.complete(parse::shelf_page(&self.music_call("browse", json!({"continuation": token}))?)?))
    }

    /// Charts of `country` (ISO code) or of the locale's country.
    pub fn charts(&self, country: Option<&str>) -> Result<ShelfPage> {
        let gl = country
            .map(str::to_owned)
            .or_else(|| self.locale().gl)
            .unwrap_or_else(|| "ZZ".to_owned()); // ZZ = global charts
        self.cached(&format!("charts:{gl}"), Ttl::CHARTS, || {
            parse::shelf_page(&self.music_call(
                "browse",
                json!({"browseId": "FEmusic_charts", "formData": {"selectedValues": [gl]}}),
            )?)
        })
    }

    pub fn new_releases(&self) -> Result<ShelfPage> {
        self.cached("new_releases", Ttl::HOME, || {
            parse::shelf_page(&self.music_call("browse", json!({"browseId": "FEmusic_new_releases"}))?)
        })
    }

    /// "Moods & genres" tiles.
    pub fn categories(&self) -> Result<Vec<Category>> {
        self.cached("categories", Ttl::CATEGORIES, || {
            let resp = self.music_call("browse", json!({"browseId": "FEmusic_moods_and_genres"}))?;
            Ok(parse::categories(&resp))
        })
    }

    /// Any browse target: a category tile, a shelf's "see all"...
    pub fn browse(&self, target: &BrowseRef) -> Result<ShelfPage> {
        let key = format!("browse:{}:{}", target.browse_id, target.params.as_deref().unwrap_or(""));
        self.cached(&key, Ttl::HOME, || {
            let mut body = json!({"browseId": target.browse_id});
            if let Some(p) = &target.params {
                body["params"] = json!(p);
            }
            parse::shelf_page(&self.music_call("browse", body)?)
        })
    }

    /// Home screen: home feed + new releases + charts. Sections that fail
    /// are reported in `partial_errors`; fails only if all three fail.
    pub fn discover(&self) -> Result<Discover> {
        let mut out = Discover::default();
        let mut first_err = None;
        let sections: [(&str, Result<ShelfPage>); 3] = [
            ("home", self.home()),
            ("new_releases", self.new_releases()),
            ("charts", self.charts(None)),
        ];
        for (name, page) in sections {
            match page {
                Ok(p) => out.shelves.extend(p.shelves),
                Err(e) => {
                    out.partial_errors.push(format!("{name}: {e}"));
                    first_err.get_or_insert(e);
                }
            }
        }
        match first_err {
            Some(e) if out.shelves.is_empty() => Err(e),
            _ => Ok(out),
        }
    }

    // ------------------------------------------------------------ search

    /// Raw search: unfiltered (mixed top results) or with a filter. Fetches
    /// continuation pages until `limit` items (at most 5 pages).
    pub fn search_items(&self, text: &str, filter: Option<SearchFilter>, limit: usize) -> Result<Vec<Item>> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let ctx = Ctx {
            type_label: filter.is_none(),
        };
        let mut body = json!({"query": text});
        if let Some(f) = filter {
            body["params"] = json!(f.params());
        }
        let (mut items, mut cont) = parse::search(&self.music_call("search", body)?, ctx)?;
        let mut pages = 1;
        while items.len() < limit && pages < 5 {
            let Some(token) = cont.take() else { break };
            let (more, next) = parse::search(&self.music_call("search", json!({"continuation": token}))?, ctx)?;
            if more.is_empty() {
                break;
            }
            items.extend(more);
            cont = next;
            pages += 1;
        }
        let mut seen = HashSet::new();
        items.retain(|i| seen.insert((std::mem::discriminant(i), i.id().to_owned())));
        Ok(self.complete(items))
    }

    /// Search mapped to the core types (`Provider::search`).
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResults> {
        let all = [SearchKind::Track, SearchKind::Album, SearchKind::Artist, SearchKind::Playlist];
        let limit = query.limit.max(1) as usize;
        let offset = query.offset as usize;
        let want = limit + offset;
        let mut results = SearchResults::default();
        if all.iter().all(|k| query.wants(*k)) && offset == 0 {
            sort_into(&mut results, self.search_items(&query.text, None, 0)?);
        } else {
            for kind in all.into_iter().filter(|k| query.wants(*k)) {
                let items = match kind {
                    SearchKind::Track => self.search_items(&query.text, Some(SearchFilter::Songs), want)?,
                    SearchKind::Album => self.search_items(&query.text, Some(SearchFilter::Albums), want)?,
                    SearchKind::Artist => self.search_items(&query.text, Some(SearchFilter::Artists), want)?,
                    SearchKind::Playlist => {
                        let mut v = self.search_items(&query.text, Some(SearchFilter::FeaturedPlaylists), want)?;
                        if v.len() < want {
                            v.extend(self.search_items(&query.text, Some(SearchFilter::CommunityPlaylists), want)?);
                        }
                        v
                    }
                };
                sort_into(&mut results, items);
            }
        }
        fn page<T>(v: &mut Vec<T>, offset: usize, limit: usize) {
            let end = v.len().min(offset + limit);
            let start = offset.min(end);
            *v = v.drain(start..end).collect();
        }
        page(&mut results.tracks, offset, limit);
        page(&mut results.albums, offset, limit);
        page(&mut results.artists, offset, limit);
        page(&mut results.playlists, offset, limit);
        Ok(results)
    }

    /// Autocomplete while typing. Not cached (one request per call: debounce
    /// in the UI, ~250 ms).
    pub fn suggestions(&self, prefix: &str) -> Result<Suggestions> {
        if prefix.trim().is_empty() {
            return Ok(Suggestions::default());
        }
        Ok(self.complete(parse::suggestions(&self.music_call(
            "music/get_search_suggestions",
            json!({"input": prefix}),
        )?)))
    }

    // ------------------------------------------------------------ pages

    pub fn album(&self, id: &str) -> Result<AlbumPage> {
        self.cached(&format!("album:{id}"), Ttl::ALBUM, || {
            parse::album_page(&self.music_call("browse", json!({"browseId": id}))?, id)
        })
    }

    pub fn artist(&self, id: &str) -> Result<ArtistPage> {
        self.cached(&format!("artist:{id}"), Ttl::ARTIST, || {
            parse::artist_page(&self.music_call("browse", json!({"browseId": id}))?, id)
        })
    }

    /// Playlist by id (with or without the `VL` prefix).
    pub fn playlist(&self, id: &str) -> Result<PlaylistPage> {
        let id = id.strip_prefix("VL").unwrap_or(id);
        self.cached(&format!("playlist:{id}"), Ttl::PLAYLIST, || {
            parse::playlist_page(&self.music_call("browse", json!({"browseId": format!("VL{id}")}))?, id)
        })
    }

    pub fn playlist_continuation(&self, token: &str) -> Result<(Vec<Track>, Option<String>)> {
        Ok(self.complete(parse::playlist_continuation(&self.music_call("browse", json!({"continuation": token}))?)))
    }

    /// Every track of a playlist, following continuations up to `max`.
    pub fn playlist_tracks(&self, id: &str, max: usize) -> Result<Vec<Track>> {
        let page = self.playlist(id)?;
        let mut tracks = page.tracks;
        let mut cont = page.continuation;
        while tracks.len() < max {
            let Some(token) = cont.take() else { break };
            let (more, next) = self.playlist_continuation(&token)?;
            if more.is_empty() {
                break;
            }
            tracks.extend(more);
            cont = next;
        }
        tracks.truncate(max);
        Ok(tracks)
    }

    fn watch_next(&self, video_id: &str) -> Result<parse::WatchNext> {
        parse::watch_next(&self.music_call(
            "next",
            json!({"videoId": video_id, "isAudioOnly": true, "enablePersistentPlaylistPanel": true}),
        )?)
    }

    /// Metadata of one track (title, artists, album, duration, cover).
    pub fn track(&self, video_id: &str) -> Result<Track> {
        self.cached(&format!("track:{video_id}"), Ttl::TRACK, || {
            self.watch_next(video_id)?
                .tracks
                .into_iter()
                .find(|t| t.id.as_str() == video_id)
                .ok_or_else(|| YtmError::NotFound(format!("track {video_id}")))
        })
    }

    /// Endless "radio" of similar tracks starting at `video_id` (auto-queue).
    /// The seed track is the first entry.
    pub fn radio(&self, video_id: &str) -> Result<Radio> {
        let playlist_id = format!("RDAMVM{video_id}");
        self.cached(&format!("radio:{video_id}"), Ttl::RADIO, || {
            let next = parse::watch_next(&self.music_call(
                "next",
                json!({
                    "videoId": video_id,
                    "playlistId": playlist_id,
                    "params": "wAEB",
                    "isAudioOnly": true,
                    "enablePersistentPlaylistPanel": true,
                    "tunerSettingValue": "AUTOMIX_SETTING_NORMAL",
                }),
            )?)?;
            Ok(Radio {
                playlist_id: next.playlist_id.unwrap_or(playlist_id.clone()),
                tracks: next.tracks,
                continuation: next.continuation,
            })
        })
    }

    /// More tracks of a radio.
    pub fn radio_continuation(&self, radio: &Radio) -> Result<Radio> {
        let token = radio
            .continuation
            .as_deref()
            .ok_or_else(|| YtmError::NotFound("radio has no continuation".into()))?;
        let next = parse::watch_next(&self.music_call(
            "next",
            json!({
                "playlistId": radio.playlist_id,
                "continuation": token,
                "isAudioOnly": true,
                "enablePersistentPlaylistPanel": true,
            }),
        )?)?;
        Ok(self.complete(Radio {
            playlist_id: radio.playlist_id.clone(),
            tracks: next.tracks,
            continuation: next.continuation,
        }))
    }

    /// Plain-text lyrics, when YouTube Music has them (`Ok(None)` if not).
    pub fn lyrics(&self, video_id: &str) -> Result<Option<Lyrics>> {
        self.cached(&format!("lyrics:{video_id}"), Ttl::LYRICS, || {
            let Some(browse) = self.watch_next(video_id)?.lyrics_browse_id else {
                return Ok(None);
            };
            Ok(parse::lyrics(&self.music_call("browse", json!({"browseId": browse}))?))
        })
    }

    // ------------------------------------------------------------ streams

    /// Resolves the best audio stream. Uses a prefetched one if still valid.
    pub fn resolve(&self, video_id: &str, quality: StreamQuality) -> Result<ResolvedStream> {
        let key = (video_id.to_owned(), quality_key(quality));
        let pre = self
            .inner
            .prefetched
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&key);
        if let Some(s) = pre.filter(|s| s.time_left() > STREAM_MARGIN) {
            return Ok(s);
        }
        self.resolve_fresh(video_id, quality)
    }

    /// Resolves now and keeps the result for the next [`Self::resolve`]
    /// (e.g. the next queue item, a few seconds before it starts).
    pub fn prefetch(&self, video_id: &str, quality: StreamQuality) -> Result<()> {
        let s = self.resolve_fresh(video_id, quality)?;
        self.inner
            .prefetched
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert((video_id.to_owned(), quality_key(quality)), s);
        Ok(())
    }

    /// Drops any prefetched stream of `video_id` (after a 403/410).
    pub fn invalidate_stream(&self, video_id: &str) {
        self.inner
            .prefetched
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retain(|(id, _), _| id != video_id);
    }

    /// Always asks YouTube (no prefetch reuse), trying each player client.
    pub fn resolve_fresh(&self, video_id: &str, quality: StreamQuality) -> Result<ResolvedStream> {
        if video_id.len() != 11 || !video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(YtmError::NotFound(format!("not a YouTube video id: {video_id:?}")));
        }
        let gated = self.session().gated.clone();
        let mut clients: Vec<PlayerClient> = self
            .inner
            .player_clients
            .iter()
            .copied()
            .filter(|c| !gated.contains(c))
            .collect();
        if clients.is_empty() {
            clients = self.inner.player_clients.clone();
        }
        let mut errors: Vec<(PlayerClient, YtmError)> = Vec::new();
        let mut visitor_reset = false;
        let mut i = 0;
        while i < clients.len() {
            let client = clients[i];
            match self.player_once(client, video_id, quality) {
                Ok(s) => return Ok(s),
                Err(e) => {
                    log::info!("ytm: {} could not resolve {video_id}: {e}", client.name());
                    // A flagged/expired visitor token looks like a bot check:
                    // get a new one once and retry the same client.
                    if e.kind() == ErrorKind::Blocked && !visitor_reset {
                        visitor_reset = true;
                        self.session().visitor_data = None;
                        continue;
                    }
                    let definitive = matches!(e.kind(), ErrorKind::RegionBlocked | ErrorKind::AgeRestricted | ErrorKind::Offline);
                    errors.push((client, e));
                    if definitive {
                        break;
                    }
                }
            }
            i += 1;
        }
        Err(pick_error(errors))
    }

    fn player_once(&self, client: PlayerClient, video_id: &str, quality: StreamQuality) -> Result<ResolvedStream> {
        let visitor = self.visitor_data()?;
        let spec = client.spec();
        let body = client.player_body(video_id, &visitor, &self.locale());
        let req = HttpRequest::post_json(PLAYER_API, body.to_string())
            .header("User-Agent", spec.user_agent)
            .header("X-Youtube-Client-Name", spec.id.to_string())
            .header("X-Youtube-Client-Version", spec.version)
            .header("X-Goog-Visitor-Id", visitor.as_str())
            .header("Origin", "https://www.youtube.com");
        let resp = self.send(&req, true)?;
        let value = Self::json_of(&resp, "player")?;
        let data = stream::parse_player(&value)?;
        let fmt = stream::select_format(&data.formats, quality).ok_or_else(|| {
            if data.formats.iter().any(|f| f.ciphered) {
                YtmError::NoStream(format!("{}: only ciphered formats", client.name()))
            } else {
                YtmError::NoStream(format!("{}: no direct audio format", client.name()))
            }
        })?;
        let resolved = stream::to_resolved(video_id, &data, fmt, client.name(), spec.user_agent, SystemTime::now())
            .ok_or_else(|| YtmError::NoStream("format without url".into()))?;
        if self.inner.verify_streams
            && let Err(e) = self.probe(&resolved)
        {
            if e.kind() == ErrorKind::Blocked {
                self.session().gated.insert(client);
            }
            return Err(e);
        }
        Ok(resolved)
    }

    /// Requests the last KiB of the stream: googlevideo serves the first
    /// ~1 MB even to clients it will cut off (PO-token enforcement).
    fn probe(&self, s: &ResolvedStream) -> Result<()> {
        let range = match s.content_length {
            Some(len) if len > 4096 => format!("bytes={}-{}", len - 1024, len - 1),
            _ => "bytes=0-1023".to_owned(),
        };
        let mut req = HttpRequest::get(s.url.clone()).header("Range", range);
        for (k, v) in &s.headers {
            req = req.header(k, v.clone());
        }
        let resp = self.send(&req, false)?;
        match resp.status {
            200 | 206 => Ok(()),
            403 => Err(YtmError::Blocked(format!("googlevideo refused the {} stream (PO token enforced?)", s.client))),
            other => Err(YtmError::from_status(other, "stream probe", resp.retry_after)),
        }
    }
}

fn sort_into(results: &mut SearchResults, items: Vec<Item>) {
    for item in items {
        match item {
            Item::Track(t) => results.tracks.push(t),
            Item::Album(a) => results.albums.push(a),
            Item::Artist(a) => results.artists.push(a),
            Item::Playlist(p) => results.playlists.push(p),
        }
    }
}

/// Most meaningful error of a failed client chain.
fn pick_error(errors: Vec<(PlayerClient, YtmError)>) -> YtmError {
    let rank = |k: ErrorKind| match k {
        ErrorKind::RegionBlocked | ErrorKind::AgeRestricted | ErrorKind::NotFound => 0,
        ErrorKind::Offline => 1,
        ErrorKind::Unavailable => 2,
        ErrorKind::RateLimited => 3,
        ErrorKind::Network => 4,
        ErrorKind::Blocked => 5,
        ErrorKind::NoStream => 6,
        ErrorKind::UpstreamChanged => 7,
        ErrorKind::Storage => 8,
    };
    if errors.len() == 1 {
        return errors.into_iter().next().map(|(_, e)| e).unwrap();
    }
    let summary: Vec<String> = errors.iter().map(|(c, e)| format!("[{}] {e}", c.name())).collect();
    match errors.into_iter().min_by_key(|(_, e)| rank(e.kind())) {
        None => YtmError::NoStream("no player client configured".into()),
        Some((_, e @ (YtmError::RegionBlocked(_) | YtmError::AgeRestricted(_) | YtmError::NotFound(_) | YtmError::Offline(_) | YtmError::Unavailable(_) | YtmError::RateLimited { .. } | YtmError::Network(_)))) => e,
        Some((_, YtmError::Blocked(_))) => YtmError::Blocked(summary.join("; ")),
        Some(_) => YtmError::NoStream(summary.join("; ")),
    }
}

/// Values that contain tracks whose metadata may need completing.
pub(crate) trait HasTracks {
    fn tracks_mut(&mut self) -> Vec<&mut Track>;
}

fn item_tracks(items: &mut [Item]) -> impl Iterator<Item = &mut Track> {
    items.iter_mut().filter_map(|i| match i {
        Item::Track(t) => Some(t),
        _ => None,
    })
}

fn shelf_tracks(shelves: &mut [Shelf]) -> impl Iterator<Item = &mut Track> {
    shelves.iter_mut().flat_map(|s| item_tracks(&mut s.items))
}

impl HasTracks for Track {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        vec![self]
    }
}
impl HasTracks for Vec<Item> {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        item_tracks(self).collect()
    }
}
impl HasTracks for (Vec<Track>, Option<String>) {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        self.0.iter_mut().collect()
    }
}
impl HasTracks for ShelfPage {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        shelf_tracks(&mut self.shelves).collect()
    }
}
impl HasTracks for AlbumPage {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        self.tracks.iter_mut().chain(shelf_tracks(&mut self.related)).collect()
    }
}
impl HasTracks for ArtistPage {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        self.top_tracks.iter_mut().chain(shelf_tracks(&mut self.shelves)).collect()
    }
}
impl HasTracks for PlaylistPage {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        self.tracks.iter_mut().chain(shelf_tracks(&mut self.related)).collect()
    }
}
impl HasTracks for Radio {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        self.tracks.iter_mut().collect()
    }
}
impl HasTracks for Suggestions {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        item_tracks(&mut self.items).collect()
    }
}
impl HasTracks for Vec<Category> {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        Vec::new()
    }
}
impl HasTracks for Option<Lyrics> {
    fn tracks_mut(&mut self) -> Vec<&mut Track> {
        Vec::new()
    }
}
