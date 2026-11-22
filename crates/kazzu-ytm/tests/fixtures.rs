//! Offline tests: the whole client runs against recorded innertube
//! responses (`tests/fixtures/*.json`, recorded 2026-11-22 in pt/BR and
//! sanitized: no tracking params, no IP, fake visitor data).

use std::sync::{Arc, Mutex};

use kazzu_core::{Codec, Provider, SearchKind, SearchQuery, StreamQuality, StreamSource, TrackSource};
use kazzu_ytm::{
    Config, ErrorKind, HttpRequest, HttpResponse, Item, Locale, Method, PlayerClient, Result, Transport, YtMusic,
};
use serde_json::Value;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[derive(Default)]
struct Fake {
    log: Mutex<Vec<HttpRequest>>,
    /// googlevideo probes return 403 for these clients (`c=` url param).
    gated: Vec<&'static str>,
    offline: bool,
}

fn ok(body: String) -> Result<HttpResponse> {
    Ok(HttpResponse {
        status: 200,
        body,
        retry_after: None,
    })
}

impl Transport for Fake {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse> {
        self.log.lock().unwrap().push(req.clone());
        if self.offline {
            return Err(kazzu_ytm::YtmError::Network("cannot resolve host (offline?)".into()));
        }
        if req.method == Method::Get {
            if req.url.contains("sw.js_data") {
                return ok(r#")]}'
[["yt.sw.adr",null,[[["pt","BR",null,null,"","","CgtGSVhUVVJFVklTSVRPUg%3D%3D","1.20261117.06.00"]]]]]"#
                    .into());
            }
            if req.url.contains("googlevideo.com") {
                let gated = self.gated.iter().any(|c| req.url.contains(&format!("c={c}")));
                return Ok(HttpResponse {
                    status: if gated { 403 } else { 206 },
                    body: String::new(),
                    retry_after: None,
                });
            }
            panic!("unexpected GET {}", req.url);
        }
        let body: Value = serde_json::from_str(req.body.as_deref().unwrap()).unwrap();
        let s = |k: &str| body.get(k).and_then(Value::as_str).unwrap_or("");
        if req.url.contains("/player") {
            let client = body["context"]["client"]["clientName"].as_str().unwrap();
            let name = match (client, s("videoId")) {
                (_, "aaaaaaaaaaa") => "player_unavailable",
                (_, "HtVdAasjOgU") => "player_agerestricted",
                (_, "botbotbotbo") => "player_visionos_novd",
                ("VISIONOS", _) => "player_visionos",
                ("IOS_PRODUCER", _) => "player_iosproducer",
                other => panic!("unexpected player request {other:?}"),
            };
            return ok(fixture(name));
        }
        if req.url.contains("music/get_queue") {
            // Echo every id back with a fixed 3:21 duration.
            let datas: Vec<Value> = body["videoIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| serde_json::json!({"content": {"playlistPanelVideoRenderer": {
                    "videoId": id, "title": {"runs": [{"text": "q"}]}, "lengthText": {"runs": [{"text": "3:21"}]}
                }}}))
                .collect();
            return ok(serde_json::json!({ "queueDatas": datas }).to_string());
        }
        let name = if req.url.contains("get_search_suggestions") {
            "sugg"
        } else if req.url.contains("/search") {
            match (s("query"), body.get("params").is_some()) {
                ("nirvana", false) => "search",
                ("nirvana", true) => "search_songs",
                ("daft punk", _) => "search_en",
                _ if body.get("continuation").is_some() => return ok(r#"{"continuationContents":{"musicShelfContinuation":{"contents":[]}}}"#.into()),
                other => panic!("unexpected search {other:?}"),
            }
        } else if req.url.contains("/next") {
            match s("videoId") {
                "hTWKbfoikeg" => "next",
                "sp86SkWKRQE" => "next_track",
                other => panic!("unexpected next {other}"),
            }
        } else {
            match s("browseId") {
                "FEmusic_home" => "home",
                "FEmusic_charts" => "charts",
                "FEmusic_new_releases" => "newrel",
                "FEmusic_moods_and_genres" => "moods",
                "MPREb_jPOYfjGgApr" => "album",
                "UCrPe3hLA51968GwxHSZ1llw" => "artist",
                "VLRDCLAK5uy_njFsM69_gqiI2S4HKpdUAkEYLKeoc4ggQ" => "playlist",
                "MPLYt_98tQCDzFEIf" => "lyrics",
                other => panic!("unexpected browse {other}"),
            }
        };
        ok(fixture(name))
    }
}

