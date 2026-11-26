//! Modo `--snapshot`: percorre as telas, em larguras desktop e mobile, e grava PNGs. É a verificação visual do app
//! (usa o renderer ativo; com FemtoVG captura o vidro desfocado de verdade).

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Model};

use crate::app::{App, AppError};
use crate::config::ThemePref;
use crate::ui;

type Action = Box<dyn Fn(&App)>;

/// Envia uma tecla (pressionar + soltar) à janela, como o teclado faria.
fn key(app: &App, text: impl Into<slint::SharedString>) {
    let text: slint::SharedString = text.into();
    let w = app.window.window();
    w.dispatch_event(slint::platform::WindowEvent::KeyPressed { text: text.clone() });
    w.dispatch_event(slint::platform::WindowEvent::KeyReleased { text });
}

struct Step {
    /// Nome do arquivo (sem extensão); `None` = só executa a ação.
    shot: Option<String>,
    action: Action,
    settle: Duration,
}

fn step(shot: Option<&str>, settle_ms: u64, action: impl Fn(&App) + 'static) -> Step {
    Step { shot: shot.map(str::to_owned), action: Box::new(action), settle: Duration::from_millis(settle_ms) }
}

fn go(tab: ui::Tab) -> impl Fn(&App) {
    move |app| {
        let s = app.window.global::<ui::AppState>();
        s.set_player_open(false);
        s.set_queue_open(false);
        s.set_detail_open(false);
        s.set_tab(tab);
    }
}

fn first_pick_id(app: &App) -> Option<String> {
    app.window.global::<ui::AppState>().get_quick_picks().row_data(0).map(|t| t.id.to_string())
}

fn first_recent_id(app: &App) -> Option<String> {
    app.window.global::<ui::AppState>().get_recents().row_data(0).map(|t| t.id.to_string())
}

fn close_all(app: &App) {
    let s = app.window.global::<ui::AppState>();
    s.set_player_open(false);
    s.set_queue_open(false);
    s.set_detail_open(false);
    s.set_menu_open(false);
    s.set_playlist_sheet_open(false);
    s.set_new_playlist_open(false);
}

