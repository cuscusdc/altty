//! The playback engine: one control/decode thread plus the device callback.
//!
//! ```text
//!  UI ──PlaybackCommand──▶ engine thread ──f32 frames──▶ SPSC ring ──▶ device callback
//!     ◀──PlaybackEvent───  (queue, decode,              (lock-free)    (Renderer: volume,
//!                           resample?, DSP)                              format conversion)
//! ```
//!
//! Gapless: when a track's decoder reaches EOF the next one is opened at
//! once and its samples are appended to the same ring, so the device never
//! sees a gap. Track boundaries are tracked as frame counters ("segments")
//! and compared to the number of frames the callback has consumed, which
//! also yields the playback position.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError, TrySendError};
use kazzu_core::{
    AudioFormat, Error, OutputInfo, PlaybackCommand, PlaybackEvent, PlaybackState, PlaybackStatus,
    QueueItem, QueueItemId, RepeatMode, Result,
};
use ringbuf::traits::{Observer, Producer, Split};
use ringbuf::{HeapProd, HeapRb};

use crate::cpal_sink::CpalSink;
use crate::decoder::{SourceSpec, TrackDecoder, append_remapped, frames_to_duration};
use crate::dsp::{Bypass, Dsp};
use crate::output::{AudioSink, OutputConfig, OutputRequest, Renderer, Shared};
use crate::queue::Queue;
use crate::resample::StreamResampler;

/// Wake-up period of the engine thread when it has nothing to decode.
const TICK: Duration = Duration::from_millis(5);
/// "Previous" restarts the current track when past this position.
const PREVIOUS_RESTART: Duration = Duration::from_secs(3);
/// Max packets decoded per loop iteration (keeps commands responsive).
const PUMP_BUDGET: usize = 32;
const EVENT_CAPACITY: usize = 4096;
/// Max time to wait for the device callback to acknowledge a flush.
const FLUSH_WAIT: Duration = Duration::from_millis(500);

/// Engine tuning.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Initial linear volume (1.0 = unity, bit-perfect).
    pub volume: f32,
    /// Length of the ring buffer between decoder and device.
    pub buffer: Duration,
    /// Interval of [`PlaybackEvent::Position`] while playing.
    pub position_interval: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            buffer: Duration::from_millis(500),
            position_interval: Duration::from_millis(100),
        }
    }
}

/// Entry point. See [`Engine::spawn`] and [`Engine::builder`].
pub struct Engine;

impl Engine {
    /// Spawns an engine on the default sound card (cpal) with default
    /// settings. The device is opened lazily, on first playback.
    pub fn spawn() -> Result<EngineHandle> {
        Self::builder().spawn()
    }

    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }
}

/// Configures and spawns an engine.
#[derive(Default)]
pub struct EngineBuilder {
    sink: Option<Box<dyn AudioSink>>,
    dsp: Option<Box<dyn Dsp>>,
    config: EngineConfig,
}

impl EngineBuilder {
    /// Output to use (default: [`CpalSink`] on the default device).
    pub fn sink(mut self, sink: impl AudioSink + 'static) -> Self {
        self.sink = Some(Box::new(sink));
        self
    }

    /// DSP stage (default: [`Bypass`]).
    pub fn dsp(mut self, dsp: impl Dsp + 'static) -> Self {
        self.dsp = Some(Box::new(dsp));
        self
    }

    pub fn config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn spawn(self) -> Result<EngineHandle> {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (ev_tx, ev_rx) = crossbeam_channel::bounded(EVENT_CAPACITY);
        let sink = self.sink.unwrap_or_else(|| Box::new(CpalSink::new()));
        let dsp = self.dsp.unwrap_or_else(|| Box::new(Bypass));
        let config = self.config;
        let thread = std::thread::Builder::new()
            .name("kazzu-engine".into())
            .spawn(move || Core::new(cmd_rx, ev_tx, sink, dsp, config).run())?;
        Ok(EngineHandle {
            tx: cmd_tx,
            events: ev_rx,
            thread: Some(thread),
        })
    }
}

enum Msg {
    Command(PlaybackCommand),
    SetDsp(Box<dyn Dsp>),
    State(Sender<PlaybackState>),
    Shutdown,
}

/// Handle to a running engine. Dropping it stops the engine thread.
pub struct EngineHandle {
    tx: Sender<Msg>,
    events: Receiver<PlaybackEvent>,
    thread: Option<JoinHandle<()>>,
}

