//! HTTP(S) streaming for [`StreamSource::Url`](kazzu_core::StreamSource).
//!
//! ```text
//!  thread "kazzu-net"           thread "kazzu-stream"              engine thread
//!  ranged GET (ureq) ──bytes──▶ HttpSource (Read+Seek) ──symphonia──▶ PCM chunks ──▶ Decoding
//!  retry/backoff, throttle      blocks only this thread             (bounded channel, try_recv)
//!  optional disk cache write
//! ```
//!
//! The engine thread never touches the network: opening, probing, seeking
//! and decoding a remote stream happen on a per-stream worker thread, and the
//! engine only polls a channel. The device callback is unaffected (it still
//! only reads the ring).

mod cache;
mod feed;
mod http;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use kazzu_core::QueueItemId;

pub use cache::StreamCache;
pub(crate) use feed::{FeedFailure, OpenPoll, Pull, RemoteFeed, RemoteSpec};

/// Disk cache of streams that were downloaded completely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheConfig {
    pub dir: PathBuf,
    /// Total size limit; least recently used files are evicted first.
    pub max_bytes: u64,
}

/// Network streaming tuning (part of [`EngineConfig`](crate::EngineConfig)).
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// TCP + TLS setup budget per request.
    pub connect_timeout: Duration,
    /// Budget to receive response headers, and to receive one ranged body.
    /// A body that is not complete in time is resumed (not restarted) by a
    /// new request from the last byte received.
    pub read_timeout: Duration,
    /// Consecutive failed attempts (without any byte received) before the
    /// stream is reported as failed.
    pub max_retries: u32,
    /// First retry delay; doubles on each attempt up to `retry_max`.
    pub retry_base: Duration,
    pub retry_max: Duration,
    /// Size of each `Range: bytes=a-b` request. Bounded ranges keep servers
    /// such as googlevideo from throttling (they slow down open-ended
    /// requests) and bound the cost of a dropped connection.
    pub request_chunk: u64,
    /// How far ahead of the decoder bytes are downloaded.
    pub read_ahead: u64,
    /// Max downloaded bytes kept in memory per stream (blocks far from the
    /// read position are dropped first and refetched if needed).
    pub memory_limit: u64,
    /// Download speed cap in bytes/s (`None` = unlimited).
    pub max_bandwidth: Option<u64>,
    /// Optional LRU disk cache (base for offline playback).
    pub cache: Option<CacheConfig>,
    /// Start opening the next remote track this long before the current
    /// one ends, so the gapless join does not wait for the network.
    pub prebuffer: Duration,
    /// How long the engine waits for [`EngineHandle::replace_stream`]
    /// after a [`StreamEvent::Expired`] before skipping the item.
    ///
    /// [`EngineHandle::replace_stream`]: crate::EngineHandle::replace_stream
    pub expired_wait: Duration,
    /// User-Agent used when the stream headers do not set one.
    pub user_agent: String,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(20),
            max_retries: 8,
            retry_base: Duration::from_millis(250),
            retry_max: Duration::from_secs(8),
            request_chunk: 4 * 1024 * 1024,
            read_ahead: 16 * 1024 * 1024,
            memory_limit: 32 * 1024 * 1024,
            max_bandwidth: None,
            cache: None,
            prebuffer: Duration::from_secs(15),
            expired_wait: Duration::from_secs(30),
            user_agent: concat!("Kazzu/", env!("CARGO_PKG_VERSION")).to_owned(),
        }
    }
}

/// Streaming notifications that need an answer from the controller.
///
/// Kept apart from [`PlaybackEvent`](kazzu_core::PlaybackEvent) so adding
/// them does not change the shared, serialized vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// The server refused the stream URL (HTTP 401/403/410): signed URLs
    /// such as YouTube's expire. Call `Provider::resolve_stream` again and
    /// hand the result to [`EngineHandle::replace_stream`]; the engine
    /// resumes at `position` (gapless if the audio before it is still
    /// buffered). Without an answer within `StreamConfig::expired_wait`
    /// the item is skipped.
    ///
    /// [`EngineHandle::replace_stream`]: crate::EngineHandle::replace_stream
    Expired { id: QueueItemId, position: Duration },
    /// The network failed after all retries. Playback pauses when the
    /// buffered audio runs out; `play()` retries from `position`.
    NetworkFailed {
        id: QueueItemId,
        position: Duration,
        message: String,
    },
}

/// Shared by every stream of one engine: one connection pool, one cache.
pub(crate) struct NetContext {
    pub agent: ureq::Agent,
    pub config: StreamConfig,
    pub cache: Option<Arc<StreamCache>>,
}

impl NetContext {
    pub fn new(config: StreamConfig) -> Arc<Self> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(config.connect_timeout))
            .timeout_recv_response(Some(config.read_timeout))
            .timeout_recv_body(Some(config.read_timeout))
            .user_agent(config.user_agent.as_str())
            .build()
            .into();
        let cache = config.cache.as_ref().and_then(|c| match StreamCache::new(&c.dir, c.max_bytes) {
            Ok(cache) => Some(Arc::new(cache)),
            Err(e) => {
                log::warn!("stream cache disabled ({}): {e}", c.dir.display());
                None
            }
        });
        Arc::new(Self { agent, config, cache })
    }
}