fn provider_with(fake: Fake) -> (YtMusic, Arc<Fake>) {
    let fake = Arc::new(fake);
    let mut config = Config::ephemeral();
    config.locale = Locale::new("pt", "BR");
    config.player_clients = vec![PlayerClient::VisionOs, PlayerClient::IosProducer];
    let ytm = YtMusic::with_transport(config, fake.clone()).unwrap();
    (ytm, fake)
}

fn provider() -> YtMusic {
    provider_with(Fake::default()).0
}

#[test]
fn unfiltered_search_classifies_items() {
    let ytm = provider();
    let items = ytm.client().search_items("nirvana", None, 0).unwrap();
    // Top-result card: the artist.
    match &items[0] {
        Item::Artist(a) => {
            assert_eq!(a.name, "Nirvana");
            assert_eq!(a.id.as_str(), "UCrPe3hLA51968GwxHSZ1llw");
        }
        other => panic!("{other:?}"),
    }
    let albums: Vec<_> = items.iter().filter_map(|i| match i { Item::Album(a) => Some(a), _ => None }).collect();
    let nevermind = albums.iter().find(|a| a.title == "Nevermind").expect("Nevermind");
    assert_eq!(nevermind.id.as_str(), "MPREb_jPOYfjGgApr");
    assert_eq!(nevermind.year, Some(1991));
    assert_eq!(nevermind.artists[0].name, "Nirvana");
    let tracks: Vec<_> = items.iter().filter_map(|i| match i { Item::Track(t) => Some(t), _ => None }).collect();
    let teen = tracks.iter().find(|t| t.title == "Smells Like Teen Spirit").expect("song");
    assert_eq!(teen.source, TrackSource::Remote { provider: kazzu_core::ProviderKind::YtMusic, id: teen.id.as_str().into() });
    assert!(tracks.iter().all(|t| t.id.as_str().len() == 11));
    // Unlinked type label ("Música") must never be taken as the artist.
    assert!(tracks.iter().flat_map(|t| &t.artists).all(|a| a.name != "Música" && a.name != "Vídeo"), "{tracks:#?}");
    // Songs inside the top-result card inherit the card's artist.
    let school = tracks.iter().find(|t| t.id.as_str() == "sp86SkWKRQE").unwrap();
    assert_eq!(school.artists[0].name, "Nirvana");
    assert_eq!(school.duration.unwrap().as_secs(), 163);
    let elbosco = tracks.iter().find(|t| t.id.as_str() == "Qfw4ps4M-bM").unwrap();
    assert_eq!(elbosco.artists[0].name, "Elbosco");
    // Podcasts, episodes and user profiles are skipped.
    assert!(items.iter().all(|i| !i.title().contains("Kurt Cobain: la cronaca")));
    let playlists: Vec<_> = items.iter().filter_map(|i| match i { Item::Playlist(p) => Some(p), _ => None }).collect();
    assert!(playlists.iter().any(|p| p.id.as_str() == "RDCLAK5uy_njFsM69_gqiI2S4HKpdUAkEYLKeoc4ggQ"), "VL prefix stripped");
    assert!(items.iter().all(|i| match i {
        Item::Track(t) => t.cover.is_some(),
        Item::Album(a) => a.cover.is_some(),
        _ => true,
    }));
}

#[test]
fn filtered_song_search_has_artist_album_duration() {
    let ytm = provider();
    let items = ytm.client().search_items("nirvana", Some(kazzu_ytm::SearchFilter::Songs), 10).unwrap();
    assert!(items.len() >= 10, "{}", items.len());
    for item in &items {
        let Item::Track(t) = item else { panic!("only songs: {item:?}") };
        assert!(!t.artists.is_empty(), "{t:?}");
        assert!(t.duration.is_some(), "{t:?}");
    }
    let with_album = items.iter().filter(|i| matches!(i, Item::Track(t) if t.album.is_some())).count();
    assert!(with_album >= items.len() / 2);
}

