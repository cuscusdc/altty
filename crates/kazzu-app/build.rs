fn main() {
    // O APK usa a interface mobile nova (ui-mobile-v6); o desktop continua na árvore v5.
    let android = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android");
    let entry = if android { "ui-mobile-v6/main.slint" } else { "ui-v5/app.slint" };
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into())
        .with_edition((1, 19));
    slint_build::compile_with_config(entry, config).expect("falha ao compilar a UI Slint");
    println!("cargo:rerun-if-changed=ui-mobile-v6");
    println!("cargo:rerun-if-changed=ui-v5");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
}
