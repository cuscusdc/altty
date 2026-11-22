//! Catalog model: tracks, albums, artists, playlists and audio formats.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::ids::{AlbumId, ArtistId, PlaylistId, TrackId};

/// Music source / service a piece of metadata comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Files on the local disk (offline library).
    Local,
    Spotify,
    YtMusic,
    Tidal,
    Qobuz,
}

impl ProviderKind {
    /// Every known provider, in display order.
    pub const ALL: [ProviderKind; 5] = [
        ProviderKind::Local,
        ProviderKind::Spotify,
        ProviderKind::YtMusic,
        ProviderKind::Tidal,
        ProviderKind::Qobuz,
    ];

    /// Human readable name.
    pub fn display_name(self) -> &'static str {
        match self {
            ProviderKind::Local => "Local",
            ProviderKind::Spotify => "Spotify",
            ProviderKind::YtMusic => "YouTube Music",
            ProviderKind::Tidal => "Tidal",
            ProviderKind::Qobuz => "Qobuz",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Audio codec of an encoded stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    Flac,
    Alac,
    /// Uncompressed PCM (WAV, AIFF, ...).
    Pcm,
    Mp3,
    Aac,
    Vorbis,
    Opus,
    /// Anything else, by name.
    Other(String),
}

impl Codec {
    /// Whether the codec is lossless (bit-exact reconstruction).
    pub fn is_lossless(&self) -> bool {
        matches!(self, Codec::Flac | Codec::Alac | Codec::Pcm)
    }
}

impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Codec::Flac => "FLAC",
            Codec::Alac => "ALAC",
            Codec::Pcm => "PCM",
            Codec::Mp3 => "MP3",
            Codec::Aac => "AAC",
            Codec::Vorbis => "Vorbis",
            Codec::Opus => "Opus",
            Codec::Other(name) => name,
        };
        f.write_str(name)
    }
}

/// Technical description of an audio stream. Every field except `codec` is
/// optional because remote providers often only know part of it up front.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioFormat {
    pub codec: Codec,
    /// Samples per second per channel, e.g. 44100, 96000.
    pub sample_rate: Option<u32>,
    /// Bits per sample of the *source* (16, 24, 32). `None` for lossy codecs.
    pub bit_depth: Option<u8>,
    pub channels: Option<u16>,
    /// Average bitrate in kbit/s.
    pub bitrate_kbps: Option<u32>,
}

impl AudioFormat {
    /// A format that only knows its codec.
    pub fn new(codec: Codec) -> Self {
        Self {
            codec,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            bitrate_kbps: None,
        }
    }

    /// Lossless and above CD quality (more than 16 bit or more than 48 kHz).
    pub fn is_hi_res(&self) -> bool {
        self.codec.is_lossless()
            && (self.bit_depth.is_some_and(|b| b > 16) || self.sample_rate.is_some_and(|r| r > 48_000))
    }
}

/// Where a track's audio comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TrackSource {
    /// A file on the local disk.
    Local { path: PathBuf },
    /// A track that must be resolved through a [`crate::Provider`].
    Remote { provider: ProviderKind, id: String },
}

impl TrackSource {
    pub fn provider(&self) -> ProviderKind {
        match self {
            TrackSource::Local { .. } => ProviderKind::Local,
            TrackSource::Remote { provider, .. } => *provider,
        }
    }
}

/// Reference to artwork. Resolution/caching is done by the UI/library layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoverRef {
    /// Picture embedded in the tags of this audio file.
    Embedded { path: PathBuf },
    /// Standalone image file (e.g. `cover.jpg` next to the album).
    File { path: PathBuf },
    /// Remote image URL. Fetched only when shown (no prefetch/telemetry).
    Url { url: String },
}

/// Lightweight link to an artist, as attached to tracks and albums.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtistRef {
    pub id: Option<ArtistId>,
    pub name: String,
}

impl ArtistRef {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
        }
    }
}

/// Lightweight link to an album, as attached to tracks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlbumRef {
    pub id: Option<AlbumId>,
    pub title: String,
}

/// A single playable recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub album: Option<AlbumRef>,
    pub duration: Option<Duration>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub format: Option<AudioFormat>,
    pub source: TrackSource,
    pub cover: Option<CoverRef>,
}

impl Track {
    /// Minimal track with only the mandatory fields.
    pub fn new(id: impl Into<TrackId>, title: impl Into<String>, source: TrackSource) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artists: Vec::new(),
            album: None,
            duration: None,
            track_number: None,
            disc_number: None,
            format: None,
            source,
            cover: None,
        }
    }

    /// Artists joined with ", " for display.
    pub fn artist_line(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn provider(&self) -> ProviderKind {
        self.source.provider()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Album {
    pub id: AlbumId,
    pub provider: ProviderKind,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub year: Option<i32>,
    pub track_count: Option<u32>,
    pub cover: Option<CoverRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub provider: ProviderKind,
    pub name: String,
    pub image: Option<CoverRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub provider: ProviderKind,
    pub name: String,
    pub description: Option<String>,
    pub cover: Option<CoverRef>,
    /// Ordered track ids; full tracks are fetched on demand.
    pub tracks: Vec<TrackId>,
    /// Whether the user can modify it (own playlist vs. followed one).
    pub editable: bool,
}
