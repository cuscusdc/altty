//! `--selftest`: entrada real (ponteiro e teclado) sobre a janela, conferindo gestos e navegação.

use std::cell::RefCell;
use std::rc::Rc;

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::data::World;
use crate::scenes;
use crate::shots::{run_steps, step, Step};
use crate::{AppState, AppWindow, Tab};

fn press(w: &AppWindow, x: f32, y: f32) {
    w.window().dispatch_event(WindowEvent::PointerMoved { position: LogicalPosition::new(x, y) });
    w.window().dispatch_event(WindowEvent::PointerPressed { position: LogicalPosition::new(x, y), button: PointerEventButton::Left });
}
fn move_to(w: &AppWindow, x: f32, y: f32) {
    w.window().dispatch_event(WindowEvent::PointerMoved { position: LogicalPosition::new(x, y) });
}
fn release(w: &AppWindow, x: f32, y: f32) {
    w.window().dispatch_event(WindowEvent::PointerReleased { position: LogicalPosition::new(x, y), button: PointerEventButton::Left });
}
fn tap(w: &AppWindow, x: f32, y: f32) {
    press(w, x, y);
    release(w, x, y);
}
fn key(w: &AppWindow, k: Key) {
    let text: slint::SharedString = k.into();
    w.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window().dispatch_event(WindowEvent::KeyReleased { text });
}

