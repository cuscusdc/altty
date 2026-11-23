//! Teste de renderização das telas Slint, sem janela de verdade: plataforma de teste com o renderer de software.
//! Garante que todas as telas compilam, instanciam e desenham nos tamanhos desktop e mobile, nos dois temas, e grava
//! PNGs em `target/ui-snapshots/` para conferência visual (o blur só existe no FemtoVG: ver `--snapshot`).
//!
//! Tudo em UM teste: a plataforma do Slint é por thread e só pode ser definida uma vez por processo.

use std::path::PathBuf;
use std::rc::Rc;

use kazzu_app::art::{self, Pixels};
use kazzu_app::config::{AccentPref, StreamQualityPref, ThemePref};
use kazzu_app::slint_view::SlintView;
use kazzu_app::ui::{AppState, AppWindow, Tab, Theme};
use kazzu_app::view::View;
use kazzu_app::vm::*;
use slint::platform::software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, PhysicalSize};
use std::cell::RefCell;

struct TestPlatform(Rc<MinimalSoftwareWindow>);

impl slint::platform::Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn cover(rgb: [u8; 3]) -> Pixels {
    let img = image::RgbaImage::from_fn(96, 96, |x, y| {
        let t = (x + y) as f32 / 190.0;
        image::Rgba([(f32::from(rgb[0]) * (1.0 - t * 0.4)) as u8, (f32::from(rgb[1]) * (1.0 - t * 0.4)) as u8, (f32::from(rgb[2]) * (1.0 - t * 0.4)) as u8, 255])
    });
    art::to_pixels(&img)
}

fn track(i: u32, title: &str, playing: bool) -> TrackVm {
    let even = i.is_multiple_of(2);
    TrackVm {
        id: i.to_string(),
        title: title.into(),
        artist: "Nina Sabiá".into(),
        album: "Calor de Março".into(),
        cover: Some(cover([220, 120 + i as u8 * 10, 80])),
        duration_label: "3:21".into(),
        codec: if even { "FLAC" } else { "MP3" }.into(),
        detail: if even { "24/96" } else { "320" }.into(),
        detail_long: if even { "24-bit/96 kHz" } else { "320 kbps" }.into(),
        hires: even,
        playing,
        ..Default::default()
    }
}

fn fill(view: &SlintView) {
    let tracks: Vec<TrackVm> = ["Quase Verão", "Calor de Março", "Sol de Quina", "Terra Firme", "Vento Sul"].iter().enumerate().map(|(i, t)| track(i as u32, t, i == 0)).collect();
    let albums: Vec<AlbumVm> = (0..6).map(|i| AlbumVm { id: i.to_string(), title: format!("Álbum {i}"), artist: "Aurora Vento".into(), cover: Some(cover([40 + i as u8 * 30, 120, 160])), cover_url: None }).collect();
    let hero = |i: u8, t: &str| HighlightVm { id: format!("yt:a{i}"), kind: "album".into(), caption: "Novo no YouTube Music".into(), title: t.into(), subtitle: "Nina Sabiá".into(), cover: Some(cover([230, 140 + i * 20, 90])), cover_url: None, tint: None };
    view.set_loading(false);
    view.set_home(&HomeVm {
        state: LoadStateVm::Ready,
        source_label: "Do YouTube Music".into(),
        banner: true,
        highlights: vec![hero(1, "Calor de Março"), hero(2, "Linha de Fuga")],
        quick_picks: tracks.iter().map(|t| TrackVm { why: "Em alta hoje".into(), source: "YouTube Music".into(), ..t.clone() }).collect(),
        mixes: vec![hero(3, "Mix Foco"), hero(4, "Mix Estrada")],
        albums: albums.clone(),
        recents: tracks.clone(),
        ..Default::default()
    });
    view.set_explore(&ExploreVm {
        state: LoadStateVm::Ready,
        new_releases: albums.clone(),
        moods: vec![HighlightVm { id: "mood:0".into(), kind: "mood".into(), title: "Foco".into(), tint: Some([40, 90, 120]), ..Default::default() }],
        trending: tracks.clone(),
        ..Default::default()
    });
    view.set_library(&LibraryVm {
        collections: vec![
            CollectionVm { id: "downloads".into(), title: "Baixados".into(), count_label: "Em breve".into(), covers: vec![], soon: true },
            CollectionVm { id: "liked".into(), title: "Curtidos".into(), count_label: "2 músicas".into(), covers: vec![], soon: false },
            CollectionVm { id: "albums".into(), title: "Álbuns".into(), count_label: "6 álbuns".into(), covers: vec![cover([30, 120, 140]), cover([200, 80, 70]), cover([230, 200, 90])], soon: false },
            CollectionVm { id: "playlists".into(), title: "Playlists".into(), count_label: "1 playlist".into(), covers: vec![cover([30, 120, 140]); 4], soon: false },
        ],
        playlists: vec![PlaylistVm { id: "1".into(), name: "Foco".into(), count_label: "5 músicas".into(), covers: vec![cover([30, 120, 140]), cover([200, 80, 70])] }],
    });
    view.set_settings(
        &SettingsVm { folders: vec!["/home/ana/Música".into()], summary: "1 pasta, 5 músicas".into(), has_library: true, track_count: 5 },
        &[ProviderVm { id: "ytmusic".into(), name: "YouTube Music".into(), status_label: "Ativo e padrão. Sem login.".into(), connected: true, available: true }],
    );
    view.set_queue(&QueueVm { tracks: tracks[1..].to_vec(), summary: "A seguir, 4 músicas, 13 min".into(), more_label: String::new(), note: "Quando a fila acabar, o Kazzu continua com músicas parecidas.".into() });
    view.set_search(&SearchVm { state: SearchStateVm::Results, songs: tracks.clone(), albums: albums.clone(), recent: vec!["aurora".into()], ..Default::default() });
    let now = TrackVm { cover: Some(cover([230, 140, 90])), ..track(0, "Quase Verão", true) };
    view.set_playback(&PlaybackVm {
        has_now: true,
        now: Some(now),
        playing: true,
        volume: 0.8,
        output_name: "Fones do estúdio".into(),
        output_note: "Original, sem equalizador".into(),
        bit_perfect: true,
        origin: "Mix Foco".into(),
        source: "Do YouTube Music".into(),
        quality_label: "OPUS 160 kbps".into(),
        error_action: "Tentar de novo".into(),
        ..Default::default()
    });
    view.set_now_art(&NowArtVm { cover: Some(cover([230, 140, 90])), thumb: Some(cover([230, 140, 90])), backdrop: Some(cover([100, 50, 30])), ..Default::default() });
    view.set_progress(0.31, "1:02", "−2:19");
}

