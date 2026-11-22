//! Headless end-to-end tests of the engine through `NullSink`.
//! Fixtures are generated at test time (no binaries in the repo).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use kazzu_audio::output::f32_to_i24;
use kazzu_audio::{Engine, EngineHandle, NullSink};
use kazzu_core::{
    PlaybackEvent, PlaybackStatus, QueueItem, QueueItemId, RepeatMode, StreamSource, Track,
    TrackSource,
};

// ---------------------------------------------------------------- fixtures

fn fixture_dir() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "kazzu-audio-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Interleaved integer samples for a test signal.
type Pcm = Vec<i32>;

fn write_wav(path: &Path, rate: u32, channels: u16, bits: u16, samples: &[i32]) {
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: bits,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for &s in samples {
        if bits == 16 {
            w.write_sample(s as i16).unwrap();
        } else {
            w.write_sample(s).unwrap();
        }
    }
    w.finalize().unwrap();
}

/// Note: flacenc 0.5 writes a final *short* block that symphonia rejects
/// (`flac -t` warns about it too), so FLAC fixtures use lengths that are a
/// multiple of the 4096-frame block size. Files from libFLAC are fine.
fn write_flac(path: &Path, rate: u32, channels: u16, bits: u16, samples: &[i32]) {
    assert_eq!(samples.len() / usize::from(channels) % 4096, 0, "see note above");
    use flacenc::component::BitRepr;
    use flacenc::error::Verify;
    let config = flacenc::config::Encoder::default().into_verified().unwrap();
    let source = flacenc::source::MemSource::from_samples(
        samples,
        usize::from(channels),
        usize::from(bits),
        rate as usize,
    );
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size).unwrap();
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).unwrap();
    std::fs::write(path, sink.as_slice()).unwrap();
}

/// Stereo 16-bit signal whose value identifies (track, frame).
fn tagged_signal(track: i32, frames: usize) -> Pcm {
    let mut out = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let v = track * 10_000 + (n % 9_000) as i32 - 16_000;
        out.push(v);
        out.push(-v);
    }
    out
}

/// Stereo 16-bit signal where the left sample encodes the frame index.
fn index_signal(frames: usize) -> Pcm {
    let mut out = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let v = (n % 30_000) as i32 - 15_000;
        out.push(v);
        out.push(0);
    }
    out
}

fn item(path: &Path) -> QueueItem {
    let name = path.file_stem().unwrap().to_string_lossy().into_owned();
    let track = Track::new(
        name.clone(),
        name,
        TrackSource::Local {
            path: path.to_path_buf(),
        },
    );
    QueueItem::new(track, StreamSource::file(path))
}

fn to_i16(samples: &[f32]) -> Vec<i32> {
    samples
        .iter()
        .map(|&s| i32::from((s * 32_768.0).round().clamp(-32_768.0, 32_767.0) as i16))
        .collect()
}

// ------------------------------------------------------------ event utils

/// Collects events until `done` matches one (inclusive) or the timeout hits.
fn wait_for(
    events: &Receiver<PlaybackEvent>,
    timeout: Duration,
    mut done: impl FnMut(&PlaybackEvent) -> bool,
) -> Vec<PlaybackEvent> {
    let deadline = Instant::now() + timeout;
    let mut seen = Vec::new();
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        match events.recv_timeout(left) {
            Ok(ev) => {
                let stop = done(&ev);
                seen.push(ev);
                if stop {
                    return seen;
                }
            }
            Err(_) => break,
        }
    }
    panic!("timed out; events seen: {seen:#?}");
}

fn track_changed_to(ev: &PlaybackEvent, id: QueueItemId) -> bool {
    matches!(ev, PlaybackEvent::TrackChanged { item: Some(i), .. } if i.id == id)
}

fn ended_ids(events: &[PlaybackEvent]) -> Vec<QueueItemId> {
    events
        .iter()
        .filter_map(|e| match e {
            PlaybackEvent::TrackEnded { id } => Some(*id),
            _ => None,
        })
        .collect()
}

fn wait_status(engine: &EngineHandle, status: PlaybackStatus) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if engine.state().is_some_and(|s| s.status == status) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("status never became {status:?}");
}

// ------------------------------------------------------------------ tests

