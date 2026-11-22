//! End-to-end tests of `StreamSource::Url` through a local HTTP server
//! (`support/server.rs`) and `NullSink`: ranged downloads, dropped
//! connections, expired URLs, network outage, seek, gapless, disk cache,
//! bandwidth cap, and that the engine thread never blocks on the network.

#[path = "support/server.rs"]
mod server;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use kazzu_audio::{CacheConfig, Engine, EngineConfig, EngineHandle, NullSink, StreamConfig, StreamEvent};
use kazzu_core::{
    PlaybackEvent, PlaybackStatus, ProviderKind, QueueItem, QueueItemId, StreamSource, Track,
    TrackSource,
};
use server::TestServer;

// ---------------------------------------------------------------- helpers

fn temp_dir() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "kazzu-stream-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn wav_bytes(rate: u32, samples: &[i32]) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::new());
    let mut w = hound::WavWriter::new(&mut cursor, spec).unwrap();
    for &s in samples {
        w.write_sample(s as i16).unwrap();
    }
    w.finalize().unwrap();
    cursor.into_inner()
}

/// Stereo 16-bit signal whose value identifies (track, frame).
fn tagged_signal(track: i32, frames: usize) -> Vec<i32> {
    let mut out = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let v = (track % 3) * 10_000 + (n % 9_000) as i32 - 16_000;
        out.push(v);
        out.push(-v);
    }
    out
}

/// Left sample encodes the frame index.
fn index_signal(frames: usize) -> Vec<i32> {
    let mut out = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        out.push((n % 30_000) as i32 - 15_000);
        out.push(0);
    }
    out
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)).unwrap()
}

fn remote_item(url: &str, video_id: &str, duration: Option<Duration>) -> QueueItem {
    let mut track = Track::new(
        video_id,
        video_id,
        TrackSource::Remote {
            provider: ProviderKind::YtMusic,
            id: video_id.into(),
        },
    );
    track.duration = duration;
    QueueItem::new(
        track,
        StreamSource::Url {
            url: url.to_owned(),
            headers: vec![("User-Agent".into(), "KazzuTest/1.0".into())],
            format: None,
        },
    )
}

/// Asserts the captured output equals `expected`, with a useful message.
#[track_caller]
fn assert_same(capture: &kazzu_audio::Capture, expected: &[i32], what: &str) {
    let got = to_i16(&capture.samples());
    if got != expected {
        let first = got.iter().zip(expected).position(|(a, b)| a != b);
        panic!(
            "{what}: got {} samples, expected {}; first difference at {first:?}",
            got.len(),
            expected.len()
        );
    }
}

fn to_i16(samples: &[f32]) -> Vec<i32> {
    samples
        .iter()
        .map(|&s| i32::from((s * 32_768.0).round().clamp(-32_768.0, 32_767.0) as i16))
        .collect()
}

/// Small chunks and read-ahead so even short test files need many requests.
fn small_net() -> StreamConfig {
    StreamConfig {
        request_chunk: 64 * 1024,
        read_ahead: 96 * 1024,
        retry_base: Duration::from_millis(20),
        retry_max: Duration::from_millis(100),
        read_timeout: Duration::from_secs(5),
        ..StreamConfig::default()
    }
}

fn engine_with(stream: StreamConfig, speed: f64) -> (EngineHandle, kazzu_audio::Capture) {
    let (sink, capture) = NullSink::new().with_speed(speed).with_capture();
    let engine = Engine::builder()
        .sink(sink)
        .config(EngineConfig {
            stream,
            ..EngineConfig::default()
        })
        .spawn()
        .unwrap();
    (engine, capture)
}

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

fn ended_ids(events: &[PlaybackEvent]) -> Vec<QueueItemId> {
    events
        .iter()
        .filter_map(|e| match e {
            PlaybackEvent::TrackEnded { id } => Some(*id),
            _ => None,
        })
        .collect()
}

