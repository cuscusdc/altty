//! Container/codec coverage for what streaming providers serve:
//! WebM (Matroska) + Opus (YouTube itag 251) and MP4/AAC, both plain and
//! fragmented DASH-style (YouTube itag 140). Fixtures are 6 s sines made
//! with ffmpeg (see `tests/fixtures/README.md`).

use std::path::PathBuf;
use std::time::Duration;

use kazzu_audio::TrackDecoder;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn decode_all(dec: &mut TrackDecoder) -> usize {
    let mut out = Vec::new();
    while dec.decode_next(&mut out).unwrap() {}
    out.len() / usize::from(dec.spec().channels)
}

fn check(name: &str, rate: u32) {
    let mut dec = TrackDecoder::open_file(&fixture(name)).unwrap();
    assert_eq!(dec.spec().sample_rate, rate, "{name}");
    assert_eq!(dec.spec().channels, 2, "{name}");
    let frames = decode_all(&mut dec);
    let secs = frames as f64 / f64::from(rate);
    assert!((5.9..6.1).contains(&secs), "{name}: decoded {secs} s");

    // Seek to the middle and decode the rest. As the engine does, the
    // container duration is reconciled with the track metadata (fragmented
    // MP4 only declares its first fragment).
    let mut dec = TrackDecoder::open_file(&fixture(name)).unwrap();
    dec.reconcile_duration(Some(Duration::from_secs(6)));
    assert!(dec.duration().unwrap() >= Duration::from_secs(6), "{name}: {:?}", dec.duration());
    dec.seek(Duration::from_secs(3)).unwrap();
    let rest = decode_all(&mut dec) as f64 / f64::from(rate);
    assert!((2.9..3.1).contains(&rest), "{name}: after seek decoded {rest} s");
}

#[cfg(feature = "opus")]
#[test]
fn webm_opus_decodes_and_seeks() {
    check("sine_opus.webm", 48_000);
}

#[test]
fn mp4_aac_decodes_and_seeks() {
    check("sine_aac.m4a", 44_100);
}

#[test]
fn fragmented_mp4_aac_decodes_and_seeks() {
    check("sine_aac_frag.m4a", 44_100);
}