#[test]
fn gapless_queue_is_sample_exact_and_bit_perfect() {
    let dir = fixture_dir();
    let lengths = [4_427usize, 3_003, 5_000];
    let mut expected = Vec::new();
    let mut items = Vec::new();
    for (t, &frames) in lengths.iter().enumerate() {
        let pcm = tagged_signal(t as i32, frames);
        let path = dir.join(format!("t{t}.wav"));
        write_wav(&path, 44_100, 2, 16, &pcm);
        expected.extend_from_slice(&pcm);
        items.push(item(&path));
    }
    let ids: Vec<_> = items.iter().map(|i| i.id).collect();

    let (sink, capture) = NullSink::new().with_speed(20.0).with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.load(items, 0);
    let events = wait_for(engine.events(), Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });

    assert_eq!(ended_ids(&events), ids, "every track ends, in order");
    let info = events
        .iter()
        .find_map(|e| match e {
            PlaybackEvent::OutputChanged { info } => Some(info.clone()),
            _ => None,
        })
        .unwrap();
    assert!(info.bit_perfect, "{info:?}");
    assert_eq!(info.sample_rate, 44_100);
    assert_eq!(capture.opens(), 1, "same format: device opened once (gapless)");

    let got = to_i16(&capture.samples());
    assert_eq!(got.len(), expected.len(), "no gaps, no duplicates");
    assert!(got == expected, "output differs from the concatenated sources");
}