/// Plano de capturas para um tamanho e um tema. `full`: inclui reprodução, folhas e detalhe.
fn plan(label: &'static str, w: f32, h: f32, theme: ThemePref, full: bool) -> Vec<Step> {
    let n = move |s: &str| format!("{label}-{s}");
    let mut v = vec![
        step(None, 500, move |app| {
            app.controller.borrow_mut().set_theme(theme);
            app.window.window().set_maximized(false);
            app.window.global::<ui::AppState>().set_mobile(w < 1100.0);
            app.window.window().set_size(slint::LogicalSize::new(w, h));
        }),
        // o Início do YouTube Music (ou só o local, com --no-youtube) precisa de tempo para chegar
        step(Some(&n("home")), 2500, go(ui::Tab::Home)),
        step(Some(&n("explore")), 2500, go(ui::Tab::Explore)),
        step(Some(&n("library")), 700, go(ui::Tab::Library)),
        step(Some(&n("search-idle")), 500, go(ui::Tab::Search)),
        step(Some(&n("search-results")), 2500, |app| {
            app.window.global::<ui::AppState>().set_search_text("aurora".into());
            app.controller.borrow_mut().search("aurora");
        }),
        step(Some(&n("settings")), 600, |app| {
            app.window.global::<ui::AppState>().set_search_text("".into());
            go(ui::Tab::Settings)(app);
        }),
        step(Some(&n("settings-appearance")), 500, |app| app.window.global::<ui::AppState>().set_settings_section(2)),
        step(Some(&n("settings-playback")), 500, |app| app.window.global::<ui::AppState>().set_settings_section(1)),
        step(Some(&n("settings-search")), 500, |app| {
            let state = app.window.global::<ui::AppState>();
            state.set_settings_section(-1);
            state.set_settings_query("vidro".into());
        }),
        step(None, 100, |app| {
            let state = app.window.global::<ui::AppState>();
            state.set_settings_query("".into());
            state.set_settings_section(0);
        }),
        // janela alta: mostra Ajustes inteira sem precisar rolar
        step(None, 500, move |app| app.window.window().set_size(slint::LogicalSize::new(w, 1900.0))),
        step(Some(&n("settings-tall")), 500, move |app| app.window.window().set_size(slint::LogicalSize::new(w, 1900.0))),
        step(None, 500, move |app| app.window.window().set_size(slint::LogicalSize::new(w, h))),
        // Início rolado: o cabeçalho recolhido (vidro) aparece. A rolagem de cada aba fica em `Metrics`; a tela a lê ao abrir.
        step(None, 300, |app| {
            go(ui::Tab::Explore)(app);
            app.window.global::<ui::Metrics>().set_scroll_home(-440.0);
        }),
        step(Some(&n("home-scrolled")), 600, |app| go(ui::Tab::Home)(app)),
        step(None, 300, |app| {
            go(ui::Tab::Explore)(app);
            app.window.global::<ui::Metrics>().set_scroll_home(0.0);
        }),
        // foco por teclado
        step(None, 300, |app| go(ui::Tab::Home)(app)),
        step(Some(&n("home-focus")), 400, |app| {
            for _ in 0..4 {
                key(app, slint::platform::Key::Tab);
            }
        }),
    ];
    if full {
        v.extend([
            step(None, 3000, |app| {
                go(ui::Tab::Home)(app);
                if let Some(id) = first_pick_id(app).or_else(|| first_recent_id(app)) {
                    app.controller.borrow_mut().play_track(&id);
                }
            }),
            step(Some(&n("home-playing")), 600, go(ui::Tab::Home)),
            step(Some(&n("library-playing")), 600, go(ui::Tab::Library)),
            step(Some(&n("player")), 900, |app| app.window.global::<ui::AppState>().set_player_open(true)),
            step(Some(&n("player-paused")), 500, |app| key(app, " ")),
            step(None, 300, |app| key(app, " ")),
            step(Some(&n("queue")), 900, |app| {
                let s = app.window.global::<ui::AppState>();
                s.set_player_open(false);
                s.set_queue_open(true);
            }),
            step(Some(&n("menu")), 700, |app| {
                let s = app.window.global::<ui::AppState>();
                s.set_queue_open(false);
                s.set_menu_track(s.get_now());
                s.set_menu_open(true);
            }),
            step(Some(&n("playlist-sheet")), 600, |app| {
                let s = app.window.global::<ui::AppState>();
                s.set_menu_open(false);
                s.set_playlist_sheet_open(true);
            }),
            step(Some(&n("new-playlist")), 600, |app| {
                let s = app.window.global::<ui::AppState>();
                s.set_playlist_sheet_open(false);
                s.set_new_playlist_open(true);
            }),
            step(Some(&n("detail")), 2500, |app| {
                close_all(app);
                let s = app.window.global::<ui::AppState>();
                if let Some(a) = s.get_albums().row_data(0) {
                    app.controller.borrow_mut().open_album(&a.id);
                } else if let Some(h) = s.get_highlights().row_data(0) {
                    app.controller.borrow_mut().open_item(&h.kind, &h.id);
                }
            }),
            step(Some(&n("detail-liked")), 900, |app| {
                close_all(app);
                app.controller.borrow_mut().open_collection("liked");
            }),
            step(None, 300, close_all),
        ]);
    }
    v
}

fn save(app: &App, dir: &Path, name: &str) {
    match app.window.window().take_snapshot() {
        Ok(buf) => {
            let path = dir.join(format!("{name}.png"));
            match image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec()) {
                Some(img) => match img.save(&path) {
                    Ok(()) => println!("capturado: {}", path.display()),
                    Err(e) => eprintln!("não foi possível salvar {}: {e}", path.display()),
                },
                None => eprintln!("buffer inesperado em {name}"),
            }
        }
        Err(e) => eprintln!("captura de {name} falhou: {e}"),
    }
}