pub fn run(win: AppWindow, world: Rc<World>) -> i32 {
    let results: Rc<RefCell<Vec<(String, bool)>>> = Rc::default();
    let rec = |r: &Rc<RefCell<Vec<(String, bool)>>>, name: &str, ok: bool| r.borrow_mut().push((name.to_string(), ok));
    let mut steps: Vec<Step> = Vec::new();
    let setup = {
        let world = world.clone();
        move |scene: &'static str| {
            let world = world.clone();
            move |w: &AppWindow| {
                w.window().set_size(slint::LogicalSize::new(390.0, 844.0));
                scenes::set_look(w, true, false, true);
                scenes::set_safe(w, 0.0, 0.0);
                scenes::apply(w, &world, scene);
            }
        }
    };

    // 1. arrastar o player 200 px para baixo fecha
    steps.push(step(600, setup("player-orange")));
    steps.push(step(100, |w| {
        press(w, 200.0, 40.0);
        move_to(w, 200.0, 120.0);
    }));
    let r = results.clone();
    steps.push(step(100, move |w| {
        move_to(w, 200.0, 250.0);
        rec(&r, "arrastar o player: a folha acompanha o dedo (não fecha durante o arraste)", w.global::<AppState>().get_player_open());
    }));
    let r = results.clone();
    steps.push(step(300, |w| release(w, 200.0, 250.0)));
    steps.push(step(100, move |w| rec(&r, "arrastar mais de 140 px fecha o player", !w.global::<AppState>().get_player_open())));
    // 2. arrastar 80 px não fecha
    steps.push(step(600, setup("player-orange")));
    steps.push(step(100, |w| {
        press(w, 200.0, 40.0);
        move_to(w, 200.0, 90.0);
        move_to(w, 200.0, 120.0);
    }));
    steps.push(step(100, |w| release(w, 200.0, 120.0)));
    let r = results.clone();
    steps.push(step(500, move |w| rec(&r, "arrastar menos de 140 px volta e mantém o player aberto", w.global::<AppState>().get_player_open())));
    // 3. botão fechar (chevron) e Esc
    steps.push(step(100, |w| tap(w, 40.0, 52.0)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "botão 'Fechar o player' fecha", !w.global::<AppState>().get_player_open())));
    steps.push(step(600, setup("player-orange")));
    steps.push(step(100, |w| key(w, Key::Escape)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "Esc fecha o player", !w.global::<AppState>().get_player_open())));
    // 4. mini player abre o player; barra inferior troca de aba; Buscar
    steps.push(step(600, setup("home-ready")));
    steps.push(step(100, |w| tap(w, 100.0, 728.0)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "tocar no mini player abre o player", w.global::<AppState>().get_player_open())));
    steps.push(step(100, |w| {
        w.invoke_go_back();
    }));
    steps.push(step(300, |w| tap(w, 150.0, 800.0)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "tocar em 'Explorar' vai para Explorar", w.global::<AppState>().get_tab() == Tab::Explore)));
    steps.push(step(100, |w| tap(w, 350.0, 800.0)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "botão Buscar vai para Buscar", w.global::<AppState>().get_tab() == Tab::Search)));
    // 5. Voltar: Busca com texto limpa o texto; depois vai ao Início; depois false
    steps.push(step(100, |w| w.global::<AppState>().set_search_text("aurora".into())));
    let r = results.clone();
    steps.push(step(100, move |w| {
        let t = w.invoke_go_back();
        rec(&r, "Voltar na Busca com texto: limpa o texto (true)", t && w.global::<AppState>().get_search_text().is_empty() && w.global::<AppState>().get_tab() == Tab::Search);
    }));
    let r = results.clone();
    steps.push(step(100, move |w| {
        let t = w.invoke_go_back();
        rec(&r, "Voltar na raiz da Busca: vai ao Início (true)", t && w.global::<AppState>().get_tab() == Tab::Home);
    }));
    let r = results.clone();
    steps.push(step(100, move |w| {
        let t = w.invoke_go_back();
        rec(&r, "Voltar no Início sem nada aberto: false (o sistema fecha o app)", !t);
    }));
    // 6. ordem de Voltar: menu → fila → player → detalhe
    steps.push(step(600, setup("queue-open")));
    steps.push(step(100, |w| w.global::<AppState>().set_menu_open(true)));
    let r = results.clone();
    steps.push(step(100, move |w| {
        let s = w.global::<AppState>();
        let a = w.invoke_go_back() && !s.get_menu_open() && s.get_queue_open() && s.get_player_open();
        let b = w.invoke_go_back() && !s.get_queue_open() && s.get_player_open();
        let c = w.invoke_go_back() && !s.get_player_open();
        rec(&r, "Voltar: menu, depois fila, depois player", a && b && c);
    }));
    // 7. tocar na aba ativa volta ao topo (Biblioteca rolada)
    steps.push(step(600, setup("library-full")));
    steps.push(step(100, |w| {
        w.window().dispatch_event(WindowEvent::PointerScrolled { position: LogicalPosition::new(200.0, 300.0), delta_x: 0.0, delta_y: -300.0 });
    }));
    let r = results.clone();
    steps.push(step(400, move |w| {
        let y = w.global::<crate::Metrics>().get_scroll_library();
        rec(&r, &format!("rolar a Biblioteca guarda a posição ({y} px)"), y < -1.0);
    }));
    steps.push(step(100, |w| tap(w, 270.0, 800.0)));
    let r = results.clone();
    steps.push(step(500, move |w| {
        let y = w.global::<crate::Metrics>().get_scroll_library();
        rec(&r, "tocar na aba ativa volta ao topo", y > -1.0);
    }));
    // 8. nova playlist: Enter no campo cria
    steps.push(step(600, setup("newplaylist-open")));
    let created = Rc::new(std::cell::Cell::new(false));
    let c2 = created.clone();
    steps.push(step(300, move |w| {
        let c3 = c2.clone();
        w.global::<AppState>().on_create_playlist_named(move |name, _| c3.set(name == "Minha playlist"));
        key(w, Key::Return);
    }));
    let (r, c) = (results.clone(), created.clone());
    steps.push(step(300, move |w| rec(&r, "Enter na Nova playlist cria 'Minha playlist' e fecha a folha", c.get() && !w.global::<AppState>().get_new_playlist_open())));
    // 9. áreas seguras: a barra sobe com safe-bottom
    steps.push(step(600, |w| {
        scenes::set_safe(w, 48.0, 24.0);
    }));
    steps.push(step(300, |w| {
        w.global::<AppState>().set_tab(Tab::Home);
        tap(w, 250.0, 826.0); // onde ficaria a barra sem a área segura
    }));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "com safe-bottom 24, tocar na faixa de gestos (y=826) não aciona a barra", w.global::<AppState>().get_tab() == Tab::Home)));
    steps.push(step(100, |w| tap(w, 250.0, 776.0)));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "com safe-bottom 24, a barra subiu: tocar em y=776 aciona 'Biblioteca'", w.global::<AppState>().get_tab() == Tab::Library)));
    // 10. alvos de toque de 48 com o visual menor: o toque a 22 px do centro (fora do visual de 40/32, dentro dos 48) aciona
    steps.push(step(600, setup("settings-all")));
    steps.push(step(100, scenes::clear_now)); // sem o mini player por cima dos controles
    steps.push(step(300, |w| {
        if let Some(e) = ElementHandle::find_by_accessible_label(w, "Alta").next() {
            let (p, s) = (e.absolute_position(), e.size());
            tap(w, p.x + s.width / 2.0, p.y + s.height / 2.0 + 22.0);
        }
    }));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "Ajustes: tocar 22 px abaixo do centro da aba 'Alta' (item de 40) a seleciona", w.global::<AppState>().get_stream_quality() == 1)));
    steps.push(step(400, |w| w.window().set_size(slint::LogicalSize::new(390.0, 1150.0)))); // janela alta: o interruptor fica acima da barra
    steps.push(step(100, |w| {
        if let Some(e) = ElementHandle::find_by_accessible_label(w, "Continuar com músicas parecidas").find(|e| e.accessible_role() == Some(AccessibleRole::Switch)) {
            let (p, s) = (e.absolute_position(), e.size());
            tap(w, p.x + s.width / 2.0 + 29.0, p.y + s.height / 2.0 - 22.0);
        }
    }));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "Ajustes: tocar 22 px acima e 29 px à direita do centro do interruptor (52×32) o aciona", !w.global::<AppState>().get_autoplay_radio())));
    steps.push(step(600, setup("search-results")));
    steps.push(step(300, |w| {
        if let Some(e) = ElementHandle::find_by_accessible_label(w, "Álbuns").find(|e| e.accessible_role() == Some(AccessibleRole::Tab)) {
            let (p, s) = (e.absolute_position(), e.size());
            tap(w, p.x + s.width / 2.0, p.y + s.height / 2.0 - 22.0);
        }
    }));
    let r = results.clone();
    steps.push(step(300, move |w| rec(&r, "Busca: tocar 22 px acima do centro do item 'Álbuns' do segmento (bolha de 40, toque de 48) o seleciona", w.global::<AppState>().get_search_kind() == 2)));
    let res = results.clone();
    steps.push(step(100, move |_| {}));
    run_steps(win, steps);
    let mut failed = 0;
    for (name, ok) in res.borrow().iter() {
        println!("[{}] {name}", if *ok { "ok" } else { "FALHOU" });
        if !ok {
            failed += 1;
        }
    }
    i32::from(failed > 0)
}
