//! A cópia dos arquivos do design em `ui/theme/` não pode divergir sem registro:
//! `tokens.slint` é idêntico (é gerado); átomos e contrato só podem ACRESCENTAR/ajustar (ver docs/UI_NOTES.md),
//! então todo nome exportado pelo design precisa continuar existindo.

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

#[test]
fn tokens_are_an_exact_copy_of_the_design() {
    let (Some(design), Some(ours)) = (read(&root().join("../../design/slint/tokens.slint")), read(&root().join("ui/theme/tokens.slint"))) else {
        return; // fora do repositório completo (ex.: crate publicada): nada a comparar
    };
    assert_eq!(design, ours, "ui/theme/tokens.slint divergiu do design: mude tokens.json, regenere e copie");
}

#[test]
fn atoms_are_the_same_file_in_the_design_and_in_the_app() {
    let (Some(design), Some(ours)) = (read(&root().join("../../design/slint/atoms.slint")), read(&root().join("ui/theme/atoms.slint"))) else {
        return;
    };
    assert_eq!(design, ours, "atoms.slint diverge: corrija nos dois lugares (design/slint e ui/theme)");
}

#[test]
fn contract_is_the_same_file_in_the_design_and_in_the_app() {
    let (Some(design), Some(ours)) = (read(&root().join("../../design/slint/app-contract.slint")), read(&root().join("ui/theme/app-contract.slint"))) else {
        return;
    };
    assert_eq!(design, ours, "app-contract.slint diverge: o contrato v2 vive nos dois lugares (design/slint e ui/theme)");
}

#[test]
fn atoms_and_contract_keep_every_name_the_design_exports() {
    for file in ["atoms.slint", "app-contract.slint"] {
        let (Some(design), Some(ours)) = (read(&root().join("../../design/slint").join(file)), read(&root().join("ui/theme").join(file))) else {
            continue;
        };
        let missing: Vec<_> = exports(&design).difference(&exports(&ours)).cloned().collect();
        assert!(missing.is_empty(), "{file}: faltam exportações do design: {missing:?}");
    }
}

#[test]
fn no_screen_uses_literal_colors_outside_the_registered_exceptions() {
    // regra do SLINT_SPEC §1.1: cores só de Theme.*; as exceções estão listadas em docs/UI_NOTES.md
    const ALLOWED: &[&str] = &["#000000a0", "#000000b0", "#000000b3", "#0c0806", "#0c0907a3", "#1a1511", "#ffffff1f", "#ffffff", "#15110d"];
    let mut offenders = Vec::new();
    for dir in ["ui/components", "ui/screens", "ui/app.slint"] {
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