#[test]
fn provider_search_maps_to_core_results() {
    let ytm = provider();
    let all = pollster::block_on(ytm.search(&SearchQuery::new("nirvana"))).unwrap();
    assert!(!all.tracks.is_empty() && !all.albums.is_empty() && !all.artists.is_empty() && !all.playlists.is_empty());
    let mut q = SearchQuery::new("nirvana");
    q.kinds = vec![SearchKind::Track];
    q.limit = 5;
    let songs = pollster::block_on(ytm.search(&q)).unwrap();
    assert_eq!(songs.tracks.len(), 5);
    assert!(songs.albums.is_empty());
}

#[test]
fn english_search_also_works() {
    let ytm = provider();
    let items = ytm.client().search_items("daft punk", None, 0).unwrap();
    assert!(matches!(&items[0], Item::Artist(a) if a.name == "Daft Punk"), "{:?}", items[0]);
    assert!(items.iter().any(|i| matches!(i, Item::Track(_))));
    assert!(items.iter().flat_map(|i| match i { Item::Track(t) => t.artists.clone(), _ => vec![] }).all(|a| a.name != "Song" && a.name != "Video"));
}

#[test]
fn suggestions() {
    let s = provider().client().suggestions("nirv").unwrap();
    assert_eq!(s.queries[0], "nirvana");
    assert!(s.queries.len() >= 5);
    assert!(!s.items.is_empty(), "direct hits");
}

#[test]
fn home_charts_new_releases_and_discover() {
    let ytm = provider();
    let c = ytm.client();
    let home = c.home().unwrap();
    assert!(!home.shelves.is_empty());
    assert!(home.shelves.iter().all(|s| !s.title.is_empty() && !s.items.is_empty()), "{:#?}", home.shelves.iter().map(|s| &s.title).collect::<Vec<_>>());
    let charts = c.charts(None).unwrap();
    assert!(charts.shelves.iter().any(|s| s.items.iter().any(|i| matches!(i, Item::Playlist(_)))));
    let new = c.new_releases().unwrap();
    assert!(new.shelves.iter().flat_map(|s| &s.items).filter(|i| matches!(i, Item::Album(_))).count() >= 10);
    let discover = c.discover().unwrap();
    assert!(discover.partial_errors.is_empty());
    assert_eq!(discover.shelves.len(), home.shelves.len() + new.shelves.len() + charts.shelves.len());
    let cats = c.categories().unwrap();
    assert!(cats.len() >= 10, "{}", cats.len());
    assert!(cats.iter().all(|k| k.browse.params.is_some()));
}

