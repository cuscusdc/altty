//! kazzu-audio: Kazzu's playback engine.
//!
//! - Decoding: symphonia (FLAC, WAV/AIFF/CAF PCM, ALAC, MP3, AAC, Vorbis,
//!   ADPCM) + libopus through `symphonia-adapter-libopus` (feature `opus`).
//! - Output: cpal ([`CpalSink`]) or [`NullSink`] for headless use/tests.
//! - Bit-perfect by default: the device is opened at the file's native rate;
//!   resampling (rubato) only happens if the device refuses that rate; the
//!   DSP stage ([`Dsp`]) defaults to [`Bypass`]; unity volume skips gain.
//! - Gapless playback, sample-accurate seek, queue with repeat/shuffle.
//! - HTTP(S) streaming ([`stream`]): ranged downloads on a network thread,
//!   resume after drops, expired-URL refresh ([`StreamEvent`]), LRU disk cache.
//!
//! ```no_run
//! use kazzu_audio::Engine;
//! use kazzu_core::{QueueItem, StreamSource, Track, TrackSource};
//!
//! let engine = Engine::spawn().unwrap();
//! let path = "/music/song.flac";
//! let track = Track::new("1", "Song", TrackSource::Local { path: path.into() });
//! engine.load(vec![QueueItem::new(track, StreamSource::file(path))], 0);
//! for event in engine.events().iter() {
//!     println!("{event:?}");
//! }
//! ```

mod cpal_sink;
mod decoder;
pub mod dsp;
mod engine;
mod null_sink;
pub mod output;
mod queue;
mod resample;
pub mod stream;

pub use cpal_sink::{CpalSink, OutputDevice};
pub use decoder::{SourceSpec, TrackDecoder};
pub use dsp::{Bypass, Dsp, DspChain};
pub use engine::{Engine, EngineBuilder, EngineConfig, EngineHandle};
pub use null_sink::{Capture, NullSink};
pub use stream::{CacheConfig, StreamCache, StreamConfig, StreamEvent};
pub use output::{AudioSink, OutputConfig, OutputRequest, OutputSample, OutputSampleFormat, Renderer};

/// Whether this build can decode Opus.
pub fn opus_supported() -> bool {
    cfg!(feature = "opus")
}
