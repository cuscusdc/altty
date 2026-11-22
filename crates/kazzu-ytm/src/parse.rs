//! Turns WEB_REMIX innertube responses into Kazzu types.
//!
//! Strategy: classify each item by its own navigation endpoint (`pageType`
//! for albums/artists/playlists, `videoId` for songs) instead of by its
//! position in the page. Layouts change every few months; endpoint types have
//! been stable for years. Unknown renderers are skipped, never fatal.

use std::time::Duration;

use kazzu_core::{
    Album, AlbumId, AlbumRef, Artist, ArtistId, ArtistRef, CoverRef, Playlist, PlaylistId,
    ProviderKind, Track, TrackId, TrackSource,
};
use serde_json::Value;

use crate::error::{Result, YtmError};
use crate::json::{
    arr_at, at, find_all, find_first, num, parse_duration, parse_year, runs, str_at, strings,
    text, text_at,
};
use crate::model::{
    AlbumPage, ArtistPage, BrowseRef, Category, Item, Lyrics, PlaylistPage, Shelf, ShelfPage,
    Suggestions,
};

const PK: ProviderKind = ProviderKind::YtMusic;

/// Edge of the square covers we ask googleusercontent for.
pub(crate) const COVER_SIZE: u32 = 544;

const ITEM_KEYS: [&str; 4] = [
    "musicResponsiveListItemRenderer",
    "musicTwoRowItemRenderer",
    "playlistPanelVideoRenderer",
    "playlistPanelVideoWrapperRenderer",
];

// ---------------------------------------------------------------- helpers

fn page_type(endpoint: &Value) -> Option<&str> {
    str_at(
        endpoint,
        &[
            "browseEndpoint",
            "browseEndpointContextSupportedConfigs",
            "browseEndpointContextMusicConfig",
            "pageType",
        ],
    )
}

fn browse_id(endpoint: &Value) -> Option<&str> {
    str_at(endpoint, &["browseEndpoint", "browseId"])
}

fn browse_ref(endpoint: Option<&Value>) -> Option<BrowseRef> {
    let ep = endpoint?.get("browseEndpoint")?;
    Some(BrowseRef {
        browse_id: ep.get("browseId")?.as_str()?.to_owned(),
        params: ep.get("params").and_then(Value::as_str).map(str::to_owned),
    })
}

fn strip_vl(id: &str) -> &str {
    id.strip_prefix("VL").unwrap_or(id)
}

pub(crate) fn remote_track(id: &str, title: String) -> Track {
    Track::new(
        TrackId::new(id),
        title,
        TrackSource::Remote {
            provider: PK,
            id: id.to_owned(),
        },
    )
}

/// Largest thumbnail below `value`, upscaled for googleusercontent covers.
pub(crate) fn best_thumbnail(value: Option<&Value>) -> Option<CoverRef> {
    let thumbs = find_first(value?, "thumbnails")?.as_array()?;
    let best = thumbs.iter().max_by_key(|t| {
        num(t.get("width")).unwrap_or(0) * num(t.get("height")).unwrap_or(0)
    })?;
    let mut url = best.get("url")?.as_str()?.to_owned();
    if url.starts_with("//") {
        url.insert_str(0, "https:");
    }
    Some(CoverRef::Url {
        url: upscale_cover(&url, COVER_SIZE),
    })
}

/// `...=w60-h60-l90-rj` → `...=w544-h544-l90-rj` on googleusercontent URLs.
pub(crate) fn upscale_cover(url: &str, size: u32) -> String {
    if !url.contains("googleusercontent.com") {
        return url.to_owned();
    }
    let Some(eq) = url.rfind('=') else {
        return url.to_owned();
    };
    let (base, params) = url.split_at(eq + 1);
    let mut has_wh = false;
    let rewritten: Vec<String> = params
        .split('-')
        .map(|p| {
            let is_dim = |prefix: char| {
                p.starts_with(prefix) && p.len() > 1 && p[1..].chars().all(|c| c.is_ascii_digit())
            };
            if is_dim('w') {
                has_wh = true;
                format!("w{size}")
            } else if is_dim('h') {
                has_wh = true;
                format!("h{size}")
            } else if is_dim('s') {
                has_wh = true;
                format!("s{size}")
            } else {
                p.to_owned()
            }
        })
        .collect();
    if has_wh {
        format!("{base}{}", rewritten.join("-"))
    } else {
        url.to_owned()
    }
}