fn statuses(events: &[PlaybackEvent]) -> Vec<PlaybackStatus> {
    events
        .iter()
        .filter_map(|e| match e {
            PlaybackEvent::StatusChanged { status } => Some(*status),
            _ => None,
        })
        .collect()
}

fn finished(e: &PlaybackEvent) -> bool {
    matches!(e, PlaybackEvent::QueueFinished)
}

fn wait_status(engine: &EngineHandle, status: PlaybackStatus, timeout: Duration) {
    let deadline = Instant::now() + timeout;
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
fn remote_wav_is_bit_exact_over_ranged_requests() {
    let server = TestServer::start();
    let pcm = tagged_signal(1, 44_100 * 2);
    let url = server.add("/a.wav", wav_bytes(44_100, &pcm), "audio/wav");
    let item = remote_item(&url, "a", None);
    let id = item.id;

    let (engine, capture) = engine_with(small_net(), 20.0);
    engine.load(vec![item], 0);
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);

    assert_eq!(ended_ids(&events), vec![id]);
    let st = statuses(&events);
    assert_eq!(st.first(), Some(&PlaybackStatus::Buffering), "opens in Buffering: {st:?}");
    assert!(st.contains(&PlaybackStatus::Playing), "{st:?}");
    let info = events
        .iter()
        .find_map(|e| match e {
            PlaybackEvent::OutputChanged { info } => Some(info.clone()),
            _ => None,
        })
        .unwrap();
    assert!(info.bit_perfect, "{info:?}");
    assert_same(&capture, &pcm, "streamed output differs from the source");

    let hits = server.hits("/a.wav");
    assert!(hits.len() >= 4, "bounded ranges: {} requests", hits.len());
    for h in &hits {
        let (a, b) = h.range.expect("every request carries Range");
        assert!(b.is_some_and(|b| b - a < 64 * 1024), "bounded range {a}-{b:?}");
        assert_eq!(h.user_agent.as_deref(), Some("KazzuTest/1.0"), "item headers are sent");
    }
}

#[test]
fn dropped_connections_resume_without_glitches() {
    let server = TestServer::start();
    let pcm = tagged_signal(2, 44_100 * 2);
    let url = server.add("/drop.wav", wav_bytes(44_100, &pcm), "audio/wav");
    {
        let mut st = server.state();
        st.drop_after = 20_000;
        st.drops = 6;
    }
    let (engine, capture) = engine_with(small_net(), 20.0);
    engine.load(vec![remote_item(&url, "drop", None)], 0);
    wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_same(&capture, &pcm, "resumed download altered the audio");
    assert!(server.hits("/drop.wav").len() >= 6 + 4);
}

#[test]
fn server_without_range_support_still_plays_and_seeks() {
    let server = TestServer::start();
    let rate = 44_100;
    let pcm = index_signal(rate as usize * 3);
    let url = server.add("/norange.wav", wav_bytes(rate, &pcm), "audio/wav");
    server.state().no_ranges = true;

    let (engine, capture) = engine_with(small_net(), 4.0);
    engine.load(vec![remote_item(&url, "norange", None)], 0);
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(5));
    let target = Duration::from_micros(1_234_567);
    let target_frame = (target.as_secs_f64() * f64::from(rate)) as usize;
    engine.seek(target);
    std::thread::sleep(Duration::from_millis(300));
    capture.clear();
    engine.play();
    wait_for(engine.events(), Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::Position { position, .. } if *position > target + Duration::from_millis(80))
    });
    let got = to_i16(&capture.samples());
    assert!(got.len() >= 2000);
    for (k, frame) in got.as_chunks::<2>().0.iter().take(1_000).enumerate() {
        assert_eq!((frame[0] + 15_000) as usize, (target_frame + k) % 30_000, "frame {k}");
    }
}

