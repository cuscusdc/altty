//! Implementação do `View` sobre o Slint. Só roda na thread de UI: guarda `VecModel`s (não são `Send`) e escreve
//! em `AppState`/`Theme`. Conversão de `Pixels` em `slint::Image` acontece aqui.

use std::rc::Rc;

use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel, Weak};

use crate::accent::{Accent, stage_palette};
use crate::art::Pixels;
use crate::config::{AccentPref, ThemePref};
use crate::ui;
use crate::view::View;
use crate::vm::*;

fn img(p: &Option<Pixels>) -> Image {
    p.as_ref().map(|p| Image::from_rgba8(p.clone())).unwrap_or_default()
}

fn images(v: &[Pixels]) -> ModelRc<Image> {
    ModelRc::new(VecModel::from(v.iter().map(|p| Image::from_rgba8(p.clone())).collect::<Vec<_>>()))
}

fn color(c: [u8; 3]) -> slint::Color {
    slint::Color::from_rgb_u8(c[0], c[1], c[2])
}

fn track(t: &TrackVm) -> ui::Track {
    ui::Track {
        id: t.id.as_str().into(),
        title: t.title.as_str().into(),
        artist: t.artist.as_str().into(),
        album: t.album.as_str().into(),
        cover: img(&t.cover),
        duration_label: t.duration_label.as_str().into(),
        codec: t.codec.as_str().into(),
        detail: t.detail.as_str().into(),
        detail_long: t.detail_long.as_str().into(),
        hires: t.hires,
        source: t.source.as_str().into(),
        liked: t.liked,
        playing: t.playing,
        why: t.why.as_str().into(),
    }
}

fn album(a: &AlbumVm) -> ui::Album {
    ui::Album { id: a.id.as_str().into(), title: a.title.as_str().into(), artist: a.artist.as_str().into(), cover: img(&a.cover) }
}

fn artist(a: &ArtistVm) -> ui::Artist {
    ui::Artist { id: a.id.as_str().into(), name: a.name.as_str().into(), subtitle: a.subtitle.as_str().into(), cover: img(&a.cover) }
}

fn highlight(h: &HighlightVm) -> ui::Highlight {
    ui::Highlight {
        id: h.id.as_str().into(),
        kind: h.kind.as_str().into(),
        caption: h.caption.as_str().into(),
        title: h.title.as_str().into(),
        subtitle: h.subtitle.as_str().into(),
        cover: img(&h.cover),
        tint: h.tint.map(color).unwrap_or_default(),
    }
}

fn load_state(s: LoadStateVm) -> ui::LoadState {
    match s {
        LoadStateVm::Loading => ui::LoadState::Loading,
        LoadStateVm::Ready => ui::LoadState::Ready,
        LoadStateVm::Empty => ui::LoadState::Empty,
        LoadStateVm::Error => ui::LoadState::Error,
        LoadStateVm::Offline => ui::LoadState::Offline,
    }
}

pub fn home_section(s: ui::HomeSection) -> HomeSectionVm {
    match s {
        ui::HomeSection::ForYou => HomeSectionVm::ForYou,
        ui::HomeSection::Discover => HomeSectionVm::Discover,
        ui::HomeSection::Mixes => HomeSectionVm::Mixes,
        ui::HomeSection::Focus => HomeSectionVm::Focus,
        ui::HomeSection::Workout => HomeSectionVm::Workout,
    }
}

type Rows<T> = Rc<VecModel<T>>;

/// Troca o conteúdo mantendo o mesmo `VecModel` (as listas já ligadas na UI continuam válidas).
fn fill<T: Clone + 'static, S>(model: &Rows<T>, src: &[S], f: impl Fn(&S) -> T) {
    model.set_vec(src.iter().map(f).collect::<Vec<_>>());
}

/// Muda só o campo `playing`/`liked` de linhas que mudaram (não recria capas nem rola a lista).
fn patch_tracks(model: &Rows<ui::Track>, f: impl Fn(&mut ui::Track) -> bool) {
    for i in 0..model.row_count() {
        if let Some(mut row) = model.row_data(i)
            && f(&mut row)
        {
            model.set_row_data(i, row);
        }
    }
}

pub struct SlintView {
    ui: Weak<ui::AppWindow>,
    highlights: Rows<ui::Highlight>,
    quick_picks: Rows<ui::Track>,
    mixes: Rows<ui::Highlight>,
    recents: Rows<ui::Track>,
    albums: Rows<ui::Album>,
    new_releases: Rows<ui::Album>,
    moods: Rows<ui::Highlight>,
    trending: Rows<ui::Track>,
    collections: Rows<ui::Collection>,
    playlists: Rows<ui::Playlist>,
    queue: Rows<ui::Track>,
    search_songs: Rows<ui::Track>,
    search_albums: Rows<ui::Album>,
    search_artists: Rows<ui::Artist>,
    search_playlists: Rows<ui::Highlight>,
    search_suggestions: Rows<SharedString>,
    recent_searches: Rows<SharedString>,
    detail_tracks: Rows<ui::Track>,
    detail_albums: Rows<ui::Album>,
    folders: Rows<SharedString>,
    providers: Rows<ui::ProviderRow>,
}