/// What a subtitle line ("Song • Nirvana • Nevermind • 4:38") tells us.
#[derive(Debug, Default)]
struct RunInfo {
    artists: Vec<ArtistRef>,
    album: Option<AlbumRef>,
    year: Option<i32>,
    duration: Option<u64>,
    /// Unlinked " • "-separated segments that are not a year or duration.
    plain: Vec<String>,
}

fn is_separator(text: &str) -> bool {
    matches!(text.trim(), "•" | "·" | "|")
}

fn scan_runs(runs: &[Value], info: &mut RunInfo) {
    let mut segment = String::new();
    let mut segment_linked = false;
    let flush = |segment: &mut String, linked: &mut bool, info: &mut RunInfo| {
        let s = segment.trim();
        if !s.is_empty() && !*linked {
            if let Some(d) = parse_duration(s) {
                info.duration.get_or_insert(d);
            } else if let Some(y) = parse_year(s) {
                info.year.get_or_insert(y);
            } else {
                info.plain.push(s.to_owned());
            }
        }
        segment.clear();
        *linked = false;
    };
    for run in runs {
        let t = run.get("text").and_then(Value::as_str).unwrap_or("");
        if is_separator(t) {
            flush(&mut segment, &mut segment_linked, info);
            continue;
        }
        if let Some(nav) = run.get("navigationEndpoint") {
            match (page_type(nav), browse_id(nav)) {
                (Some("MUSIC_PAGE_TYPE_ARTIST" | "MUSIC_PAGE_TYPE_USER_CHANNEL"), Some(id)) => {
                    if !info.artists.iter().any(|a| a.name == t) {
                        info.artists.push(ArtistRef {
                            id: Some(ArtistId::new(id)),
                            name: t.to_owned(),
                        });
                    }
                    segment_linked = true;
                    continue;
                }
                (Some("MUSIC_PAGE_TYPE_ALBUM" | "MUSIC_PAGE_TYPE_AUDIOBOOK"), Some(id)) => {
                    info.album.get_or_insert(AlbumRef {
                        id: Some(AlbumId::new(id)),
                        title: t.to_owned(),
                    });
                    segment_linked = true;
                    continue;
                }
                _ => {}
            }
        }
        segment.push_str(t);
    }
    flush(&mut segment, &mut segment_linked, info);
}

fn run_info(texts: &[Option<&Value>]) -> RunInfo {
    let mut info = RunInfo::default();
    for t in texts {
        scan_runs(runs(*t), &mut info);
    }
    info
}

/// Artist names of an unlinked subtitle. With 2+ segments the first one is
/// the item type ("Song", "Álbum"...), whatever the language.
fn fallback_artists(plain: &[String], has_type_label: bool) -> Vec<ArtistRef> {
    let candidate = if has_type_label || plain.len() >= 2 {
        plain.get(1)
    } else {
        plain.first()
    };
    candidate
        .filter(|s| !s.chars().any(|c| c.is_ascii_digit()))
        .map(|s| vec![ArtistRef::named(s.clone())])
        .unwrap_or_default()
}

fn make_album(id: &str, title: String, info: &RunInfo, cover: Option<CoverRef>) -> Album {
    let artists = if info.artists.is_empty() {
        fallback_artists(&info.plain, false)
    } else {
        info.artists.clone()
    };
    Album {
        id: AlbumId::new(id),
        provider: PK,
        title,
        artists,
        year: info.year,
        track_count: None,
        cover,
    }
}

fn make_playlist(id: &str, name: String, description: Option<String>, cover: Option<CoverRef>) -> Playlist {
    Playlist {
        id: PlaylistId::new(strip_vl(id)),
        provider: PK,
        name,
        description,
        cover,
        tracks: Vec::new(),
        editable: false,
    }
}

fn make_artist(id: &str, name: String, image: Option<CoverRef>) -> Artist {
    Artist {
        id: ArtistId::new(id),
        provider: PK,
        name,
        image,
    }
}

/// Item behind a browse endpoint (album/artist/playlist), or `None` for
/// podcasts, profiles, episodes...
fn browse_item(
    nav: &Value,
    title: String,
    subtitle: &RunInfo,
    subtitle_text: Option<String>,
    cover: Option<CoverRef>,
) -> Option<Item> {
    let id = browse_id(nav)?;
    match page_type(nav)? {
        "MUSIC_PAGE_TYPE_ALBUM" | "MUSIC_PAGE_TYPE_AUDIOBOOK" => {
            Some(Item::Album(make_album(id, title, subtitle, cover)))
        }
        "MUSIC_PAGE_TYPE_ARTIST" => Some(Item::Artist(make_artist(id, title, cover))),
        "MUSIC_PAGE_TYPE_PLAYLIST" => Some(Item::Playlist(make_playlist(id, title, subtitle_text, cover))),
        _ => None,
    }
}