#[test]
fn remote_seek_is_sample_accurate() {
    let server = TestServer::start();
    let rate = 44_100;
    let pcm = index_signal(rate as usize * 6);
    let url = server.add("/seek.wav", wav_bytes(rate, &pcm), "audio/wav");

    let (engine, capture) = engine_with(small_net(), 4.0);
    engine.load(vec![remote_item(&url, "seek", None)], 0);
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(5));
    // Far beyond the read-ahead: needs a new ranged request.
    let target = Duration::from_micros(4_321_000);
    let target_frame = (target.as_secs_f64() * f64::from(rate)) as usize;
    engine.seek(target);
    std::thread::sleep(Duration::from_millis(300));
    capture.clear();
    engine.play();
    wait_for(engine.events(), Duration::from_secs(10), |e| {
        matches!(e, PlaybackEvent::Position { position, .. } if *position > target + Duration::from_millis(80))
    });
    let got = to_i16(&capture.samples());
    assert!(got.len() >= 2000);
    assert_eq!((got[0] + 15_000) as usize, target_frame % 30_000, "seek landed on the wrong frame");
    for (k, frame) in got.as_chunks::<2>().0.iter().take(1_000).enumerate() {
        assert_eq!((frame[0] + 15_000) as usize, (target_frame + k) % 30_000);
    }
    assert!(engine.state().unwrap().position >= target);
    // Some request started near the target instead of reading everything.
    let wav_offset = 44 + target_frame as u64 * 4;
    assert!(
        server
            .hits("/seek.wav")
            .iter()
            .any(|h| h.range.is_some_and(|(a, _)| a <= wav_offset && wav_offset - a < 128 * 1024)),
        "{:?}",
        server.hits("/seek.wav")
    );
}

#[test]
fn gapless_across_remote_tracks() {
    let server = TestServer::start();
    let lengths = [30_011usize, 25_007, 40_000];
    let mut expected = Vec::new();
    let mut items = Vec::new();
    for (t, &frames) in lengths.iter().enumerate() {
        let pcm = tagged_signal(t as i32, frames);
        let url = server.add(&format!("/g{t}.wav"), wav_bytes(44_100, &pcm), "audio/wav");
        expected.extend_from_slice(&pcm);
        items.push(remote_item(&url, &format!("g{t}"), None));
    }
    server.state().latency = Duration::from_millis(30);
    let ids: Vec<_> = items.iter().map(|i| i.id).collect();
    let (engine, capture) = engine_with(small_net(), 5.0);
    engine.load(items, 0);
    let events = wait_for(engine.events(), Duration::from_secs(20), finished);
    assert_eq!(ended_ids(&events), ids);
    assert_eq!(capture.opens(), 1, "same format: one device open");
    let got = to_i16(&capture.samples());
    assert_eq!(got.len(), expected.len(), "no gaps, no duplicates");
    assert!(got == expected, "remote gapless output differs");
}

#[test]
fn expired_url_is_refreshed_and_resumes_exactly() {
    let server = TestServer::start();
    let pcm = tagged_signal(3, 44_100 * 3);
    let bytes = wav_bytes(44_100, &pcm);
    let url = server.add("/old.wav", bytes.clone(), "audio/wav");
    let fresh = server.add("/new.wav", bytes, "audio/wav");
    // The signed URL dies after 4 requests (mid-track).
    server.state().expire_after.insert("/old.wav".into(), 4);

    let item = remote_item(&url, "exp", None);
    let id = item.id;
    let (engine, capture) = engine_with(small_net(), 10.0);
    engine.load(vec![item], 0);

    let ev = engine.stream_events().recv_timeout(Duration::from_secs(10)).expect("Expired event");
    let StreamEvent::Expired { id: eid, position } = ev else {
        panic!("unexpected {ev:?}");
    };
    assert_eq!(eid, id);
    assert!(position > Duration::ZERO, "expired mid-track at {position:?}");
    // What a controller does: resolve again and hand over the new URL.
    engine.replace_stream(
        id,
        StreamSource::Url {
            url: fresh,
            headers: Vec::new(),
            format: None,
        },
    );
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_eq!(ended_ids(&events), vec![id]);
    assert_same(&capture, &pcm, "resume after refresh is not exact");
    // The fresh URL was read from the resume point (after the header probe).
    let resumed = server.hits("/new.wav");
    assert!(
        resumed.iter().any(|h| h.range.is_some_and(|(a, _)| a > 48 * 1024)),
        "resumed mid-file: {resumed:?}"
    );
}

