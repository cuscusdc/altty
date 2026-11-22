//! Montagem do app: janela Slint, controller, threads de trabalho e ponte do motor de áudio.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use kazzu_audio::{Engine, EngineConfig, EngineHandle, NullSink};
use kazzu_core::PlaybackEvent;
use slint::ComponentHandle;

use crate::config::{AccentPref, Config, ThemePref};
use crate::controller::{Controller, Msg, MsgSink, NavTab};
use crate::slint_view::SlintView;
use crate::ui;
use crate::worker::{QueueMode, ThreadedRunner};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("não foi possível abrir a biblioteca: {0}")]
    Library(#[from] kazzu_library::Error),
    #[error("não foi possível iniciar o motor de áudio: {0}")]
    Audio(#[from] kazzu_core::Error),
    #[error("falha ao criar a janela: {0}")]
    Window(#[from] slint::PlatformError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("diretório de dados do usuário indisponível")]
    NoDataDir,
    /// O renderer com vidro não iniciou; quem chamou deve reabrir em modo software.
    #[error("renderer indisponível")]
    RetrySoftware,
}

/// Como iniciar.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Onde ficam banco e cache (padrão: diretório de dados do usuário).
    pub data_dir: Option<PathBuf>,
    /// Arquivo de configuração (padrão: diretório de configuração do usuário).
    pub config_path: Option<PathBuf>,
    /// Pastas de música extras a indexar já na abertura.
    pub extra_folders: Vec<PathBuf>,
    /// Saída de áudio simulada (sem placa de som): para CI e capturas de tela.
    pub null_audio: bool,
    /// O renderer ativo não suporta `backdrop-blur` (software).
    pub software_renderer: bool,
    /// Mostra a janela já na criação para descobrir se o renderer iniciou (falha = `AppError::RetrySoftware`).
    pub probe_renderer: bool,
}

thread_local! {
    static CONTROLLER: RefCell<Option<Rc<RefCell<Controller<SlintView>>>>> = const { RefCell::new(None) };
}

fn dispatch(msg: Msg) {
    CONTROLLER.with(|slot| {
        let Some(ctrl) = slot.borrow().clone() else { return };
        match ctrl.try_borrow_mut() {
            Ok(mut c) => c.handle(msg),
            Err(_) => log::error!("mensagem descartada: controller ocupado"),
        }
    });
}

pub struct App {
    pub window: ui::AppWindow,
    pub controller: Rc<RefCell<Controller<SlintView>>>,
}

impl App {
    pub fn build(opts: &Options) -> Result<Self, AppError> {
        // 1. Janela (e, se pedido, teste do renderer) ANTES de subir motor de áudio, worker e banco: se o OpenGL não
        //    iniciar, quem chamou reabre em modo software sem ter deixado threads nem conexões para trás.
        let window = ui::AppWindow::new()?;
        if opts.probe_renderer
            && let Err(e) = window.show()
        {
            log::warn!("renderer com vidro indisponível: {e}");
            return Err(AppError::RetrySoftware);
        }
        window.global::<ui::Theme>().set_blur_supported(!opts.software_renderer);
        window.global::<ui::AppState>().set_licenses_text(licenses_text().into());
        window.global::<ui::AppState>().set_version_label(format!("Kazzu {}", env!("CARGO_PKG_VERSION")).into());

        // 2. Configuração. Arquivo ilegível vai para `.bak` (nunca é sobrescrito sem aviso).
        let cfg_path = opts.config_path.clone().or_else(Config::default_path);
        let mut notice: Option<String> = None;
        let cfg = match &cfg_path {
            Some(p) => match Config::load(p) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("{e}; usando a configuração padrão");
                    notice = Some(match Config::back_up_corrupt(p) {
                        Ok(bak) => format!("A configuração estava ilegível. Guardei uma cópia em {} e voltei ao padrão.", bak.display()),
                        Err(_) => "A configuração estava ilegível; voltei ao padrão.".to_owned(),
                    });
                    Config::default()
                }
            },
            None => Config::default(),
        };
        let data_dir = match &opts.data_dir {
            Some(d) => d.clone(),
            None => kazzu_library::Library::default_data_dir().map_err(|_| AppError::NoDataDir)?,
        };

        // 3. Motor de áudio. O dispositivo só é aberto na primeira reprodução.
        let config = EngineConfig { volume: crate::controller::sane_unit(cfg.volume, 1.0), ..EngineConfig::default() };
        let engine: EngineHandle = if opts.null_audio {
            Engine::builder().sink(NullSink::new()).config(config).spawn()?
        } else {
            Engine::builder().config(config).spawn()?
        };

        let sink: MsgSink = Arc::new(|msg| {
            let _ = slint::invoke_from_event_loop(move || dispatch(msg));
        });
        spawn_engine_bridge(&engine, sink.clone())?;

        let emit_sink = sink.clone();
        let runner = ThreadedRunner::spawn(data_dir, Arc::new(move |r| emit_sink(Msg::Job(r))))?;

        let view = SlintView::new(&window);
        let controller = Rc::new(RefCell::new(Controller::new(view, engine, Box::new(runner), sink.clone(), cfg, cfg_path)));
        CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller.clone()));
        wire(&window, &controller);

        controller.borrow_mut().init(Default::default());
        if let Some(text) = notice {
            controller.borrow().notice(&text, true);
        }
        // preferências do sistema chegam depois (leitura fora da thread de UI)
        let sys_sink = sink;
        let _ = std::thread::Builder::new().name("kazzu-sysprefs".into()).spawn(move || sys_sink(Msg::System(crate::platform::system_prefs())));
        for folder in &opts.extra_folders {
            controller.borrow_mut().add_folder_path(folder.clone());
        }
        Ok(Self { window, controller })
    }

    /// Mostra a janela e roda o laço de eventos até fechar; depois grava a configuração e para o motor.
    pub fn run(self) -> Result<(), AppError> {
        let result = self.window.run();
        self.finish();
        result.map_err(AppError::from)
    }

    /// Encerramento: o controller é desligado por referência (as closures da UI só guardam `Weak`, então não há
    /// como o `Rc` ficar preso e o desligamento ser pulado) e solto em seguida, o que encerra o motor e as threads.
    pub fn finish(self) {
        CONTROLLER.with(|slot| slot.borrow_mut().take());
        match self.controller.try_borrow_mut() {
            Ok(mut c) => c.shutdown(),
            Err(_) => log::error!("controller ocupado no encerramento"),
        }
    }
}

