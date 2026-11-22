//! Cenas: cada uma põe a janela real (`AppWindow`) num estado conhecido. Tudo passa por AppState/Theme, como no app.

use slint::{ComponentHandle, Image, Model, ModelRc};

use crate::art::{self, Kind};
use crate::data::{model, ss, strings, World};
use crate::{AppState, AppWindow, DetailKind, HomeSection, LoadState, Metrics, RepeatMode, SearchState, Tab, Theme, ThemeMode, Track};

/// Cenas com a tela já rolada (testam o conteúdo passando sob a barra de status e os botões do sistema).
pub const SCROLL_SCENES: &[&str] = &["home-scrolled", "library-scrolled"];

pub const SCENES: &[&str] = &[
    "home-loading", "home-ready", "home-first-run", "home-offline", "home-offline-nolib", "home-error", "home-stale",
    "explore-ready", "explore-loading", "explore-error",
    "library-full", "library-empty", "library-scan",
    "search-idle", "search-results", "search-empty", "search-loading", "search-error", "search-offline",
    "detail-album", "detail-artist", "detail-empty", "detail-grid",
    "player-orange", "player-pale", "player-gray", "player-loading", "player-error", "player-longnote",
    "queue-open", "menu-open", "playlist-open", "newplaylist-open",
    "settings-all", "toast-action", "perf-offer",
    "home-nocover", "library-nocover", "player-nocover", "search-nocover",
];

const NOW_KINDS: [(usize, Kind); 3] = [(0, Kind::Sunset), (2, Kind::Pale), (5, Kind::Gray)];

/// Faixa atual: capa grande, backdrop, palco, vidro e acento derivados do matiz da capa (a receita do Rust do app).
pub fn set_now(win: &AppWindow, world: &World, idx: usize, playing: bool) {
    let s = win.global::<AppState>();
    let th = win.global::<Theme>();
    let d = &crate::data::TRACKS[idx];
    s.set_now(Track { playing, liked: idx.is_multiple_of(5), ..world.track(idx) });
    s.set_has_now(true);
    s.set_playing(playing);
    s.set_now_art(art::image(d.kind, 512));
    s.set_backdrop(art::backdrop(d.kind));
    let c = art::stage_colors(art::cover_hue(d.kind));
    th.set_stage_a_dark(c.dark[0]);
    th.set_stage_b_dark(c.dark[1]);
    th.set_stage_c_dark(c.dark[2]);
    th.set_stage_d_dark(c.dark[3]);
    th.set_stage_a_light(c.light[0]);
    th.set_stage_b_light(c.light[1]);
    th.set_stage_c_light(c.light[2]);
    th.set_stage_d_light(c.light[3]);
    th.set_glass_tint_dark(c.tint_dark);
    th.set_glass_tint_light(c.tint_light);
    s.set_accent_dark(c.accent_dark);
    s.set_accent_dark_ink(c.accent_dark_ink);
    s.set_accent_light(c.accent_light);
    s.set_accent_light_ink(c.accent_light_ink);
    th.set_use_dynamic_accent(true);
    win.invoke_apply_accent();
    s.set_now_origin(ss("Mix Foco"));
    s.set_now_source(ss("Do YouTube Music"));
    s.set_quality_label(ss(&format!("{} {}", d.codec, d.detail_long)));
    s.set_output_note(ss("Original, sem equalizador"));
    s.set_position_label(ss("1:02"));
    s.set_remaining_label(ss("−2:19"));
    s.set_progress(0.31);
}

pub fn clear_now(win: &AppWindow) {
    let s = win.global::<AppState>();
    s.set_has_now(false);
    s.set_playing(false);
    s.set_backdrop(Image::default());
    let th = win.global::<Theme>();
    th.set_use_dynamic_accent(false);
}

fn blank_tracks(m: ModelRc<Track>) -> ModelRc<Track> {
    model(m.iter().map(|t| Track { cover: Image::default(), ..t }).collect())
}

