//! `--hit-targets`: mede a ÁREA DE TOQUE de cada elemento interativo de todas as cenas (≥ 48×48 lógicos).
//!
//! Usa a API de testes do Slint (`i-slint-backend-testing`, só na galeria; licença GPL-3.0-only OR royalty-free, a mesma do Slint):
//! o backend de testes não desenha, só calcula o layout, então roda sem servidor X. Dois passes por cena e tamanho:
//!  1. elementos com `accessible-role` button/tab/switch/checkbox/radio-button/slider/combobox: a área é a união dos
//!     `TouchArea` dentro do elemento (é ela que recebe o toque, mesmo maior que o visual, como no Chip); sem `TouchArea`, o próprio elemento;
//!  2. todo `TouchArea` visível da janela, com ou sem papel (pega controles que esqueceram o `accessible-role`).
//!
//! Exceções legítimas (documentadas em docs/UI_NOTES.md): `TouchArea` que cobre a janela inteira (Scrim) e a barra de
//! rolagem do ListView (std-widgets), que é arrasto e não toque. O resultado sai ≠ 0 se houver violação.

use std::collections::BTreeMap;
use std::ops::ControlFlow;

use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementRoot};
use slint::{ComponentHandle, LogicalSize};

use crate::data::World;
use crate::{scenes, AppWindow};

const MIN: f32 = 48.0;
const TOL: f32 = 0.01;
// Tamanhos reais e, de cada largura, uma versão MUITO ALTA: o Slint só visita os itens dentro da área visível das listas
// rolantes; com a janela alta todo o conteúdo vertical de cada tela entra na medição (a largura, e portanto o layout, é a mesma).
const SIZES: [(u32, u32); 10] = [(390, 844), (360, 740), (320, 640), (768, 1024), (1280, 800), (390, 6000), (360, 6000), (320, 6000), (768, 6000), (1280, 6000)];
const ROLES: [AccessibleRole; 7] = [
    AccessibleRole::Button,
    AccessibleRole::Tab,
    AccessibleRole::Switch,
    AccessibleRole::Checkbox,
    AccessibleRole::RadioButton,
    AccessibleRole::Slider,
    AccessibleRole::Combobox,
];

#[derive(Default)]
struct Violation {
    count: usize,
    min_w: f32,
    min_h: f32,
    example: String,
}

struct Rect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

fn rect_of(e: &ElementHandle) -> Rect {
    let (p, s) = (e.absolute_position(), e.size());
    Rect { x0: p.x, y0: p.y, x1: p.x + s.width, y1: p.y + s.height }
}

fn describe(e: &ElementHandle) -> String {
    let t = e.type_name().map(|s| s.to_string()).unwrap_or_default();
    let id = e.id().map(|s| s.to_string()).unwrap_or_default();
    let label = e.accessible_label().map(|s| s.to_string()).unwrap_or_default();
    let role = e.accessible_role().map(|r| format!("{r:?}")).unwrap_or_default();
    format!("{role} {t} {id} \"{label}\"")
}

fn record(out: &mut BTreeMap<String, Violation>, key: String, scene: &str, size: (u32, u32), w: f32, h: f32, example: String) {
    if std::env::var_os("HIT_VERBOSE").is_some() {
        println!("  . {scene} {}x{} {key}: {w:.0}×{h:.0} {example}", size.0, size.1);
    }
    let v = out.entry(key).or_insert_with(|| Violation { min_w: f32::MAX, min_h: f32::MAX, ..Default::default() });
    v.count += 1;
    if v.count == 1 || w < v.min_w || h < v.min_h {
        v.example = format!("{scene} {}x{}: {example}", size.0, size.1);
    }
    v.min_w = v.min_w.min(w);
    v.min_h = v.min_h.min(h);
}

fn all_elements(win: &AppWindow) -> Vec<ElementHandle> {
    let mut v = Vec::new();
    win.root_element().visit_descendants(|e| {
        v.push(e);
        ControlFlow::<()>::Continue(())
    });
    v
}