fn licenses_text() -> String {
    "Kazzu é software livre.\n\n\
     Slint, usado sob GPL-3.0: interface gráfica.\n\
     Hanken Grotesk e Geist Mono (SIL OFL 1.1): fontes, incluídas com o app.\n\
     Symphonia (MPL-2.0): decodificação de áudio. libopus (BSD-3-Clause): decodificador Opus.\n\
     cpal, rusqlite, lofty, image, rayon, rfd e demais bibliotecas: MIT ou Apache-2.0.\n\n\
     A lista completa, com versões, está em docs/DEPENDENCIES.md."
        .to_owned()
}

/// Lê os eventos do motor numa thread e os entrega à UI (agrupando posições, que chegam a ~10 Hz).
fn spawn_engine_bridge(engine: &EngineHandle, sink: MsgSink) -> std::io::Result<()> {
    let rx = engine.events().clone();
    std::thread::Builder::new().name("kazzu-events".into()).spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut batch = vec![first];
            batch.extend(rx.try_iter());
            // só a última posição do lote interessa
            let last_position = batch.iter().rposition(|e| matches!(e, PlaybackEvent::Position { .. }));
            for (i, ev) in batch.into_iter().enumerate() {
                if matches!(ev, PlaybackEvent::Position { .. }) && Some(i) != last_position {
                    continue;
                }
                sink(Msg::Engine(ev));
            }
        }
    })?;
    Ok(())
}

macro_rules! cb {
    ($ctrl:expr, |$c:ident $(, $arg:ident)*| $body:expr) => {{
        // `Weak`: a janela guarda estas closures; um `Rc` aqui manteria o controller vivo para sempre
        let ctrl = Rc::downgrade($ctrl);
        move |$($arg),*| {
            if let Some(rc) = ctrl.upgrade() {
                match rc.try_borrow_mut() {
                    Ok(mut $c) => {
                        $body;
                    }
                    Err(_) => log::error!("ação ignorada: controller ocupado"),
                }
            }
        }
    }};
}

fn tab_of(t: ui::Tab) -> NavTab {
    match t {
        ui::Tab::Home => NavTab::Home,
        ui::Tab::Explore => NavTab::Explore,
        ui::Tab::Library => NavTab::Library,
        ui::Tab::Search => NavTab::Search,
        ui::Tab::Settings => NavTab::Settings,
    }
}