/// Sem nenhuma capa (Android com só a permissão de áudio: sem cover.jpg da pasta; só capas embutidas na tag).
fn strip_covers(win: &AppWindow, world: &World, only_some: bool) {
    let s = win.global::<AppState>();
    let blank_hl = |m: ModelRc<crate::Highlight>| model(m.iter().map(|h| crate::Highlight { cover: Image::default(), ..h }).collect::<Vec<_>>());
    let blank_albums = |m: ModelRc<crate::Album>| model(m.iter().map(|a| crate::Album { cover: Image::default(), ..a }).collect::<Vec<_>>());
    s.set_highlights(blank_hl(s.get_highlights()));
    s.set_mixes(blank_hl(s.get_mixes()));
    s.set_quick_picks(blank_tracks(s.get_quick_picks()));
    s.set_albums(blank_albums(s.get_albums()));
    s.set_new_releases(blank_albums(s.get_new_releases()));
    s.set_recents(blank_tracks(s.get_recents()));
    s.set_queue(blank_tracks(s.get_queue()));
    s.set_search_songs(blank_tracks(s.get_search_songs()));
    s.set_search_albums(blank_albums(s.get_search_albums()));
    s.set_search_best(crate::Highlight { cover: Image::default(), ..s.get_search_best() });
    s.set_detail_tracks(blank_tracks(s.get_detail_tracks()));
    s.set_detail_cover(Image::default());
    s.set_now(Track { cover: Image::default(), ..s.get_now() });
    s.set_menu_track(Track { cover: Image::default(), ..s.get_menu_track() });
    s.set_now_art(Image::default());
    s.set_backdrop(Image::default());
    // coleções e playlists: Álbuns com 1 de 4 capas, Playlists e "Para correr" sem nenhuma (e uma com 2)
    let mut cols = world.collections(10);
    if let Some(c) = cols.row_data(2) {
        let one = c.covers.row_data(0).map(|i| vec![i]).unwrap_or_default();
        let mut c2 = c;
        c2.covers = model(one);
        cols = model((0..4).map(|i| if i == 2 { c2.clone() } else { cols.row_data(i).unwrap() }).collect());
        if let Some(mut p) = cols.row_data(3) {
            p.covers = model(vec![]);
            let v: Vec<_> = (0..4).map(|i| if i == 3 { p.clone() } else { cols.row_data(i).unwrap() }).collect();
            cols = model(v);
        }
    }
    s.set_collections(cols);
    let mut pls = world.playlists().iter().collect::<Vec<_>>();
    pls[0].covers = model(vec![]);
    pls[1].covers = model(pls[1].covers.iter().take(2).collect());
    s.set_playlists(model(pls));
    let _ = only_some;
}

