//! kazzu-slint-shot-v4: renderiza `design/v4/slint/preview.slint` e grava PNGs.
//!
//! Uso:  kazzu-slint-shot-v4 [--renderer femtovg|software] --shot "chave=valor ..." [--shot ...]
//!   scene   tela[.estado][+folha]   ex.: home, home.loading, search.results, album+menu, player+queue
//!   size    LxA lógico              ex.: 390x844
//!   theme   dark | light
//!   rt      0 | 1                   Reduzir transparência
//!   rm      0 | 1                   Reduzir movimento
//!   scroll  px rolados              0 = topo
//!   cover   1..22                   capa do player / do álbum
//!   safe    topo,base               áreas seguras em px (ex.: 32,24)
//!   chrome  1 | 0                   0 esconde a doca e o cabeçalho (linha de base da prova de vidro)
//!   now     1 | 0                   0 = sem faixa tocando (sem mini-player)
//!   out     arquivo.png
//! O renderer software não tem backdrop-blur: mostra o fallback sólido.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use slint::ComponentHandle;

slint::include_modules!();

#[derive(Clone)]
struct Shot {
    screen: String,
    state: String,
    overlay: String,
    w: f32,
    h: f32,
    dark: bool,
    rt: bool,
    rm: bool,
    scroll: f32,
    cover: i32,
    safe: (f32, f32),
    chrome: bool,
    now: bool,
    out: String,
}

fn parse_shot(s: &str) -> Shot {
    let kv: HashMap<&str, &str> = s.split_whitespace().filter_map(|p| p.split_once('=')).collect();
    let get = |k: &str, d: &'static str| kv.get(k).copied().unwrap_or(d).to_string();
    let scene = get("scene", "home");
    let (rest, overlay) = scene.split_once('+').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((scene.clone(), "none".into()));
    let (screen, state) = rest.split_once('.').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((rest.clone(), "ready".into()));
    let size = get("size", "390x844");
    let (w, h) = size.split_once('x').expect("size=LxA");
    let safe = get("safe", "0,0");
    let (st, sb) = safe.split_once(',').expect("safe=topo,base");
    Shot {
        screen,
        state,
        overlay,
        w: w.parse().unwrap(),
        h: h.parse().unwrap(),
        dark: get("theme", "dark") != "light",
        rt: get("rt", "0") == "1",
        rm: get("rm", "0") == "1",
        scroll: get("scroll", "0").parse().unwrap(),
        cover: get("cover", "5").parse().unwrap(),
        safe: (st.parse().unwrap(), sb.parse().unwrap()),
        chrome: get("chrome", "1") != "0",
        now: get("now", "1") != "0",
        out: kv.get("out").expect("out=arquivo.png").to_string(),
    }
}

fn next(win: Preview, steps: Rc<RefCell<Vec<Shot>>>, i: usize, software: bool) {
    if i >= steps.borrow().len() {
        let _ = slint::quit_event_loop();
        return;
    }
    let s = steps.borrow()[i].clone();
    win.window().set_size(slint::LogicalSize::new(s.w, s.h));
    let theme = win.global::<Theme>();
    theme.set_dark(s.dark);
    theme.set_reduce_transparency(s.rt);
    theme.set_reduce_motion(s.rm);
    theme.set_blur_supported(!software);
    theme.set_window_w(s.w);
    win.set_screen(s.screen.as_str().into());
    win.set_state(s.state.as_str().into());
    win.set_overlay(s.overlay.as_str().into());
    win.set_scroll_y(s.scroll);
    win.set_cover(s.cover);
    win.set_safe_top(s.safe.0);
    win.set_safe_bottom(s.safe.1);
    win.set_chrome(s.chrome);
    win.set_has_now(s.now);
    let w2 = win.clone_strong();
    slint::Timer::single_shot(Duration::from_millis(if i == 0 { 1600 } else { 900 }), move || {
        match w2.window().take_snapshot() {
            Ok(buf) => {
                let img = image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec()).expect("buffer");
                img.save(&s.out).expect("salvar png");
                println!("capturado: {} ({}x{})", s.out, buf.width(), buf.height());
            }
            Err(e) => eprintln!("captura falhou: {e}"),
        }
        next(w2.clone_strong(), steps, i + 1, software);
    });
}

fn main() {
    let mut renderer = "femtovg".to_string();
    let mut shots = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--renderer" => renderer = it.next().expect("--renderer precisa de um nome"),
            "--shot" => shots.push(parse_shot(&it.next().expect("--shot precisa de um valor"))),
            other => panic!("argumento desconhecido: {other}"),
        }
    }
    let software = renderer == "software";
    slint::BackendSelector::new().backend_name("winit".into()).renderer_name(renderer.clone()).select().expect("renderer indisponível");
    let win = Preview::new().expect("janela");
    let steps = Rc::new(RefCell::new(shots));
    let win2 = win.clone_strong();
    slint::Timer::single_shot(Duration::from_millis(300), move || next(win2, steps, 0, software));
    win.run().expect("executar");
}
