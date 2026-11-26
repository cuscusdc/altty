//! kazzu-spotify: Spotify provider for Kazzu — metadata-only in Phase 1.
//!
//! - Search, track/album/artist details via the Spotify Web API.
//! - OAuth 2.0 PKCE flow for user authentication.
//! - `resolve_stream()` returns an error directing to Spotify Connect:
//!   the Web API does not emit audio stream URLs (known limitation).
//! - Future: librespot integration (GPL, not suitable for this crate).
//!
//! Pure Rust, no tokio: blocking HTTP (ureq + rustls) behind async methods,
//! usable from any executor (`pollster`, tokio, smol...).
//! No network call happens unless a method is called (no telemetry).

use url::form_urlencoded;
use async_trait::async_trait;
use base64::Engine;
use serde_json::Value;
use sha2::Digest;

use kazzu_core::{
    provider::{Provider, ProviderCapabilities, SearchQuery, SearchResults, StreamQuality, StreamSource},
    model::{Album, Artist, ProviderKind, Track},
    ids::{AlbumId, ArtistId, PlaylistId, TrackId},
    Error as CoreError,
};

use thiserror::Error;

/// Spotify-specific errors
#[derive(Error, Debug)]
pub enum SpotifyError {
    #[error("OAuth 2.0 error: {0}")]
    OAuth(String),
    #[error("Web API error: {0}")]
    WebApi(String),
    #[error("Authentication required")]
    AuthRequired,
    #[error("Track not found: {0}")]
    NotFound(String),
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<SpotifyError> for CoreError {
    fn from(e: SpotifyError) -> Self {
        CoreError::Other(format!("kazzu-spotify: {}", e))
    }
}

/// Spotify Provider implementation
///
/// Follows the Provider trait pattern from kazzu-core.
/// Phase 1: Metadata-only via Spotify Web API.
/// - Search, library, album/artist/track details available
/// - resolve_stream() returns error directing to Spotify Connect
/// - No audio stream URLs via Web API (known limitation)
pub struct SpotifyProvider {
    /// Client ID from Spotify Developer Dashboard
    client_id: String,
    /// Client Secret from Spotify Developer Dashboard
    client_secret: String,
    /// Redirect URI for OAuth 2.0 flow
    redirect_uri: String,
    /// HTTP agent for API requests
    agent: ureq::Agent,
    /// Access token (managed internally)
    access_token: std::sync::Mutex<Option<String>>,
    /// Token expiration time
    token_expiry: std::sync::Mutex<Option<std::time::Instant>>,
}

impl SpotifyProvider {
    /// Create a new Spotify provider with client credentials
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(std::time::Duration::from_secs(10)))
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .max_redirects(0)
            .https_only(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0")
            .build();
        Self {
            client_id,
            client_secret,
            redirect_uri,
            agent: ureq::Agent::new_with_config(config),
            access_token: std::sync::Mutex::new(None),
            token_expiry: std::sync::Mutex::new(None),
        }
    }

