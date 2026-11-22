fn main() {
    // Mesmas opções do build.rs do kazzu-app: a galeria compila a UI real.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into()).with_debug_info(true); // debug info: a API de testes (--hit-targets) precisa dos nomes dos elementos
    slint_build::compile_with_config("../kazzu-app/ui/app.slint", config).expect("falha ao compilar a UI Slint");
    println!("cargo:rerun-if-changed=../kazzu-app/ui");
}