/// Desenha a janela e devolve RGBA 8 bits.
fn render(win: &MinimalSoftwareWindow, w: u32, h: u32) -> image::RgbaImage {
    win.set_size(PhysicalSize::new(w, h));
    let mut buf = vec![PremultipliedRgbaColor::default(); (w * h) as usize];
    // dois quadros: o primeiro aplica layout/animações iniciais; a pausa deixa terminar a entrada do player e das folhas
    // (esmaecer de 120 ms com Reduzir movimento), que usam o relógio de verdade
    for _ in 0..2 {
        std::thread::sleep(std::time::Duration::from_millis(140));
        slint::platform::update_timers_and_animations();
        win.draw_if_needed(|r| {
            r.render(&mut buf, w as usize);
        });
    }
    let mut img = image::RgbaImage::new(w, h);
    for (i, p) in buf.iter().enumerate() {
        img.put_pixel(i as u32 % w, i as u32 / w, image::Rgba([p.red, p.green, p.blue, 255]));
    }
    img
}

/// Quantidade de cores distintas numa região (aproximação de "tem conteúdo").
fn distinct(img: &image::RgbaImage, x: u32, y: u32, w: u32, h: u32) -> usize {
    let mut set = std::collections::HashSet::new();
    for yy in y..(y + h).min(img.height()) {
        for xx in x..(x + w).min(img.width()) {
            let p = img.get_pixel(xx, yy);
            set.insert([p[0] >> 2, p[1] >> 2, p[2] >> 2]);
        }
    }
    set.len()
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-snapshots");
    std::fs::create_dir_all(&dir).expect("criar target/ui-snapshots");
    dir
}