    /// Generate the OAuth 2.0 authorization URL (PKCE flow start)
    pub fn authorize_url(&self, state: &str, code_verifier: &str) -> String {
        format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&code_challenge_method=S256&code_challenge={}",
            url_encode(&self.client_id),
            url_encode(&self.redirect_uri),
            url_encode("user-read-private user-read-email user-library-read playlist-read-private playlist-modify-public playlist-modify-private user-follow-modify user-follow-read"),
            url_encode(state),
            url_encode(&sha256_base64url(code_verifier)),
        )
    }

    /// Exchange authorization code for tokens (PKCE flow)
    pub fn exchange_code_for_token(&self, code: &str, code_verifier: &str) -> Result<String, SpotifyError> {
        let url = "https://accounts.spotify.com/api/token";

        let mut req = self.agent.post(url);
        req = req.header("Content-Type", "application/x-form-urlencoded");
        req = req.header("Authorization", &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
        ));

        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("code_verifier", code_verifier)
            .finish();

        let resp = req.send(body)
            .map_err(|e| SpotifyError::WebApi(format!("request failed: {}", e)))?;

        let status = resp.status().as_u16();
        let body_bytes = resp.into_body()
            .with_config()
            .limit(32 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| SpotifyError::WebApi(format!("read body failed: {}", e)))?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        let resp: Value = serde_json::from_str(&body_str)
            .map_err(|e| SpotifyError::WebApi(format!("invalid JSON: {}", e)))?;

        if status != 200 {
            return Err(SpotifyError::WebApi(format!("token exchange failed: {}", body_str)));
        }

        let access_token = resp["access_token"]
            .as_str()
            .ok_or_else(|| SpotifyError::WebApi("no access_token in response".to_owned()))?
            .to_owned();

        let expires_in = resp["expires_in"].as_u64().unwrap_or(3600);
        let expiry = std::time::Instant::now() + std::time::Duration::from_secs(expires_in);

        *self.access_token.lock().unwrap() = Some(access_token.clone());
        *self.token_expiry.lock().unwrap() = Some(expiry);

        Ok(access_token)
    }

    /// Refresh the access token using a refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<String, SpotifyError> {
        let url = "https://accounts.spotify.com/api/token";

        let mut req = self.agent.post(url);
        req = req.header("Content-Type", "application/x-form-urlencoded");
        req = req.header("Authorization", &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
        ));

        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "refresh_token")
            .append_pair("refresh_token", refresh_token)
            .finish();

        let resp = req.send(body)
            .map_err(|e| SpotifyError::WebApi(format!("request failed: {}", e)))?;

        let status = resp.status().as_u16();
        let body_bytes = resp.into_body()
            .with_config()
            .limit(32 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| SpotifyError::WebApi(format!("read body failed: {}", e)))?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        let resp: Value = serde_json::from_str(&body_str)
            .map_err(|e| SpotifyError::WebApi(format!("invalid JSON: {}", e)))?;

        if status != 200 {
            return Err(SpotifyError::WebApi(format!("token refresh failed: {}", body_str)));
        }

        let access_token = resp["access_token"]
            .as_str()
            .ok_or_else(|| SpotifyError::WebApi("no access_token in response".to_owned()))?
            .to_owned();

        let expires_in = resp["expires_in"].as_u64().unwrap_or(3600);
        let expiry = std::time::Instant::now() + std::time::Duration::from_secs(expires_in);

        *self.access_token.lock().unwrap() = Some(access_token.clone());
        *self.token_expiry.lock().unwrap() = Some(expiry);

        Ok(access_token)
    }

    /// Set a stored access token (e.g. from a previous session)
    pub fn set_access_token(&self, token: String) {
        let expiry = std::time::Instant::now() + std::time::Duration::from_secs(3600);
        *self.access_token.lock().unwrap() = Some(token);
        *self.token_expiry.lock().unwrap() = Some(expiry);
    }

    /// Get a bearer token for API requests
    fn bearer_token(&self) -> Result<String, SpotifyError> {
        let token = self.access_token.lock().unwrap();
        token.clone().ok_or(SpotifyError::AuthRequired)
    }

    /// Make a GET request to the Spotify Web API with bearer auth
    fn web_api_get(&self, path: &str) -> Result<Value, CoreError> {
        let token = self.bearer_token()
            .map_err(|e| CoreError::Other(format!("kazzu-spotify: {}", e)))?;

        let url = format!("https://api.spotify.com/v1{}", path);

        let mut req = self.agent.get(&url);
        req = req.header("Authorization", &format!("Bearer {}", token));

        let resp = req.call()
            .map_err(|e| CoreError::Other(format!("Spotify Web API request failed: {}", e)))?;

        let status = resp.status().as_u16();
        let body_bytes = resp.into_body()
            .with_config()
            .limit(32 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| CoreError::Other(format!("read body failed: {}", e)))?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        let value: Value = serde_json::from_str(&body_str)
            .map_err(|e| CoreError::Other(format!("invalid JSON response: {}", e)))?;

        if status != 200 {
            return Err(CoreError::Other(format!("Spotify Web API error ({}): {}", status, body_str)));
        }

        Ok(value)
    }
}

