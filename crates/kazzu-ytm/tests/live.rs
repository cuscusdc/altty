//! Live tests against the real YouTube Music. Ignored by default (network,
//! upstream changes). Run with:
//!
//! ```text
//! cargo test -p kazzu-ytm --test live -- --ignored --test-threads=1
//! ```
//!
//! They make a few dozen small requests in total.

use std::sync::Arc;
use std::time::Duration;

use kazzu_core::{Codec, StreamQuality};
use kazzu_ytm::{
    Config, ErrorKind, HttpRequest, Item, Locale, PlayerClient, SearchFilter, Transport, UreqTransport, YtClient,
};

fn client() -> YtClient {
    let mut config = Config::ephemeral();
    config.locale = Locale::new("pt", "BR");
    YtClient::new(config).unwrap()
}

const SONG: &str = "sp86SkWKRQE"; // Nirvana - School (audio track, ATV)
const VIDEO: &str = "hTWKbfoikeg"; // Smells Like Teen Spirit (music video, OMV)

#[test]
#[ignore]
fn live_search_and_suggestions() {
    let c = client();
    let items = c.search_items("nirvana", None, 0).unwrap();
    assert!(matches!(&items[0], Item::Artist(a) if a.name == "Nirvana"), "{:?}", items.first());
    let songs = c.search_items("nirvana", Some(SearchFilter::Songs), 30).unwrap();
    assert!(songs.len() >= 30, "continuation paging: {}", songs.len());
    assert!(songs.iter().all(|i| matches!(i, Item::Track(t) if !t.artists.is_empty())));
    let albums = c.search_items("nirvana", Some(SearchFilter::Albums), 5).unwrap();
    assert!(albums.iter().any(|i| matches!(i, Item::Album(a) if a.title == "Nevermind")));
    let mixed_tracks: Vec<_> = items.iter().filter_map(|i| match i { Item::Track(t) => Some(t), _ => None }).collect();
    assert!(!mixed_tracks.is_empty());
    assert!(mixed_tracks.iter().all(|t| t.duration.is_some()), "durations filled via get_queue");
    let home = c.home().unwrap();
    assert!(home.shelves.iter().flat_map(|s| &s.items).all(|i| !matches!(i, Item::Track(t) if t.duration.is_none())));
    let s = c.suggestions("nirv").unwrap();
    assert!(s.queries.iter().any(|q| q.starts_with("nirvana")));
}

#[test]
#[ignore]
fn live_discover_pages() {
    let c = client();
    let d = c.discover().unwrap();
    assert!(d.partial_errors.is_empty(), "{:?}", d.partial_errors);
    assert!(d.shelves.len() >= 4);
    let charts_us = c.charts(Some("US")).unwrap();
    assert!(!charts_us.shelves.is_empty());
    let cats = c.categories().unwrap();
    assert!(!cats.is_empty());
    let mood = c.browse(&cats[0].browse).unwrap();
    assert!(!mood.shelves.is_empty(), "category {}", cats[0].title);
}

#[test]
#[ignore]
fn live_album_artist_playlist() {
    let c = client();
    let album = c.album("MPREb_jPOYfjGgApr").unwrap();
    assert_eq!(album.album.title, "Nevermind");
    assert!(album.tracks.len() >= 12);
    let artist = c.artist("UCrPe3hLA51968GwxHSZ1llw").unwrap();
    assert!(!artist.top_tracks.is_empty());
    if let Some(all) = &artist.all_tracks {
        let pl = c.playlist(&all.browse_id).unwrap();
        assert!(pl.tracks.len() >= 20, "all songs playlist: {}", pl.tracks.len());
    }
    // A long playlist exercises the shelf continuation (100 per page).
    let charts = c.charts(Some("US")).unwrap();
    let long = charts
        .shelves
        .iter()
        .flat_map(|s| &s.items)
        .find_map(|i| match i {
            Item::Playlist(p) => Some(p.id.as_str().to_owned()),
            _ => None,
        })
        .unwrap();
    let tracks = c.playlist_tracks(&long, 300).unwrap();
    assert!(!tracks.is_empty());
}

#[test]
#[ignore]
fn live_radio_track_lyrics() {
    let c = client();
    let radio = c.radio(VIDEO).unwrap();
    assert!(radio.tracks.len() >= 20);
    let more = c.radio_continuation(&radio).unwrap();
    assert!(!more.tracks.is_empty(), "radio continuation");
    let t = c.track(SONG).unwrap();
    assert_eq!(t.artists[0].name, "Nirvana");
    let lyrics = c.lyrics(VIDEO).unwrap().expect("lyrics");
    assert!(lyrics.text.to_lowercase().contains("hello"));
}

/// Downloads the first and last 256 KiB of the resolved stream: proves the
/// URL is not limited to the first megabyte (PO token enforcement).
fn assert_full_download(s: &kazzu_ytm::ResolvedStream) {
    let len = s.content_length.expect("content length");
    let t = UreqTransport::with_timeouts(Duration::from_secs(10), Duration::from_secs(60));
    for range in ["bytes=0-262143".to_owned(), format!("bytes={}-{}", len - 262_144, len - 1)] {
        let mut req = HttpRequest::get(s.url.clone()).header("Range", range.clone());
        for (k, v) in &s.headers {
            req = req.header(k, v.clone());
        }
        let resp = t.send(&req).unwrap();
        assert_eq!(resp.status, 206, "{range} via {}", s.client);
    }
}

#[test]
#[ignore]
fn live_resolve_visionos_opus() {
    let mut config = Config::ephemeral();
    config.player_clients = vec![PlayerClient::VisionOs];
    let c = YtClient::new(config).unwrap();
    for id in [SONG, VIDEO] {
        let s = c.resolve(id, StreamQuality::Original).unwrap();
        assert_eq!(s.format.codec, Codec::Opus, "{id}");
        assert_eq!(s.itag, 251);
        assert!(s.time_left() > Duration::from_secs(3600));
        assert_full_download(&s);
    }
}

#[test]
#[ignore]
fn live_resolve_ios_producer_aac() {
    let mut config = Config::ephemeral();
    config.player_clients = vec![PlayerClient::IosProducer];
    let c = YtClient::new(config).unwrap();
    let s = c.resolve(SONG, StreamQuality::Original).unwrap();
    assert_eq!(s.format.codec, Codec::Aac);
    assert_full_download(&s);
}

#[test]
#[ignore]
fn live_resolve_errors() {
    let c = client();
    assert_eq!(c.resolve("aaaaaaaaaaa", StreamQuality::Original).unwrap_err().kind(), ErrorKind::Unavailable);
}

/// Documents the state of the clients we do NOT use by default (expected to
/// fail today: googlevideo cuts them after ~1 MB without a PO token).
#[test]
#[ignore]
fn live_report_experimental_clients() {
    for client in [PlayerClient::AndroidVr, PlayerClient::Ios] {
        let mut config = Config::ephemeral();
        config.player_clients = vec![client];
        let c = YtClient::with_transport(config, Arc::new(UreqTransport::new())).unwrap();
        match c.resolve(SONG, StreamQuality::Original) {
            Ok(s) => eprintln!("{}: OK ({}) — consider re-enabling", client.name(), s.badge()),
            Err(e) => eprintln!("{}: {} [{}]", client.name(), e, e.kind().code()),
        }
    }
}