/// Liga os callbacks de `AppState` ao controller.
fn wire(window: &ui::AppWindow, ctrl: &Rc<RefCell<Controller<SlintView>>>) {
    let s = window.global::<ui::AppState>();
    let weak = window.as_weak();

    // navegação (estado de tela vive no Slint; o controller só precisa saber a aba e se há detalhe)
    s.on_navigate({
        let ctrl = Rc::downgrade(ctrl);
        let weak = weak.clone();
        move |tab| {
            if let Some(w) = weak.upgrade() {
                let st = w.global::<ui::AppState>();
                st.set_tab(tab);
                st.set_detail_open(false);
                st.set_queue_open(false);
                st.set_player_open(false);
            }
            if let Some(rc) = ctrl.upgrade()
                && let Ok(mut c) = rc.try_borrow_mut()
            {
                c.navigate(tab_of(tab));
            }
        }
    });
    s.on_open_player({
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                w.global::<ui::AppState>().set_player_open(true);
            }
        }
    });
    s.on_close_player({
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                w.global::<ui::AppState>().set_player_open(false);
            }
        }
    });
    s.on_close_detail({
        let ctrl = Rc::downgrade(ctrl);
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                w.global::<ui::AppState>().set_detail_open(false);
            }
            if let Some(rc) = ctrl.upgrade()
                && let Ok(mut c) = rc.try_borrow_mut()
            {
                c.detail_closed();
            }
        }
    });

    // reprodução
    s.on_toggle_play(cb!(ctrl, |c| c.toggle_play()));
    s.on_next(cb!(ctrl, |c| c.next()));
    s.on_previous(cb!(ctrl, |c| c.previous()));
    s.on_seek(cb!(ctrl, |c, v| c.seek(v)));
    s.on_seek_relative(cb!(ctrl, |c, secs| c.seek_relative(i64::from(secs))));
    // volume: grava a configuração 1 s depois do último ajuste (arrastar o slider não grava a cada pixel)
    let volume_timer = Rc::new(slint::Timer::default());
    let save_later = {
        let ctrl = Rc::downgrade(ctrl);
        move |timer: &slint::Timer| {
            let ctrl = ctrl.clone();
            timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_secs(1), move || {
                if let Some(rc) = ctrl.upgrade()
                    && let Ok(mut c) = rc.try_borrow_mut()
                {
                    c.flush_config();
                }
            });
        }
    };
    {
        let (set, t, later) = (cb!(ctrl, |c, v| c.set_volume(v)), volume_timer.clone(), save_later.clone());
        s.on_set_volume(move |v| {
            set(v);
            later(&t);
        });
        let (step, t, later) = (cb!(ctrl, |c, dir| c.volume_step(dir)), volume_timer, save_later);
        s.on_volume_step(move |d| {
            step(d);
            later(&t);
        });
    }
    s.on_toggle_shuffle(cb!(ctrl, |c| c.toggle_shuffle()));
    s.on_cycle_repeat(cb!(ctrl, |c| c.cycle_repeat()));
    s.on_toggle_like(cb!(ctrl, |c, id| c.toggle_like(&id)));
    s.on_play_track(cb!(ctrl, |c, id| c.play_track(&id)));
    s.on_play_next(cb!(ctrl, |c, id| c.queue_track(&id, QueueMode::PlayNext)));
    s.on_enqueue(cb!(ctrl, |c, id| c.queue_track(&id, QueueMode::Enqueue)));
    s.on_retry_track(cb!(ctrl, |c| c.retry_track()));
    s.on_open_lyrics(cb!(ctrl, |c| c.coming_soon("A letra das músicas")));
    s.on_open_output_picker(cb!(ctrl, |c| c.coming_soon("A troca de saída de áudio")));
    s.on_recognize_ambient(cb!(ctrl, |c| c.coming_soon("O reconhecimento de música pelo som ambiente")));
    s.on_connect_provider(cb!(ctrl, |c, _id| c.coming_soon("A conexão com provedores")));

    // fila
    s.on_queue_remove(cb!(ctrl, |c, i| c.queue_remove(usize::try_from(i).unwrap_or(usize::MAX))));
    s.on_queue_jump(cb!(ctrl, |c, i| c.queue_jump(usize::try_from(i).unwrap_or(usize::MAX))));
    s.on_queue_clear(cb!(ctrl, |c| c.queue_clear()));

    // biblioteca e detalhe
    s.on_open_album(cb!(ctrl, |c, id| c.open_album(&id)));
    s.on_open_playlist(cb!(ctrl, |c, id| c.open_playlist(&id)));
    s.on_open_collection(cb!(ctrl, |c, id| c.open_collection(&id)));
    s.on_open_genre(cb!(ctrl, |c, id| c.open_genre(&id)));
    s.on_play_all(cb!(ctrl, |c| c.play_all(false)));
    s.on_shuffle_all(cb!(ctrl, |c| c.play_all(true)));
    s.on_create_playlist_named(cb!(ctrl, |c, name| c.create_playlist(&name)));
    s.on_add_folder(cb!(ctrl, |c| c.add_folder()));
    s.on_remove_folder(cb!(ctrl, |c, path| c.remove_folder(&path)));
    s.on_rescan(cb!(ctrl, |c| c.rescan()));

    // busca
    s.on_search(cb!(ctrl, |c, text| c.search(&text)));
    s.on_search_filter_changed(cb!(ctrl, |c, kind, sources| {
        use slint::Model;
        c.search_filters(kind, sources.row_data(0).unwrap_or(true));
    }));
    s.on_clear_recent_searches(cb!(ctrl, |c| c.clear_recent_searches()));

    // configurações
    s.on_theme_mode_changed(cb!(ctrl, |c, mode| c.set_theme(match mode {
        ui::ThemeMode::Auto => ThemePref::Auto,
        ui::ThemeMode::Light => ThemePref::Light,
        ui::ThemeMode::Dark => ThemePref::Dark,
    })));
    s.on_accent_mode_changed(cb!(ctrl, |c, mode| c.set_accent_pref(match mode {
        ui::AccentMode::FromCover => AccentPref::Cover,
        ui::AccentMode::Brand => AccentPref::Brand,
    })));
    s.on_reduce_transparency_changed(cb!(ctrl, |c, on| c.set_reduce_transparency(on)));
    s.on_reduce_motion_changed(cb!(ctrl, |c, on| c.set_reduce_motion(on)));
    s.on_dismiss_toast(|| {});
}
