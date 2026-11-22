//! Streaming sample-rate conversion (only used when the output device cannot
//! run at the source rate). Wraps rubato's asynchronous sinc resampler and
//! removes its group delay so tracks stay sample-aligned for gapless play.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{
    Async, FixedAsync, Indexing, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

const CHUNK_FRAMES: usize = 1024;

pub struct StreamResampler {
    inner: Async<f32>,
    channels: usize,
    ratio: f64,
    /// Interleaved input waiting for a full chunk.
    pending: Vec<f32>,
    out_buf: Vec<f32>,
    /// Output frames still to discard (resampler delay).
    delay_left: usize,
    frames_in: u64,
    frames_out: u64,
}

impl StreamResampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: u16) -> Result<Self, String> {
        let channels = usize::from(channels.max(1));
        let ratio = f64::from(to_rate) / f64::from(from_rate);
        let params = SincInterpolationParameters::new(256, WindowFunction::BlackmanHarris2)
            .oversampling_factor(256)
            .interpolation(SincInterpolationType::Cubic);
        let inner = Async::<f32>::new_sinc(
            ratio,
            1.0,
            &params,
            CHUNK_FRAMES,
            channels,
            FixedAsync::Input,
        )
        .map_err(|e| e.to_string())?;
        let delay_left = inner.output_delay();
        let out_len = inner.output_frames_max() * channels;
        Ok(Self {
            inner,
            channels,
            ratio,
            pending: Vec::with_capacity(CHUNK_FRAMES * channels * 2),
            out_buf: vec![0.0; out_len],
            delay_left,
            frames_in: 0,
            frames_out: 0,
        })
    }

    /// Feeds interleaved input; appends any produced output to `out`.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) -> Result<(), String> {
        self.pending.extend_from_slice(input);
        self.frames_in += (input.len() / self.channels) as u64;
        loop {
            let need = self.inner.input_frames_next();
            if self.pending.len() / self.channels < need {
                return Ok(());
            }
            self.run_chunk(need, None, out)?;
            self.pending.drain(..need * self.channels);
        }
    }

    /// Flushes the tail at end of stream, emitting exactly
    /// `round(frames_in * ratio)` frames in total.
    pub fn finish(&mut self, out: &mut Vec<f32>) -> Result<(), String> {
        let expected = (self.frames_in as f64 * self.ratio).round() as u64;
        let mut partial = self.pending.len() / self.channels;
        let mut guard = 0;
        while self.frames_out < expected && guard < 64 {
            let need = self.inner.input_frames_next();
            self.pending.resize(need * self.channels, 0.0);
            self.run_chunk(need, Some(partial), out)?;
            self.pending.clear();
            partial = 0;
            guard += 1;
        }
        if self.frames_out > expected {
            let excess = (self.frames_out - expected) as usize * self.channels;
            out.truncate(out.len().saturating_sub(excess));
            self.frames_out = expected;
        }
        Ok(())
    }

    fn run_chunk(
        &mut self,
        frames: usize,
        partial: Option<usize>,
        out: &mut Vec<f32>,
    ) -> Result<(), String> {
        let input = InterleavedSlice::new(&self.pending[..frames * self.channels], self.channels, frames)
            .map_err(|e| e.to_string())?;
        let out_frames = self.out_buf.len() / self.channels;
        let mut output = InterleavedSlice::new_mut(&mut self.out_buf, self.channels, out_frames)
            .map_err(|e| e.to_string())?;
        let indexing = Indexing {
            partial_len: partial,
            ..Indexing::default()
        };
        let (_, produced) = self
            .inner
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .map_err(|e| e.to_string())?;
        let skip = self.delay_left.min(produced);
        self.delay_left -= skip;
        let produced_samples = &self.out_buf[skip * self.channels..produced * self.channels];
        out.extend_from_slice(produced_samples);
        self.frames_out += (produced - skip) as u64;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_length_matches_ratio() {
        let mut rs = StreamResampler::new(44_100, 48_000, 2).unwrap();
        let input = vec![0.25f32; 44_100 * 2];
        let mut out = Vec::new();
        for chunk in input.chunks(777 * 2) {
            rs.process(chunk, &mut out).unwrap();
        }
        rs.finish(&mut out).unwrap();
        assert_eq!(out.len(), 48_000 * 2);
        // DC passes through (away from the edges).
        let mid = out[48_000];
        assert!((mid - 0.25).abs() < 1e-3, "{mid}");
    }
}
