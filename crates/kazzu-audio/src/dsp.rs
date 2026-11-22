//! Pluggable DSP stage (stage 3: equalizer).
//!
//! The stage runs on the decode thread, after decoding/resampling and before
//! the ring buffer, so implementations do not need to be realtime safe (they
//! may allocate in `configure`). Parameter changes become audible after the
//! ring latency (~0.3 s). The default stage is [`Bypass`], which leaves the
//! samples untouched; while the active stage reports `is_bypassed() == true`
//! the engine keeps reporting the path as bit-perfect.

/// An audio processor working on interleaved `f32` frames.
pub trait Dsp: Send {
    /// Called before the first `process` and whenever the stream format
    /// changes. Allocate filters/buffers here.
    fn configure(&mut self, sample_rate: u32, channels: u16);

    /// Processes `samples` in place (interleaved, `channels` per frame).
    fn process(&mut self, samples: &mut [f32]);

    /// Clears internal state (filter memories); called on seek/track jump.
    fn reset(&mut self) {}

    /// True when `process` is a no-op. The engine then skips the call and
    /// treats the path as bit-perfect.
    fn is_bypassed(&self) -> bool {
        false
    }
}

/// The identity processor, used by default.
#[derive(Debug, Default, Clone, Copy)]
pub struct Bypass;

impl Dsp for Bypass {
    fn configure(&mut self, _sample_rate: u32, _channels: u16) {}

    fn process(&mut self, _samples: &mut [f32]) {}

    fn is_bypassed(&self) -> bool {
        true
    }
}

/// Runs several processors in order. Bypassed when all of them are.
#[derive(Default)]
pub struct DspChain {
    stages: Vec<Box<dyn Dsp>>,
}

impl DspChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, stage: Box<dyn Dsp>) {
        self.stages.push(stage);
    }

    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}

impl Dsp for DspChain {
    fn configure(&mut self, sample_rate: u32, channels: u16) {
        for s in &mut self.stages {
            s.configure(sample_rate, channels);
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        for s in &mut self.stages {
            if !s.is_bypassed() {
                s.process(samples);
            }
        }
    }

    fn reset(&mut self) {
        for s in &mut self.stages {
            s.reset();
        }
    }

    fn is_bypassed(&self) -> bool {
        self.stages.iter().all(|s| s.is_bypassed())
    }
}
