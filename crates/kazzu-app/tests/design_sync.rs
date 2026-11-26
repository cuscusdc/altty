//! A cópia dos arquivos do design v4 no app não pode divergir sem registro: `tokens.slint` é idêntico (é gerado); átomos,
//! cromo e telas são ADAPTADOS ao `AppState` (ver docs/UI_NOTES.md), mas todo nome exportado pelo design continua existindo.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(p: &Path) -> Option<String> {
    std::fs::read_to_string(p).ok()
}

/// Nomes de `export global|component|struct|enum Nome`.
fn exports(src: &str) -> BTreeSet<String> {
    src.lines()
        .filter_map(|l| {
            let l = l.trim_start().strip_prefix("export ")?;
            let rest = ["global ", "component ", "struct ", "enum "].iter().find_map(|k| l.strip_prefix(k))?;
            Some(rest.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-')).next()?.to_owned())
        })
        .collect()
}

const DESIGN: &str = "../../design/v4";

#[test]
fn tokens_are_an_exact_copy_of_the_design() {
    let (Some(design), Some(ours)) = (read(&root().join(DESIGN).join("slint/tokens.slint")), read(&root().join("ui/theme/tokens.slint"))) else {
        return; // fora do repositório completo (ex.: crate publicada): nada a comparar
    };
    assert_eq!(design, ours, "ui/theme/tokens.slint divergiu do design v4: mude design/v4/tokens.json, regenere (gen_tokens.py) e copie");
}

#[test]
fn fonts_are_the_design_fonts() {
    for f in ["Inter-Regular.ttf", "Inter-SemiBold.ttf", "Inter-Bold.ttf", "InterDisplay-Bold.ttf"] {
        let (Ok(design), Ok(ours)) = (std::fs::read(root().join(DESIGN).join("fonts").join(f)), std::fs::read(root().join("ui/fonts").join(f))) else {
            continue;
        };
        assert!(design == ours, "ui/fonts/{f} diverge de design/v4/fonts");
    }
}

#[test]
fn atoms_keep_every_name_the_design_exports() {
    // O app ADAPTA os átomos do design (Track no lugar de TrackItem, mais alguns acréscimos), mas todo componente exportado
    // pelo design continua existindo com o mesmo nome. Exceções registradas em docs/UI_NOTES.md.
    const ADAPTED: &[&str] = &["TrackItem"];
    let (Some(design), Some(ours)) = (read(&root().join(DESIGN).join("slint/atoms.slint")), read(&root().join("ui/components/atoms.slint"))) else {
        return;
    };
    let missing: Vec<_> = exports(&design).difference(&exports(&ours)).filter(|n| !ADAPTED.contains(&n.as_str())).cloned().collect();
    assert!(missing.is_empty(), "atoms.slint: faltam exportações do design: {missing:?}");
}

#[test]
fn platform_screens_cover_every_design_name() {
    // O design v4 tinha uma árvore única (`screens.slint` + `chrome.slint`). A implementação separa
    // mobile/desktop e dobra Explorar no Início; cada nome do design continua coberto por um
    // componente de cada família (ou pelos átomos/folhas compartilhados). Renomes registrados aqui.
    const MAP: &[(&str, &[&str])] = &[
        ("HomeScreen", &["MobileHome", "DesktopHome"]),
        ("ExploreScreen", &["MobileHome", "DesktopHome"]),
        ("SearchField", &["SearchField"]),
        ("SearchScreen", &["MobileSearch", "DesktopSearch"]),
        ("LibraryScreen", &["MobileLibrary", "DesktopLibrary"]),
        ("DetailScreen", &["MobileDetail", "DesktopDetail"]),
        ("PlayerScreen", &["MobilePlayer", "DesktopPlayer"]),
        ("QueueSheet", &["QueueSheet"]),
        ("MenuSheet", &["MenuSheet"]),
        ("NewPlaylistSheet", &["NewPlaylistSheet"]),
        ("SettingsScreen", &["MobileSettings", "DesktopSettings"]),
        ("MiniPlayer", &["MobileDock", "DesktopNowBar", "DesktopCompactBar"]),
        ("TabBar", &["MobileDock", "DesktopSidebar"]),
        ("SearchOrb", &["MobileDock", "DesktopSidebar"]),
        ("Dock", &["MobileDock", "DesktopCompactBar"]),
        ("TopBar", &["MobileTopBar"]),
    ];
    let mut ours = BTreeSet::new();
    for dir in ["ui/mobile", "ui/desktop"] {
        if let Ok(entries) = std::fs::read_dir(root().join(dir)) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(src) = read(&entry.path()) {
                    ours.extend(exports(&src));
                }
            }
        }
    }
    for file in ["ui/components/atoms.slint", "ui/components/sheets.slint"] {
        if let Some(src) = read(&root().join(file)) {
            ours.extend(exports(&src));
        }
    }
    let mut design_names = BTreeSet::new();
    for file in ["slint/screens.slint", "slint/chrome.slint"] {
        if let Some(src) = read(&root().join(DESIGN).join(file)) {
            design_names.extend(exports(&src));
        }
    }
    if design_names.is_empty() {
        return;
    }
    let mapped: BTreeSet<String> = MAP.iter().map(|(name, _)| (*name).to_owned()).collect();
    let unmapped: Vec<_> = design_names.difference(&mapped).cloned().collect();
    assert!(unmapped.is_empty(), "nomes do design sem equivalente registrado: {unmapped:?}");
    let mut missing = Vec::new();
    for (design, equivalents) in MAP {
        if !design_names.contains(*design) {
            continue;
        }
        for name in *equivalents {
            if !ours.contains(*name) {
                missing.push(format!("{design} -> {name}"));
            }
        }
    }
    assert!(missing.is_empty(), "componentes equivalentes ausentes: {missing:?}");
}

