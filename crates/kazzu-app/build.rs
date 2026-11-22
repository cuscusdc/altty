fn main() {
    // "fluent" (claro/escuro conforme o sistema). Só ListView e Palette vêm de std-widgets; o resto é o design system do Kazzu.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("ui/app.slint", config).expect("falha ao compilar a UI Slint");
    println!("cargo:rerun-if-changed=ui");
}
