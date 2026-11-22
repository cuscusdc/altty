//! Manual check on a real sound card:
//! `cargo run -p kazzu-audio --example play -- file1.flac file2.opus ...`
//! Set `KAZZU_DEVICE=<id>` to pick a device; `--list` prints device ids.

use kazzu_audio::{CpalSink, Engine};
use kazzu_core::{PlaybackEvent, QueueItem, StreamSource, Track, TrackSource};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--list") {
        for d in CpalSink::devices() {
            println!("{}{}  ({})", if d.is_default { "* " } else { "  " }, d.id, d.name);
        }
        return;
    }
    let sink = match std::env::var("KAZZU_DEVICE") {
        Ok(id) => CpalSink::with_device(id),
        Err(_) => CpalSink::new(),
    };
    let items: Vec<QueueItem> = args
        .iter()
        .map(|p| {
            let track = Track::new(p.as_str(), p.as_str(), TrackSource::Local { path: p.into() });
            QueueItem::new(track, StreamSource::file(p))
        })
        .collect();
    if items.is_empty() {
        eprintln!("usage: play [--list] FILE...");
        return;
    }
    let engine = Engine::builder().sink(sink).spawn().expect("engine");
    engine.load(items, 0);
    for event in engine.events().iter() {
        match &event {
            PlaybackEvent::Position { .. } | PlaybackEvent::QueueChanged { .. } => {}
            PlaybackEvent::TrackChanged { item: Some(item), duration } => {
                println!("▶ {} ({duration:?})", item.track.title)
            }
            PlaybackEvent::OutputChanged { info } => println!("  output: {info:?}"),
            PlaybackEvent::QueueFinished => break,
            other => println!("  {other:?}"),
        }
    }
}