fn run_steps(app: Rc<App>, steps: Rc<Vec<Step>>, i: usize, dir: Rc<PathBuf>) {
    let Some(st) = steps.get(i) else {
        let _ = slint::quit_event_loop();
        return;
    };
    (st.action)(&app);
    let settle = st.settle;
    slint::Timer::single_shot(settle, move || {
        if let Some(name) = &steps[i].shot {
            save(&app, &dir, name);
        }
        run_steps(app.clone(), steps.clone(), i + 1, dir.clone());
    });
}

/// Espera a biblioteca carregar e roda o plano. Devolve ao terminar.
pub fn run(app: App, dir: PathBuf, software: bool) -> Result<(), AppError> {
    std::fs::create_dir_all(&dir)?;
    let tag_env = std::env::var("KAZZU_SNAPSHOT_TAG").unwrap_or_default();
    let prefix = format!("{}{}", if tag_env.is_empty() { String::new() } else { format!("{tag_env}-") }, if software { "sw" } else { "gl" });
    let mut steps: Vec<Step> = Vec::new();
    let tag = |s: &'static str| -> &'static str { Box::leak(format!("{prefix}-{s}").into_boxed_str()) };
    // A native run may target selected sizes to verify window constraints without replaying the whole matrix.
    let only = std::env::var("KAZZU_SNAPSHOT_ONLY").unwrap_or_default();
    for (label, w, h, theme, full) in [
        ("mobile-dark", 390.0, 844.0, ThemePref::Dark, true),
        ("narrow-dark", 360.0, 740.0, ThemePref::Dark, true),
        ("mobile-light", 390.0, 844.0, ThemePref::Light, true),
        ("tablet-dark", 768.0, 1024.0, ThemePref::Dark, false),
        ("desktop-dark", 1280.0, 800.0, ThemePref::Dark, true),
        ("desktop-light", 1280.0, 800.0, ThemePref::Light, false),
    ] {
        if only.is_empty() || only.split(',').any(|s| s.trim() == label) {
            steps.extend(plan(tag(label), w, h, theme, full));
        }
    }
    steps.push(step(Some(&format!("{prefix}-mobile-light-reduce-transparency")), 700, |app| {
        app.window.global::<ui::AppState>().set_mobile(true);
        app.window.window().set_size(slint::LogicalSize::new(390.0, 844.0));
        app.controller.borrow_mut().set_reduce_transparency(true);
        go(ui::Tab::Library)(app);
    }));
    let app = Rc::new(app);
    let steps = Rc::new(steps);
    let dir = Rc::new(dir);
    // aguarda os dados (scan do demo) antes de começar
    let waiter = Rc::new(slint::Timer::default());
    let started = std::time::Instant::now();
    let w2 = waiter.clone();
    let (a2, s2, d2) = (app.clone(), steps.clone(), dir.clone());
    waiter.start(slint::TimerMode::Repeated, Duration::from_millis(200), move || {
        let st = a2.window.global::<ui::AppState>();
        let loaded = !st.get_loading() && !st.get_scanning();
        // biblioteca vazia (primeira execução): sem faixas, espera só um instante
        let ready = loaded && (st.get_recents().row_count() > 0 || st.get_quick_picks().row_count() > 0 || started.elapsed() > Duration::from_secs(6));
        if ready || started.elapsed() > Duration::from_secs(30) {
            w2.stop();
            // pequena folga para as capas do primeiro desenho
            let (a3, s3, d3) = (a2.clone(), s2.clone(), d2.clone());
            slint::Timer::single_shot(Duration::from_millis(600), move || run_steps(a3, s3, 0, d3));
        }
    });
    app.window.run()?;
    Ok(())
}