impl EngineHandle {
    /// Sends any command.
    pub fn send(&self, command: PlaybackCommand) {
        let _ = self.tx.send(Msg::Command(command));
    }

    /// Event stream. The receiver can be cloned; events are dropped (not
    /// queued forever) if nobody reads them.
    pub fn events(&self) -> &Receiver<PlaybackEvent> {
        &self.events
    }

    /// Snapshot of the full player state.
    pub fn state(&self) -> Option<PlaybackState> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.tx.send(Msg::State(tx)).ok()?;
        rx.recv_timeout(Duration::from_secs(2)).ok()
    }

    /// Replaces the DSP stage (equalizer etc.).
    pub fn set_dsp(&self, dsp: impl Dsp + 'static) {
        let _ = self.tx.send(Msg::SetDsp(Box::new(dsp)));
    }

    pub fn load(&self, items: Vec<QueueItem>, start: usize) {
        self.send(PlaybackCommand::Load { items, start });
    }

    pub fn enqueue(&self, items: Vec<QueueItem>) {
        self.send(PlaybackCommand::Enqueue { items });
    }

    pub fn play_next(&self, items: Vec<QueueItem>) {
        self.send(PlaybackCommand::PlayNext { items });
    }

    pub fn play(&self) {
        self.send(PlaybackCommand::Play);
    }

    pub fn pause(&self) {
        self.send(PlaybackCommand::Pause);
    }

    pub fn toggle_pause(&self) {
        self.send(PlaybackCommand::TogglePause);
    }

    pub fn stop(&self) {
        self.send(PlaybackCommand::Stop);
    }

    pub fn seek(&self, position: Duration) {
        self.send(PlaybackCommand::Seek { position });
    }

    pub fn set_volume(&self, volume: f32) {
        self.send(PlaybackCommand::SetVolume { volume });
    }

    pub fn next(&self) {
        self.send(PlaybackCommand::Next);
    }

    pub fn previous(&self) {
        self.send(PlaybackCommand::Previous);
    }

    pub fn jump_to(&self, id: QueueItemId) {
        self.send(PlaybackCommand::JumpTo { id });
    }

    pub fn remove(&self, id: QueueItemId) {
        self.send(PlaybackCommand::Remove { id });
    }

    pub fn clear(&self) {
        self.send(PlaybackCommand::Clear);
    }

    pub fn set_repeat(&self, mode: RepeatMode) {
        self.send(PlaybackCommand::SetRepeat { mode });
    }

    pub fn set_shuffle(&self, enabled: bool) {
        self.send(PlaybackCommand::SetShuffle { enabled });
    }

    /// Stops the engine and waits for its thread.
    pub fn shutdown(mut self) {
        self.stop_thread();
    }

    fn stop_thread(&mut self) {
        let _ = self.tx.send(Msg::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for EngineHandle {
    fn drop(&mut self) {
        self.stop_thread();
    }
}

// ---------------------------------------------------------------------------
// Engine thread internals
// ---------------------------------------------------------------------------

/// The open output stream and the producer side of its ring.
struct Output {
    config: OutputConfig,
    /// Source spec the output was opened for.
    request: OutputRequest,
    shared: Arc<Shared>,
    prod: HeapProd<f32>,
    /// Frames pushed into the ring since the stream was opened.
    written: u64,
    flush_gen: u64,
}

impl Output {
    fn flush_pending(&self) -> bool {
        self.shared.flush_ack.load(Ordering::Acquire) != self.flush_gen
    }

    fn played(&self) -> u64 {
        self.shared.frames_played.load(Ordering::Acquire)
    }

    /// Can a source with `spec` be appended without reopening (gapless)?
    fn accepts(&self, spec: SourceSpec) -> bool {
        let same_source = self.request.sample_rate == spec.sample_rate
            && self.request.channels == spec.channels
            && self.request.bits_per_sample.unwrap_or(0) >= spec.bits_per_sample.unwrap_or(0);
        let native = self.config.sample_rate == spec.sample_rate
            && channels_compatible(spec.channels, self.config.channels)
            && depth_ok(self.config.sample_format.bits(), spec.bits_per_sample);
        same_source || native
    }
}

fn channels_compatible(source: u16, output: u16) -> bool {
    source == output || (source == 1 && output == 2)
}

/// Whether a device of `device_bits` (None = float) carries a source of
/// `source_bits` without losing precision.
fn depth_ok(device_bits: Option<u32>, source_bits: Option<u32>) -> bool {
    match (device_bits, source_bits) {
        // f32 is exact up to 24-bit integers; lossy sources are float anyway.
        (None, s) => s.is_none_or(|b| b <= 24),
        (Some(_), None) => true,
        (Some(d), Some(s)) => d >= s,
    }
}

/// A track being decoded into the ring.
struct Decoding {
    id: QueueItemId,
    decoder: TrackDecoder,
    resampler: Option<StreamResampler>,
    out_channels: usize,
    /// Processed samples (output format) not yet in the ring.
    pcm: Vec<f32>,
    pcm_pos: usize,
    raw: Vec<f32>,
    mapped: Vec<f32>,
    eof: bool,
}

impl Decoding {
    fn new(id: QueueItemId, decoder: TrackDecoder, out: &OutputConfig) -> Result<Self> {
        let spec = decoder.spec();
        let resampler = if spec.sample_rate != out.sample_rate {
            Some(
                StreamResampler::new(spec.sample_rate, out.sample_rate, out.channels)
                    .map_err(Error::Output)?,
            )
        } else {
            None
        };
        Ok(Self {
            id,
            decoder,
            resampler,
            out_channels: usize::from(out.channels),
            pcm: Vec::new(),
            pcm_pos: 0,
            raw: Vec::new(),
            mapped: Vec::new(),
            eof: false,
        })
    }

    fn has_pending(&self) -> bool {
        self.pcm_pos < self.pcm.len()
    }

    /// Decodes one packet into `pcm` (channel map → resample → DSP).
    fn fill(&mut self, dsp: &mut dyn Dsp) -> Result<()> {
        if !self.has_pending() {
            self.pcm.clear();
            self.pcm_pos = 0;
        }
        let start = self.pcm.len();
        self.raw.clear();
        let more = self.decoder.decode_next(&mut self.raw)?;

        let src_channels = usize::from(self.decoder.spec().channels);
        let stage: &[f32] = if src_channels == self.out_channels {
            &self.raw
        } else {
            self.mapped.clear();
            append_remapped(&self.raw, src_channels, self.out_channels, &mut self.mapped);
            &self.mapped
        };
        match &mut self.resampler {
            Some(rs) => {
                rs.process(stage, &mut self.pcm).map_err(Error::Output)?;
                if !more {
                    rs.finish(&mut self.pcm).map_err(Error::Output)?;
                }
            }
            None => self.pcm.extend_from_slice(stage),
        }
        if !dsp.is_bypassed() && self.pcm.len() > start {
            dsp.process(&mut self.pcm[start..]);
        }
        if !more {
            self.eof = true;
        }
        Ok(())
    }

    /// Pushes whole frames into the ring. Returns frames pushed.
    fn push(&mut self, prod: &mut HeapProd<f32>) -> u64 {
        let vacant = prod.vacant_len();
        let room = vacant - vacant % self.out_channels;
        let n = room.min(self.pcm.len() - self.pcm_pos);
        if n == 0 {
            return 0;
        }
        let pushed = prod.push_slice(&self.pcm[self.pcm_pos..self.pcm_pos + n]);
        self.pcm_pos += pushed;
        (pushed / self.out_channels) as u64
    }
}

/// A track's span inside the ring, in output frames.
struct Segment {
    id: QueueItemId,
    start: u64,
    end: Option<u64>,
    /// Track position at `start` (non-zero after a seek).
    offset: Duration,
    rate: u32,
    duration: Option<Duration>,
    format: AudioFormat,
    source_channels: u16,
    resampled_from: Option<u32>,
}

struct Core {
    rx: Receiver<Msg>,
    events: Sender<PlaybackEvent>,
    sink: Box<dyn AudioSink>,
    dsp: Box<dyn Dsp>,
    config: EngineConfig,
    queue: Queue,
    status: PlaybackStatus,
    volume: f32,
    out: Option<Output>,
    dec: Option<Decoding>,
    /// Last decoded item when nothing followed it (resumes gapless if the
    /// queue grows before playback drains).
    tail: Option<QueueItemId>,
    /// Next track, opened but waiting for the ring to drain because it
    /// needs a different output format.
    pending_reopen: Option<(QueueItemId, TrackDecoder)>,
    segments: VecDeque<Segment>,
    last_position: Instant,
    running: bool,
}

impl Core {
    fn new(
        rx: Receiver<Msg>,
        events: Sender<PlaybackEvent>,
        sink: Box<dyn AudioSink>,
        dsp: Box<dyn Dsp>,
        config: EngineConfig,
    ) -> Self {
        let volume = config.volume.clamp(0.0, 1.0);
        Self {
            rx,
            events,
            sink,
            dsp,
            config,
            queue: Queue::new(),
            status: PlaybackStatus::Stopped,
            volume,
            out: None,
            dec: None,
            tail: None,
            pending_reopen: None,
            segments: VecDeque::new(),
            last_position: Instant::now(),
            running: true,
        }
    }

    fn run(mut self) {
        while self.running {
            loop {
                match self.rx.try_recv() {
                    Ok(msg) => self.handle(msg),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.running = false;
                        break;
                    }
                }
            }
            if !self.running {
                break;
            }
            let worked = self.pump();
            self.update();
            if !worked {
                match self.rx.recv_timeout(TICK) {
                    Ok(msg) => self.handle(msg),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => self.running = false,
                }
            }
        }
        self.sink.close();
    }

    fn emit(&self, event: PlaybackEvent) {
        match self.events.try_send(event) {
            Ok(()) | Err(TrySendError::Disconnected(_)) => {}
            Err(TrySendError::Full(_)) => log::trace!("event channel full, event dropped"),
        }
    }

    fn handle(&mut self, msg: Msg) {
        match msg {
            Msg::Command(cmd) => self.command(cmd),
            Msg::SetDsp(mut dsp) => {
                if let Some(out) = &self.out {
                    dsp.configure(out.config.sample_rate, out.config.channels);
                }
                self.dsp = dsp;
                self.emit_output_info();
            }
            Msg::State(reply) => {
                let _ = reply.send(self.snapshot());
            }
            Msg::Shutdown => self.running = false,
        }
    }

    fn command(&mut self, cmd: PlaybackCommand) {
        match cmd {
            PlaybackCommand::Load { items, start } => {
                // The device stays open: start_item flushes, and only reopens
                // it if the new track needs another format.
                self.reset_pipeline();
                self.queue.load(items, start);
                self.emit_queue();
                match self.queue.current_id() {
                    Some(id) => {
                        if self.start_item(id, Duration::ZERO) {
                            self.set_status(PlaybackStatus::Playing);
                        }
                    }
                    None => self.stop(),
                }
            }
            PlaybackCommand::Enqueue { items } => {
                let was_empty = self.queue.is_empty();
                self.queue.enqueue(items);
                if was_empty && let Some(first) = self.queue.first_id() {
                    self.queue.set_current(first);
                }
                self.emit_queue();
                self.revalidate_prefetch();
            }
            PlaybackCommand::PlayNext { items } => {
                let was_empty = self.queue.is_empty();
                self.queue.play_next(items);
                if was_empty && let Some(first) = self.queue.first_id() {
                    self.queue.set_current(first);
                }
                self.emit_queue();
                self.revalidate_prefetch();
            }
            PlaybackCommand::Play => self.play(),
            PlaybackCommand::Pause => {
                if self.status == PlaybackStatus::Playing {
                    self.set_status(PlaybackStatus::Paused);
                }
            }
            PlaybackCommand::TogglePause => match self.status {
                PlaybackStatus::Playing => self.set_status(PlaybackStatus::Paused),
                _ => self.play(),
            },
            PlaybackCommand::Stop => self.stop(),
            PlaybackCommand::Seek { position } => self.seek(position),
            PlaybackCommand::SetVolume { volume } => {
                self.volume = if volume.is_finite() { volume.clamp(0.0, 1.0) } else { 1.0 };
                if let Some(out) = &self.out {
                    out.shared.volume.store(self.volume.to_bits(), Ordering::Relaxed);
                }
                self.emit(PlaybackEvent::VolumeChanged {
                    volume: self.volume,
                });
                self.emit_output_info();
            }
            PlaybackCommand::Next => {
                if let Some(cur) = self.queue.current_id() {
                    match self.queue.next_after(cur, false) {
                        Some(next) => self.select(next),
                        None => {
                            self.stop();
                            self.emit(PlaybackEvent::QueueFinished);
                        }
                    }
                }
            }
            PlaybackCommand::Previous => {
                if let Some(cur) = self.queue.current_id() {
                    let restart = self.position() > PREVIOUS_RESTART;
                    match self.queue.previous_before(cur) {
                        Some(prev) if !restart => self.select(prev),
                        _ => self.seek(Duration::ZERO),
                    }
                }
            }
            PlaybackCommand::JumpTo { id } => {
                if self.queue.get(id).is_some() {
                    let stopped = self.status == PlaybackStatus::Stopped;
                    if self.start_item(id, Duration::ZERO) && stopped {
                        self.set_status(PlaybackStatus::Playing);
                    }
                }
            }
            PlaybackCommand::Remove { id } => {
                let was_current = self.queue.remove(id);
                self.emit_queue();
                if was_current {
                    match self.queue.current_id() {
                        Some(next) if self.status != PlaybackStatus::Stopped => {
                            self.start_item(next, Duration::ZERO);
                        }
                        Some(_) => self.emit_track_changed(),
                        None => self.stop(),
                    }
                } else {
                    self.revalidate_prefetch();
                }
            }
            PlaybackCommand::Clear => {
                self.stop();
                self.queue.clear();
                self.emit_queue();
                self.emit(PlaybackEvent::TrackChanged {
                    item: None,
                    duration: None,
                });
            }
            PlaybackCommand::SetRepeat { mode } => {
                self.queue.set_repeat(mode);
                self.emit(PlaybackEvent::RepeatChanged { mode });
                self.revalidate_prefetch();
            }
            PlaybackCommand::SetShuffle { enabled } => {
                self.queue.set_shuffle(enabled);
                self.emit(PlaybackEvent::ShuffleChanged { enabled });
                self.emit_queue();
                self.revalidate_prefetch();
            }
        }
    }

    fn set_status(&mut self, status: PlaybackStatus) {
        if let Some(out) = &self.out {
            out.shared
                .paused
                .store(status != PlaybackStatus::Playing, Ordering::Release);
        }
        if status != self.status {
            self.status = status;
            self.emit(PlaybackEvent::StatusChanged { status });
        }
    }

    /// Skips to `id`: starts it if a track is active, otherwise only moves
    /// the cursor (the device is not opened while stopped).
    fn select(&mut self, id: QueueItemId) {
        if self.status == PlaybackStatus::Stopped {
            self.queue.set_current(id);
            self.emit_track_changed();
            self.emit_queue();
        } else {
            self.start_item(id, Duration::ZERO);
        }
    }

    fn play(&mut self) {
        match self.status {
            PlaybackStatus::Playing => {}
            PlaybackStatus::Paused | PlaybackStatus::Buffering if self.out.is_some() => {
                self.set_status(PlaybackStatus::Playing);
            }
            _ => {
                let id = self.queue.current_id().or_else(|| self.queue.first_id());
                if let Some(id) = id
                    && self.start_item(id, Duration::ZERO)
                {
                    self.set_status(PlaybackStatus::Playing);
                }
            }
        }
    }

    fn stop(&mut self) {
        self.stop_output();
        self.set_status(PlaybackStatus::Stopped);
        self.emit(PlaybackEvent::Position {
            position: Duration::ZERO,
            duration: None,
        });
    }

    /// Drops decoding state and releases the device.
    fn stop_output(&mut self) {
        self.reset_pipeline();
        if self.out.take().is_some() {
            self.sink.close();
        }
    }

    fn reset_pipeline(&mut self) {
        self.segments.clear();
        self.dec = None;
        self.pending_reopen = None;
        self.tail = None;
    }

    /// Discards everything buffered (ring included) without closing.
    fn flush(&mut self) {
        if let Some(out) = &mut self.out {
            out.flush_gen += 1;
            out.shared.flush_req.store(out.flush_gen, Ordering::Release);
        }
        self.reset_pipeline();
        self.dsp.reset();
    }

    fn seek(&mut self, position: Duration) {
        if let Some(cur) = self.queue.current_id()
            && self.status != PlaybackStatus::Stopped
        {
            self.start_item(cur, position);
        }
    }

    /// Flushes and starts decoding `id` at `position`. Unplayable items are
    /// reported and skipped. Returns false when nothing could be started.
    fn start_item(&mut self, id: QueueItemId, position: Duration) -> bool {
        self.flush();
        let mut candidate = Some(id);
        let mut attempts = 0;
        while let Some(id) = candidate {
            attempts += 1;
            if attempts > self.queue.len().max(1) {
                break;
            }
            let opened = self
                .queue
                .get(id)
                .ok_or_else(|| Error::NotFound(format!("queue item {id}")))
                .and_then(|item| TrackDecoder::open(&item.stream));
            let mut decoder = match opened {
                Ok(d) => d,
                Err(e) => {
                    self.emit_error(Some(id), &e);
                    candidate = self.queue.next_after(id, false);
                    continue;
                }
            };
            let mut offset = Duration::ZERO;
            if !position.is_zero() {
                match decoder.seek(position) {
                    Ok(actual) => offset = actual,
                    Err(e) => self.emit_error(Some(id), &e),
                }
            }
            if let Err(e) = self.ensure_output(decoder.spec()) {
                self.emit_error(Some(id), &e);
                self.stop_output();
                self.set_status(PlaybackStatus::Stopped);
                return false;
            }
            if let Err(e) = self.begin_segment(id, decoder, offset) {
                self.emit_error(Some(id), &e);
                candidate = self.queue.next_after(id, false);
                continue;
            }
            self.queue.set_current(id);
            self.emit_track_changed();
            self.emit_queue();
            self.emit_position();
            return true;
        }
        self.stop();
        self.emit(PlaybackEvent::QueueFinished);
        false
    }

    /// Opens (or reopens) the output if it cannot carry `spec` as is.
    fn ensure_output(&mut self, spec: SourceSpec) -> Result<()> {
        if self.out.as_ref().is_some_and(|o| o.accepts(spec)) {
            return Ok(());
        }
        if self.out.take().is_some() {
            self.sink.close();
        }
        let request = OutputRequest {
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            bits_per_sample: spec.bits_per_sample,
        };
        let volume = self.volume;
        let paused = self.status != PlaybackStatus::Playing;
        let buffer = self.config.buffer;
        let mut built: Option<(HeapProd<f32>, Arc<Shared>)> = None;
        let config = self
            .sink
            .open(request, &mut |cfg: &OutputConfig| {
                let frames = (buffer.as_secs_f64() * f64::from(cfg.sample_rate)) as usize;
                let channels = usize::from(cfg.channels.max(1));
                let ring = HeapRb::<f32>::new(frames.max(8192) * channels);
                let (prod, cons) = ring.split();
                let shared = Arc::new(Shared::new(volume));
                shared.paused.store(paused, Ordering::Release);
                built = Some((prod, Arc::clone(&shared)));
                Renderer::new(cons, shared, cfg.channels)
            })
            .map_err(Error::Output)?;
        let (prod, shared) =
            built.ok_or_else(|| Error::Output("sink did not create a renderer".into()))?;
        self.dsp.configure(config.sample_rate, config.channels);
        self.out = Some(Output {
            config,
            request,
            shared,
            prod,
            written: 0,
            flush_gen: 0,
        });
        Ok(())
    }

    /// Starts decoding `id` into the current output, appending a segment.
    fn begin_segment(&mut self, id: QueueItemId, decoder: TrackDecoder, offset: Duration) -> Result<()> {
        let out = self
            .out
            .as_ref()
            .ok_or_else(|| Error::Output("no output".into()))?;
        let spec = decoder.spec();
        let segment = Segment {
            id,
            start: out.written,
            end: None,
            offset,
            rate: out.config.sample_rate,
            duration: decoder.duration(),
            format: decoder.audio_format().clone(),
            source_channels: spec.channels,
            resampled_from: (spec.sample_rate != out.config.sample_rate).then_some(spec.sample_rate),
        };
        self.dec = Some(Decoding::new(id, decoder, &out.config)?);
        self.segments.push_back(segment);
        Ok(())
    }

    /// Feeds the ring. Returns true if any progress was made.
    fn pump(&mut self) -> bool {
        let Some(out) = self.out.as_mut() else {
            return false;
        };
        if out.flush_pending() {
            return false;
        }
        let mut worked = false;
        let mut failure = None;
        for _ in 0..PUMP_BUDGET {
            let Some(dec) = self.dec.as_mut() else {
                break;
            };
            if dec.has_pending() {
                let pushed = dec.push(&mut out.prod);
                out.written += pushed;
                if pushed == 0 {
                    break; // ring full
                }
                worked = true;
                continue;
            }
            if dec.eof {
                let finished = dec.id;
                if let Some(seg) = self.segments.back_mut()
                    && seg.end.is_none()
                {
                    seg.end = Some(out.written);
                }
                self.dec = None;
                self.prefetch_after(finished);
                return true;
            }
            if let Err(e) = dec.fill(self.dsp.as_mut()) {
                // Treat a broken stream as ended: play what was decoded.
                dec.eof = true;
                failure = Some((dec.id, e));
                worked = true;
                break;
            }
            worked = true;
        }
        if let Some((id, e)) = failure {
            self.emit_error(Some(id), &e);
        }

        if self.dec.is_none()
            && self.pending_reopen.is_none()
            && let Some(tail) = self.tail
            && self.queue.next_after(tail, true).is_some()
        {
            self.tail = None;
            self.prefetch_after(tail);
            worked = true;
        }
        worked
    }

    /// Opens the item following `prev` for gapless continuation.
    fn prefetch_after(&mut self, prev: QueueItemId) {
        let mut candidate = self.queue.next_after(prev, true);
        let mut attempts = 0;
        while let Some(id) = candidate {
            attempts += 1;
            if attempts > self.queue.len() {
                break;
            }
            let opened = self
                .queue
                .get(id)
                .ok_or_else(|| Error::NotFound(format!("queue item {id}")))
                .and_then(|item| TrackDecoder::open(&item.stream));
            match opened {
                Ok(decoder) => {
                    let gapless = self.out.as_ref().is_some_and(|o| o.accepts(decoder.spec()));
                    if gapless {
                        if let Err(e) = self.begin_segment(id, decoder, Duration::ZERO) {
                            self.emit_error(Some(id), &e);
                            candidate = self.queue.next_after(id, false);
                            continue;
                        }
                    } else {
                        self.pending_reopen = Some((id, decoder));
                    }
                    return;
                }
                Err(e) => {
                    self.emit_error(Some(id), &e);
                    candidate = self.queue.next_after(id, false);
                }
            }
        }
        self.tail = if attempts > self.queue.len() { None } else { Some(prev) };
    }

    /// Re-decodes from the current position if what was pre-decoded after
    /// the current track no longer matches the queue.
    fn revalidate_prefetch(&mut self) {
        let Some(cur) = self.queue.current_id() else {
            return;
        };
        if self.status == PlaybackStatus::Stopped || self.segments.is_empty() {
            return;
        }
        let prefetched = self
            .segments
            .get(1)
            .map(|s| s.id)
            .or_else(|| self.pending_reopen.as_ref().map(|(id, _)| *id));
        if let Some(p) = prefetched
            && Some(p) != self.queue.next_after(cur, true)
        {
            self.restart_current_exact(cur);
        }
    }

    /// Discards everything decoded ahead and resumes `cur` exactly at the
    /// frame the listener is hearing, so nothing is skipped or repeated.
    fn restart_current_exact(&mut self, cur: QueueItemId) {
        let Some(out) = self.out.as_mut() else {
            return;
        };
        out.flush_gen += 1;
        out.shared.flush_req.store(out.flush_gen, Ordering::Release);
        let deadline = Instant::now() + FLUSH_WAIT;
        while out.flush_pending() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        let heard = if out.flush_pending() {
            out.played()
        } else {
            out.shared.flushed_at.load(Ordering::Acquire)
        };
        // Retire the segments that were fully heard before the flush; the
        // audible one may already be the next (pre-decoded) track.
        let mut last_ended = None;
        while let Some(front) = self.segments.front()
            && front.end.is_some_and(|end| heard >= end)
        {
            let ended = front.id;
            self.segments.pop_front();
            self.emit(PlaybackEvent::TrackEnded { id: ended });
            last_ended = Some(ended);
        }
        match self.segments.front() {
            Some(front) => {
                let id = front.id;
                let position =
                    front.offset + frames_to_duration(heard.saturating_sub(front.start), front.rate);
                self.start_item(id, position);
            }
            None => {
                let after = last_ended.unwrap_or(cur);
                match self.queue.next_after(after, true) {
                    Some(next) => {
                        self.start_item(next, Duration::ZERO);
                    }
                    None => self.finish_queue(),
                }
            }
        }
    }

    /// Tracks segment boundaries and emits position/track events.
    fn update(&mut self) {
        if self.out.is_some() && self.sink.failed() {
            let position = self.position();
            self.emit(PlaybackEvent::Error {
                id: self.queue.current_id(),
                message: "audio device lost; reopening".into(),
            });
            self.stop_output();
            if let Some(cur) = self.queue.current_id() {
                self.start_item(cur, position);
            }
            return;
        }

        loop {
            let Some(out) = self.out.as_ref() else {
                return;
            };
            if out.flush_pending() {
                break;
            }
            let played = out.played();
            let Some(front) = self.segments.front() else {
                break;
            };
            let Some(end) = front.end else {
                break;
            };
            if played < end {
                break;
            }
            let ended = front.id;
            self.segments.pop_front();
            self.emit(PlaybackEvent::TrackEnded { id: ended });

            if let Some(next) = self.segments.front() {
                let id = next.id;
                self.queue.set_current(id);
                self.emit_track_changed();
                self.emit_queue();
            } else if let Some((id, decoder)) = self.pending_reopen.take() {
                let res = self
                    .ensure_output(decoder.spec())
                    .and_then(|()| self.begin_segment(id, decoder, Duration::ZERO));
                match res {
                    Ok(()) => {
                        self.queue.set_current(id);
                        self.emit_track_changed();
                        self.emit_queue();
                    }
                    Err(e) => {
                        self.emit_error(Some(id), &e);
                        if let Some(next) = self.queue.next_after(id, false) {
                            self.start_item(next, Duration::ZERO);
                        } else {
                            self.finish_queue();
                        }
                        return;
                    }
                }
            } else if self.dec.is_none() {
                self.finish_queue();
                return;
            }
        }

        if self.status == PlaybackStatus::Playing
            && self.last_position.elapsed() >= self.config.position_interval
        {
            self.emit_position();
        }
    }

    fn finish_queue(&mut self) {
        self.stop_output();
        self.set_status(PlaybackStatus::Stopped);
        self.emit(PlaybackEvent::Position {
            position: Duration::ZERO,
            duration: None,
        });
        self.emit(PlaybackEvent::QueueFinished);
    }

    fn position(&self) -> Duration {
        let (Some(out), Some(seg)) = (self.out.as_ref(), self.segments.front()) else {
            return Duration::ZERO;
        };
        let mut played = out.played();
        if let Some(end) = seg.end {
            played = played.min(end);
        }
        let frames = played.saturating_sub(seg.start);
        seg.offset + frames_to_duration(frames, seg.rate)
    }

    fn emit_position(&mut self) {
        self.last_position = Instant::now();
        let duration = self.segments.front().and_then(|s| s.duration);
        self.emit(PlaybackEvent::Position {
            position: self.position(),
            duration,
        });
    }

    fn emit_track_changed(&mut self) {
        let item = self.queue.current().cloned().map(Box::new);
        let duration = self.segments.front().and_then(|s| s.duration);
        self.emit(PlaybackEvent::TrackChanged { item, duration });
        self.emit_output_info();
    }

    fn emit_queue(&self) {
        self.emit(PlaybackEvent::QueueChanged {
            items: self.queue.ordered_items(),
            current: self.queue.current_position(),
        });
    }

    fn emit_output_info(&self) {
        if let Some(info) = self.output_info() {
            self.emit(PlaybackEvent::OutputChanged { info });
        }
    }

    fn emit_error(&self, id: Option<QueueItemId>, error: &Error) {
        log::warn!("playback error ({id:?}): {error}");
        self.emit(PlaybackEvent::Error {
            id,
            message: error.to_string(),
        });
    }

    fn output_info(&self) -> Option<OutputInfo> {
        let out = self.out.as_ref()?;
        let seg = self.segments.front();
        let bit_perfect = seg.is_some_and(|s| {
            s.resampled_from.is_none()
                && s.source_channels == out.config.channels
                && depth_ok(
                    out.config.sample_format.bits(),
                    s.format.bit_depth.map(u32::from),
                )
        }) && self.dsp.is_bypassed()
            && self.volume == 1.0;
        Some(OutputInfo {
            device: out.config.device.clone(),
            sample_rate: out.config.sample_rate,
            channels: out.config.channels,
            source: seg.map(|s| s.format.clone()),
            bit_perfect,
            resampled_from: seg.and_then(|s| s.resampled_from),
        })
    }

    fn snapshot(&self) -> PlaybackState {
        PlaybackState {
            status: self.status,
            queue: self.queue.ordered_items(),
            current: self.queue.current_position(),
            position: self.position(),
            duration: self.segments.front().and_then(|s| s.duration),
            volume: self.volume,
            repeat: self.queue.repeat(),
            shuffle: self.queue.shuffle(),
            output: self.output_info(),
        }
    }
}
