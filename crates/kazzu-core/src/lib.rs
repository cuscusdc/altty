//! kazzu-core: domain types shared by every Kazzu crate.
//!
//! - [`ids`]: strongly typed ids (`TrackId`, `AlbumId`, `ArtistId`,
//!   `PlaylistId`, `QueueItemId`).
//! - [`model`]: `Track`, `Album`, `Artist`, `Playlist`, `AudioFormat`,
//!   `ProviderKind`, ...
//! - [`provider`]: the async `Provider` trait and its request/response types.
//! - [`playback`]: queue items, playback commands, events and state.
//! - [`error`]: the shared `Error` / `Result`.
//!
//! This crate has no I/O and no runtime dependency (no tokio); it only
//! defines vocabulary.

pub mod error;
pub mod ids;
pub mod model;
pub mod playback;
pub mod provider;

pub use error::{Error, Result};
pub use ids::{AlbumId, ArtistId, PlaylistId, QueueItemId, TrackId};
pub use model::{
    Album, AlbumRef, Artist, ArtistRef, AudioFormat, Codec, CoverRef, Playlist, ProviderKind,
    Track, TrackSource,
};
pub use playback::{
    OutputInfo, PlaybackCommand, PlaybackEvent, PlaybackState, PlaybackStatus, QueueItem,
    RepeatMode,
};
pub use provider::{
    LibrarySnapshot, Provider, ProviderCapabilities, SearchKind, SearchQuery, SearchResults,
    StreamQuality, StreamSource,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_track() -> Track {
        let mut t = Track::new(
            "1",
            "Song",
            TrackSource::Local {
                path: "/music/a.flac".into(),
            },
        );
        t.artists = vec![ArtistRef::named("A"), ArtistRef::named("B")];
        t.duration = Some(Duration::from_secs(180));
        t.format = Some(AudioFormat {
            codec: Codec::Flac,
            sample_rate: Some(96_000),
            bit_depth: Some(24),
            channels: Some(2),
            bitrate_kbps: None,
        });
        t
    }

    #[test]
    fn ids_are_transparent_in_json() {
        let id = TrackId::new("abc");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"abc\"");
        let a = QueueItemId::next();
        let b = QueueItemId::next();
        assert_ne!(a, b);
    }

    #[test]
    fn track_roundtrip_and_helpers() {
        let t = sample_track();
        assert_eq!(t.artist_line(), "A, B");
        assert_eq!(t.provider(), ProviderKind::Local);
        assert!(t.format.as_ref().unwrap().is_hi_res());
        let json = serde_json::to_string(&t).unwrap();
        let back: Track = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn local_queue_item_uses_file_stream() {
        let item = QueueItem::local(sample_track()).unwrap();
        assert_eq!(item.stream, StreamSource::file("/music/a.flac"));
        let remote = Track::new(
            "x",
            "Remote",
            TrackSource::Remote {
                provider: ProviderKind::Tidal,
                id: "42".into(),
            },
        );
        assert!(QueueItem::local(remote).is_none());
    }

    #[test]
    fn commands_serialize_tagged() {
        let cmd = PlaybackCommand::Seek {
            position: Duration::from_millis(1500),
        };
        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["cmd"], "seek");
        let back: PlaybackCommand = serde_json::from_value(json).unwrap();
        assert_eq!(back, cmd);
    }

    // Compile-time check that Provider is object safe.
    #[allow(dead_code)]
    fn provider_is_dyn(_: &dyn Provider) {}
}