fn watch_video_id(value: Option<&Value>) -> Option<&str> {
    let v = value?;
    str_at(v, &["watchEndpoint", "videoId"])
}

fn fill_track(track: &mut Track, info: RunInfo, has_type_label: bool) {
    track.artists = if info.artists.is_empty() {
        fallback_artists(&info.plain, has_type_label)
    } else {
        info.artists
    };
    track.album = info.album;
    track.duration = info.duration.map(Duration::from_secs);
}

// ---------------------------------------------------------------- items

/// Parsing context.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Ctx {
    /// Rows start with a localized type label ("Song • ..."), as in the
    /// unfiltered search.
    pub type_label: bool,
}

/// `musicResponsiveListItemRenderer`: rows of lists (search, album tracks,
/// playlists, top songs).
pub(crate) fn list_item(r: &Value, ctx: Ctx) -> Option<Item> {
    let cols: Vec<&Value> = arr_at(r, &["flexColumns"])
        .iter()
        .filter_map(|c| at(c, &["musicResponsiveListItemFlexColumnRenderer", "text"]))
        .collect();
    let title = cols.first().and_then(|c| text(c))?;
    let cover = best_thumbnail(r.get("thumbnail"));
    let subtitle: Vec<Option<&Value>> = cols.iter().skip(1).map(|c| Some(*c)).collect();

    if let Some(nav) = r.get("navigationEndpoint")
        && nav.get("browseEndpoint").is_some()
    {
        let info = run_info(&subtitle[..subtitle.len().min(1)]);
        let sub_text = cols.get(1).and_then(|c| text(c));
        return browse_item(nav, title, &info, sub_text, cover);
    }

    let id = str_at(r, &["playlistItemData", "videoId"])
        .or_else(|| watch_video_id(runs(cols.first().copied()).first().and_then(|x| x.get("navigationEndpoint"))))
        .or_else(|| r.get("overlay").and_then(|o| find_first(o, "videoId")).and_then(Value::as_str))?;
    if str_at(r, &["musicItemRendererDisplayPolicy"]) == Some("MUSIC_ITEM_RENDERER_DISPLAY_POLICY_GREY_OUT") {
        return None;
    }
    // Podcast episodes are videos too; their title/subtitle link to
    // non-music pages.
    if cols.iter().any(|c| {
        runs(Some(c)).iter().filter_map(|x| x.get("navigationEndpoint")).filter_map(page_type).any(|p| {
            p.contains("NON_MUSIC") || p.contains("PODCAST")
        })
    }) {
        return None;
    }
    let mut track = remote_track(id, title);
    track.cover = cover;
    // Artists come from the first subtitle column only (others hold play
    // counts); albums and durations may be in any column.
    let mut info = run_info(&subtitle[..subtitle.len().min(1)]);
    let rest = run_info(subtitle.get(1..).unwrap_or(&[]));
    if info.album.is_none() {
        info.album = rest.album;
    }
    if info.artists.is_empty() && !rest.artists.is_empty() {
        info.artists = rest.artists;
    }
    if let Some(d) = arr_at(r, &["fixedColumns"])
        .iter()
        .filter_map(|c| text_at(c, &["musicResponsiveListItemFixedColumnRenderer", "text"]))
        .find_map(|s| parse_duration(&s))
    {
        info.duration = Some(d);
    }
    info.duration = info.duration.or(rest.duration);
    fill_track(&mut track, info, ctx.type_label);
    track.track_number = text_at(r, &["index"]).and_then(|s| s.trim().parse().ok());
    Some(Item::Track(track))
}

/// `musicTwoRowItemRenderer`: cards of carousels and grids.
pub(crate) fn two_row_item(r: &Value) -> Option<Item> {
    let title = text_at(r, &["title"])?;
    let cover = best_thumbnail(r.get("thumbnailRenderer"));
    let info = run_info(&[r.get("subtitle")]);
    let sub_text = text_at(r, &["subtitle"]);
    let nav = r.get("navigationEndpoint")?;
    if nav.get("browseEndpoint").is_some() {
        return browse_item(nav, title, &info, sub_text, cover);
    }
    if let Some(id) = watch_video_id(Some(nav)) {
        let mut track = remote_track(id, title);
        track.cover = cover;
        fill_track(&mut track, info, false);
        return Some(Item::Track(track));
    }
    if let Some(pid) = str_at(nav, &["watchPlaylistEndpoint", "playlistId"]) {
        return Some(Item::Playlist(make_playlist(pid, title, sub_text, cover)));
    }
    None
}

