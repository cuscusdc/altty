//! Headless output for tests, CI and machines without a sound card.
//!
//! A worker thread plays the role of the device callback, rendering one
//! period at a time at `speed`× real time. Optionally it records every
//! sample that came from the ring (silence from pauses/underruns is not
//! recorded), so tests can check bit-exactness and gapless joins.
//!
//! The capture uses a mutex; that is fine here because the worker is not a
//! real audio callback. Real sinks must never lock.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::output::{AudioSink, OutputConfig, OutputRequest, OutputSampleFormat, Renderer};

/// Recorded output, shared with the test.
#[derive(Debug, Clone, Default)]
pub struct Capture {
    inner: Arc<Mutex<CaptureData>>,
}

#[derive(Debug, Default)]
struct CaptureData {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    opens: u32,
}

impl Capture {
    /// Interleaved samples recorded so far.
    pub fn samples(&self) -> Vec<f32> {
        self.inner.lock().map(|d| d.samples.clone()).unwrap_or_default()
    }

    pub fn frames(&self) -> usize {
        self.inner
            .lock()
            .map(|d| d.samples.len() / usize::from(d.channels.max(1)))
            .unwrap_or(0)
    }

    pub fn clear(&self) {
        if let Ok(mut d) = self.inner.lock() {
            d.samples.clear();
        }
    }

    /// Format of the last opened stream.
    pub fn format(&self) -> (u32, u16) {
        self.inner
            .lock()
            .map(|d| (d.sample_rate, d.channels))
            .unwrap_or((0, 0))
    }

    /// How many times the sink was opened.
    pub fn opens(&self) -> u32 {
        self.inner.lock().map(|d| d.opens).unwrap_or(0)
    }
}

/// A sink that discards (or records) audio without any device.
pub struct NullSink {
    speed: f64,
    period_frames: usize,
    rates: Option<Vec<u32>>,
    capture: Option<Capture>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Default for NullSink {
    fn default() -> Self {
        Self::new()
    }
}

impl NullSink {
    /// Real-time pacing, accepts any rate/channel count.
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            period_frames: 512,
            rates: None,
            capture: None,
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
        }
    }

    /// Runs `speed` times faster than real time (tests).
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed.max(0.01);
        self
    }

    /// Frames rendered per simulated callback.
    pub fn with_period(mut self, frames: usize) -> Self {
        self.period_frames = frames.max(16);
        self
    }

    /// Restricts the "device" to these sample rates (exercises resampling).
    pub fn with_rates(mut self, rates: Vec<u32>) -> Self {
        self.rates = Some(rates);
        self
    }

    /// Records rendered samples into the returned [`Capture`].
    pub fn with_capture(mut self) -> (Self, Capture) {
        let capture = Capture::default();
        self.capture = Some(capture.clone());
        (self, capture)
    }
}

impl AudioSink for NullSink {
    fn open(
        &mut self,
        request: OutputRequest,
        make_renderer: &mut dyn FnMut(&OutputConfig) -> Renderer,
    ) -> Result<OutputConfig, String> {
        self.close();
        let sample_rate = match &self.rates {
            Some(rates) if !rates.contains(&request.sample_rate) => *rates
                .iter()
                .max()
                .ok_or_else(|| "null sink has no rates".to_owned())?,
            _ => request.sample_rate,
        };
        let config = OutputConfig {
            device: "null".to_owned(),
            sample_rate,
            channels: request.channels,
            sample_format: OutputSampleFormat::F32,
        };
        let mut renderer = make_renderer(&config);

        if let Some(capture) = &self.capture
            && let Ok(mut d) = capture.inner.lock()
        {
            d.sample_rate = sample_rate;
            d.channels = request.channels;
            d.opens += 1;
        }

        let stop = Arc::new(AtomicBool::new(false));
        self.stop = Arc::clone(&stop);
        let capture = self.capture.clone();
        let channels = usize::from(request.channels.max(1));
        let period_frames = self.period_frames;
        let period = Duration::from_secs_f64(period_frames as f64 / f64::from(sample_rate) / self.speed);

        let worker = std::thread::Builder::new()
            .name("kazzu-null-sink".into())
            .spawn(move || {
                let mut buf = vec![0.0f32; period_frames * channels];
                let mut next = Instant::now();
                while !stop.load(Ordering::Acquire) {
                    let frames = renderer.render(&mut buf);
                    if frames > 0
                        && let Some(capture) = &capture
                        && let Ok(mut d) = capture.inner.lock()
                    {
                        d.samples.extend_from_slice(&buf[..frames * channels]);
                    }
                    next += period;
                    let now = Instant::now();
                    if next > now {
                        std::thread::sleep(next - now);
                    } else {
                        next = now;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        self.worker = Some(worker);
        Ok(config)
    }

    fn close(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for NullSink {
    fn drop(&mut self) {
        self.close();
    }
}
