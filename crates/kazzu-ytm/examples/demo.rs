//! Manual demo against the real YouTube Music (makes network requests).
//!
//! ```text
//! cargo run -p kazzu-ytm --example demo -- search 'nirvana'
//! cargo run -p kazzu-ytm --example demo -- songs 'nirvana'
//! cargo run -p kazzu-ytm --example demo -- suggest 'nirv'
//! cargo run -p kazzu-ytm --example demo -- home | charts [BR] | new | moods
//! cargo run -p kazzu-ytm --example demo -- album MPREb_jPOYfjGgApr
//! cargo run -p kazzu-ytm --example demo -- artist UCrPe3hLA51968GwxHSZ1llw
//! cargo run -p kazzu-ytm --example demo -- playlist RDCLAK5uy_...
//! cargo run -p kazzu-ytm --example demo -- radio <videoId> | lyrics <videoId>
//! cargo run -p kazzu-ytm --example demo -- play <videoId> [seconds] [--cpal]
//! ```
//!
//! `play` resolves the stream (prints URL host, codec, bitrate, expiry) and
//! hands it to kazzu-audio: by default to a `NullSink` (no sound, but the
//! whole download/decode path runs), with `--cpal` to the sound card.
//! Locale: system; override with `KAZZU_LOCALE=pt-BR`.

use std::time::{Duration, Instant, SystemTime};

use kazzu_core::{PlaybackEvent, QueueItem, StreamQuality};
use kazzu_ytm::{Config, Item, Locale, SearchFilter, ShelfPage, YtMusic};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let arg = args.get(1).cloned().unwrap_or_default();
    let mut config = Config::desktop();
    if let Ok(tag) = std::env::var("KAZZU_LOCALE")
        && let Some(l) = Locale::from_tag(&tag)
    {
        config.locale = l;
    }
    eprintln!("locale: hl={:?} gl={:?}", config.locale.hl, config.locale.gl);
    let ytm = YtMusic::new(config).expect("init");
    let started = Instant::now();
    let result = pollster::block_on(run(&ytm, cmd, arg, &args));
    eprintln!("({} ms)", started.elapsed().as_millis());
    if let Err(e) = result {
        eprintln!("error [{}] {e} (retryable: {})", e.kind().code(), e.is_retryable());
        std::process::exit(1);
    }
}

fn line(item: &Item) -> String {
    match item {
        Item::Track(t) => format!(
            "song     {}  {} — {}{}{}",
            t.id,
            t.title,
            t.artist_line(),
            t.album.as_ref().map(|a| format!(" · {}", a.title)).unwrap_or_default(),
            t.duration.map(|d| format!(" · {}:{:02}", d.as_secs() / 60, d.as_secs() % 60)).unwrap_or_default()
        ),
        Item::Album(a) => format!(
            "album    {}  {} — {}{}",
            a.id,
            a.title,
            a.artists.iter().map(|x| x.name.as_str()).collect::<Vec<_>>().join(", "),
            a.year.map(|y| format!(" ({y})")).unwrap_or_default()
        ),
        Item::Artist(a) => format!("artist   {}  {}", a.id, a.name),
        Item::Playlist(p) => format!("playlist {}  {}", p.id, p.name),
    }
}

fn print_page(page: &ShelfPage) {
    for s in &page.shelves {
        println!("\n## {} ({} items){}", s.title, s.items.len(), if s.more.is_some() { " [see all]" } else { "" });
        for i in s.items.iter().take(6) {
            println!("  {}", line(i));
        }
    }
}

