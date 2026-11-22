//! Output side: the realtime renderer that runs inside the device callback
//! and the [`AudioSink`] abstraction implemented by the cpal and null sinks.
//!
//! Real-time rules for everything reachable from [`Renderer::render`]: no
//! allocation, no locks, no syscalls, no logging. Communication with the
//! engine thread happens only through the lock-free ring buffer and atomics.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use ringbuf::HeapCons;
use ringbuf::traits::{Consumer, Observer};

/// Interleaved frames are moved through the renderer in chunks of at most
/// this many frames, using a scratch buffer allocated up front.
const SCRATCH_FRAMES: usize = 4096;

/// What the engine asks the sink to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputRequest {
    /// Native sample rate of the source.
    pub sample_rate: u32,
    pub channels: u16,
    /// Bits per sample of the source, used to pick a device sample format
    /// that does not lose precision.
    pub bits_per_sample: Option<u32>,
}

/// Device sample format actually used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputSampleFormat {
    I16,
    I24,
    I32,
    F32,
    /// Other formats, converted through f32 by the backend.
    Other,
}

impl OutputSampleFormat {
    /// Effective integer precision, `None` for float.
    pub fn bits(self) -> Option<u32> {
        match self {
            OutputSampleFormat::I16 => Some(16),
            OutputSampleFormat::I24 => Some(24),
            OutputSampleFormat::I32 => Some(32),
            OutputSampleFormat::F32 | OutputSampleFormat::Other => None,
        }
    }
}

/// What the sink actually opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputConfig {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: OutputSampleFormat,
}

/// A destination for rendered audio (sound card, null device, ...).
///
/// `open` is called on the engine thread whenever the source format changes.
/// The sink negotiates a configuration, asks `make_renderer` for a
/// [`Renderer`] built for it, and must then call [`Renderer::render`] from
/// its realtime context until `close` (or the next `open`).
pub trait AudioSink: Send {
    /// Opens (or reopens) the output. The sink should honour the requested
    /// rate and channel count when the hardware allows it (bit-perfect path)
    /// and report what it really opened; the engine resamples/remaps if the
    /// result differs.
    fn open(
        &mut self,
        request: OutputRequest,
        make_renderer: &mut dyn FnMut(&OutputConfig) -> Renderer,
    ) -> Result<OutputConfig, String>;

    /// Stops rendering and releases the device.
    fn close(&mut self);

    /// Whether the device reported a fatal error since the last `open`
    /// (device unplugged, stream invalidated). The engine then reopens.
    fn failed(&self) -> bool {
        false
    }
}

/// State shared between the engine thread and the renderer.
#[derive(Debug)]
pub(crate) struct Shared {
    /// Frames consumed from the ring (played or flushed), since stream open.
    pub frames_played: AtomicU64,
    /// When true the renderer outputs silence without consuming.
    pub paused: AtomicBool,
    /// `f32` bits of the linear gain.
    pub volume: AtomicU32,
    /// Flush handshake: the engine bumps `flush_req`, the renderer drops all
    /// buffered samples and copies the value into `flush_ack`.
    pub flush_req: AtomicU64,
    pub flush_ack: AtomicU64,
    /// `frames_played` at the instant of the last flush, before the dropped
    /// frames were added: the exact amount of audio that was really heard.
    pub flushed_at: AtomicU64,
    /// Number of callbacks that ran out of data while playing.
    pub underruns: AtomicU64,
}

impl Shared {
    pub fn new(volume: f32) -> Self {
        Self {
            frames_played: AtomicU64::new(0),
            paused: AtomicBool::new(true),
            volume: AtomicU32::new(volume.to_bits()),
            flush_req: AtomicU64::new(0),
            flush_ack: AtomicU64::new(0),
            flushed_at: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
        }
    }
}

/// Realtime half of the pipeline, moved into the device callback.
pub struct Renderer {
    cons: HeapCons<f32>,
    shared: Arc<Shared>,
    channels: usize,
    scratch: Box<[f32]>,
}

impl Renderer {
    pub(crate) fn new(cons: HeapCons<f32>, shared: Arc<Shared>, channels: u16) -> Self {
        let channels = usize::from(channels.max(1));
        Self {
            cons,
            shared,
            channels,
            scratch: vec![0.0; SCRATCH_FRAMES * channels].into_boxed_slice(),
        }
    }