#[test]
fn album_page() {
    let page = provider().client().album("MPREb_jPOYfjGgApr").unwrap();
    assert_eq!(page.album.title, "Nevermind");
    assert_eq!(page.album.year, Some(1991));
    assert_eq!(page.album.artists[0].name, "Nirvana");
    assert_eq!(page.album.artists[0].id.as_ref().unwrap().as_str(), "UCrPe3hLA51968GwxHSZ1llw");
    assert_eq!(page.tracks.len(), 13);
    assert_eq!(page.album.track_count, Some(13));
    let first = &page.tracks[0];
    assert_eq!(first.title, "Smells Like Teen Spirit");
    assert_eq!(first.track_number, Some(1));
    assert_eq!(first.duration.unwrap().as_secs(), 302);
    assert_eq!(first.artists[0].name, "Nirvana");
    assert_eq!(first.album.as_ref().unwrap().title, "Nevermind");
    assert_eq!(page.tracks[12].track_number, Some(13));
    assert_eq!(page.audio_playlist_id.as_deref(), Some("OLAK5uy_lYnxawfGdkGePjdFhIYaS6LjP-Md6UYf0"));
    assert!(page.description.unwrap().contains("Nevermind"));
    assert!(!page.related.is_empty());
    match page.album.cover.unwrap() {
        kazzu_core::CoverRef::Url { url } => assert!(url.contains("w544-h544"), "{url}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn artist_page() {
    let page = provider().client().artist("UCrPe3hLA51968GwxHSZ1llw").unwrap();
    assert_eq!(page.artist.name, "Nirvana");
    assert!(page.top_tracks.len() >= 5);
    assert_eq!(page.top_tracks[0].title, "Smells Like Teen Spirit");
    assert!(page.top_tracks.iter().all(|t| !t.artists.is_empty()));
    assert!(page.all_tracks.is_some());
    let albums: Vec<_> = page.shelves.iter().flat_map(|s| &s.items).filter_map(|i| match i { Item::Album(a) => Some(a), _ => None }).collect();
    assert!(albums.len() >= 5);
    assert!(albums.iter().all(|a| !a.artists.is_empty()));
    assert!(page.shelves.iter().any(|s| s.items.iter().any(|i| matches!(i, Item::Artist(_)))), "related artists");
    assert!(page.description.is_some());
}

#[test]
fn playlist_page() {
    let page = provider().client().playlist("RDCLAK5uy_njFsM69_gqiI2S4HKpdUAkEYLKeoc4ggQ").unwrap();
    assert_eq!(page.playlist.name, "Presenting Edis");
    // Header says "18 músicas".
    assert_eq!(page.tracks.len(), 18);
    assert_eq!(page.playlist.tracks.len(), 18);
    assert!(page.summary.as_deref().unwrap().starts_with("18"));
    assert!(page.tracks.iter().all(|t| t.artists.iter().any(|a| a.name.eq_ignore_ascii_case("EDIS")) && t.duration.is_some()));
    assert_eq!(page.playlist.id.as_str(), "RDCLAK5uy_njFsM69_gqiI2S4HKpdUAkEYLKeoc4ggQ");
}

#[test]
fn radio_track_and_lyrics() {
    let c = provider();
    let c = c.client();
    let radio = c.radio("hTWKbfoikeg").unwrap();
    assert!(radio.tracks.len() >= 40);
    assert_eq!(radio.tracks[0].id.as_str(), "hTWKbfoikeg");
    assert!(radio.continuation.is_some());
    assert!(radio.tracks.iter().all(|t| !t.artists.is_empty() && t.duration.is_some()));
    let t = c.track("sp86SkWKRQE").unwrap();
    assert_eq!(t.title, "School");
    assert_eq!(t.artists[0].name, "Nirvana");
    let lyrics = c.lyrics("hTWKbfoikeg").unwrap().unwrap();
    assert!(lyrics.text.starts_with("Load up on guns"));
    assert!(!lyrics.text.contains('\r'));
}

#[test]
fn resolves_best_opus_stream_with_visionos() {
    let (ytm, fake) = provider_with(Fake::default());
    let track = pollster::block_on(ytm.track(&"sp86SkWKRQE".into())).unwrap();
    let s = ytm.client().resolve("sp86SkWKRQE", StreamQuality::Original).unwrap();
    assert_eq!(s.itag, 251);
    assert_eq!(s.format.codec, Codec::Opus);
    assert_eq!(s.format.sample_rate, Some(48_000));
    assert_eq!(s.format.bitrate_kbps, Some(112));
    assert_eq!(s.client, "visionos");
    assert_eq!(s.badge(), "Opus · 112 kbps · 48 kHz");
    assert!(s.mime.contains("webm"));
    assert!(s.headers.iter().any(|(k, v)| k == "User-Agent" && v.contains("Safari")));
    assert_eq!(s.expires_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), 1_795_347_254);
    assert_eq!(s.content_length, Some(2_274_755));
    let mut bare = kazzu_core::Track::new("sp86SkWKRQE", "School", track.source.clone());
    s.fill_track(&mut bare);
    assert_eq!(bare.duration.unwrap().as_millis(), 162_161);
    assert_eq!(bare.format.as_ref().unwrap().codec, Codec::Opus);
    let low = ytm.client().resolve("sp86SkWKRQE", StreamQuality::Low).unwrap();
    assert_eq!(low.itag, 249);
    let src = pollster::block_on(ytm.resolve_stream(&track, StreamQuality::Original)).unwrap();
    assert!(matches!(src, StreamSource::Url { format: Some(f), .. } if f.codec == Codec::Opus));

    let log = fake.log.lock().unwrap();
    // One bootstrap per session; the probe asked for the tail of the file.
    assert_eq!(log.iter().filter(|r| r.url.contains("sw.js_data")).count(), 1);
    let probe = log.iter().find(|r| r.url.contains("googlevideo")).unwrap();
    assert_eq!(probe.header_value("Range"), Some("bytes=2273731-2274754"));
    // Privacy: metadata requests carry no visitor id, no cookie.
    for r in log.iter().filter(|r| r.url.contains("music.youtube.com/youtubei")) {
        assert!(r.header_value("Cookie").is_none());
        assert!(r.header_value("X-Goog-Visitor-Id").is_none());
        assert!(!r.body.as_deref().unwrap_or("").contains("visitorData"));
    }
}

#[test]
fn falls_back_to_ios_producer_when_googlevideo_gates_visionos() {
    let (ytm, fake) = provider_with(Fake {
        gated: vec!["VISIONOS"],
        ..Fake::default()
    });
    // The fixture URL keeps `c=VISIONOS`, so the fake answers its probe with
    // 403 (what googlevideo does when it enforces PO tokens on a client).
    let s = ytm.client().resolve("sp86SkWKRQE", StreamQuality::Original).unwrap();
    assert_eq!(s.client, "ios_producer");
    assert_eq!(s.format.codec, Codec::Aac);
    assert_eq!(s.itag, 140);
    // VisionOS is skipped for the rest of the session.
    let before = fake.log.lock().unwrap().len();
    let again = ytm.client().resolve("sp86SkWKRQE", StreamQuality::Original).unwrap();
    assert_eq!(again.client, "ios_producer");
    let log = fake.log.lock().unwrap();
    assert!(log[before..].iter().all(|r| !r.body.as_deref().unwrap_or("").contains("VISIONOS")));
}

#[test]
fn stream_errors_are_typed() {
    let c = provider();
    let c = c.client();
    assert_eq!(c.resolve("aaaaaaaaaaa", StreamQuality::Original).unwrap_err().kind(), ErrorKind::Unavailable);
    assert_eq!(c.resolve("HtVdAasjOgU", StreamQuality::Original).unwrap_err().kind(), ErrorKind::AgeRestricted);
    let bot = c.resolve("botbotbotbo", StreamQuality::Original).unwrap_err();
    assert_eq!(bot.kind(), ErrorKind::Blocked);
    assert!(bot.is_retryable());
    assert_eq!(c.resolve("bad id", StreamQuality::Original).unwrap_err().kind(), ErrorKind::NotFound);
    // Through the core boundary the kind is still recoverable.
    let core: kazzu_core::Error = c.resolve("aaaaaaaaaaa", StreamQuality::Original).unwrap_err().into();
    assert_eq!(ErrorKind::of(&core), Some(ErrorKind::Unavailable));
}

#[test]
fn prefetch_is_reused_once() {
    let (ytm, fake) = provider_with(Fake::default());
    let c = ytm.client();
    c.prefetch("sp86SkWKRQE", StreamQuality::Original).unwrap();
    let n = fake.log.lock().unwrap().len();
    // The fixture's URL expires in 2026-11-22 + 6 h; that is in the past for
    // anyone running this later, so only assert reuse when still valid.
    let s = c.resolve("sp86SkWKRQE", StreamQuality::Original).unwrap();
    if s.time_left().as_secs() > 1200 {
        assert_eq!(fake.log.lock().unwrap().len(), n, "served from prefetch");
    }
    c.invalidate_stream("sp86SkWKRQE");
}

#[test]
fn disk_cache_serves_stale_pages_offline() {
    let dir = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::default());
    let mut config = Config::with_data_dir(dir.path());
    config.locale = Locale::new("pt", "BR");
    let ytm = YtMusic::with_transport(config.clone(), fake.clone()).unwrap();
    let album = ytm.client().album("MPREb_jPOYfjGgApr").unwrap();
    assert!(ytm.client().peek_home().is_none());
    ytm.client().home().unwrap();
    assert!(ytm.client().peek_home().is_some());

    // New session, network down: the cached album still opens.
    let down = Arc::new(Fake { offline: true, ..Fake::default() });
    let ytm2 = YtMusic::with_transport(config.clone(), down.clone()).unwrap();
    assert_eq!(ytm2.client().album("MPREb_jPOYfjGgApr").unwrap(), album);
    let err = ytm2.client().artist("UCrPe3hLA51968GwxHSZ1llw").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Network);
    assert!(err.is_retryable());

    // Offline mode: no request at all.
    ytm2.client().set_offline(true);
    let before = down.log.lock().unwrap().len();
    assert_eq!(ytm2.client().album("MPREb_jPOYfjGgApr").unwrap(), album);
    assert_eq!(ytm2.client().artist("x").unwrap_err().kind(), ErrorKind::Offline);
    assert_eq!(down.log.lock().unwrap().len(), before);

    // Another locale does not reuse pt/BR pages.
    ytm2.client().set_locale(Locale::new("en", "US"));
    assert!(ytm2.client().album("MPREb_jPOYfjGgApr").is_err());
}

