//! Sound card output through cpal.
//!
//! Bit-perfect strategy: open the device at the source's native rate and
//! channel count, with a sample format at least as deep as the source. Only
//! if the device does not support the rate does the engine resample.
//! Note: on Linux the "default" device usually goes through PipeWire/Pulse,
//! which may resample/mix on its own; choose a `hw:` device for a truly
//! bit-perfect path.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, Device, ErrorKind, Sample, SampleFormat, SizedSample, StreamConfig,
    SupportedStreamConfigRange, I24,
};

use crate::output::{
    AudioSink, OutputConfig, OutputRequest, OutputSample, OutputSampleFormat, Renderer, f32_to_i24,
};

impl OutputSample for I24 {
    const SILENCE: Self = I24::EQUILIBRIUM;
    #[inline]
    fn from_f32(sample: f32) -> Self {
        I24::new_unchecked(f32_to_i24(sample))
    }
}

/// An output device as listed by [`CpalSink::devices`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDevice {
    /// Stable id to pass to [`CpalSink::with_device`].
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// cpal-backed [`AudioSink`].
pub struct CpalSink {
    device_id: Option<String>,
    stream: Option<cpal::Stream>,
    failed: Arc<AtomicBool>,
}

impl Default for CpalSink {
    fn default() -> Self {
        Self::new()
    }
}

impl CpalSink {
    /// Uses the system default output device.
    pub fn new() -> Self {
        Self {
            device_id: None,
            stream: None,
            failed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Uses a specific device (id from [`CpalSink::devices`]).
    pub fn with_device(id: impl Into<String>) -> Self {
        Self {
            device_id: Some(id.into()),
            ..Self::new()
        }
    }

    /// Lists output devices of the default host.
    pub fn devices() -> Vec<OutputDevice> {
        let host = cpal::default_host();
        let default_id = host.default_output_device().and_then(|d| d.id().ok());
        let Ok(devices) = host.output_devices() else {
            return Vec::new();
        };
        devices
            .filter_map(|d| {
                let id = d.id().ok()?;
                let name = d
                    .description()
                    .map(|desc| desc.name().to_owned())
                    .unwrap_or_else(|_| id.to_string());
                Some(OutputDevice {
                    is_default: Some(&id) == default_id.as_ref(),
                    id: id.to_string(),
                    name,
                })
            })
            .collect()
    }

    fn device(&self) -> Result<Device, String> {
        let host = cpal::default_host();
        match &self.device_id {
            None => host
                .default_output_device()
                .ok_or_else(|| "no default output device".to_owned()),
            Some(wanted) => host
                .output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.id().is_ok_and(|id| id.to_string() == *wanted))
                .ok_or_else(|| format!("output device not found: {wanted}")),
        }
    }
}

impl AudioSink for CpalSink {
    fn open(
        &mut self,
        request: OutputRequest,
        make_renderer: &mut dyn FnMut(&OutputConfig) -> Renderer,
    ) -> Result<OutputConfig, String> {
        self.close();
        self.failed.store(false, Ordering::Release);

        let device = self.device()?;
        let device_name = device
            .description()
            .map(|d| d.name().to_owned())
            .unwrap_or_else(|_| "audio device".to_owned());
        let ranges: Vec<SupportedStreamConfigRange> = device
            .supported_output_configs()
            .map_err(|e| e.to_string())?
            .collect();
        let (config, format) = choose_config(&ranges, request).or_else(|| {
            let default = device.default_output_config().ok()?;
            Some((default.config(), default.sample_format()))
        })
        .ok_or_else(|| "device reports no usable output configuration".to_owned())?;

        let output = OutputConfig {
            device: device_name,
            sample_rate: config.sample_rate,
            channels: config.channels,
            sample_format: map_format(format),
        };
        let renderer = make_renderer(&output);

        let stream = match format {
            SampleFormat::I16 => build::<i16>(&device, config, renderer, &self.failed),
            SampleFormat::I24 => build::<I24>(&device, config, renderer, &self.failed),
            SampleFormat::I32 => build::<i32>(&device, config, renderer, &self.failed),
            SampleFormat::F64 => build::<f64>(&device, config, renderer, &self.failed),
            _ => build::<f32>(&device, config, renderer, &self.failed),
        }?;
        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(stream);
        Ok(output)
    }

    fn close(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
        }
    }

    fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}

fn build<T>(
    device: &Device,
    config: StreamConfig,
    mut renderer: Renderer,
    failed: &Arc<AtomicBool>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + OutputSample,
{
    let failed = Arc::clone(failed);
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _info: &cpal::OutputCallbackInfo| {
                renderer.render(data);
            },
            move |err: cpal::Error| match err.kind() {
                ErrorKind::DeviceNotAvailable
                | ErrorKind::StreamInvalidated
                | ErrorKind::DeviceChanged => {
                    log::warn!("audio stream failed: {err}");
                    failed.store(true, Ordering::Release);
                }
                _ => log::debug!("audio stream: {err}"),
            },
            None,
        )
        .map_err(|e| e.to_string())
}

fn map_format(format: SampleFormat) -> OutputSampleFormat {
    match format {
        SampleFormat::I16 => OutputSampleFormat::I16,
        SampleFormat::I24 => OutputSampleFormat::I24,
        SampleFormat::I32 => OutputSampleFormat::I32,
        SampleFormat::F32 => OutputSampleFormat::F32,
        _ => OutputSampleFormat::Other,
    }
}

/// Preference of sample formats for a source of the given depth.
fn format_rank(format: SampleFormat, source_bits: Option<u32>) -> Option<u8> {
    let order: &[SampleFormat] = match source_bits {
        // 16-bit (or less) lossless: native int first; deeper ones are exact too.
        Some(bits) if bits <= 16 => &[
            SampleFormat::I16,
            SampleFormat::I32,
            SampleFormat::I24,
            SampleFormat::F32,
        ],
        // 24/32-bit lossless.
        Some(_) => &[
            SampleFormat::I32,
            SampleFormat::I24,
            SampleFormat::F32,
            SampleFormat::I16,
        ],
        // Lossy codecs decode to float.
        None => &[
            SampleFormat::F32,
            SampleFormat::I32,
            SampleFormat::I24,
            SampleFormat::I16,
        ],
    };
    order.iter().position(|f| *f == format).map(|p| p as u8)
}

/// Picks the best stream config: native rate > integer multiple > common
/// rates; requested channels > stereo; deepest suitable sample format.
pub(crate) fn choose_config(
    ranges: &[SupportedStreamConfigRange],
    request: OutputRequest,
) -> Option<(StreamConfig, SampleFormat)> {
    let rates = [
        request.sample_rate,
        request.sample_rate * 2,
        request.sample_rate * 4,
        48_000,
        44_100,
        96_000,
        192_000,
    ];
    let channel_prefs = [request.channels, 2];

    for channels in channel_prefs {
        for rate in rates {
            let best = ranges
                .iter()
                .filter(|r| r.channels() == channels && r.contains_rate(rate))
                .filter_map(|r| {
                    format_rank(r.sample_format(), request.bits_per_sample).map(|rank| (rank, r))
                })
                .min_by_key(|(rank, _)| *rank);
            if let Some((_, range)) = best {
                return Some((
                    StreamConfig {
                        channels,
                        sample_rate: rate,
                        buffer_size: BufferSize::Default,
                    },
                    range.sample_format(),
                ));
            }
        }
    }
    None
}