async fn run(ytm: &YtMusic, cmd: &str, arg: String, args: &[String]) -> kazzu_ytm::Result<()> {
    match cmd {
        "search" => {
            for i in ytm.search_items(arg, None, 0).await? {
                println!("{}", line(&i));
            }
        }
        "songs" => {
            for i in ytm.search_items(arg, Some(SearchFilter::Songs), 20).await? {
                println!("{}", line(&i));
            }
        }
        "suggest" => {
            let s = ytm.suggestions(arg).await?;
            for q in &s.queries {
                println!("query    {q}");
            }
            for i in &s.items {
                println!("{}", line(i));
            }
        }
        "home" => {
            let d = ytm.discover().await?;
            print_page(&ShelfPage {
                shelves: d.shelves,
                ..Default::default()
            });
            for e in d.partial_errors {
                eprintln!("partial error: {e}");
            }
        }
        "charts" => print_page(&ytm.charts((!arg.is_empty()).then_some(arg)).await?),
        "new" => print_page(&ytm.new_releases().await?),
        "moods" => {
            for c in ytm.categories().await? {
                println!("{:<24} {:<32} {}", c.group.unwrap_or_default(), c.title, c.browse.browse_id);
            }
        }
        "album" => {
            let a = ytm.album(arg).await?;
            println!("{} — {} ({:?}) {}", a.album.title, a.album.artists.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", "), a.album.year, a.summary.unwrap_or_default());
            for t in &a.tracks {
                println!("  {:>2}. {}", t.track_number.unwrap_or(0), line(&Item::Track(t.clone())));
            }
        }
        "artist" => {
            let a = ytm.artist(arg).await?;
            println!("{} — {}", a.artist.name, a.audience.unwrap_or_default());
            for t in &a.top_tracks {
                println!("  {}", line(&Item::Track(t.clone())));
            }
            print_page(&ShelfPage {
                shelves: a.shelves,
                ..Default::default()
            });
        }
        "playlist" => {
            let p = ytm.playlist(arg).await?;
            println!("{} — {} ({})", p.playlist.name, p.author.unwrap_or_default(), p.summary.unwrap_or_default());
            for t in &p.tracks {
                println!("  {}", line(&Item::Track(t.clone())));
            }
            if p.continuation.is_some() {
                println!("  … more");
            }
        }
        "radio" => {
            let r = ytm.radio(arg).await?;
            println!("radio {} ({} tracks, more: {})", r.playlist_id, r.tracks.len(), r.continuation.is_some());
            for t in &r.tracks {
                println!("  {}", line(&Item::Track(t.clone())));
            }
        }
        "lyrics" => match ytm.lyrics(arg).await? {
            Some(l) => println!("{}\n\n— {}", l.text, l.source.unwrap_or_default()),
            None => println!("(no lyrics)"),
        },
        "play" => play(ytm, arg, args).await?,
        _ => {
            eprintln!("usage: demo search|songs|suggest|home|charts|new|moods|album|artist|playlist|radio|lyrics|play <arg>");
        }
    }
    Ok(())
}

async fn play(ytm: &YtMusic, id: String, args: &[String]) -> kazzu_ytm::Result<()> {
    let seconds: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(15);
    let cpal = args.iter().any(|a| a == "--cpal");
    let track = match ytm.client().track(&id) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("metadata unavailable ({e}); playing anyway");
            kazzu_core::Track::new(id.as_str(), id.as_str(), kazzu_core::TrackSource::Remote {
                provider: kazzu_core::ProviderKind::YtMusic,
                id: id.clone(),
            })
        }
    };
    println!("track:   {} — {}", track.title, track.artist_line());
    let s = ytm.resolve(id.clone(), StreamQuality::Original).await?;
    let host = s.url.split('/').nth(2).unwrap_or("?");
    println!("client:  {}", s.client);
    println!("format:  {} (itag {}, {})", s.badge(), s.itag, s.mime);
    println!("size:    {:?} bytes, duration {:?}", s.content_length, s.duration);
    println!("expires: in {} min ({:?})", s.time_left().as_secs() / 60, s.expires_at.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
    println!("url:     https://{host}/videoplayback?… ({} chars)", s.url.len());
    if let Ok(full) = std::env::var("KAZZU_PRINT_URL")
        && full == "1"
    {
        println!("{}", s.url);
    }

    let builder = kazzu_audio::Engine::builder();
    let engine = if cpal {
        kazzu_audio::Engine::spawn()
    } else {
        builder.sink(kazzu_audio::NullSink::new()).spawn()
    }
    .map_err(|e| kazzu_ytm::YtmError::Storage(format!("audio engine: {e}")))?;
    let mut track = track;
    s.fill_track(&mut track);
    engine.load(vec![QueueItem::new(track, s.to_stream_source())], 0);
    println!("\nplaying {seconds}s on {} …", if cpal { "the sound card" } else { "NullSink" });
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut last_pos = 0;
    while Instant::now() < deadline {
        match engine.events().recv_timeout(Duration::from_millis(200)) {
            Ok(PlaybackEvent::Position { position, .. }) => {
                if position.as_secs() != last_pos {
                    last_pos = position.as_secs();
                    println!("  position {last_pos}s");
                }
            }
            Ok(PlaybackEvent::OutputChanged { info }) => println!("  output: {} Hz, {} ch, source {:?}", info.sample_rate, info.channels, info.source),
            Ok(PlaybackEvent::StatusChanged { status }) => println!("  status: {status:?}"),
            Ok(PlaybackEvent::Error { message, .. }) => {
                println!("  engine error: {message}");
                if message.to_lowercase().contains("unsupported") || message.contains("not supported") {
                    println!("  (kazzu-audio does not open StreamSource::Url yet; the stream itself resolved fine)");
                }
                break;
            }
            Ok(PlaybackEvent::QueueFinished) => {
                println!("  finished");
                break;
            }
            Ok(_) | Err(_) => {}
        }
    }
    engine.shutdown();
    Ok(())
}