impl SlintView {
    /// Liga os modelos à janela uma única vez.
    pub fn new(window: &ui::AppWindow) -> Self {
        let v = Self {
            ui: window.as_weak(),
            highlights: Rc::default(),
            quick_picks: Rc::default(),
            mixes: Rc::default(),
            recents: Rc::default(),
            albums: Rc::default(),
            new_releases: Rc::default(),
            moods: Rc::default(),
            trending: Rc::default(),
            collections: Rc::default(),
            playlists: Rc::default(),
            queue: Rc::default(),
            search_songs: Rc::default(),
            search_albums: Rc::default(),
            search_artists: Rc::default(),
            search_playlists: Rc::default(),
            search_suggestions: Rc::default(),
            recent_searches: Rc::default(),
            detail_tracks: Rc::default(),
            detail_albums: Rc::default(),
            folders: Rc::default(),
            providers: Rc::default(),
        };
        let s = window.global::<ui::AppState>();
        s.set_highlights(ModelRc::from(v.highlights.clone()));
        s.set_quick_picks(ModelRc::from(v.quick_picks.clone()));
        s.set_mixes(ModelRc::from(v.mixes.clone()));
        s.set_recents(ModelRc::from(v.recents.clone()));
        s.set_albums(ModelRc::from(v.albums.clone()));
        s.set_new_releases(ModelRc::from(v.new_releases.clone()));
        s.set_moods(ModelRc::from(v.moods.clone()));
        s.set_trending(ModelRc::from(v.trending.clone()));
        s.set_collections(ModelRc::from(v.collections.clone()));
        s.set_playlists(ModelRc::from(v.playlists.clone()));
        s.set_queue(ModelRc::from(v.queue.clone()));
        s.set_search_songs(ModelRc::from(v.search_songs.clone()));
        s.set_search_albums(ModelRc::from(v.search_albums.clone()));
        s.set_search_artists(ModelRc::from(v.search_artists.clone()));
        s.set_search_playlists(ModelRc::from(v.search_playlists.clone()));
        s.set_search_suggestions(ModelRc::from(v.search_suggestions.clone()));
        s.set_recent_searches(ModelRc::from(v.recent_searches.clone()));
        s.set_detail_tracks(ModelRc::from(v.detail_tracks.clone()));
        s.set_detail_albums(ModelRc::from(v.detail_albums.clone()));
        s.set_folders(ModelRc::from(v.folders.clone()));
        s.set_providers(ModelRc::from(v.providers.clone()));
        v
    }

    fn with(&self, f: impl FnOnce(&ui::AppWindow)) {
        if let Some(w) = self.ui.upgrade() {
            f(&w);
        }
    }

    fn track_models(&self) -> [&Rows<ui::Track>; 6] {
        [&self.recents, &self.queue, &self.search_songs, &self.detail_tracks, &self.quick_picks, &self.trending]
    }
}