#[test]
fn local_library_through_provider() {
    let ytm = provider();
    let t = ytm.client().track("sp86SkWKRQE").unwrap();
    ytm.library_store().like(&t).unwrap();
    let snap = pollster::block_on(ytm.library()).unwrap();
    assert_eq!(snap.liked, vec![t.id.clone()]);
    let caps = ytm.capabilities();
    assert!(caps.search && caps.stream && caps.like && caps.library && !caps.edit_playlists);
    assert_eq!(ytm.kind(), kazzu_core::ProviderKind::YtMusic);
}

#[test]
fn no_network_until_asked() {
    let (_ytm, fake) = provider_with(Fake::default());
    assert!(fake.log.lock().unwrap().is_empty());
}


/// kazzu-audio needs `Track.duration` (fMP4 itag 140 only declares its first
/// fragment): every track the client returns must have one.
#[test]
fn every_returned_track_has_a_duration() {
    let (ytm, fake) = provider_with(Fake::default());
    let c = ytm.client();
    let mut tracks: Vec<kazzu_core::Track> = Vec::new();
    let items = |v: &[Item]| v.iter().filter_map(|i| match i { Item::Track(t) => Some(t.clone()), _ => None }).collect::<Vec<_>>();
    tracks.extend(items(&c.search_items("nirvana", None, 0).unwrap()));
    tracks.extend(items(&c.search_items("daft punk", None, 0).unwrap()));
    tracks.extend(items(&c.search_items("nirvana", Some(kazzu_ytm::SearchFilter::Songs), 10).unwrap()));
    tracks.extend(items(&c.suggestions("nirv").unwrap().items));
    for page in [c.home().unwrap(), c.charts(None).unwrap(), c.new_releases().unwrap()] {
        for s in &page.shelves {
            tracks.extend(items(&s.items));
        }
    }
    let artist = c.artist("UCrPe3hLA51968GwxHSZ1llw").unwrap();
    tracks.extend(artist.top_tracks.clone());
    for s in &artist.shelves {
        tracks.extend(items(&s.items));
    }
    tracks.extend(c.album("MPREb_jPOYfjGgApr").unwrap().tracks);
    tracks.extend(c.playlist("RDCLAK5uy_njFsM69_gqiI2S4HKpdUAkEYLKeoc4ggQ").unwrap().tracks);
    tracks.extend(c.radio("hTWKbfoikeg").unwrap().tracks);
    tracks.push(c.track("sp86SkWKRQE").unwrap());
    assert!(tracks.len() > 100, "{}", tracks.len());
    let missing: Vec<_> = tracks.iter().filter(|t| t.duration.is_none()).map(|t| (&t.id, &t.title)).collect();
    assert!(missing.is_empty(), "{missing:?}");
    // Real durations are kept; only the missing ones were looked up.
    let teen = tracks.iter().find(|t| t.id.as_str() == "ljUtuoFt-8c").unwrap();
    assert_eq!(teen.duration.unwrap().as_secs(), 302);
    let queue_calls = fake.log.lock().unwrap().iter().filter(|r| r.url.contains("get_queue")).count();
    assert!(queue_calls <= 12, "batched: {queue_calls}");
}