#[test]
fn expired_url_without_refresh_is_skipped() {
    let server = TestServer::start();
    let good_pcm = tagged_signal(4, 10_000);
    let url_bad = server.add("/dead.wav", wav_bytes(44_100, &tagged_signal(5, 10_000)), "audio/wav");
    let url_good = server.add("/ok.wav", wav_bytes(44_100, &good_pcm), "audio/wav");
    server.state().expired.insert("/dead.wav".into());
    let bad = remote_item(&url_bad, "dead", None);
    let good = remote_item(&url_good, "ok", None);
    let (bad_id, good_id) = (bad.id, good.id);
    let net = StreamConfig {
        expired_wait: Duration::from_millis(300),
        ..small_net()
    };
    let (engine, capture) = engine_with(net, 10.0);
    engine.load(vec![bad, good], 0);
    let ev = engine.stream_events().recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(ev, StreamEvent::Expired { id, position } if id == bad_id && position.is_zero()));
    let events = wait_for(engine.events(), Duration::from_secs(10), finished);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, PlaybackEvent::Error { id: Some(x), .. } if *x == bad_id))
    );
    assert_eq!(ended_ids(&events), vec![good_id]);
    assert_same(&capture, &good_pcm, "output");
}

#[test]
fn network_outage_pauses_and_play_retries() {
    let server = TestServer::start();
    let pcm = tagged_signal(6, 44_100 * 2);
    let url = server.add("/net.wav", wav_bytes(44_100, &pcm), "audio/wav");
    let net = StreamConfig {
        max_retries: 2,
        ..small_net()
    };
    let item = remote_item(&url, "net", None);
    let id = item.id;
    let (engine, capture) = engine_with(net, 10.0);
    engine.load(vec![item], 0);
    // Wait until a couple of requests went through, then cut the network.
    let deadline = Instant::now() + Duration::from_secs(5);
    while server.hits("/net.wav").len() < 2 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    server.state().down = true;

    let ev = engine.stream_events().recv_timeout(Duration::from_secs(10)).expect("NetworkFailed");
    let StreamEvent::NetworkFailed { id: fid, .. } = ev else {
        panic!("unexpected {ev:?}");
    };
    assert_eq!(fid, id);
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(10));

    server.state().down = false;
    engine.play();
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_eq!(ended_ids(&events), vec![id]);
    assert_same(&capture, &pcm, "retry after outage is not exact");
}

#[test]
fn engine_stays_responsive_while_the_network_is_slow() {
    let server = TestServer::start();
    let url = server.add("/slow.wav", wav_bytes(44_100, &tagged_signal(7, 20_000)), "audio/wav");
    server.state().latency = Duration::from_millis(1500);
    let (engine, _capture) = engine_with(small_net(), 10.0);
    engine.load(vec![remote_item(&url, "slow", None)], 0);
    std::thread::sleep(Duration::from_millis(50));
    let t = Instant::now();
    let state = engine.state().expect("engine answers while opening");
    assert!(t.elapsed() < Duration::from_millis(200), "engine blocked for {:?}", t.elapsed());
    assert_eq!(state.status, PlaybackStatus::Buffering);
    // Pause/play while buffering keeps the intent.
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(1));
    engine.play();
    wait_status(&engine, PlaybackStatus::Buffering, Duration::from_secs(1));
    wait_for(engine.events(), Duration::from_secs(10), finished);
}

