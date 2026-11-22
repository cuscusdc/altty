//! Error type shared by every Kazzu crate.

use crate::model::ProviderKind;

/// Convenience alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{provider}: authentication required or failed: {message}")]
    Auth {
        provider: ProviderKind,
        message: String,
    },

    #[error("{provider}: {message}")]
    Provider {
        provider: ProviderKind,
        message: String,
    },

    #[error("network error: {0}")]
    Network(String),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("audio output error: {0}")]
    Output(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        Error::Other(message.into())
    }
}
