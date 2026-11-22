//! Page-level types returned by the YouTube Music provider, on top of the
//! `kazzu-core` catalog types (`Track`, `Album`, `Artist`, `Playlist`).

use std::time::{Duration, SystemTime};

use kazzu_core::{Album, AudioFormat, Codec, Artist, Playlist, StreamSource, Track};
use serde::{Deserialize, Serialize};

/// One entry of a shelf / search result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "item")]
pub enum Item {
    Track(Track),
    Album(Album),
    Artist(Artist),
    Playlist(Playlist),
}

impl Item {
    pub fn title(&self) -> &str {
        match self {
            Item::Track(t) => &t.title,
            Item::Album(a) => &a.title,
            Item::Artist(a) => &a.name,
            Item::Playlist(p) => &p.name,
        }
    }

    /// Provider id of the item (video id, album browse id, channel id,
    /// playlist id).
    pub fn id(&self) -> &str {
        match self {
            Item::Track(t) => t.id.as_str(),
            Item::Album(a) => a.id.as_str(),
            Item::Artist(a) => a.id.as_str(),
            Item::Playlist(p) => p.id.as_str(),
        }
    }
}

/// Opaque link to another browse page (a "see all" / category target).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrowseRef {
    pub browse_id: String,
    pub params: Option<String>,
}

/// A titled row of items (carousel, list or grid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shelf {
    pub title: String,
    pub subtitle: Option<String>,
    pub items: Vec<Item>,
    /// "See all" target, if the shelf has one.
    pub more: Option<BrowseRef>,
}

/// A browse page made of shelves (home, charts, new releases, a mood...).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShelfPage {
    pub title: Option<String>,
    pub shelves: Vec<Shelf>,
    /// Token for [`crate::YtClient::page_continuation`], if there is more.
    pub continuation: Option<String>,
}

/// A mood / genre tile of "Explore → Moods & genres".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
    /// Group the tile belongs to ("Moods & moments", "Genres"...).
    pub group: Option<String>,
    pub browse: BrowseRef,
    /// ARGB tile color, when YouTube gives one.
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumPage {
    pub album: Album,
    pub tracks: Vec<Track>,
    pub description: Option<String>,
    /// Localized "13 songs • 49 minutes".
    pub summary: Option<String>,
    /// Album type, localized ("Album", "Single", "EP").
    pub album_type: Option<String>,
    /// Playlist id (`OLAK5uy_...`) that plays the whole album.
    pub audio_playlist_id: Option<String>,
    /// Other versions / related albums shown on the page.
    pub related: Vec<Shelf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtistPage {
    pub artist: Artist,
    pub description: Option<String>,
    /// Localized "65M monthly audience" / subscriber line.
    pub audience: Option<String>,
    pub top_tracks: Vec<Track>,
    /// Albums, singles, videos, featured on, related artists...
    pub shelves: Vec<Shelf>,
    /// Playlist with all the artist's songs ("See all" of top songs).
    pub all_tracks: Option<BrowseRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaylistPage {
    pub playlist: Playlist,
    pub tracks: Vec<Track>,
    pub author: Option<String>,
    /// Localized "19 songs • 1 hour".
    pub summary: Option<String>,
    /// Token for [`crate::YtClient::playlist_continuation`].
    pub continuation: Option<String>,
    pub related: Vec<Shelf>,
}

/// "Up next" / radio queue for auto-play.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Radio {
    pub playlist_id: String,
    pub tracks: Vec<Track>,
    /// Token for [`crate::YtClient::radio_continuation`] (radios are endless).
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lyrics {
    /// Plain text, lines separated by `\n`. Not time-synced.
    pub text: String,
    /// Attribution line, e.g. "Source: Musixmatch". Must be shown with the
    /// lyrics.
    pub source: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Suggestions {
    /// Completed queries, most relevant first.
    pub queries: Vec<String>,
    /// Direct hits (songs, artists, albums).
    pub items: Vec<Item>,
}

/// Everything the Home ("Início") screen needs, assembled from the home feed,
/// new releases and charts of the user's country.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Discover {
    pub shelves: Vec<Shelf>,
    /// Sections that failed (the rest is still shown).
    pub partial_errors: Vec<String>,
}

/// A resolved, directly downloadable audio stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedStream {
    pub video_id: String,
    /// googlevideo URL. Bound to the requesting IP; never persist it.
    pub url: String,
    /// Headers to send on every request of this stream (User-Agent of the
    /// innertube client that produced the URL).
    pub headers: Vec<(String, String)>,
    pub format: AudioFormat,
    /// e.g. `audio/webm; codecs="opus"`.
    pub mime: String,
    pub itag: u32,
    pub content_length: Option<u64>,
    pub duration: Option<Duration>,
    pub expires_at: SystemTime,
    /// Innertube client that produced the URL (for diagnostics).
    pub client: String,
    /// Loudness relative to YouTube's reference, in dB (for an opt-in
    /// normalization; never applied by default).
    pub loudness_db: Option<f32>,
}

impl ResolvedStream {
    /// What the audio engine opens.
    pub fn to_stream_source(&self) -> StreamSource {
        StreamSource::Url {
            url: self.url.clone(),
            headers: self.headers.clone(),
            format: Some(self.format.clone()),
        }
    }

    /// Completes a track with what only the stream knows: the duration (when
    /// the metadata lacked it, e.g. songs of the mixed search) and the audio
    /// format. Call before building the `QueueItem`: kazzu-audio relies on
    /// `Track.duration` for the seek bar of fragmented MP4 (itag 140), whose
    /// header only declares the first fragment.
    pub fn fill_track(&self, track: &mut Track) {
        if track.duration.is_none() {
            track.duration = self.duration;
        }
        track.format = Some(self.format.clone());
    }

    /// Short quality badge for the UI, e.g. `Opus · 128 kbps · 48 kHz`.
    pub fn badge(&self) -> String {
        let mut parts = vec![match &self.format.codec {
            Codec::Aac => "AAC".to_owned(),
            other => other.to_string(),
        }];
        if let Some(kbps) = self.format.bitrate_kbps {
            parts.push(format!("{kbps} kbps"));
        }
        if let Some(rate) = self.format.sample_rate {
            parts.push(if rate % 1000 == 0 {
                format!("{} kHz", rate / 1000)
            } else {
                format!("{:.1} kHz", rate as f64 / 1000.0)
            });
        }
        parts.join(" · ")
    }

    /// Time left before the URL expires (zero when already expired).
    pub fn time_left(&self) -> Duration {
        self.expires_at
            .duration_since(SystemTime::now())
            .unwrap_or_default()
    }
}