#[cfg(feature = "opus")]
#[test]
fn webm_opus_streams_like_youtube_itag_251() {
    let server = TestServer::start();
    let url = server.add("/videoplayback", fixture("sine_opus.webm"), "audio/webm; codecs=\"opus\"");
    let (engine, capture) = engine_with(small_net(), 20.0);
    engine.load(vec![remote_item(&url, "opus", Some(Duration::from_secs(6)))], 0);
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_eq!(ended_ids(&events).len(), 1);
    assert_eq!(capture.format(), (48_000, 2));
    let secs = capture.frames() as f64 / 48_000.0;
    assert!((5.9..6.1).contains(&secs), "played {secs} s");
}

#[test]
fn fragmented_m4a_streams_and_seeks_like_youtube_itag_140() {
    let server = TestServer::start();
    let url = server.add("/videoplayback", fixture("sine_aac_frag.m4a"), "audio/mp4");
    let (engine, capture) = engine_with(small_net(), 20.0);
    // The provider knows the duration; the fragmented file only declares 1 s.
    let item = remote_item(&url, "aac", Some(Duration::from_secs(6)));
    engine.load(vec![item.clone()], 0);
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_eq!(ended_ids(&events).len(), 1);
    let secs = capture.frames() as f64 / 44_100.0;
    assert!((5.9..6.1).contains(&secs), "played {secs} s");

    // Seek past the first fragment.
    capture.clear();
    let item = QueueItem::new(item.track.clone(), item.stream.clone());
    engine.load(vec![item], 0);
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(5));
    engine.seek(Duration::from_secs(3));
    std::thread::sleep(Duration::from_millis(300));
    capture.clear();
    engine.play();
    wait_for(engine.events(), Duration::from_secs(15), finished);
    let rest = capture.frames() as f64 / 44_100.0;
    assert!((2.9..3.1).contains(&rest), "after seek played {rest} s");
}

#[test]
fn disk_cache_replays_without_network() {
    let server = TestServer::start();
    let pcm = tagged_signal(8, 44_100);
    let url = server.add("/c.wav", wav_bytes(44_100, &pcm), "audio/wav");
    let dir = temp_dir();
    let net = StreamConfig {
        cache: Some(CacheConfig {
            dir: dir.clone(),
            max_bytes: 64 * 1024 * 1024,
        }),
        ..small_net()
    };
    let item = remote_item(&url, "cached", None);
    let (engine, capture) = engine_with(net, 20.0);
    engine.load(vec![item.clone()], 0);
    wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_same(&capture, &pcm, "output");
    let cache = engine.cache().expect("cache configured");
    let deadline = Instant::now() + Duration::from_secs(5);
    while cache.cached(&item.track).is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let file = cache.cached(&item.track).expect("complete stream cached");
    assert_eq!(file.extension().and_then(|e| e.to_str()), Some("wav"));

    // Second play: the server is gone, the cache answers.
    let before = server.hits("/c.wav").len();
    server.state().down = true;
    capture.clear();
    engine.load(vec![QueueItem::new(item.track.clone(), item.stream.clone())], 0);
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert!(!events.iter().any(|e| matches!(e, PlaybackEvent::Error { .. })), "{events:?}");
    assert_same(&capture, &pcm, "output");
    assert_eq!(server.hits("/c.wav").len(), before, "no request on a cache hit");
}

#[test]
fn disk_cache_fills_gaps_left_by_seeks() {
    let server = TestServer::start();
    let rate = 44_100;
    let pcm = index_signal(rate as usize * 4);
    let url = server.add("/gap.wav", wav_bytes(rate, &pcm), "audio/wav");
    let net = StreamConfig {
        cache: Some(CacheConfig {
            dir: temp_dir(),
            max_bytes: 64 * 1024 * 1024,
        }),
        ..small_net()
    };
    let item = remote_item(&url, "gap", None);
    let (engine, _capture) = engine_with(net, 10.0);
    engine.load(vec![item.clone()], 0);
    engine.pause();
    wait_status(&engine, PlaybackStatus::Paused, Duration::from_secs(5));
    // Jump over most of the file: the skipped bytes are a hole.
    engine.seek(Duration::from_millis(3_000));
    engine.play();
    wait_for(engine.events(), Duration::from_secs(15), finished);
    let cache = engine.cache().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while cache.cached(&item.track).is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let file = cache.cached(&item.track).expect("holes filled and file cached");
    assert_eq!(std::fs::read(file).unwrap(), wav_bytes(rate, &pcm), "cached file is the whole stream");
}