#[test]
fn every_screen_renders_at_mobile_tablet_and_desktop_sizes_in_both_themes_and_input_works() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(window.clone()))).expect("plataforma de teste");
    let ui = AppWindow::new().expect("AppWindow");
    ui.global::<Theme>().set_blur_supported(false);
    let view = SlintView::new(&ui);
    fill(&view);
    ui.show().expect("show");
    let state = ui.global::<AppState>();
    let dir = out_dir();
    let prefs = |theme| PrefsVm { theme, accent: AccentPref::Cover, reduce_motion: true, reduce_transparency: false, stream_quality: StreamQualityPref::Original, autoplay_radio: true };

    let screens: [(&str, Tab); 5] = [("home", Tab::Home), ("explore", Tab::Explore), ("library", Tab::Library), ("search", Tab::Search), ("settings", Tab::Settings)];
    for (theme, pref) in [("dark", ThemePref::Dark), ("light", ThemePref::Light)] {
        view.set_prefs(&prefs(pref));
        // mobile primeiro (o desenho parte do celular), depois tablet e desktop
        for (size_name, w, h) in [("mobile", 390u32, 844u32), ("narrow", 360, 740), ("tablet", 768, 1024), ("desktop", 1280, 800)] {
            for (name, tab) in screens {
                state.set_player_open(false);
                state.set_queue_open(false);
                state.set_detail_open(false);
                state.set_tab(tab);
                let img = render(&window, w, h);
                img.save(dir.join(format!("{theme}-{size_name}-{name}.png"))).expect("salvar png");
                // há conteúdo de verdade: texto, capas e painéis (não só o fundo)
                assert!(distinct(&img, 0, 0, w, h) > 200, "{theme} {size_name} {name} parece em branco");
                if w < 1100 {
                    // celular e tablet: a cápsula de abas (doca de vidro) fica embaixo
                    let left = (w.saturating_sub(520)) / 2;
                    assert!(distinct(&img, left + 12, h - 80, w.min(520) - 24, 64) > 20, "{theme} {size_name} {name}: sem a barra inferior");
                } else {
                    // desktop: barra lateral sólida à esquerda, com os destinos
                    assert!(distinct(&img, 12, 60, 240, 300) > 20, "{theme} {size_name} {name}: sem barra lateral");
                }
            }
            for (name, set) in [("player", 0), ("queue", 1), ("detail", 2)] {
                state.set_player_open(set == 0);
                state.set_queue_open(set == 1);
                state.set_detail_open(set == 2);
                let img = render(&window, w, h);
                img.save(dir.join(format!("{theme}-{size_name}-{name}.png"))).expect("salvar png");
                assert!(distinct(&img, 0, 0, w, h) > 200, "{name} em branco");
            }
            state.set_player_open(false);
            state.set_queue_open(false);
            state.set_detail_open(false);
        }
    }

    // Voltar: fecha o que está aberto antes de sair (o botão Voltar do Android chama isto)
    state.set_tab(Tab::Library);
    state.set_player_open(true);
    state.set_queue_open(true);
    render(&window, 390, 844);
    assert!(ui.invoke_go_back(), "fecha a fila");
    assert!(!state.get_queue_open() && state.get_player_open());
    assert!(ui.invoke_go_back(), "fecha o player");
    assert!(!state.get_player_open());
    assert!(ui.invoke_go_back(), "raiz da Biblioteca volta ao Início");
    assert_eq!(state.get_tab(), Tab::Home);
    assert!(!ui.invoke_go_back(), "no Início sem nada aberto o sistema decide (sair)");

    // Interação real: clique no botão Buscar e teclas. (Um `FocusScope` por cima de um `TouchArea` engole o clique:
    // este teste existe por causa disso.)
    let events: Rc<RefCell<Vec<String>>> = Rc::default();
    {
        let ev = events.clone();
        state.on_toggle_play(move || ev.borrow_mut().push("toggle-play".into()));
        let ev = events.clone();
        state.on_navigate(move |t| ev.borrow_mut().push(format!("navigate:{t:?}")));
    }
    render(&window, 390, 844);
    click(&window, 350.0, 800.0); // botão circular Buscar
    window.dispatch_event(WindowEvent::WindowActiveChanged(true));
    assert_eq!(state.get_tab(), Tab::Search, "o clique em Buscar chegou: {:?}", events.borrow());

    // Digitação: o campo da Busca recebe as teclas e dispara `search` com o texto acumulado.
    let typed: Rc<RefCell<Vec<String>>> = Rc::default();
    {
        let t = typed.clone();
        state.on_search(move |q| t.borrow_mut().push(q.to_string()));
    }
    render(&window, 390, 844);
    for ch in ["a", "u", "r", "ó"] {
        window.dispatch_event(WindowEvent::KeyPressed { text: ch.into() });
        window.dispatch_event(WindowEvent::KeyReleased { text: ch.into() });
    }
    assert_eq!(typed.borrow().last().map(String::as_str), Some("auró"), "digitado: {:?}", typed.borrow());
    assert_eq!(state.get_search_text(), "auró");
    // Espaço dentro do campo é texto, não play/pause
    let before = events.borrow().iter().filter(|e| *e == "toggle-play").count();
    window.dispatch_event(WindowEvent::KeyPressed { text: " ".into() });
    assert_eq!(events.borrow().iter().filter(|e| *e == "toggle-play").count(), before, "Espaço no campo não pode acionar o atalho global");
    state.set_search_text("".into());

    // O app inteiro (App::build) não deixa o controller preso em closures: só o dono e o slot de mensagens o seguram,
    // então o desligamento (grava a configuração) realmente roda. Sem YouTube Music: nenhuma rede no teste.
    {
        use kazzu_app::app::{App, Options};
        let app_dir = out_dir().join("app-shutdown");
        let _ = std::fs::remove_dir_all(&app_dir);
        let cfg = app_dir.join("config.toml");
        let app = App::build(&Options { data_dir: Some(app_dir.join("data")), config_path: Some(cfg.clone()), null_audio: true, software_renderer: true, no_remote: true, ..Default::default() }).expect("App::build");
        assert_eq!(Rc::strong_count(&app.controller), 2, "alguma closure da UI segura o controller com Rc forte");
        app.controller.borrow_mut().set_volume(0.3);
        assert!(!cfg.exists(), "o volume só é gravado com atraso ou no encerramento");
        app.finish();
        let saved = kazzu_app::config::Config::load(&cfg).expect("config gravada no encerramento");
        assert!((saved.volume - 0.3).abs() < 1e-6, "volume salvo: {}", saved.volume);
    }
}

/// Clique de verdade (mover, apertar, soltar) numa posição lógica da janela.
fn click(win: &MinimalSoftwareWindow, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    win.dispatch_event(WindowEvent::PointerMoved { position });
    win.dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
    win.dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
}
