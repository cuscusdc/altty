use crate::{AppWindow, State};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::{ComponentHandle, PhysicalSize};
use std::{io::Write, path::Path, rc::Rc};
struct Platform(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
pub fn install() -> Rc<MinimalSoftwareWindow> {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platform(window.clone()))).expect("renderer de captura");
    window
}
fn verify_callbacks(ui: &AppWindow) {
    let state = ui.global::<State>();
    state.invoke_select(0, false);
    assert_eq!(state.get_current(), 0);
    assert!(state.get_playing());
    state.invoke_seek(0.5);
    assert!((state.get_progress() - 0.5).abs() < 0.001);
    state.invoke_command("previous".into());
    assert_eq!(state.get_current(), 0);
    assert_eq!(state.get_progress(), 0.0);
    state.invoke_command("previous".into());
    assert_eq!(state.get_current(), 7);
    state.set_shuffle(true);
    for _ in 0..64 {
        let previous = state.get_current();
        state.invoke_command("next".into());
        assert_ne!(state.get_current(), previous);
    }
    state.invoke_volume_changed(1.5);
    assert_eq!(state.get_volume(), 1.0);
    state.invoke_volume_changed(-1.0);
    assert_eq!(state.get_volume(), 0.0);
    let solar = ui.global::<crate::Theme>().get_accent();
    state.invoke_theme(1);
    assert_ne!(ui.global::<crate::Theme>().get_accent(), solar);
    state.invoke_theme(0);
    state.invoke_select(0, false);
    state.invoke_command("play".into());
    state.set_shuffle(false);
    state.invoke_volume_changed(0.7);
    println!("Callbacks verificados: transporte, anterior, seek, shuffle, volume e tema.");
}
pub fn capture(
    ui: &AppWindow,
    window: &MinimalSoftwareWindow,
    directory: &Path,
) -> std::io::Result<()> {
    verify_callbacks(ui);
    std::fs::create_dir_all(directory)?;
    ui.show().map_err(std::io::Error::other)?;
    for width in [320, 420, 900] {
        window.set_size(PhysicalSize::new(width, 860));
        for (name, page, player, queue) in [
            ("inicio", 0, false, false),
            ("explorar", 1, false, false),
            ("biblioteca", 2, false, false),
            ("pesquisa", 3, false, false),
            ("configuracoes", 4, false, false),
            ("player", 0, true, false),
            ("fila", 0, true, true),
        ] {
            let s = ui.global::<State>();
            s.set_page(page);
            s.set_expanded(player);
            s.set_queue(queue);
            slint::platform::update_timers_and_animations();
            let mut pixels = vec![PremultipliedRgbaColor::default(); width as usize * 860];
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width as usize);
            });
            let mut file = std::fs::File::create(directory.join(format!("{width}-{name}.ppm")))?;
            write!(file, "P6\n{width} 860\n255\n")?;
            for pixel in pixels {
                file.write_all(&[pixel.red, pixel.green, pixel.blue])?;
            }
        }
    }
    Ok(())
}
