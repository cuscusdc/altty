//! Modo `--shot`: percorre a matriz tamanho × tema × transparência × cena e grava PNGs.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer};

use crate::data::World;
use crate::scenes;
use crate::AppWindow;

pub struct Step {
    pub action: Box<dyn Fn(&AppWindow)>,
    pub settle: Duration,
    pub capture: Option<Box<dyn FnMut(SharedPixelBuffer<Rgba8Pixel>)>>,
}

pub fn step(settle_ms: u64, action: impl Fn(&AppWindow) + 'static) -> Step {
    Step { action: Box::new(action), settle: Duration::from_millis(settle_ms), capture: None }
}

pub fn capture_step(settle_ms: u64, action: impl Fn(&AppWindow) + 'static, capture: impl FnMut(SharedPixelBuffer<Rgba8Pixel>) + 'static) -> Step {
    Step { action: Box::new(action), settle: Duration::from_millis(settle_ms), capture: Some(Box::new(capture)) }
}

/// Executa os passos em sequência (ação, espera, captura opcional) e encerra o laço de eventos no fim.
pub fn run_steps(win: AppWindow, steps: Vec<Step>) {
    let win = Rc::new(win);
    let steps = Rc::new(RefCell::new(steps));
    // pequena folga para o primeiro desenho
    let (w2, s2) = (win.clone(), steps.clone());
    slint::Timer::single_shot(Duration::from_millis(900), move || next(w2, s2, 0));
    win.run().expect("falha ao executar a janela");
}

fn next(win: Rc<AppWindow>, steps: Rc<RefCell<Vec<Step>>>, i: usize) {
    if i >= steps.borrow().len() {
        let _ = slint::quit_event_loop();
        return;
    }
    let settle = {
        let st = steps.borrow();
        (st[i].action)(&win);
        st[i].settle
    };
    slint::Timer::single_shot(settle, move || {
        let cap = steps.borrow_mut()[i].capture.take();
        if let Some(mut cap) = cap {
            match win.window().take_snapshot() {
                Ok(buf) => cap(buf),
                Err(e) => eprintln!("captura falhou: {e}"),
            }
        }
        next(win, steps, i + 1);
    });
}

pub fn save_png(dir: &Path, name: &str, buf: &SharedPixelBuffer<Rgba8Pixel>) {
    let path = dir.join(name);
    match image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec()) {
        Some(img) => match img.save(&path) {
            Ok(()) => println!("capturado: {}", path.display()),
            Err(e) => eprintln!("não foi possível salvar {}: {e}", path.display()),
        },
        None => eprintln!("buffer inesperado em {name}"),
    }
}

#[derive(Clone)]
pub struct Job {
    pub scene: &'static str,
    pub w: u32,
    pub h: u32,
    pub dark: bool,
    pub rt: bool,
    pub safe: bool,
}

impl Job {
    pub fn file(&self, software: bool) -> String {
        let (screen, state) = self.scene.split_once('-').unwrap_or((self.scene, "x"));
        format!("{screen}-{state}-{}x{}-{}{}{}{}.png", self.w, self.h, if self.dark { "dark" } else { "light" }, if self.rt { "-rt" } else { "" }, if self.safe { "-safe" } else { "" }, if software { "-sw" } else { "" })
    }
}

const RT_SUBSET: [&str; 7] = ["home-ready", "library-full", "search-results", "player-orange", "queue-open", "menu-open", "settings-all"];
const SMALL_SUBSET: [&str; 6] = ["home-ready", "library-full", "search-results", "settings-all", "player-orange", "menu-open"];
const SAFE_SUBSET: [&str; 10] = ["newplaylist-open", "home-ready", "library-full", "search-idle", "detail-album", "player-orange", "queue-open", "menu-open", "settings-all", "toast-action"];

pub fn matrix(quick: bool) -> Vec<Job> {
    let mut jobs = Vec::new();
    let mut push = |scene: &'static str, w, h, dark, rt, safe| jobs.push(Job { scene, w, h, dark, rt, safe });
    // celular primeiro
    for (w, h) in [(390, 844), (360, 740)] {
        for scene in scenes::SCENES {
            for dark in [true, false] {
                push(scene, w, h, dark, false, false);
            }
        }
        if !quick {
            for scene in RT_SUBSET {
                for dark in [true, false] {
                    push(scene, w, h, dark, true, false);
                }
            }
        }
    }
    // telas roladas (conteúdo sob a barra de status e os botões do sistema)
    for (w, h, safe) in [(390, 844, false), (390, 844, true)] {
        for scene in scenes::SCROLL_SCENES {
            for dark in [true, false] {
                push(scene, w, h, dark, false, safe);
            }
        }
    }
    if quick {
        return jobs;
    }
    for (w, h) in [(768, 1024), (1280, 800)] {
        for scene in scenes::SCENES {
            for dark in [true, false] {
                push(scene, w, h, dark, false, false);
            }
        }
        for scene in RT_SUBSET {
            push(scene, w, h, true, true, false);
        }
    }
    for scene in SMALL_SUBSET {
        push(scene, 320, 640, true, false, false);
    }
    for scene in SAFE_SUBSET {
        for dark in [true, false] {
            push(scene, 390, 844, dark, false, true);
        }
    }
    jobs
}

pub fn run(win: AppWindow, world: Rc<World>, dir: PathBuf, software: bool, only: Option<String>, quick: bool) {
    let blur = !software;
    let mut steps = Vec::new();
    let mut last_size = (0, 0);
    let dir = Rc::new(dir);
    for job in matrix(quick) {
        let file = job.file(software);
        if let Some(o) = &only
            && !file.contains(o.as_str())
        {
            continue;
        }
        let settle = if last_size != (job.w, job.h) { 1100 } else { 750 };
        last_size = (job.w, job.h);
        let world = world.clone();
        let d = dir.clone();
        let j = job.clone();
        if job.scene.ends_with("-scrolled") {
            // a tela só restaura a rolagem ao ser criada: passa antes por outra aba
            let (world, j) = (world.clone(), job.clone());
            steps.push(step(350, move |w| {
                scenes::apply(w, &world, if j.scene.starts_with("home") { "library-full" } else { "home-ready" });
            }));
        }
        steps.push(capture_step(
            settle,
            move |w| {
                w.window().set_size(slint::LogicalSize::new(j.w as f32, j.h as f32));
                scenes::set_look(w, j.dark, j.rt, blur);
                scenes::set_safe(w, if j.safe { 48.0 } else { 0.0 }, if j.safe { 24.0 } else { 0.0 });
                scenes::apply(w, &world, j.scene);
            },
            move |buf| save_png(&d, &file, &buf),
        ));
    }
    println!("{} capturas planejadas", steps.len());
    run_steps(win, steps);
}