impl View for SlintView {
    fn set_playback(&self, p: &PlaybackVm) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_has_now(p.has_now);
            s.set_now(p.now.as_ref().map(track).unwrap_or_default());
            s.set_playing(p.playing);
            s.set_player_loading(p.loading);
            s.set_shuffle(p.shuffle);
            s.set_repeat(match p.repeat {
                RepeatVm::Off => ui::RepeatMode::Off,
                RepeatVm::All => ui::RepeatMode::All,
                RepeatVm::One => ui::RepeatMode::One,
            });
            s.set_output_name(p.output_name.as_str().into());
            s.set_output_note(p.output_note.as_str().into());
            s.set_bit_perfect(p.bit_perfect);
            s.set_now_origin(p.origin.as_str().into());
            s.set_now_source(p.source.as_str().into());
            s.set_quality_label(p.quality_label.as_str().into());
            s.set_player_error(p.error.as_str().into());
            s.set_player_error_action(p.error_action.as_str().into());
        });
    }

    fn set_progress(&self, progress: f32, position: &str, remaining: &str) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_progress(progress);
            s.set_position_label(position.into());
            s.set_remaining_label(remaining.into());
        });
    }

    fn set_now_art(&self, art: &NowArtVm) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_now_art(img(&art.cover));
            s.set_backdrop(img(&art.backdrop));
        });
    }

    fn set_ambient(&self, backdrop: Option<&Pixels>) {
        // sem faixa tocando, o palco usa a capa de uma faixa da biblioteca; com faixa, `set_now_art` manda
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            if !s.get_has_now() {
                s.set_backdrop(backdrop.map(|p| Image::from_rgba8(p.clone())).unwrap_or_default());
            }
        });
    }

    fn set_accent(&self, accent: &Accent, dynamic: bool) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_accent_dark(color(accent.dark.accent));
            s.set_accent_dark_ink(color(accent.dark.ink));
            s.set_accent_light(color(accent.light.accent));
            s.set_accent_light_ink(color(accent.light.ink));
            let t = w.global::<ui::Theme>();
            t.set_use_dynamic_accent(dynamic);
            // O palco herda o matiz da capa em qualquer modo de acento (é ambiente, não "cor de destaque").
            let stage = stage_palette(accent.hue);
            t.set_stage_a_dark(color(stage.dark[0]));
            t.set_stage_b_dark(color(stage.dark[1]));
            t.set_stage_c_dark(color(stage.dark[2]));
            t.set_stage_d_dark(color(stage.dark[3]));
            t.set_stage_a_light(color(stage.light[0]));
            t.set_stage_b_light(color(stage.light[1]));
            t.set_stage_c_light(color(stage.light[2]));
            t.set_stage_d_light(color(stage.light[3]));
            t.set_glass_tint_dark(color(stage.tint_dark));
            t.set_glass_tint_light(color(stage.tint_light));
            w.invoke_apply_accent();
        });
    }

    fn set_home(&self, home: &HomeVm) {
        fill(&self.highlights, &home.highlights, highlight);
        fill(&self.quick_picks, &home.quick_picks, track);
        fill(&self.mixes, &home.mixes, highlight);
        fill(&self.recents, &home.recents, track);
        fill(&self.albums, &home.albums, album);
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_home_state(load_state(home.state));
            s.set_home_section(match home.section {
                HomeSectionVm::ForYou => ui::HomeSection::ForYou,
                HomeSectionVm::Discover => ui::HomeSection::Discover,
                HomeSectionVm::Mixes => ui::HomeSection::Mixes,
                HomeSectionVm::Focus => ui::HomeSection::Focus,
                HomeSectionVm::Workout => ui::HomeSection::Workout,
            });
            s.set_home_stale(home.stale);
            s.set_home_error_title(home.error_title.as_str().into());
            s.set_home_error_text(home.error_text.as_str().into());
            s.set_home_error_code(home.error_code.as_str().into());
            s.set_source_label(home.source_label.as_str().into());
            s.set_first_run_banner(home.banner);
        });
    }

    fn set_explore(&self, explore: &ExploreVm) {
        fill(&self.new_releases, &explore.new_releases, album);
        fill(&self.moods, &explore.moods, highlight);
        fill(&self.trending, &explore.trending, track);
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_explore_state(load_state(explore.state));
            s.set_explore_error_text(explore.error_text.as_str().into());
        });
    }

    fn set_library(&self, library: &LibraryVm) {
        fill(&self.collections, &library.collections, |c| ui::Collection {
            id: c.id.as_str().into(),
            title: c.title.as_str().into(),
            count_label: c.count_label.as_str().into(),
            covers: images(&c.covers),
            soon: c.soon,
        });
        fill(&self.playlists, &library.playlists, |p| ui::Playlist {
            id: p.id.as_str().into(),
            name: p.name.as_str().into(),
            count_label: p.count_label.as_str().into(),
            covers: images(&p.covers),
        });
    }

    fn set_settings(&self, settings: &SettingsVm, providers: &[ProviderVm]) {
        fill(&self.folders, &settings.folders, |f| f.as_str().into());
        fill(&self.providers, providers, |p| ui::ProviderRow {
            id: p.id.as_str().into(),
            name: p.name.as_str().into(),
            status_label: p.status_label.as_str().into(),
            connected: p.connected,
            available: p.available,
        });
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_library_summary(settings.summary.as_str().into());
            s.set_has_library(settings.has_library);
        });
    }

    fn set_loading(&self, loading: bool) {
        self.with(|w| w.global::<ui::AppState>().set_loading(loading));
    }

    fn set_queue(&self, queue: &QueueVm) {
        fill(&self.queue, &queue.tracks, track);
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_queue_summary(queue.summary.as_str().into());
            s.set_queue_more_label(queue.more_label.as_str().into());
            s.set_queue_note(queue.note.as_str().into());
        });
    }

    fn set_search(&self, search: &SearchVm) {
        fill(&self.search_songs, &search.songs, track);
        fill(&self.search_albums, &search.albums, album);
        fill(&self.search_artists, &search.artists, artist);
        fill(&self.search_playlists, &search.playlists, highlight);
        fill(&self.search_suggestions, &search.suggestions, |s| s.as_str().into());
        fill(&self.recent_searches, &search.recent, |s| s.as_str().into());
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_search_state(match search.state {
                SearchStateVm::Idle => ui::SearchState::Idle,
                SearchStateVm::Results => ui::SearchState::Results,
                SearchStateVm::Empty => ui::SearchState::Empty,
                SearchStateVm::Loading => ui::SearchState::Loading,
                SearchStateVm::Error => ui::SearchState::Error,
                SearchStateVm::Offline => ui::SearchState::Offline,
            });
            s.set_search_empty_text(search.empty_text.as_str().into());
            s.set_search_error_text(search.error_text.as_str().into());
            s.set_search_has_best(search.best.is_some());
            s.set_search_best(search.best.as_ref().map(highlight).unwrap_or_default());
        });
    }

    fn set_detail(&self, detail: &DetailVm) {
        fill(&self.detail_tracks, &detail.tracks, track);
        fill(&self.detail_albums, &detail.albums, album);
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_detail_kind(match detail.kind {
                DetailKindVm::Tracks => ui::DetailKind::Tracks,
                DetailKindVm::Albums => ui::DetailKind::Albums,
                DetailKindVm::Artist => ui::DetailKind::Artist,
            });
            s.set_detail_state(load_state(detail.state));
            s.set_detail_title(detail.title.as_str().into());
            s.set_detail_subtitle(detail.subtitle.as_str().into());
            s.set_detail_cover(img(&detail.cover));
            s.set_detail_source(detail.source.as_str().into());
            s.set_detail_empty_text(detail.empty_text.as_str().into());
            s.set_detail_error_text(detail.error_text.as_str().into());
            s.set_detail_saved(detail.saved);
            s.set_detail_can_save(detail.can_save);
        });
    }

    fn open_detail(&self, open: bool) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_detail_open(open);
            if open {
                s.set_queue_open(false);
            }
        });
    }

    fn set_scan(&self, scan: Option<&ScanVm>) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_scanning(scan.is_some());
            if let Some(p) = scan {
                s.set_scan_progress(p.progress);
                s.set_scan_label(p.label.as_str().into());
            }
        });
    }

    fn toast(&self, text: &str, error: bool, action: &str) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_toast_error(error);
            s.set_toast_text(text.into());
            s.set_toast_action_label(action.into());
            s.set_toast_seq(s.get_toast_seq().wrapping_add(1));
        });
    }

    fn mark_playing(&self, id: Option<&str>) {
        let f = |t: &mut ui::Track| {
            let now = id.is_some_and(|id| t.id == id);
            let changed = t.playing != now;
            t.playing = now;
            changed
        };
        for m in self.track_models() {
            patch_tracks(m, f);
        }
    }

    fn mark_liked(&self, id: &str, liked: bool) {
        let f = |t: &mut ui::Track| {
            let hit = t.id == id && t.liked != liked;
            if hit {
                t.liked = liked;
            }
            hit
        };
        for m in self.track_models() {
            patch_tracks(m, f);
        }
    }

    fn set_prefs(&self, prefs: &PrefsVm) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_theme_mode(match prefs.theme {
                ThemePref::Auto => ui::ThemeMode::Auto,
                ThemePref::Light => ui::ThemeMode::Light,
                ThemePref::Dark => ui::ThemeMode::Dark,
            });
            s.set_accent_mode(match prefs.accent {
                AccentPref::Cover => ui::AccentMode::FromCover,
                AccentPref::Brand => ui::AccentMode::Brand,
            });
            s.set_stream_quality(prefs.stream_quality.index());
            s.set_autoplay_radio(prefs.autoplay_radio);
            let t = w.global::<ui::Theme>();
            t.set_reduce_motion(prefs.reduce_motion);
            t.set_reduce_transparency(prefs.reduce_transparency);
            w.invoke_apply_theme();
        });
    }

    fn set_volume(&self, volume: f32) {
        self.with(|w| w.global::<ui::AppState>().set_volume(volume));
    }

    fn set_online(&self, online: bool) {
        self.with(|w| w.global::<ui::AppState>().set_online(online));
    }

    fn set_perf_offer(&self, offer: bool) {
        self.with(|w| w.global::<ui::AppState>().set_perf_offer(offer));
    }

    fn close_sheets(&self) {
        self.with(|w| {
            let s = w.global::<ui::AppState>();
            s.set_menu_open(false);
            s.set_playlist_sheet_open(false);
            s.set_new_playlist_open(false);
        });
    }
}
