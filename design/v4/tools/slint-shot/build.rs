fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("../../slint/preview.slint", config).expect("falha ao compilar design/v4/slint/preview.slint");
    println!("cargo:rerun-if-changed=../../slint");
    println!("cargo:rerun-if-changed=../../assets/icons");
    println!("cargo:rerun-if-changed=../../assets/covers");
}
