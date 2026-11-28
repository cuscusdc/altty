//! Ponto de entrada Android (NativeActivity via `android-activity` + backend Slint).
//! Sem Gradle/Kotlin: o `scripts/build-apk.sh` empacota a cdylib direto no APK.
//! A interface é a v6 mobile (`ui-mobile-v6`), a mesma usada pelo preview.

use slint::ComponentHandle;
use slint::android::AndroidApp;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("kazzu"),
    );

    if let Err(e) = slint::android::init(app.clone()) {
        log::error!("não foi possível iniciar o backend Android do Slint: {e}");
        return;
    }

    let window = match crate::ui::AppWindow::new() {
        Ok(window) => window,
        Err(e) => {
            log::error!("não foi possível criar a janela Kazzu: {e}");
            return;
        }
    };

    let rect = app.content_rect();
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width > 0 && height > 0 {
        window.window().set_size(slint::PhysicalSize::new(width as u32, height as u32));
    }

    if let Err(e) = window.run() {
        log::error!("kazzu: {e}");
    }
}