/// SHA-256 of `s`, base64url-encoded without padding (PKCE code_challenge).
fn sha256_base64url(s: &str) -> String {
    let digest = sha2::Sha256::digest(s.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

/// URL-encode a string using percent-encoding.
fn url_encode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// Helper to collect artists from a JSON array
fn collect_artists(arr: &[Value]) -> Vec<kazzu_core::model::ArtistRef> {
    arr.iter()
        .map(|a| {
            kazzu_core::model::ArtistRef::named(
                a.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_owned()
            )
        })
        .collect()
}

#[async_trait]
impl Provider for SpotifyProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Spotify
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            stream: false,
            download: false,
            library: false,
            edit_playlists: false,
            like: false,
            recommendations: false,
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResults, CoreError> {
        let search_type = match query.kinds.first() {
            Some(kazzu_core::provider::SearchKind::Track) => "track",
            Some(kazzu_core::provider::SearchKind::Album) => "album",
            Some(kazzu_core::provider::SearchKind::Artist) => "artist",
            Some(kazzu_core::provider::SearchKind::Playlist) => "playlist",
            None => "track,album,artist",
        };

        let path = format!(
            "/search?q={}&type={}&limit={}&offset={}",
            url_encode(&query.text),
            search_type,
            query.limit,
            query.offset
        );

        let resp = self.web_api_get(&path)?;

        let mut tracks = Vec::new();
        let mut albums = Vec::new();
        let mut artists = Vec::new();
        let mut playlists = Vec::new();

        if let Some(items) = resp.get("tracks").and_then(|t| t.get("items")).and_then(|i| i.as_array()) {
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_owned();
                let artists_list = item
                    .get("artists")
                    .and_then(|a| a.as_array())
                    .map(|arr| collect_artists(arr))
                    .unwrap_or_default();
                let album_name = item
                    .get("album")
                    .and_then(|a| a.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_owned();
                let duration_ms = item.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_owned();

                tracks.push(Track {
                    id: TrackId::new(id.clone()),
                    title: name,
                    artists: artists_list,
                    album: Some(kazzu_core::model::AlbumRef {
                        id: None,
                        title: album_name,
                    }),
                    duration: Some(std::time::Duration::from_millis(duration_ms)),
                    track_number: None,
                    disc_number: None,
                    format: None,
                    source: kazzu_core::model::TrackSource::Remote {
                        provider: ProviderKind::Spotify,
                        id,
                    },
                    cover: None,
                });
            }
        }

        if let Some(items) = resp.get("albums").and_then(|a| a.get("items")).and_then(|i| i.as_array()) {
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_owned();
                let artists_list = item
                    .get("artists")
                    .and_then(|a| a.as_array())
                    .map(|arr| collect_artists(arr))
                    .unwrap_or_default();
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_owned();
                let total_tracks = item.get("total_tracks").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let release_date = item.get("release_date").and_then(|v| v.as_str()).unwrap_or("");
                let year = release_date.split('-').next().and_then(|s| s.parse().ok());

                albums.push(Album {
                    id: AlbumId::new(id.clone()),
                    provider: ProviderKind::Spotify,
                    title: name,
                    artists: artists_list,
                    year,
                    track_count: Some(total_tracks),
                    cover: None,
                });
            }
        }

        if let Some(items) = resp.get("artists").and_then(|a| a.get("items")).and_then(|i| i.as_array()) {
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_owned();
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_owned();
                let genres: Vec<String> = item
                    .get("genres")
                    .and_then(|g| g.as_array())
                    .map(|arr| arr.iter().map(|g| g.as_str().unwrap_or("").to_owned()).collect())
                    .unwrap_or_default();
                let popularity = item.get("popularity").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                artists.push(Artist {
                    id: ArtistId::new(id.clone()),
                    provider: ProviderKind::Spotify,
                    name,
                    image: None,
                    genres,
                    popularity,
                });
            }
        }

        if let Some(items) = resp.get("playlists").and_then(|p| p.get("items")).and_then(|i| i.as_array()) {
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_owned();
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_owned();
                let description = item.get("description").and_then(|v| v.as_str()).map(|s| s.to_owned());

                playlists.push(kazzu_core::model::Playlist {
                    id: PlaylistId::new(id.clone()),
                    provider: ProviderKind::Spotify,
                    name,
                    description,
                    cover: None,
                    tracks: Vec::new(),
                    editable: false,
                });
            }
        }

        Ok(SearchResults {
            tracks,
            albums,
            artists,
            playlists,
        })
    }

    async fn resolve_stream(
        &self,
        _track: &Track,
        _quality: StreamQuality,
    ) -> Result<StreamSource, CoreError> {
        Err(CoreError::other(
            "Spotify Web API does not provide audio stream URLs. \
             Use Spotify Connect to play this track, or play local files. \
             To enable streaming, integrate librespot or the Spotify Connect protocol.",
        ))
    }

    async fn library(&self) -> Result<kazzu_core::provider::LibrarySnapshot, CoreError> {
        Err(CoreError::other(
            "Spotify library access requires user authentication. \
             Please log in via Spotify OAuth and grant library read scope.",
        ))
    }

    async fn track(&self, id: &TrackId) -> Result<Track, CoreError> {
        let spotify_id = id.as_str();
        let resp = self.web_api_get(&format!("/tracks/{}", spotify_id))?;

        let name = resp.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_owned();
        let artists_list = resp
            .get("artists")
            .and_then(|a| a.as_array())
            .map(|arr| collect_artists(arr))
            .unwrap_or_default();
        let album_name = resp
            .get("album")
            .and_then(|a| a.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_owned();
        let duration_ms = resp.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);

        Ok(Track {
            id: TrackId::new(spotify_id.to_owned()),
            title: name,
            artists: artists_list,
            album: Some(kazzu_core::model::AlbumRef {
                id: None,
                title: album_name,
            }),
            duration: Some(std::time::Duration::from_millis(duration_ms)),
            track_number: None,
            disc_number: None,
            format: None,
            source: kazzu_core::model::TrackSource::Remote {
                provider: ProviderKind::Spotify,
                id: spotify_id.to_owned(),
            },
            cover: None,
        })
    }

    async fn album_tracks(&self, id: &AlbumId) -> Result<Vec<Track>, CoreError> {
        let album_id = id.as_str();
        let resp = self.web_api_get(&format!("/albums/{}", album_id))?;

        let album_name = resp
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_owned();
        let tracks_data = resp
            .get("tracks")
            .and_then(|t| t.get("items"))
            .and_then(|i| i.as_array())
            .ok_or_else(|| CoreError::other("No tracks found in album response"))?;

        let mut tracks = Vec::new();
        for track_data in tracks_data {
            let name = track_data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_owned();
            let artists_list = track_data
                .get("artists")
                .and_then(|a| a.as_array())
                .map(|arr| collect_artists(arr))
                .unwrap_or_default();
            let duration_ms = track_data.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            let track_id = track_data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_owned();

            tracks.push(Track {
                id: TrackId::new(track_id.clone()),
                title: name,
                artists: artists_list,
                album: Some(kazzu_core::model::AlbumRef {
                    id: Some(id.clone()),
                    title: album_name.clone(),
                }),
                duration: Some(std::time::Duration::from_millis(duration_ms)),
                track_number: None,
                disc_number: None,
                format: None,
                source: kazzu_core::model::TrackSource::Remote {
                    provider: ProviderKind::Spotify,
                    id: track_id,
                },
                cover: None,
            });
        }

        Ok(tracks)
    }

    async fn playlist_tracks(&self, _id: &PlaylistId) -> Result<Vec<Track>, CoreError> {
        Err(CoreError::other(
            "Spotify playlist retrieval requires proper pagination support. \
             Use the Spotify Web API with user authentication to fetch playlist tracks.",
        ))
    }
}