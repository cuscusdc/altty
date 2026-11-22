//! Strongly typed identifiers.
//!
//! All ids are opaque strings so that every provider can use its native id
//! format (a SQLite row id or a path hash for local files, a Spotify URI, a
//! YouTube video id, ...). Ids are only unique *within* a provider; pair them
//! with [`crate::ProviderKind`] when a global key is required.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps a provider-specific id.
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Borrows the raw id.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Returns the raw id.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(
    /// Identifier of a [`crate::Track`].
    TrackId
);
string_id!(
    /// Identifier of an [`crate::Album`].
    AlbumId
);
string_id!(
    /// Identifier of an [`crate::Artist`].
    ArtistId
);
string_id!(
    /// Identifier of a [`crate::Playlist`].
    PlaylistId
);

/// Identifier of one entry in the playback queue.
///
/// Unlike [`TrackId`], it is unique per *enqueue*: the same track enqueued
/// twice gets two different `QueueItemId`s. Ids are process-local and are
/// allocated with [`QueueItemId::next`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueueItemId(pub u64);

impl QueueItemId {
    /// Allocates a fresh, process-unique id.
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for QueueItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q{}", self.0)
    }
}
