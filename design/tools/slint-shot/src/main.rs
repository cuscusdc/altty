//! kazzu-slint-shot: renderiza `design/slint/preview.slint` e grava PNGs.
//!
//! Uso:  kazzu-slint-shot [--renderer femtovg|software] --shot cena,LxA,tema,rt,scroll,arquivo.png [--shot ...]
//!   cena   nome de cena do `Preview` (home, home-wide, player, library, settings, sheet, proof, ...)
//!   LxA    tamanho lógico da janela (ex.: 390x844)
//!   tema   dark | light
//!   rt     0 | 1  (Reduzir transparência)
//!   scroll deslocamento de rolagem em px (0 = topo)
//! O renderer software não tem backdrop-blur: serve para mostrar o fallback sólido.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::ComponentHandle;

slint::include_modules!();

struct Shot {
    scene: String,
    w: f32,
    h: f32,
    dark: bool,
    rt: bool,
    scroll: f32,
    out: String,
}

fn parse_shot(s: &str) -> Shot {
    let p: Vec<&str> = s.split(',').collect();
    assert!(p.len() == 6, "use --shot cena,LxA,tema,rt,scroll,arquivo.png");
    let (w, h) = p[1].split_once('x').expect("tamanho LxA");
    Shot { scene: p[0].into(), w: w.parse().unwrap(), h: h.parse().unwrap(), dark: p[2] != "light", rt: p[3] == "1", scroll: p[4].parse().unwrap(), out: p[5].into() }
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
    fn next(win: Preview, steps: Rc<RefCell<Vec<Shot>>>, i: usize, software: bool) {
        if i >= steps.borrow().len() {
            let _ = slint::quit_event_loop();
            return;
        }
        {
            let st = steps.borrow();
            let s = &st[i];
            win.window().set_size(slint::LogicalSize::new(s.w, s.h));
            let theme = win.global::<Theme>();
            theme.set_dark(s.dark);
            theme.set_reduce_transparency(s.rt);
            theme.set_blur_supported(!software);
            theme.set_window_w(s.w);
            win.set_scene(s.scene.as_str().into());
            win.set_scroll_y(s.scroll);
        }
        let w2 = win.clone_strong();
        slint::Timer::single_shot(Duration::from_millis(if i == 0 { 1500 } else { 900 }), move || {
            match w2.window().take_snapshot() {
                Ok(buf) => {
                    let st = steps.borrow();
                    let s = &st[i];
                    let img = image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec()).expect("buffer");
                    img.save(&s.out).expect("salvar png");
                    println!("capturado: {} ({}x{})", s.out, buf.width(), buf.height());
                }
                Err(e) => eprintln!("captura falhou: {e}"),
            }
            next(w2.clone_strong(), steps, i + 1, software);
        });
    }
    slint::Timer::single_shot(Duration::from_millis(300), move || next(win2, steps, 0, software));
    win.run().expect("executar");
}