#[test]
fn icons_are_the_design_icons() {
    let (Some(design), Some(ours)) = (read(&root().join(DESIGN).join("slint/data.slint")), read(&root().join("ui/theme/icons.slint"))) else {
        return;
    };
    let names = |s: &str| -> BTreeSet<String> { s.lines().filter_map(|l| l.trim().strip_prefix("out property <image> ")).filter_map(|l| l.split(':').next()).map(str::to_owned).collect() };
    let design_icons: BTreeSet<String> = names(design.split("export global Demo").next().unwrap_or(""));
    assert_eq!(design_icons, names(&ours), "a lista de ícones divergiu de design/v4/slint/data.slint");
}

#[test]
fn no_screen_uses_literal_colors_outside_the_registered_exceptions() {
    // regra do SLINT_SPEC §1.1: cores só de Theme.*; as exceções estão listadas em docs/UI_NOTES.md
    const ALLOWED: &[&str] = &[];
    let mut offenders = Vec::new();
    for dir in ["ui/components", "ui/mobile", "ui/desktop", "ui/app.slint", "ui/theme/metrics.slint", "ui/theme/nav-state.slint"] {
        let path = root().join(dir);
        let files: Vec<PathBuf> = if path.is_dir() {
            std::fs::read_dir(&path).map(|r| r.filter_map(Result::ok).map(|e| e.path()).collect()).unwrap_or_default()
        } else {
            vec![path]
        };
        for f in files {
            let Some(src) = read(&f) else { continue };
            for (n, line) in src.lines().enumerate() {
                let mut rest = line;
                while let Some(i) = rest.find('#') {
                    let tail = &rest[i..];
                    let len = tail[1..].find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(tail.len() - 1);
                    if matches!(len, 6 | 8) && !ALLOWED.contains(&&tail[..=len]) {
                        offenders.push(format!("{}:{}: {}", f.display(), n + 1, &tail[..=len]));
                    }
                    rest = &tail[1..];
                }
            }
        }
    }
    assert!(offenders.is_empty(), "cores literais fora da lista de exceções: {offenders:#?}");
}