/// Estado base: tudo pronto, nada aberto, faixa tocando.
pub fn reset(win: &AppWindow, world: &World) {
    let s = win.global::<AppState>();
    let m = win.global::<Metrics>();
    m.set_scroll_home(0.0);
    m.set_scroll_explore(0.0);
    m.set_scroll_library(0.0);
    m.set_scroll_search(0.0);
    m.set_scroll_settings(0.0);
    s.set_tab(Tab::Home);
    s.set_player_open(false);
    s.set_queue_open(false);
    s.set_menu_open(false);
    s.set_playlist_sheet_open(false);
    s.set_new_playlist_open(false);
    s.set_detail_open(false);
    s.set_online(true);
    s.set_toast_text(ss(""));
    s.set_toast_action_label(ss(""));
    s.set_perf_offer(false);
    s.set_player_loading(false);
    s.set_player_error(ss(""));
    s.set_shuffle(false);
    s.set_repeat(RepeatMode::Off);
    s.set_search_text(ss(""));
    s.set_search_kind(0);
    s.set_search_state(SearchState::Idle);

    s.set_home_state(LoadState::Ready);
    s.set_home_stale(false);
    s.set_first_run_banner(false);
    s.set_home_section(HomeSection::ForYou);
    s.set_source_label(ss("Do YouTube Music"));
    s.set_highlights(world.highlights());
    s.set_quick_picks(world.quick_picks());
    s.set_mixes(world.mixes());
    s.set_albums(world.albums());
    s.set_recents(world.tracks(&[0, 3, 8, 12, 16, 20]));
    s.set_home_error_title(ss(""));
    s.set_home_error_text(ss(""));
    s.set_home_error_code(ss(""));

    s.set_explore_state(LoadState::Ready);
    s.set_new_releases(world.new_releases());
    s.set_moods(world.moods());
    s.set_trending(world.trending());

    s.set_collections(world.collections(10));
    s.set_playlists(world.playlists());
    s.set_has_library(true);
    s.set_scanning(false);
    s.set_scan_progress(0.0);
    s.set_loading(false);
    s.set_folders(strings(&["/storage/emulated/0/Music", "/storage/emulated/0/Download/Albuns"]));
    s.set_library_summary(ss("2 pastas, 1.284 músicas"));

    s.set_recent_searches(strings(&["Nina Sabiá", "música para foco", "Meridiano"]));
    s.set_search_suggestions(strings(&["aurora vento", "aurora boreal", "aurora ao vivo"]));
    s.set_search_has_best(true);
    s.set_search_best(world.hl("ar0", "artist", "", "Aurora Vento", "Artista, 2,1 mi de ouvintes", Kind::Teal));
    s.set_search_songs(world.tracks(&[4, 12, 1, 7, 9, 2, 3, 10]));
    s.set_search_albums(world.new_releases());
    s.set_search_artists(world.artists());
    s.set_search_playlists(world.search_playlists());

    s.set_queue(world.tracks(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]));
    s.set_queue_summary(ss("A seguir, 12 músicas, 41 min"));
    s.set_queue_more_label(ss("e mais 1.900 músicas"));
    s.set_queue_note(ss("Quando a fila acabar, o Kazzu continua com músicas parecidas do YouTube Music."));
    s.set_menu_track(world.track(1));
    s.set_playlist_sheet_track(ss("t1"));

    s.set_detail_state(LoadState::Ready);
    s.set_detail_kind(DetailKind::Tracks);
    s.set_detail_title(ss("Calor de Março"));
    s.set_detail_subtitle(ss("Nina Sabiá, 2025, 10 músicas"));
    s.set_detail_cover(art::image(Kind::Sunset, 384));
    s.set_detail_source(ss("Do YouTube Music"));
    s.set_detail_can_save(true);
    s.set_detail_saved(false);
    s.set_detail_tracks(world.tracks(&[0, 10, 17, 3, 5, 8, 11, 13]));
    s.set_detail_albums(world.albums());
    s.set_detail_empty_text(ss("Curta músicas para elas aparecerem aqui."));

    s.set_providers(world.providers());
    s.set_outputs(world.outputs());
    s.set_output_name(ss("Este celular"));
    s.set_sleep_timer_label(ss("Timer"));
    s.set_stream_quality(0);
    s.set_autoplay_radio(true);
    s.set_version_label(ss("Kazzu 0.1.0"));
    s.set_licenses_text(ss("Kazzu: GPL-3.0-or-later\nSlint: GPL-3.0-only\nHanken Grotesk, Geist Mono: SIL OFL 1.1\nsymphonia, cpal, rusqlite: MIT/Apache-2.0"));

    set_now(win, world, 0, true);
}

