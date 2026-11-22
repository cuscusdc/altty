//! Typed errors of the YouTube Music provider.
//!
//! [`YtmError`] is what the crate returns. When it crosses the
//! [`kazzu_core::Provider`] boundary it becomes a [`kazzu_core::Error`]; the
//! message then starts with a stable `[ytm:<code>]` tag so the UI can recover
//! the [`ErrorKind`] with [`ErrorKind::of`] and offer the right action
//! ("try again", "not available", "update the app").

use std::time::Duration;

use kazzu_core::{Error as CoreError, ProviderKind};
use serde::{Deserialize, Serialize};

pub type Result<T, E = YtmError> = std::result::Result<T, E>;

/// Coarse classification of a failure, meant for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// No connection, DNS failure, timeout, TLS error or a 5xx.
    Network,
    /// YouTube refused the request (HTTP 403, "confirm you're not a bot").
    Blocked,
    /// HTTP 429.
    RateLimited,
    /// The item exists but not in the user's country.
    RegionBlocked,
    /// Needs a signed-in, age-verified account.
    AgeRestricted,
    /// Removed, private or otherwise unplayable item.
    Unavailable,
    /// Unknown id / empty page.
    NotFound,
    /// Every player client failed to give a usable audio stream.
    NoStream,
    /// The response no longer has the expected shape (YouTube changed).
    UpstreamChanged,
    /// The app is in offline mode and nothing is cached.
    Offline,
    /// Local disk cache / library file problem.
    Storage,
}

impl ErrorKind {
    const ALL: [ErrorKind; 11] = [
        ErrorKind::Network,
        ErrorKind::Blocked,
        ErrorKind::RateLimited,
        ErrorKind::RegionBlocked,
        ErrorKind::AgeRestricted,
        ErrorKind::Unavailable,
        ErrorKind::NotFound,
        ErrorKind::NoStream,
        ErrorKind::UpstreamChanged,
        ErrorKind::Offline,
        ErrorKind::Storage,
    ];

    /// Stable code used in the `[ytm:<code>]` message tag.
    pub fn code(self) -> &'static str {
        match self {
            ErrorKind::Network => "network",
            ErrorKind::Blocked => "blocked",
            ErrorKind::RateLimited => "rate_limited",
            ErrorKind::RegionBlocked => "region_blocked",
            ErrorKind::AgeRestricted => "age_restricted",
            ErrorKind::Unavailable => "unavailable",
            ErrorKind::NotFound => "not_found",
            ErrorKind::NoStream => "no_stream",
            ErrorKind::UpstreamChanged => "upstream_changed",
            ErrorKind::Offline => "offline",
            ErrorKind::Storage => "storage",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.code() == code)
    }

    /// Whether showing a "try again" button makes sense.
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            ErrorKind::Network
                | ErrorKind::Blocked
                | ErrorKind::RateLimited
                | ErrorKind::NoStream
                | ErrorKind::Offline
        )
    }

    /// Recovers the kind from an error that went through the `Provider`
    /// trait. Returns `None` for errors that did not come from this crate.
    pub fn of(error: &CoreError) -> Option<Self> {
        let message = match error {
            CoreError::Provider {
                provider: ProviderKind::YtMusic,
                message,
            } => message.as_str(),
            CoreError::Network(message) | CoreError::NotFound(message) => message.as_str(),
            _ => return None,
        };
        let rest = message.strip_prefix("[ytm:")?;
        let end = rest.find(']')?;
        Self::from_code(&rest[..end])
    }
}

/// Error returned by every operation of this crate.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum YtmError {
    #[error("network error: {0}")]
    Network(String),

    #[error("blocked by YouTube: {0}")]
    Blocked(String),

    #[error("rate limited by YouTube")]
    RateLimited { retry_after: Option<Duration> },

    #[error("not available in this country: {0}")]
    RegionBlocked(String),

    #[error("age-restricted: {0}")]
    AgeRestricted(String),

    #[error("unavailable: {0}")]
    Unavailable(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("no playable audio stream: {0}")]
    NoStream(String),

    #[error("unexpected response from YouTube ({0}); the app may need an update")]
    UpstreamChanged(String),

    #[error("offline and not cached: {0}")]
    Offline(String),

    #[error("storage error: {0}")]
    Storage(String),
}