#[test]
fn flac_24bit_hires_is_bit_perfect() {
    let dir = fixture_dir();
    let frames = 4_096 * 3;
    let mut state = 0x1234_5678u32;
    let pcm: Pcm = (0..frames * 2)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 8) as i32 - (1 << 23)
        })
        .collect();
    let path = dir.join("hires.flac");
    write_flac(&path, 96_000, 2, 24, &pcm);

    let (sink, capture) = NullSink::new().with_speed(20.0).with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.load(vec![item(&path)], 0);
    let events = wait_for(engine.events(), Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    let info = events
        .iter()
        .find_map(|e| match e {
            PlaybackEvent::OutputChanged { info } => Some(info.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no output opened: {events:#?}"));
    assert_eq!(info.sample_rate, 96_000);
    let source = info.source.unwrap();
    assert_eq!(source.bit_depth, Some(24));
    assert!(source.is_hi_res());

    let got: Vec<i32> = capture.samples().iter().map(|&s| f32_to_i24(s)).collect();
    assert_eq!(got.len(), pcm.len());
    assert!(got == pcm, "24-bit samples were altered");
}

fn check_seek(path: &Path, rate: u32) {
    let (sink, capture) = NullSink::new().with_speed(4.0).with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.load(vec![item(path)], 0);
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused);

    let target = Duration::from_micros(1_234_567);
    let target_frame = (target.as_secs_f64() * f64::from(rate)) as usize;
    engine.seek(target);
    // Let the flush complete and the ring refill before resuming.
    std::thread::sleep(Duration::from_millis(100));
    capture.clear();
    engine.play();
    wait_for(engine.events(), Duration::from_secs(5), |e| {
        matches!(e, PlaybackEvent::Position { position, .. } if *position > target + Duration::from_millis(50))
    });

    let got = to_i16(&capture.samples());
    assert!(got.len() >= 2);
    let first_index = (got[0] + 15_000) as usize;
    assert_eq!(first_index, target_frame % 30_000, "seek landed on the wrong frame");
    // And it continues contiguously.
    for (k, frame) in got.as_chunks::<2>().0.iter().take(1_000).enumerate() {
        assert_eq!((frame[0] + 15_000) as usize, (target_frame + k) % 30_000);
    }
    let state = engine.state().unwrap();
    assert!(state.position >= target, "{:?}", state.position);
}

#[test]
fn seek_is_sample_accurate_wav() {
    let dir = fixture_dir();
    let path = dir.join("seek.wav");
    write_wav(&path, 44_100, 2, 16, &index_signal(44_100 * 3));
    check_seek(&path, 44_100);
}

#[test]
fn seek_is_sample_accurate_flac() {
    let dir = fixture_dir();
    let path = dir.join("seek.flac");
    write_flac(&path, 44_100, 2, 16, &index_signal(4_096 * 33));
    check_seek(&path, 44_100);
}

#[test]
fn queue_navigation_and_track_end() {
    let dir = fixture_dir();
    let items: Vec<_> = (0..3)
        .map(|t| {
            let path = dir.join(format!("nav{t}.wav"));
            write_wav(&path, 44_100, 2, 16, &tagged_signal(t, 44_100 * 2));
            item(&path)
        })
        .collect();
    let (a, b, c) = (items[0].id, items[1].id, items[2].id);

    let sink = NullSink::new().with_speed(2.0);
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    let events = engine.events().clone();
    let t = Duration::from_secs(5);

    engine.load(items, 0);
    wait_for(&events, t, |e| track_changed_to(e, a));
    engine.next();
    wait_for(&events, t, |e| track_changed_to(e, b));
    engine.previous(); // near the start: goes back
    wait_for(&events, t, |e| track_changed_to(e, a));
    engine.jump_to(c);
    wait_for(&events, t, |e| track_changed_to(e, c));
    let tail = wait_for(&events, Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    assert_eq!(ended_ids(&tail), vec![c]);
    assert!(tail.iter().any(|e| matches!(
        e,
        PlaybackEvent::StatusChanged {
            status: PlaybackStatus::Stopped
        }
    )));
}

#[test]
fn repeat_one_replays_then_stops() {
    let dir = fixture_dir();
    let path = dir.join("loop.wav");
    let pcm = tagged_signal(1, 2_205);
    write_wav(&path, 44_100, 2, 16, &pcm);
    let it = item(&path);
    let id = it.id;

    let (sink, capture) = NullSink::new().with_speed(10.0).with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.set_repeat(RepeatMode::One);
    engine.load(vec![it], 0);
    let mut ends = 0;
    wait_for(engine.events(), Duration::from_secs(5), |e| {
        if matches!(e, PlaybackEvent::TrackEnded { id: x } if *x == id) {
            ends += 1;
        }
        ends == 2
    });
    engine.set_repeat(RepeatMode::Off);
    wait_for(engine.events(), Duration::from_secs(5), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    let got = to_i16(&capture.samples());
    assert!(got.len() >= pcm.len() * 3, "looped at least three times");
    assert_eq!(got.len() % pcm.len(), 0, "loops are whole and gapless");
    assert!(got.chunks_exact(pcm.len()).all(|c| c == pcm.as_slice()));
}

#[test]
fn enqueue_while_playing_continues_gapless() {
    let dir = fixture_dir();
    let pa = tagged_signal(0, 8_192);
    let pb = tagged_signal(2, 4_000);
    let a = dir.join("ea.flac");
    let b = dir.join("eb.wav");
    write_flac(&a, 48_000, 2, 16, &pa);
    write_wav(&b, 48_000, 2, 16, &pb);

    let (sink, capture) = NullSink::new().with_speed(2.0).with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.load(vec![item(&a)], 0);
    std::thread::sleep(Duration::from_millis(20));
    engine.enqueue(vec![item(&b)]);
    wait_for(engine.events(), Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    let mut expected = pa;
    expected.extend_from_slice(&pb);
    assert!(to_i16(&capture.samples()) == expected);
}

#[test]
fn resamples_only_when_device_refuses_rate() {
    let dir = fixture_dir();
    let path = dir.join("rs.wav");
    write_wav(&path, 44_100, 2, 16, &tagged_signal(0, 44_100));

    let (sink, capture) = NullSink::new()
        .with_speed(20.0)
        .with_rates(vec![48_000])
        .with_capture();
    let engine = Engine::builder().sink(sink).spawn().unwrap();
    engine.load(vec![item(&path)], 0);
    let events = wait_for(engine.events(), Duration::from_secs(15), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    let info = events
        .iter()
        .find_map(|e| match e {
            PlaybackEvent::OutputChanged { info } => Some(info.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(info.sample_rate, 48_000);
    assert_eq!(info.resampled_from, Some(44_100));
    assert!(!info.bit_perfect);
    assert_eq!(capture.frames(), 48_000);
}

#[test]
fn volume_and_pause_report_state() {
    let dir = fixture_dir();
    let path = dir.join("vol.wav");
    write_wav(&path, 44_100, 2, 16, &tagged_signal(0, 44_100 * 2));

    let engine = Engine::builder().sink(NullSink::new()).spawn().unwrap();
    engine.load(vec![item(&path)], 0);
    wait_status(&engine, PlaybackStatus::Playing);
    engine.set_volume(0.5);
    let events = wait_for(engine.events(), Duration::from_secs(5), |e| {
        matches!(e, PlaybackEvent::OutputChanged { info } if !info.bit_perfect)
    });
    assert!(
        events
            .iter()
            .any(|e| matches!(e, PlaybackEvent::VolumeChanged { volume } if *volume == 0.5))
    );
    engine.toggle_pause();
    wait_status(&engine, PlaybackStatus::Paused);
    let p1 = engine.state().unwrap().position;
    std::thread::sleep(Duration::from_millis(150));
    let p2 = engine.state().unwrap().position;
    assert_eq!(p1, p2, "position frozen while paused");
    engine.clear();
    wait_status(&engine, PlaybackStatus::Stopped);
    assert!(engine.state().unwrap().queue.is_empty());
}

#[test]
fn unplayable_item_is_reported_and_skipped() {
    let dir = fixture_dir();
    let bad = dir.join("missing.flac");
    let good = dir.join("good.wav");
    write_wav(&good, 44_100, 2, 16, &tagged_signal(0, 2_000));
    let (bad_item, good_item) = (item(&bad), item(&good));
    let (bad_id, good_id) = (bad_item.id, good_item.id);

    let engine = Engine::builder()
        .sink(NullSink::new().with_speed(10.0))
        .spawn()
        .unwrap();
    engine.load(vec![bad_item, good_item], 0);
    let events = wait_for(engine.events(), Duration::from_secs(5), |e| {
        matches!(e, PlaybackEvent::QueueFinished)
    });
    assert!(
        events
            .iter()
            .any(|e| matches!(e, PlaybackEvent::Error { id: Some(x), .. } if *x == bad_id))
    );
    assert_eq!(ended_ids(&events), vec![good_id]);
}