fn is_touch_area(e: &ElementHandle) -> bool {
    e.type_name().is_some_and(|t| t == "TouchArea")
}

fn check_scene(win: &AppWindow, scene: &str, size: (u32, u32), out: &mut BTreeMap<String, Violation>, checked: &mut usize) {
    let win_w = size.0 as f32;
    let win_h = size.1 as f32;

    for e in &all_elements(win) {
        let role = e.accessible_role();
        let is_ta = is_touch_area(e);
        let interactive_role = role.is_some_and(|r| ROLES.contains(&r));
        if !interactive_role && !is_ta {
            continue;
        }
        if e.accessible_enabled() == Some(false) {
            continue;
        }
        if is_ta {
            // passe 2: todo TouchArea visível (com ou sem papel)
            let r = rect_of(e);
            let (w, h) = (r.x1 - r.x0, r.y1 - r.y0);
            if w >= win_w - 1.0 && h >= win_h - 1.0 {
                continue; // Scrim e áreas de tela cheia
            }
            let id = e.id().map(|s| s.to_string()).unwrap_or_default();
            if id.starts_with("ScrollBar::") {
                continue; // barra de rolagem do ListView (std-widgets): é arrasto de rolagem, não alvo de toque; a lista rola por gesto
            }
            *checked += 1;
            if w + TOL < MIN || h + TOL < MIN {
                record(out, format!("[toque] TouchArea {id}"), scene, size, w, h, format!("em ({:.0},{:.0})", r.x0, r.y0));
            }
            continue;
        }
        // passe 1: papel interativo; a área é a união dos TouchArea do elemento (o que recebe o toque), senão o próprio elemento
        let mut areas: Vec<Rect> = Vec::new();
        e.visit_descendants(|d| {
            if is_touch_area(&d) {
                areas.push(rect_of(&d));
            }
            ControlFlow::<()>::Continue(())
        });
        let r = areas.into_iter().reduce(|a, b| Rect { x0: a.x0.min(b.x0), y0: a.y0.min(b.y0), x1: a.x1.max(b.x1), y1: a.y1.max(b.y1) }).unwrap_or_else(|| rect_of(e));
        *checked += 1;
        let (w, h) = (r.x1 - r.x0, r.y1 - r.y0);
        if w + TOL < MIN || h + TOL < MIN {
            let t = e.type_name().map(|s| s.to_string()).unwrap_or_default();
            let id = e.id().map(|s| s.to_string()).unwrap_or_default();
            record(out, format!("[papel] {:?} {t} {id}", role.unwrap()), scene, size, w, h, describe(e));
        }
    }
}

pub fn run(win: AppWindow, world: &World, only: Option<&str>) -> i32 {
    let mut out: BTreeMap<String, Violation> = BTreeMap::new();
    let (mut checked, mut cases) = (0usize, 0usize);
    for size in SIZES {
        for scene in scenes::SCENES {
            if let Some(o) = only
                && !scene.contains(o)
            {
                continue;
            }
            win.window().set_size(LogicalSize::new(size.0 as f32, size.1 as f32));
            scenes::set_look(&win, true, false, true);
            scenes::set_safe(&win, 0.0, 0.0);
            scenes::apply(&win, world, scene);
            i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(600));
            check_scene(&win, scene, size, &mut out, &mut checked);
            cases += 1;
        }
    }
    println!("hit-targets: {cases} cenas × tamanhos, {checked} áreas de toque medidas (mínimo {MIN}×{MIN} px lógicos)");
    if out.is_empty() {
        println!("OK: nenhum alvo abaixo de {MIN}×{MIN}");
        return 0;
    }
    println!("{} VIOLAÇÕES (por tipo de elemento):", out.len());
    for (k, v) in &out {
        println!("  {k}: {}x em menor caso {:.0}×{:.0}  ({})", v.count, v.min_w, v.min_h, v.example);
    }
    1
}