/// `playlistPanelVideoRenderer`: rows of the "up next" queue.
pub(crate) fn panel_video(r: &Value) -> Option<Track> {
    let r = if let Some(inner) = r.get("primaryRenderer") {
        inner.get("playlistPanelVideoRenderer")?
    } else {
        r
    };
    let id = r.get("videoId")?.as_str()?;
    let title = text_at(r, &["title"])?;
    let mut track = remote_track(id, title);
    track.cover = best_thumbnail(r.get("thumbnail"));
    let mut info = run_info(&[r.get("longBylineText")]);
    if info.artists.is_empty()
        && let Some(name) = text_at(r, &["shortBylineText"])
    {
        info.artists.push(ArtistRef::named(name));
        info.plain.clear();
    }
    let length = text_at(r, &["lengthText"]).and_then(|s| parse_duration(&s));
    fill_track(&mut track, info, false);
    if length.is_some() {
        track.duration = length.map(Duration::from_secs);
    }
    Some(track)
}

/// `musicCardShelfRenderer`: the "top result" card of a search.
fn card_item(r: &Value) -> Option<Item> {
    let title = text_at(r, &["title"])?;
    let cover = best_thumbnail(r.get("thumbnail"));
    let info = run_info(&[r.get("subtitle")]);
    let sub_text = text_at(r, &["subtitle"]);
    let nav = runs(r.get("title"))
        .first()
        .and_then(|x| x.get("navigationEndpoint"))
        .or_else(|| r.get("onTap"))?;
    if nav.get("browseEndpoint").is_some() {
        return browse_item(nav, title, &info, sub_text, cover);
    }
    let id = watch_video_id(Some(nav))?;
    let mut track = remote_track(id, title);
    track.cover = cover;
    fill_track(&mut track, info, true);
    Some(Item::Track(track))
}

/// Top-result card plus the songs listed inside it. Those songs often say
/// only "Song • 2:43": they inherit the card's artist (or album).
fn card_with_contents(v: &Value) -> Vec<Item> {
    let main = card_item(v);
    let mut items: Vec<Item> = items_of(arr_at(v, &["contents"]), Ctx { type_label: true });
    for item in &mut items {
        let Item::Track(t) = item else { continue };
        match &main {
            Some(Item::Artist(a)) if t.artists.is_empty() => t.artists.push(ArtistRef {
                id: Some(a.id.clone()),
                name: a.name.clone(),
            }),
            Some(Item::Album(a)) => {
                if t.artists.is_empty() {
                    t.artists = a.artists.clone();
                }
                t.album.get_or_insert(AlbumRef {
                    id: Some(a.id.clone()),
                    title: a.title.clone(),
                });
            }
            _ => {}
        }
    }
    main.into_iter().chain(items).collect()
}

fn item_from(key: &str, v: &Value, ctx: Ctx) -> Option<Item> {
    match key {
        "musicResponsiveListItemRenderer" => list_item(v, ctx),
        "musicTwoRowItemRenderer" => two_row_item(v),
        "playlistPanelVideoRenderer" | "playlistPanelVideoWrapperRenderer" => {
            panel_video(v).map(Item::Track)
        }
        _ => None,
    }
}