    /// Number of interleaved channels the renderer produces.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Fills `out` (interleaved, `channels` per frame) with the next samples,
    /// padding with silence on underrun or pause. Returns how many frames
    /// came from the ring. Realtime safe.
    pub fn render<T: OutputSample>(&mut self, out: &mut [T]) -> usize {
        let req = self.shared.flush_req.load(Ordering::Acquire);
        if req != self.shared.flush_ack.load(Ordering::Relaxed) {
            let heard = self.shared.frames_played.load(Ordering::Relaxed);
            self.shared.flushed_at.store(heard, Ordering::Relaxed);
            let dropped = self.cons.skip(self.cons.occupied_len());
            self.shared
                .frames_played
                .fetch_add((dropped / self.channels) as u64, Ordering::Release);
            self.shared.flush_ack.store(req, Ordering::Release);
        }

        if self.shared.paused.load(Ordering::Acquire) {
            out.fill(T::SILENCE);
            return 0;
        }

        let gain = f32::from_bits(self.shared.volume.load(Ordering::Relaxed));
        let unity = gain == 1.0;
        let mut starved = false;
        let mut rendered = 0;

        for chunk in out.chunks_mut(self.scratch.len()) {
            if starved {
                // Once starved, the rest of this callback stays silent so the
                // audio that arrives later is not split by a gap.
                chunk.fill(T::SILENCE);
                continue;
            }
            // Only whole frames are taken from the ring.
            let wanted = chunk.len() - chunk.len() % self.channels;
            let available = self.cons.occupied_len();
            let take = wanted.min(available - available % self.channels);
            let got = self.cons.pop_slice(&mut self.scratch[..take]);
            let samples = &mut self.scratch[..got];
            if !unity {
                for s in samples.iter_mut() {
                    *s *= gain;
                }
            }
            for (dst, src) in chunk.iter_mut().zip(samples.iter()) {
                *dst = T::from_f32(*src);
            }
            if got < chunk.len() {
                chunk[got..].fill(T::SILENCE);
                starved = true;
            }
            rendered += got / self.channels;
            self.shared
                .frames_played
                .fetch_add((got / self.channels) as u64, Ordering::Release);
        }

        if starved {
            self.shared.underruns.fetch_add(1, Ordering::Relaxed);
        }
        rendered
    }
}

/// Device sample types the renderer can write. Conversions are exact
/// inverses of symphonia's integer→f32 scaling (`x / 2^(bits-1)`), so a
/// 16/24-bit source rendered at unity gain to a device of equal or greater
/// depth is bit-exact.
pub trait OutputSample: Copy + Send + 'static {
    const SILENCE: Self;
    fn from_f32(sample: f32) -> Self;
}

impl OutputSample for f32 {
    const SILENCE: Self = 0.0;
    #[inline]
    fn from_f32(sample: f32) -> Self {
        sample
    }
}

impl OutputSample for f64 {
    const SILENCE: Self = 0.0;
    #[inline]
    fn from_f32(sample: f32) -> Self {
        f64::from(sample)
    }
}

impl OutputSample for i16 {
    const SILENCE: Self = 0;
    #[inline]
    fn from_f32(sample: f32) -> Self {
        (sample * 32_768.0).round().clamp(-32_768.0, 32_767.0) as i16
    }
}

impl OutputSample for i32 {
    const SILENCE: Self = 0;
    #[inline]
    fn from_f32(sample: f32) -> Self {
        // Computed in f64 so full-scale values do not overflow.
        (f64::from(sample) * 2_147_483_648.0)
            .round()
            .clamp(-2_147_483_648.0, 2_147_483_647.0) as i32
    }
}

/// 24-bit value stored in the low bits of an `i32`.
pub fn f32_to_i24(sample: f32) -> i32 {
    (sample * 8_388_608.0).round().clamp(-8_388_608.0, 8_388_607.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_roundtrip_is_exact() {
        for v in [i16::MIN, -12345, -1, 0, 1, 777, i16::MAX] {
            let f = f32::from(v) / 32_768.0;
            assert_eq!(i16::from_f32(f), v);
        }
        for v in [-8_388_608, -1_234_567, -1, 0, 1, 4_321_000, 8_388_607] {
            let f = v as f32 / 8_388_608.0;
            assert_eq!(f32_to_i24(f), v);
            // 24-bit source into a 32-bit device: exact left shift.
            assert_eq!(i32::from_f32(f), v << 8);
        }
    }
}