#[test]
fn expired_next_track_is_refreshed_gapless() {
    let server = TestServer::start();
    let a = tagged_signal(0, 30_000);
    let b = tagged_signal(1, 30_000);
    let url_a = server.add("/na.wav", wav_bytes(44_100, &a), "audio/wav");
    let url_b = server.add("/nb-old.wav", wav_bytes(44_100, &b), "audio/wav");
    let fresh_b = server.add("/nb-new.wav", wav_bytes(44_100, &b), "audio/wav");
    server.state().expired.insert("/nb-old.wav".into());
    let (ia, ib) = (remote_item(&url_a, "na", None), remote_item(&url_b, "nb", None));
    let ids = vec![ia.id, ib.id];
    let (engine, capture) = engine_with(small_net(), 5.0);
    engine.load(vec![ia, ib], 0);
    let ev = engine.stream_events().recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(ev, StreamEvent::Expired { id: ids[1], position: Duration::ZERO });
    engine.replace_stream(
        ids[1],
        StreamSource::Url {
            url: fresh_b,
            headers: Vec::new(),
            format: None,
        },
    );
    let events = wait_for(engine.events(), Duration::from_secs(15), finished);
    assert_eq!(ended_ids(&events), ids);
    let mut expected = a;
    expected.extend_from_slice(&b);
    assert_same(&capture, &expected, "refreshed next track joins gaplessly");
}

#[test]
fn disk_cache_evicts_least_recently_used() {
    let server = TestServer::start();
    let dir = temp_dir();
    let one = wav_bytes(44_100, &tagged_signal(9, 30_000)); // ~120 KB
    let net = StreamConfig {
        cache: Some(CacheConfig {
            dir: dir.clone(),
            max_bytes: one.len() as u64 * 5 / 2, // room for two
        }),
        ..small_net()
    };
    let (engine, _capture) = engine_with(net, 50.0);
    let mut items = Vec::new();
    for k in 0..3 {
        let url = server.add(&format!("/e{k}.wav"), one.clone(), "audio/wav");
        items.push(remote_item(&url, &format!("e{k}"), None));
    }
    let cache = engine.cache().unwrap().clone();
    for item in &items {
        engine.load(vec![QueueItem::new(item.track.clone(), item.stream.clone())], 0);
        wait_for(engine.events(), Duration::from_secs(15), finished);
        let deadline = Instant::now() + Duration::from_secs(5);
        while cache.cached(&item.track).is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(20)); // distinct mtimes
    }
    assert!(cache.size() <= one.len() as u64 * 5 / 2);
    assert!(cache.cached(&items[0].track).is_none(), "oldest entry evicted");
    assert!(cache.cached(&items[2].track).is_some());
}

#[test]
fn bandwidth_cap_is_respected() {
    let server = TestServer::start();
    let pcm = tagged_signal(10, 44_100 * 3); // ~529 KB
    let bytes = wav_bytes(44_100, &pcm);
    let len = bytes.len() as f64;
    let url = server.add("/bw.wav", bytes, "audio/wav");
    let cap = 400_000u64;
    let net = StreamConfig {
        max_bandwidth: Some(cap),
        read_ahead: 8 * 1024 * 1024,
        ..small_net()
    };
    let (engine, capture) = engine_with(net, 50.0);
    let t = Instant::now();
    engine.load(vec![remote_item(&url, "bw", None)], 0);
    wait_for(engine.events(), Duration::from_secs(20), finished);
    let elapsed = t.elapsed().as_secs_f64();
    // Unthrottled, 3 s at 50x takes ~0.1 s.
    assert!(elapsed >= len / cap as f64 * 0.8, "took {elapsed:.2} s for {len} B at {cap} B/s");
    assert_same(&capture, &pcm, "output");
}
