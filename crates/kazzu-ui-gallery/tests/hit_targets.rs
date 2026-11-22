//! Alvos de toque ≥ 48×48 em todas as cenas e tamanhos (AC-02). Roda o próprio `--hit-targets` da galeria, que usa o
//! backend de testes do Slint (sem janela, sem Xvfb) e sai com código ≠ 0 se algum controle interativo for menor.

#[test]
fn every_interactive_element_has_a_touch_area_of_at_least_48_by_48() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_kazzu-ui-gallery")).arg("--hit-targets").output().expect("rodar a galeria");
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "alvos abaixo de 48×48:\n{text}");
    assert!(text.contains("OK: nenhum alvo abaixo de 48×48"), "{text}");
}