/// Every item below `value`, in document order (top-result card first).
pub(crate) fn collect_items(value: &Value, ctx: Ctx) -> Vec<Item> {
    fn go(value: &Value, ctx: Ctx, out: &mut Vec<Item>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    if ITEM_KEYS.contains(&k.as_str()) {
                        out.extend(item_from(k, v, ctx));
                    } else if k == "musicCardShelfRenderer" {
                        out.extend(card_with_contents(v));
                    } else if !matches!(k.as_str(), "menu" | "header" | "thumbnail" | "overlay") {
                        go(v, ctx, out);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| go(v, ctx, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    go(value, ctx, &mut out);
    out
}

fn items_of(contents: &[Value], ctx: Ctx) -> Vec<Item> {
    contents
        .iter()
        .filter_map(|c| {
            let (k, v) = c.as_object()?.iter().next()?;
            item_from(k, v, ctx)
        })
        .collect()
}

// ---------------------------------------------------------------- shelves

fn shelf(key: &str, v: &Value) -> Option<Shelf> {
    let ctx = Ctx::default();
    let (title, subtitle, items, more) = match key {
        "musicCarouselShelfRenderer" | "musicImmersiveCarouselShelfRenderer" => {
            let header = v
                .get("header")
                .and_then(|h| h.as_object()?.values().next());
            let title = header.and_then(|h| text_at(h, &["title"]));
            let subtitle = header.and_then(|h| text_at(h, &["strapline"]));
            let more = header.and_then(|h| {
                browse_ref(at(h, &["moreContentButton", "buttonRenderer", "navigationEndpoint"]))
                    .or_else(|| browse_ref(runs(h.get("title")).first().and_then(|r| r.get("navigationEndpoint"))))
            });
            (title, subtitle, items_of(arr_at(v, &["contents"]), ctx), more)
        }
        "musicShelfRenderer" | "musicPlaylistShelfRenderer" => (
            text_at(v, &["title"]),
            None,
            items_of(arr_at(v, &["contents"]), ctx),
            browse_ref(v.get("bottomEndpoint")).or_else(|| {
                browse_ref(runs(v.get("title")).first().and_then(|r| r.get("navigationEndpoint")))
            }),
        ),
        "gridRenderer" => (
            text_at(v, &["header", "gridHeaderRenderer", "title"]),
            None,
            items_of(arr_at(v, &["items"]), ctx),
            None,
        ),
        "musicCardShelfRenderer" => {
            let items = card_with_contents(v);
            (
                text_at(v, &["header", "musicCardShelfHeaderBasicRenderer", "title"]),
                None,
                items,
                None,
            )
        }
        _ => return None,
    };
    (!items.is_empty()).then(|| Shelf {
        title: title.unwrap_or_default(),
        subtitle,
        items,
        more,
    })
}

fn shelves_of(contents: &[Value]) -> Vec<Shelf> {
    contents
        .iter()
        .filter_map(|c| {
            let (k, v) = c.as_object()?.iter().next()?;
            shelf(k, v)
        })
        .collect()
}

/// Contents of every section list in the response (one- or two-column
/// browse pages and section-list continuations).
fn section_contents(resp: &Value) -> Vec<&Value> {
    let mut out: Vec<&Value> = find_all(resp, "sectionListRenderer")
        .into_iter()
        .flat_map(|sl| arr_at(sl, &["contents"]))
        .collect();
    if let Some(cont) = at(resp, &["continuationContents", "sectionListContinuation"]) {
        out.extend(arr_at(cont, &["contents"]));
    }
    out
}

fn first_continuation(value: &Value) -> Option<String> {
    for key in ["nextContinuationData", "nextRadioContinuationData", "reloadContinuationData"] {
        if let Some(c) = find_first(value, key).and_then(|c| c.get("continuation")).and_then(Value::as_str) {
            return Some(c.to_owned());
        }
    }
    find_first(value, "continuationCommand")
        .and_then(|c| c.get("token"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// Home, charts, new releases, moods, "see all" pages.
pub(crate) fn shelf_page(resp: &Value) -> Result<ShelfPage> {
    if resp.get("contents").is_none() && resp.get("continuationContents").is_none() {
        return Err(YtmError::shape("browse contents"));
    }
    let contents: Vec<Value> = section_contents(resp).into_iter().cloned().collect();
    let continuation = find_first(resp, "sectionListRenderer")
        .and_then(|sl| sl.get("continuations"))
        .or_else(|| at(resp, &["continuationContents", "sectionListContinuation", "continuations"]))
        .and_then(first_continuation);
    let title = resp
        .get("header")
        .and_then(|h| h.as_object()?.values().next())
        .and_then(|h| text_at(h, &["title"]));
    let mut shelves = shelves_of(&contents);
    for s in shelves.iter_mut().filter(|s| s.title.is_empty()) {
        s.title = title.clone().unwrap_or_default();
    }
    Ok(ShelfPage {
        title,
        shelves,
        continuation,
    })
}

// ---------------------------------------------------------------- search

/// Items and continuation of a search (or search continuation) response.
pub(crate) fn search(resp: &Value, ctx: Ctx) -> Result<(Vec<Item>, Option<String>)> {
    let root = resp
        .get("contents")
        .or_else(|| resp.get("continuationContents"))
        .ok_or_else(|| YtmError::shape("search contents"))?;
    let items = collect_items(root, ctx);
    let continuation = find_first(root, "musicShelfRenderer")
        .or_else(|| find_first(root, "musicShelfContinuation"))
        .and_then(|s| s.get("continuations"))
        .and_then(first_continuation);
    Ok((items, continuation))
}

pub(crate) fn suggestions(resp: &Value) -> Suggestions {
    let queries = find_all(resp, "searchSuggestionRenderer")
        .into_iter()
        .filter_map(|s| {
            str_at(s, &["navigationEndpoint", "searchEndpoint", "query"])
                .map(str::to_owned)
                .or_else(|| text_at(s, &["suggestion"]))
        })
        .collect();
    Suggestions {
        queries,
        items: collect_items(resp.get("contents").unwrap_or(resp), Ctx { type_label: true }),
    }
}

// ---------------------------------------------------------------- pages

fn header_of(resp: &Value) -> Option<&Value> {
    [
        "musicResponsiveHeaderRenderer",
        "musicDetailHeaderRenderer",
        "musicImmersiveHeaderRenderer",
        "musicVisualHeaderRenderer",
    ]
    .iter()
    .find_map(|k| find_first(resp, k))
}

fn description_of(header: &Value) -> Option<String> {
    at(header, &["description"])
        .and_then(|d| {
            d.get("musicDescriptionShelfRenderer")
                .and_then(|s| text_at(s, &["description"]))
                .or_else(|| text(d))
        })
        .map(|s| s.replace("\r\n", "\n"))
}

pub(crate) fn album_page(resp: &Value, id: &str) -> Result<AlbumPage> {
    let header = header_of(resp).ok_or_else(|| YtmError::NotFound(format!("album {id}")))?;
    let title = text_at(header, &["title"]).ok_or_else(|| YtmError::shape("album title"))?;
    let sub = run_info(&[header.get("subtitle")]);
    let strap = run_info(&[header.get("straplineTextOne")]);
    let cover = best_thumbnail(header.get("thumbnail"));
    let mut artists = strap.artists.clone();
    if artists.is_empty() {
        artists = sub.artists.clone();
    }
    if artists.is_empty()
        && let Some(name) = text_at(header, &["straplineTextOne"])
    {
        artists.push(ArtistRef::named(name));
    }
    let album_type = sub.plain.first().cloned();

    let tracks_shelf = find_first(resp, "musicShelfRenderer");
    let album_ref = AlbumRef {
        id: Some(AlbumId::new(id)),
        title: title.clone(),
    };
    let tracks: Vec<Track> = tracks_shelf
        .map(|s| items_of(arr_at(s, &["contents"]), Ctx::default()))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| match i {
            Item::Track(t) => Some(t),
            _ => None,
        })
        .enumerate()
        .map(|(n, mut t)| {
            if t.artists.is_empty() {
                t.artists = artists.clone();
            }
            t.album = Some(album_ref.clone());
            t.cover = cover.clone();
            t.track_number = t.track_number.or(Some(n as u32 + 1));
            t
        })
        .collect();

    let audio_playlist_id = str_at(resp, &["microformat", "microformatDataRenderer", "urlCanonical"])
        .and_then(|u| u.split("list=").nth(1))
        .map(|s| s.split('&').next().unwrap_or(s).to_owned())
        .or_else(|| {
            strings(header)
                .into_iter()
                .find(|s| s.starts_with("OLAK5uy_"))
                .map(str::to_owned)
        });
    let contents: Vec<Value> = section_contents(resp).into_iter().cloned().collect();
    let related = shelves_of(&contents)
        .into_iter()
        .filter(|s| !s.items.iter().all(|i| matches!(i, Item::Track(_))))
        .collect();
    Ok(AlbumPage {
        album: Album {
            id: AlbumId::new(id),
            provider: PK,
            title,
            artists,
            year: sub.year,
            track_count: Some(tracks.len() as u32),
            cover,
        },
        tracks,
        description: description_of(header),
        summary: text_at(header, &["secondSubtitle"]),
        album_type,
        audio_playlist_id,
        related,
    })
}

pub(crate) fn artist_page(resp: &Value, id: &str) -> Result<ArtistPage> {
    let header = resp
        .get("header")
        .and_then(|h| h.as_object()?.values().next())
        .or_else(|| header_of(resp))
        .ok_or_else(|| YtmError::NotFound(format!("artist {id}")))?;
    let name = text_at(header, &["title"]).ok_or_else(|| YtmError::shape("artist name"))?;
    let audience = find_first(header, "monthlyListenerCount")
        .and_then(text)
        .or_else(|| find_first(header, "subscriberCountText").and_then(text));
    let artist = make_artist(id, name, best_thumbnail(header.get("thumbnail")));
    let contents: Vec<Value> = section_contents(resp).into_iter().cloned().collect();
    let mut shelves = shelves_of(&contents);
    let mut top_tracks = Vec::new();
    let mut all_tracks = None;
    if let Some(pos) = shelves
        .iter()
        .position(|s| s.items.iter().all(|i| matches!(i, Item::Track(_))))
    {
        let s = shelves.remove(pos);
        all_tracks = s.more;
        top_tracks = s
            .items
            .into_iter()
            .filter_map(|i| match i {
                Item::Track(t) => Some(t),
                _ => None,
            })
            .collect();
    }
    let me = ArtistRef {
        id: Some(artist.id.clone()),
        name: artist.name.clone(),
    };
    for s in &mut shelves {
        for item in &mut s.items {
            match item {
                Item::Album(a) if a.artists.is_empty() => a.artists.push(me.clone()),
                Item::Track(t) if t.artists.is_empty() => t.artists.push(me.clone()),
                _ => {}
            }
        }
    }
    for t in &mut top_tracks {
        if t.artists.is_empty() {
            t.artists.push(me.clone());
        }
    }
    Ok(ArtistPage {
        description: description_of(header),
        artist,
        audience,
        top_tracks,
        shelves,
        all_tracks,
    })
}

fn tracks_of(items: Vec<Item>) -> Vec<Track> {
    items
        .into_iter()
        .filter_map(|i| match i {
            Item::Track(t) => Some(t),
            _ => None,
        })
        .collect()
}

fn shelf_continuation(shelf: &Value) -> Option<String> {
    shelf
        .get("continuations")
        .and_then(first_continuation)
        .or_else(|| {
            arr_at(shelf, &["contents"])
                .last()
                .and_then(|c| c.get("continuationItemRenderer"))
                .and_then(first_continuation)
        })
}

pub(crate) fn playlist_page(resp: &Value, id: &str) -> Result<PlaylistPage> {
    let header = header_of(resp).ok_or_else(|| YtmError::NotFound(format!("playlist {id}")))?;
    let name = text_at(header, &["title"]).ok_or_else(|| YtmError::shape("playlist title"))?;
    let shelf = find_first(resp, "musicPlaylistShelfRenderer").or_else(|| find_first(resp, "musicShelfRenderer"));
    let tracks = shelf
        .map(|s| tracks_of(items_of(arr_at(s, &["contents"]), Ctx::default())))
        .unwrap_or_default();
    let author = text_at(header, &["straplineTextOne"]).or_else(|| {
        run_info(&[header.get("subtitle")])
            .artists
            .first()
            .map(|a| a.name.clone())
    });
    let contents: Vec<Value> = section_contents(resp).into_iter().cloned().collect();
    let related = shelves_of(&contents)
        .into_iter()
        .filter(|s| !s.items.iter().all(|i| matches!(i, Item::Track(_))))
        .collect();
    let mut playlist = make_playlist(id, name, description_of(header), best_thumbnail(header.get("thumbnail")));
    playlist.tracks = tracks.iter().map(|t| t.id.clone()).collect();
    Ok(PlaylistPage {
        playlist,
        continuation: shelf.and_then(shelf_continuation),
        tracks,
        author,
        summary: text_at(header, &["secondSubtitle"]).or_else(|| text_at(header, &["subtitle"])),
        related,
    })
}

/// Tracks + next token of a playlist-shelf continuation response.
pub(crate) fn playlist_continuation(resp: &Value) -> (Vec<Track>, Option<String>) {
    if let Some(c) = at(resp, &["continuationContents", "musicPlaylistShelfContinuation"]) {
        return (
            tracks_of(items_of(arr_at(c, &["contents"]), Ctx::default())),
            shelf_continuation(c),
        );
    }
    let items: Vec<&Value> = find_all(resp, "continuationItems")
        .into_iter()
        .flat_map(|v| v.as_array().map(Vec::as_slice).unwrap_or(&[]))
        .collect();
    let owned: Vec<Value> = items.iter().map(|v| (*v).clone()).collect();
    let token = owned
        .last()
        .and_then(|c| c.get("continuationItemRenderer"))
        .and_then(first_continuation);
    (tracks_of(items_of(&owned, Ctx::default())), token)
}

/// Result of the `next` endpoint.
#[derive(Debug, Default)]
pub(crate) struct WatchNext {
    pub tracks: Vec<Track>,
    pub continuation: Option<String>,
    pub playlist_id: Option<String>,
    pub lyrics_browse_id: Option<String>,
}

pub(crate) fn watch_next(resp: &Value) -> Result<WatchNext> {
    let panel = find_first(resp, "playlistPanelRenderer")
        .or_else(|| at(resp, &["continuationContents", "playlistPanelContinuation"]))
        .ok_or_else(|| YtmError::shape("playlistPanelRenderer"))?;
    let tracks = arr_at(panel, &["contents"])
        .iter()
        .filter_map(|c| {
            c.get("playlistPanelVideoRenderer")
                .or_else(|| c.get("playlistPanelVideoWrapperRenderer"))
                .and_then(panel_video)
        })
        .collect();
    let lyrics_browse_id = find_all(resp, "tabRenderer").into_iter().find_map(|tab| {
        if tab.get("unselectable").and_then(Value::as_bool) == Some(true) {
            return None;
        }
        let id = str_at(tab, &["endpoint", "browseEndpoint", "browseId"])?;
        id.starts_with("MPLY").then(|| id.to_owned())
    });
    Ok(WatchNext {
        tracks,
        continuation: panel.get("continuations").and_then(first_continuation),
        playlist_id: panel.get("playlistId").and_then(Value::as_str).map(str::to_owned),
        lyrics_browse_id,
    })
}

pub(crate) fn lyrics(resp: &Value) -> Option<Lyrics> {
    let shelf = find_first(resp, "musicDescriptionShelfRenderer")?;
    let text = text_at(shelf, &["description"])?.replace("\r\n", "\n");
    Some(Lyrics {
        text,
        source: text_at(shelf, &["footer"]),
    })
}

pub(crate) fn categories(resp: &Value) -> Vec<Category> {
    let mut out = Vec::new();
    for section in section_contents(resp) {
        let group = find_first(section, "gridHeaderRenderer")
            .and_then(|h| text_at(h, &["title"]))
            .or_else(|| {
                find_first(section, "musicCarouselShelfBasicHeaderRenderer").and_then(|h| text_at(h, &["title"]))
            });
        for b in find_all(section, "musicNavigationButtonRenderer") {
            let (Some(title), Some(browse)) = (
                text_at(b, &["buttonText"]),
                browse_ref(b.get("clickCommand")),
            ) else {
                continue;
            };
            let color = num(at(b, &["solid", "leftStripeColor"])).map(|c| c as u32);
            out.push(Category {
                title,
                group: group.clone(),
                browse,
                color,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upscales_googleusercontent_covers() {
        assert_eq!(
            upscale_cover("https://lh3.googleusercontent.com/abc=w60-h60-l90-rj", 544),
            "https://lh3.googleusercontent.com/abc=w544-h544-l90-rj"
        );
        assert_eq!(
            upscale_cover("https://yt3.googleusercontent.com/abc=s120-c-k", 544),
            "https://yt3.googleusercontent.com/abc=s544-c-k"
        );
        let yt = "https://i.ytimg.com/vi/x/hqdefault.jpg";
        assert_eq!(upscale_cover(yt, 544), yt);
    }

    #[test]
    fn subtitle_runs_are_classified() {
        let v: Value = serde_json::from_str(
            r#"{"runs":[{"text":"Song"},{"text":" • "},{"text":"Nirvana","navigationEndpoint":{"browseEndpoint":{"browseId":"UC1","browseEndpointContextSupportedConfigs":{"browseEndpointContextMusicConfig":{"pageType":"MUSIC_PAGE_TYPE_ARTIST"}}}}},{"text":" • "},{"text":"Nevermind","navigationEndpoint":{"browseEndpoint":{"browseId":"MPREb_1","browseEndpointContextSupportedConfigs":{"browseEndpointContextMusicConfig":{"pageType":"MUSIC_PAGE_TYPE_ALBUM"}}}}},{"text":" • "},{"text":"4:38"},{"text":" • "},{"text":"1991"}]}"#,
        )
        .unwrap();
        let info = run_info(&[Some(&v)]);
        assert_eq!(info.artists.len(), 1);
        assert_eq!(info.artists[0].id.as_ref().unwrap().as_str(), "UC1");
        assert_eq!(info.album.as_ref().unwrap().title, "Nevermind");
        assert_eq!(info.duration, Some(278));
        assert_eq!(info.year, Some(1991));
        assert_eq!(info.plain, vec!["Song".to_owned()]);
    }

    #[test]
    fn unlinked_artist_fallback() {
        let plain = vec!["Música".to_owned(), "Elbosco".to_owned()];
        assert_eq!(fallback_artists(&plain, true)[0].name, "Elbosco");
        let plain = vec!["Elbosco".to_owned()];
        assert_eq!(fallback_artists(&plain, false)[0].name, "Elbosco");
        assert!(fallback_artists(&["Tocou 3 bi vezes".to_owned()], false).is_empty());
    }
}