impl YtmError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            YtmError::Network(_) => ErrorKind::Network,
            YtmError::Blocked(_) => ErrorKind::Blocked,
            YtmError::RateLimited { .. } => ErrorKind::RateLimited,
            YtmError::RegionBlocked(_) => ErrorKind::RegionBlocked,
            YtmError::AgeRestricted(_) => ErrorKind::AgeRestricted,
            YtmError::Unavailable(_) => ErrorKind::Unavailable,
            YtmError::NotFound(_) => ErrorKind::NotFound,
            YtmError::NoStream(_) => ErrorKind::NoStream,
            YtmError::UpstreamChanged(_) => ErrorKind::UpstreamChanged,
            YtmError::Offline(_) => ErrorKind::Offline,
            YtmError::Storage(_) => ErrorKind::Storage,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.kind().is_retryable()
    }

    /// Whether a stale cached copy is an acceptable answer to this failure
    /// (the network is the problem, not the item).
    pub(crate) fn allows_stale(&self) -> bool {
        matches!(
            self.kind(),
            ErrorKind::Network | ErrorKind::Blocked | ErrorKind::RateLimited | ErrorKind::Offline
        )
    }

    /// Maps an HTTP status of an innertube/googlevideo response.
    pub(crate) fn from_status(status: u16, what: &str, retry_after: Option<Duration>) -> Self {
        match status {
            403 => YtmError::Blocked(format!("HTTP 403 on {what}")),
            429 => YtmError::RateLimited { retry_after },
            404 | 410 => YtmError::NotFound(format!("HTTP {status} on {what}")),
            400 | 412 => YtmError::UpstreamChanged(format!("HTTP {status} on {what}")),
            500..=599 => YtmError::Network(format!("HTTP {status} on {what}")),
            _ => YtmError::Network(format!("unexpected HTTP {status} on {what}")),
        }
    }

    pub(crate) fn shape(what: impl std::fmt::Display) -> Self {
        YtmError::UpstreamChanged(format!("missing {what}"))
    }
}

impl From<std::io::Error> for YtmError {
    fn from(e: std::io::Error) -> Self {
        YtmError::Storage(e.to_string())
    }
}

impl From<YtmError> for CoreError {
    fn from(e: YtmError) -> Self {
        let message = format!("[ytm:{}] {e}", e.kind().code());
        match e.kind() {
            ErrorKind::Network | ErrorKind::Offline => CoreError::Network(message),
            ErrorKind::NotFound => CoreError::NotFound(message),
            _ => CoreError::Provider {
                provider: ProviderKind::YtMusic,
                message,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_survives_the_core_boundary() {
        for err in [
            YtmError::Network("x".into()),
            YtmError::Blocked("x".into()),
            YtmError::RateLimited { retry_after: None },
            YtmError::RegionBlocked("x".into()),
            YtmError::AgeRestricted("x".into()),
            YtmError::Unavailable("x".into()),
            YtmError::NotFound("x".into()),
            YtmError::NoStream("x".into()),
            YtmError::UpstreamChanged("x".into()),
            YtmError::Offline("x".into()),
            YtmError::Storage("x".into()),
        ] {
            let kind = err.kind();
            let core: CoreError = err.into();
            assert_eq!(ErrorKind::of(&core), Some(kind), "{core}");
        }
        assert_eq!(ErrorKind::of(&CoreError::other("nope")), None);
    }

    #[test]
    fn status_mapping() {
        assert_eq!(YtmError::from_status(403, "x", None).kind(), ErrorKind::Blocked);
        assert_eq!(YtmError::from_status(429, "x", None).kind(), ErrorKind::RateLimited);
        assert_eq!(YtmError::from_status(503, "x", None).kind(), ErrorKind::Network);
        assert_eq!(YtmError::from_status(400, "x", None).kind(), ErrorKind::UpstreamChanged);
        assert!(ErrorKind::Network.is_retryable());
        assert!(!ErrorKind::Unavailable.is_retryable());
    }
}