pub fn apply(win: &AppWindow, world: &World, name: &str) {
    reset(win, world);
    let s = win.global::<AppState>();
    match name {
        "home-loading" => s.set_home_state(LoadState::Loading),
        "home-ready" => {}
        "home-first-run" => {
            s.set_first_run_banner(true);
            clear_now(win);
        }
        "home-offline" => {
            s.set_online(false);
            s.set_home_state(LoadState::Offline);
        }
        "home-offline-nolib" => {
            // sem pastas e sem músicas: o cartão oferece "Adicionar pasta"
            s.set_online(false);
            s.set_home_state(LoadState::Offline);
            s.set_has_library(false);
            s.set_recents(model(vec![]));
            s.set_library_summary(ss(""));
        }
        "home-error" => {
            s.set_home_state(LoadState::Error);
            s.set_home_error_code(ss("YTM-5xx · 2026-11-22T10:41"));
        }
        "home-stale" => {
            s.set_home_state(LoadState::Error);
            s.set_home_stale(true);
        }
        "explore-ready" => s.set_tab(Tab::Explore),
        "explore-loading" => {
            s.set_tab(Tab::Explore);
            s.set_explore_state(LoadState::Loading);
        }
        "explore-error" => {
            s.set_tab(Tab::Explore);
            s.set_explore_state(LoadState::Error);
        }
        "library-full" => s.set_tab(Tab::Library),
        "library-empty" => {
            s.set_tab(Tab::Library);
            s.set_has_library(false);
            s.set_playlists(model(vec![]));
            s.set_recents(model(vec![]));
            s.set_collections(world.collections(0));
            s.set_library_summary(ss(""));
            s.set_folders(strings(&[]));
        }
        "library-scan" => {
            s.set_tab(Tab::Library);
            s.set_scanning(true);
            s.set_scan_progress(0.42);
            s.set_scan_label(ss("Lendo /storage/emulated/0/Music, 538 de 1.284"));
        }
        "search-idle" => s.set_tab(Tab::Search),
        "search-results" => {
            s.set_tab(Tab::Search);
            s.set_search_text(ss("aurora"));
            s.set_search_state(SearchState::Results);
        }
        "search-empty" => {
            s.set_tab(Tab::Search);
            s.set_search_text(ss("xyzqw"));
            s.set_search_state(SearchState::Empty);
            s.set_search_empty_text(ss("Nada para “xyzqw”."));
        }
        "search-loading" => {
            s.set_tab(Tab::Search);
            s.set_search_text(ss("aurora"));
            s.set_search_state(SearchState::Loading);
        }
        "search-error" => {
            s.set_tab(Tab::Search);
            s.set_search_text(ss("aurora"));
            s.set_search_state(SearchState::Error);
        }
        "search-offline" => {
            s.set_tab(Tab::Search);
            s.set_online(false);
            s.set_search_text(ss("aurora"));
            s.set_search_state(SearchState::Offline);
        }
        "detail-album" => s.set_detail_open(true),
        "detail-artist" => {
            s.set_detail_open(true);
            s.set_detail_kind(DetailKind::Artist);
            s.set_detail_title(ss("Os Faróis"));
            s.set_detail_subtitle(ss("Banda, 1,4 mi de ouvintes"));
            s.set_detail_cover(art::image(Kind::Radar, 384));
            s.set_detail_can_save(false);
        }
        "detail-empty" => {
            s.set_detail_open(true);
            s.set_detail_state(LoadState::Empty);
            s.set_detail_title(ss("Curtidas"));
            s.set_detail_subtitle(ss("0 músicas"));
            s.set_detail_source(ss(""));
            s.set_detail_can_save(false);
        }
        "detail-grid" => {
            s.set_detail_open(true);
            s.set_detail_kind(DetailKind::Albums);
            s.set_detail_title(ss("Álbuns"));
            s.set_detail_subtitle(ss("8 álbuns"));
            s.set_detail_source(ss(""));
        }
        "player-orange" => {
            s.set_player_open(true);
        }
        "player-pale" => {
            set_now(win, world, NOW_KINDS[1].0, true);
            s.set_player_open(true);
        }
        "player-gray" => {
            set_now(win, world, NOW_KINDS[2].0, true);
            s.set_player_open(true);
        }
        "player-loading" => {
            s.set_player_open(true);
            s.set_player_loading(true);
            s.set_progress(0.0);
            s.set_position_label(ss("0:00"));
        }
        "player-error" => {
            s.set_player_open(true);
            s.set_player_error(ss("Não deu para tocar esta faixa. Esta faixa não está disponível na sua região."));
        }
        "player-longnote" => {
            // nota da saída longa: tem de quebrar/terminar em reticências sem estourar a folha (360 px de largura)
            s.set_player_open(true);
            s.set_output_note(ss("Qualidade do YouTube Music, sem equalizador, reamostrado de 96 para 48 kHz pelo sistema"));
        }
        "queue-open" => {
            s.set_player_open(true);
            s.set_queue_open(true);
        }
        "menu-open" => {
            s.set_player_open(false);
            s.set_menu_track(world.track(1));
            s.set_menu_open(true);
        }
        "playlist-open" => {
            s.set_playlist_sheet_open(true);
        }
        "newplaylist-open" => {
            s.set_new_playlist_open(true);
        }
        "home-scrolled" => {
            win.global::<Metrics>().set_scroll_home(-430.0);
            s.set_tab(Tab::Home);
        }
        "library-scrolled" => {
            win.global::<Metrics>().set_scroll_library(-300.0);
            s.set_tab(Tab::Library);
        }
        "home-nocover" => strip_covers(win, world, false),
        "library-nocover" => {
            s.set_tab(Tab::Library);
            strip_covers(win, world, true);
        }
        "player-nocover" => {
            s.set_player_open(true);
            strip_covers(win, world, false);
        }
        "search-nocover" => {
            s.set_tab(Tab::Search);
            s.set_search_text(ss("aurora"));
            s.set_search_state(SearchState::Results);
            strip_covers(win, world, false);
        }
        "settings-all" => s.set_tab(Tab::Settings),
        "toast-action" => {
            s.set_toast_text(ss("Adicionada às Curtidas"));
            s.set_toast_action_label(ss("Desfazer"));
            s.set_toast_seq(s.get_toast_seq() + 1);
        }
        "perf-offer" => s.set_perf_offer(true),
        other => eprintln!("cena desconhecida: {other}"),
    }
}

