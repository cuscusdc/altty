//! Galeria da UI do Kazzu: abre a janela real (`AppWindow`) com dados falsos, gera capturas (`--shot`) e prova o vidro (`--glass-proof`).

mod art;
mod data;
mod hit_targets;
mod proof;
mod scenes;
mod selftest;
mod shots;

slint::include_modules!();

use std::path::PathBuf;
use std::rc::Rc;

use slint::ComponentHandle;

const HELP: &str = "kazzu-ui-gallery: galeria da UI do Kazzu (dados falsos)

  --interactive          só abre a janela (padrão se nada mais for pedido)
  --state <nome>         abre a janela já num estado (use --list para ver os nomes)
  --shot <dir>           percorre a matriz de capturas e grava PNGs em <dir>
  --only <texto>         (com --shot) só as capturas cujo nome contém <texto>
  --quick                (com --shot) só 390x844 e 360x740, escuro e claro, sem variantes
  --selftest             entrada real (ponteiro/teclado): arrastar o player, Voltar, abas, rolagem por aba
  --hit-targets          mede a área de toque (≥ 48×48) de todo elemento interativo em todas as cenas; sai ≠ 0 se falhar
  --glass-proof          prova por pixels que o vidro desfoca o conteúdo e que o sólido não deixa passar nada
  --renderer <nome>      femtovg (padrão, com blur) ou software (sem blur; capturas ganham o sufixo -sw)
  --size <LxA>           tamanho da janela no modo interativo (padrão 390x844)
  --light | --rt | --safe   (interativo) tema claro, reduzir transparência, áreas seguras 48/24
  --list                 lista os estados
";

struct Cli {
    shot: Option<PathBuf>,
    only: Option<String>,
    quick: bool,
    glass_proof: bool,
    selftest: bool,
    hit_targets: bool,
    state: Option<String>,
    renderer: String,
    size: (u32, u32),
    light: bool,
    rt: bool,
    safe: bool,
    list: bool,
    help: bool,
}

fn parse_args() -> Result<Cli, String> {
    let mut cli = Cli { shot: None, only: None, quick: false, glass_proof: false, selftest: false, hit_targets: false, state: None, renderer: "femtovg".into(), size: (390, 844), light: false, rt: false, safe: false, list: false, help: false };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--shot" => cli.shot = Some(PathBuf::from(it.next().ok_or("--shot precisa de um diretório")?)),
            "--only" => cli.only = Some(it.next().ok_or("--only precisa de um texto")?),
            "--quick" => cli.quick = true,
            "--glass-proof" => cli.glass_proof = true,
            "--selftest" => cli.selftest = true,
            "--hit-targets" => cli.hit_targets = true,
            "--state" => cli.state = Some(it.next().ok_or("--state precisa de um nome")?),
            "--renderer" => cli.renderer = it.next().ok_or("--renderer precisa de um nome")?,
            "--size" => {
                let v = it.next().ok_or("--size precisa de LxA")?;
                let (w, h) = v.split_once('x').ok_or("use --size 390x844")?;
                cli.size = (w.parse().map_err(|_| "largura inválida")?, h.parse().map_err(|_| "altura inválida")?);
            }
            "--interactive" => {}
            "--light" => cli.light = true,
            "--rt" => cli.rt = true,
            "--safe" => cli.safe = true,
            "--list" => cli.list = true,
            "--help" | "-h" => cli.help = true,
            other => return Err(format!("argumento desconhecido: {other}")),
        }
    }
    Ok(cli)
}

fn main() {
    let cli = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}\n\n{HELP}");
            std::process::exit(2);
        }
    };
    if cli.help {
        print!("{HELP}");
        return;
    }
    if cli.list {
        for s in scenes::SCENES {
            println!("{s}");
        }
        return;
    }
    if cli.hit_targets {
        // sem janela real: o backend de testes só calcula o layout (não precisa de servidor X)
        i_slint_backend_testing::init_no_event_loop();
        let win = AppWindow::new().expect("não foi possível criar a janela");
        let world = Rc::new(data::World::new());
        scenes::reset(&win, &world);
        std::process::exit(hit_targets::run(win, &world, cli.only.as_deref()));
    }
    let software = cli.renderer == "software";
    if let Err(e) = slint::BackendSelector::new().backend_name("winit".into()).renderer_name(cli.renderer.clone()).select() {
        eprintln!("renderer {} indisponível: {e}", cli.renderer);
        std::process::exit(2);
    }
    let win = AppWindow::new().expect("não foi possível criar a janela");
    let world = Rc::new(data::World::new());
    scenes::reset(&win, &world);
    let blur = !software;

    if let Some(dir) = cli.shot {
        std::fs::create_dir_all(&dir).expect("não foi possível criar o diretório de saída");
        shots::run(win, world, dir, software, cli.only, cli.quick);
        return;
    }
    if cli.glass_proof {
        if software {
            eprintln!("--glass-proof precisa do renderer femtovg (o software não tem backdrop-blur)");
            std::process::exit(2);
        }
        std::process::exit(proof::run(win, world));
    }
    if cli.selftest {
        scenes::wire(&win, world.clone());
        std::process::exit(selftest::run(win, world));
    }
    // interativo
    scenes::wire(&win, world.clone());
    scenes::set_look(&win, !cli.light, cli.rt, blur);
    if cli.safe {
        scenes::set_safe(&win, 48.0, 24.0);
    }
    if let Some(name) = &cli.state {
        scenes::apply(&win, &world, name);
    }
    win.window().set_size(slint::LogicalSize::new(cli.size.0 as f32, cli.size.1 as f32));
    win.run().expect("falha ao executar a janela");
}