/// Tema e transparência da janela (Rust do app faz o mesmo: escreve em Theme e chama apply-theme).
pub fn set_look(win: &AppWindow, dark: bool, reduce_transparency: bool, blur_supported: bool) {
    let th = win.global::<Theme>();
    th.set_reduce_transparency(reduce_transparency);
    th.set_blur_supported(blur_supported);
    win.global::<AppState>().set_theme_mode(if dark { ThemeMode::Dark } else { ThemeMode::Light });
    win.invoke_apply_theme();
}

pub fn set_safe(win: &AppWindow, top: f32, bottom: f32) {
    let s = win.global::<AppState>();
    s.set_safe_top(top);
    s.set_safe_bottom(bottom);
}

/// Callbacks mínimos para o modo `--interactive` (a UI real age sobre dados falsos).
pub fn wire(win: &AppWindow, world: std::rc::Rc<World>) {
    let s = win.global::<AppState>();
    let w = win.as_weak();
    s.on_toggle_play({
        let w = w.clone();
        move || {
            if let Some(w) = w.upgrade() {
                let st = w.global::<AppState>();
                st.set_playing(!st.get_playing());
            }
        }
    });
    s.on_toggle_like({
        let w = w.clone();
        move |_| {
            if let Some(w) = w.upgrade() {
                let st = w.global::<AppState>();
                let mut n = st.get_now();
                n.liked = !n.liked;
                st.set_now(n);
                st.set_toast_text(ss(if st.get_now().liked { "Adicionada às Curtidas" } else { "Removida das Curtidas" }));
                st.set_toast_action_label(ss("Desfazer"));
                st.set_toast_seq(st.get_toast_seq() + 1);
            }
        }
    });
    s.on_seek({
        let w = w.clone();
        move |v| {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_progress(v);
            }
        }
    });
    s.on_play_track({
        let w = w.clone();
        let world = world.clone();
        move |id| {
            if let Some(w) = w.upgrade() {
                let idx: usize = id.trim_start_matches('t').parse().unwrap_or(0);
                set_now(&w, &world, idx, true);
            }
        }
    });
    s.on_search({
        let w = w.clone();
        move |text| {
            if let Some(w) = w.upgrade() {
                let st = w.global::<AppState>();
                st.set_search_state(if text.is_empty() { SearchState::Idle } else if text.contains("xyz") { SearchState::Empty } else { SearchState::Results });
                st.set_search_empty_text(ss(&format!("Nada para “{text}”.")));
            }
        }
    });
    s.on_open_album({
        let w = w.clone();
        move |_| {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_detail_open(true);
            }
        }
    });
    s.on_open_item({
        let w = w.clone();
        move |_, _| {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_detail_open(true);
            }
        }
    });
    s.on_open_collection({
        let w = w.clone();
        move |_| {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_detail_open(true);
            }
        }
    });
    s.on_queue_remove({
        let w = w.clone();
        move |_| {
            if let Some(w) = w.upgrade() {
                let st = w.global::<AppState>();
                st.set_toast_text(ss("Removida da fila"));
                st.set_toast_seq(st.get_toast_seq() + 1);
            }
        }
    });
    s.on_dismiss_first_run_banner({
        let w = w.clone();
        move || {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_first_run_banner(false);
            }
        }
    });
    s.on_perf_offer_dismiss({
        let w = w.clone();
        move || {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_perf_offer(false);
            }
        }
    });
    s.on_toast_action({
        let w = w.clone();
        move || {
            if let Some(w) = w.upgrade() {
                w.global::<AppState>().set_toast_text(ss(""));
            }
        }
    });
    s.on_theme_mode_changed({
        let w = w.clone();
        move |_| {
            if let Some(w) = w.upgrade() {
                w.invoke_apply_theme();
            }
        }
    });
    s.on_create_playlist_named({
        let w = w.clone();
        move |_, _| {
            if let Some(w) = w.upgrade() {
                let st = w.global::<AppState>();
                st.set_toast_text(ss("Playlist criada"));
                st.set_toast_seq(st.get_toast_seq() + 1);
            }
        }
    });
}
