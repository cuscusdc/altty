mod audio;
mod models;
mod snapshot;
use audio::{Audio, SimulatedAudio};
use models::{Settings, Track};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
slint::include_modules!();
struct Controller {
    tracks: Vec<Track>,
    index: usize,
    audio: SimulatedAudio,
    settings: Settings,
    rows: Vec<serde_json::Value>,
    visible_rows: Vec<usize>,
    palettes: Vec<serde_json::Value>,
    seed: u64,
    toast_until: Instant,
    scan_until: Option<Instant>,
    scan_reset: Option<Instant>,
    clipboard: Option<arboard::Clipboard>,
}
fn color(value: &str) -> slint::Color {
    let value = u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0);
    slint::Color::from_rgb_u8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
fn rgba(css: &str) -> slint::Color {
    let parts: Vec<f32> = css
        .trim_start_matches("rgba(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    slint::Color::from_argb_u8(
        (parts[3] * 255.0) as u8,
        parts[0] as u8,
        parts[1] as u8,
        parts[2] as u8,
    )
}
fn colors(gradient: &str) -> Vec<slint::Color> {
    gradient
        .split('#')
        .skip(1)
        .map(|s| color(&s[..6]))
        .collect()
}
fn time(value: f32) -> String {
    let value = value.max(0.0) as u32;
    format!("{}:{:02}", value / 60, value % 60)
}
fn songs(tracks: &[Track]) -> ModelRc<Song> {
    ModelRc::new(VecModel::from(
        tracks
            .iter()
            .map(|t| {
                let c = colors(&t.art_gradient);
                Song {
                    title: t.title.as_str().into(),
                    artist: t.artist.as_str().into(),
                    duration: time(t.duration).into(),
                    first: c[0],
                    last: *c.last().unwrap(),
                }
            })
            .collect::<Vec<_>>(),
    ))
}
impl Controller {
    fn search(&mut self, ui: &AppWindow, query: &str) {
        let query = query.to_lowercase();
        let results = self
            .tracks
            .iter()
            .filter(|t| {
                !query.is_empty()
                    && format!("{} {} {}", t.title, t.artist, t.lyrics.join(" "))
                        .to_lowercase()
                        .contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>();
        ui.global::<State>().set_results(songs(&results));
    }
    fn publish(&self, ui: &AppWindow) {
        let s = ui.global::<State>();
        let t = &self.tracks[self.index];
        s.set_current(self.index as i32);
        s.set_playing(self.audio.playing());
        s.set_volume(self.audio.volume());
        s.set_progress(t.position / t.duration);
        s.set_position(time(t.position).into());
        s.set_remaining(format!("-{}", time(t.duration - t.position)).into());
        s.set_album(t.album.as_str().into());
        s.set_lines(ModelRc::new(VecModel::from(
            t.lyrics
                .iter()
                .map(|s| s.as_str().into())
                .collect::<Vec<slint::SharedString>>(),
        )));
        let atmosphere = colors(&t.background);
        s.set_bg_first(atmosphere[0]);
        s.set_bg_mid(atmosphere[1]);
        s.set_bg_last(atmosphere[2]);
    }
    fn toast(&mut self, ui: &AppWindow, message: &str) {
        ui.global::<State>().set_toast(message.into());
        self.toast_until = Instant::now() + Duration::from_millis(2200);
    }
    fn random(&mut self, exclude_current: bool) -> usize {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        if exclude_current {
            (self.index + 1 + self.seed as usize % (self.tracks.len() - 1)) % self.tracks.len()
        } else {
            self.seed as usize % self.tracks.len()
        }
    }
    fn next(&mut self, ui: &AppWindow) {
        self.index = if ui.global::<State>().get_shuffle() {
            self.random(true)
        } else {
            (self.index + 1) % self.tracks.len()
        };
        self.tracks[self.index].position = 0.0;
        self.audio.set_playing(true);
    }
    fn rows(&mut self, ui: &AppWindow) {
        let query = ui.global::<State>().get_settings_query().to_lowercase();
        self.visible_rows = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                format!("{} {}", r["title"], r["sub"])
                    .to_lowercase()
                    .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        let rows = self
            .visible_rows
            .iter()
            .map(|&i| {
                let r = &self.rows[i];
                let action = r["action"].as_str().unwrap_or("");
                let toggle_index = [
                    "toggleNormalization",
                    "toggleLiquidGlass",
                    "toggleAnimations",
                    "toggleHistory",
                ]
                .iter()
                .position(|key| action.contains(key));
                Setting {
                    title: r["title"].as_str().unwrap_or("").into(),
                    subtitle: r["sub"].as_str().unwrap_or("").into(),
                    value: r["value"].as_str().unwrap_or("").into(),
                    group: r["group"].as_str().unwrap_or("").into(),
                    toggle: toggle_index.is_some(),
                    checked: toggle_index.is_some_and(|i| self.settings.toggles[i]),
                }
            })
            .collect::<Vec<_>>();
        ui.global::<State>()
            .set_settings(ModelRc::new(VecModel::from(rows)));
    }
    fn palette(&mut self, ui: &AppWindow, index: usize) {
        if let Some(p) = self.palettes.get(index) {
            let theme = ui.global::<Theme>();
            theme.set_accent(color(p["primary"].as_str().unwrap()));
            theme.set_secondary(color(p["secondary"].as_str().unwrap()));
            theme.set_base(color(p["baseDark"].as_str().unwrap()));
            theme.set_card(rgba(p["cardBg"].as_str().unwrap()));
            ui.global::<State>().set_palette(index as i32);
            self.rows[4]["sub"] = p["sub"].clone();
            self.rows[4]["value"] = p["name"].clone();
        }
        self.rows(ui);
    }
    fn setting(&mut self, ui: &AppWindow, visible_index: usize) {
        let Some(&i) = self.visible_rows.get(visible_index) else {
            return;
        };
        let action = self.rows[i]["action"].as_str().unwrap_or("").to_owned();
        if action.contains("cycleAudioQuality") {
            self.settings.quality = (self.settings.quality + 1) % 4;
            let (label, pill) = [
                ("Alta fidelidade Lossless • 320 kbps", "320k"),
                ("Master Hi-Res FLAC • 24-bit/96kHz", "Hi-Res"),
                ("Padrão Econômico • 160 kbps", "160k"),
                ("Automático (Adapta à conexão)", "Auto"),
            ][self.settings.quality];
            self.rows[i]["sub"] = label.into();
            self.rows[i]["value"] = pill.into();
        } else if action.contains("cycleCrossfade") {
            self.settings.crossfade = (self.settings.crossfade + 1) % 5;
            let seconds = self.settings.crossfade * 2;
            self.rows[i]["value"] = format!("{seconds}s").into();
            self.rows[i]["sub"] = if seconds == 0 {
                "Desativado (Corte seco)".into()
            } else {
                format!("Transição suave de {seconds}s entre músicas").into()
            };
        } else if action.contains("cycleThemeNext") {
            self.palette(ui, (ui.global::<State>().get_palette() as usize + 1) % 6);
        } else if action.contains("toggleSetting") {
            let index = [
                "toggleNormalization",
                "toggleLiquidGlass",
                "toggleAnimations",
                "toggleHistory",
            ]
            .iter()
            .position(|k| action.contains(k))
            .unwrap();
            self.settings.toggles[index] = !self.settings.toggles[index];
            ui.global::<Theme>().set_glass(self.settings.toggles[1]);
        } else if action.contains("scanLibraryNow") {
            self.rows[i]["sub"] = "Indexando arquivos locais de áudio...".into();
            self.rows[i]["value"] = "Buscando...".into();
            self.scan_until = Some(Instant::now() + Duration::from_millis(1200));
        } else if action.contains("clearMusicCache") {
            ui.global::<State>().set_confirm(1);
        } else if action.contains("clearSearchHistory") {
            ui.global::<State>().set_confirm(2);
        } else if let Some(message) = action
            .strip_prefix("showToast('")
            .and_then(|s| s.strip_suffix("')"))
        {
            self.toast(ui, &message.replace("\\n", "\n"));
        }
        self.rows(ui);
    }
    fn command(&mut self, ui: &AppWindow, command: &str) {
        match command {
            "play" => self.audio.set_playing(!self.audio.playing()),
            "shuffle" => ui
                .global::<State>()
                .set_shuffle(!ui.global::<State>().get_shuffle()),
            "repeat" => ui
                .global::<State>()
                .set_repeat(!ui.global::<State>().get_repeat()),
            "mute" => self
                .audio
                .set_volume(if self.audio.volume() > 0.0 { 0.0 } else { 0.7 }),
            "next" => self.next(ui),
            "previous" => {
                if self.tracks[self.index].position <= 4.0 {
                    self.index = (self.index + self.tracks.len() - 1) % self.tracks.len();
                    self.audio.set_playing(true);
                }
                self.tracks[self.index].position = 0.0;
            }
            "random" => {
                self.index = self.random(false);
                self.tracks[self.index].position = 0.0;
                self.audio.set_playing(true);
            }
            "clear-cache" => {
                self.rows[9]["sub"] = "0 KB utilizados • 0 faixas em cache".into();
                self.rows(ui);
                self.toast(ui, "Cache limpo com sucesso");
            }
            "clear-history" => {
                ui.global::<State>().set_history(false);
                self.toast(ui, "Histórico de buscas apagado");
            }
            "share" => {
                let track = &self.tracks[self.index];
                let text = format!("Ouvindo {} de {} no kazzu!", track.title, track.artist);
                if self.clipboard.is_none() {
                    self.clipboard = arboard::Clipboard::new().ok();
                }
                match self
                    .clipboard
                    .as_mut()
                    .ok_or(())
                    .and_then(|clipboard| clipboard.set_text(text).map_err(|_| ()))
                {
                    Ok(()) => self.toast(ui, "Compartilhamento copiado!"),
                    Err(_) => self.toast(ui, "Não foi possível acessar a área de transferência"),
                }
            }
            "video" => self.toast(ui, "Reprodução em vídeo indisponível nesta versão"),
            "timer" => self.toast(ui, "Sleep Timer: Desligar após 30 minutos"),
            "more" => self.toast(
                ui,
                "Opções: Adicionar à Playlist, Ver Álbum, Ajustes de Áudio, Equalizador",
            ),
            "device" => self.toast(ui, "Saída: Alto-falantes internos (This phone)"),
            "profile" => self.toast(ui, "Perfil: cuscusdoce rael\nMembro Free desde Out/2024"),
            "history" => self.toast(ui, "Histórico de reprodução"),
            "playlist" => self.toast(ui, "Criação de playlists indisponível neste protótipo"),
            command if command.starts_with("find:") => {
                if let Some(index) = self.tracks.iter().position(|t| t.title == &command[5..]) {
                    self.index = index;
                    self.tracks[index].position = 0.0;
                    self.audio.set_playing(true);
                }
            }
            message => self.toast(ui, &format!("Abrindo {message}")),
        }
        self.publish(ui);
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let snapshot_dir = std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--snapshot-dir=")
            .map(std::path::PathBuf::from)
    });
    let snapshot_window = snapshot_dir.as_ref().map(|_| snapshot::install());
    let ui = AppWindow::new()?;
    let controller = Rc::new(RefCell::new(Controller {
        tracks: serde_json::from_str(include_str!("tracks.json")).expect("catálogo do protótipo"),
        index: 0,
        audio: SimulatedAudio::default(),
        settings: Settings::default(),
        rows: serde_json::from_str(include_str!("settings.json")).expect("ajustes"),
        visible_rows: vec![],
        palettes: serde_json::from_str(include_str!("palettes.json")).expect("paletas"),
        seed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
            | 1,
        toast_until: Instant::now(),
        scan_until: None,
        scan_reset: None,
        clipboard: None,
    }));
    ui.global::<State>()
        .set_songs(songs(&controller.borrow().tracks));
    controller.borrow().publish(&ui);
    controller.borrow_mut().rows(&ui);
    // A playlist is a model ready for future creation; the HTML provides no creation flow.
    let playlist = models::Playlist {
        name: "💿 Continue ouvindo".into(),
        tracks: vec![0, 1, 3],
    };
    ui.global::<State>()
        .set_continue_title(playlist.name.into());
    ui.global::<State>()
        .set_continue_tracks(ModelRc::new(VecModel::from(
            playlist
                .tracks
                .into_iter()
                .map(|index| index as i32)
                .collect::<Vec<_>>(),
        )));
    macro_rules! bind { ($method:ident, |$c:ident, $w:ident $(, $arg:ident)*| $body:block) => {{
        let weak = ui.as_weak(); let state = controller.clone();
        ui.global::<State>().$method(move |$($arg),*| { if let Some($w) = weak.upgrade() { let mut $c = state.borrow_mut(); $body } });
    }}; }
    bind!(on_command, |c, w, command| {
        c.command(&w, &command);
    });
    bind!(on_select, |c, w, index, preserve| {
        if (0..c.tracks.len() as i32).contains(&index) {
            c.index = index as usize;
            if !preserve {
                c.tracks[index as usize].position = 0.0;
            }
            c.audio.set_playing(true);
            c.publish(&w);
        }
    });
    bind!(on_seek, |c, w, value| {
        let index = c.index;
        c.tracks[index].position = value.clamp(0.0, 1.0) * c.tracks[index].duration;
        c.publish(&w);
    });
    bind!(on_volume_changed, |c, w, value| {
        c.audio.set_volume(value);
        c.publish(&w);
    });
    bind!(on_theme, |c, w, index| {
        c.palette(&w, index as usize);
    });
    bind!(on_setting, |c, w, index| {
        c.setting(&w, index as usize);
    });
    bind!(on_filter_settings, |c, w, _query| {
        c.rows(&w);
    });
    bind!(on_search, |c, w, query| {
        c.search(&w, &query);
    });
    let timer = slint::Timer::default();
    let weak = ui.as_weak();
    let state = controller.clone();
    let last = Rc::new(RefCell::new(Instant::now()));
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(w) = weak.upgrade() {
                let now = Instant::now();
                let elapsed = now.duration_since(*last.borrow()).as_secs_f32();
                *last.borrow_mut() = now;
                let mut c = state.borrow_mut();
                let index = c.index;
                let mut position = c.tracks[index].position;
                c.audio.advance(&mut position, elapsed);
                c.tracks[index].position = position;
                if position >= c.tracks[index].duration {
                    if w.global::<State>().get_repeat() {
                        c.tracks[index].position = 0.0;
                    } else {
                        c.next(&w);
                    }
                }
                if now >= c.toast_until {
                    w.global::<State>().set_toast("".into());
                }
                if c.scan_until.is_some_and(|deadline| now >= deadline) {
                    c.scan_until = None;
                    c.scan_reset = Some(now + Duration::from_millis(2500));
                    c.rows[8]["sub"] = "742 músicas locais encontradas • Atualizado agora".into();
                    c.rows[8]["value"] = "Concluído!".into();
                    c.rows(&w);
                }
                if c.scan_reset.is_some_and(|deadline| now >= deadline) {
                    c.scan_reset = None;
                    c.rows[8]["value"] = "Escanear".into();
                    c.rows(&w);
                }
                c.publish(&w);
            }
        },
    );
    if let (Some(directory), Some(window)) = (snapshot_dir, snapshot_window) {
        snapshot::capture(&ui, &window, &directory).expect("gravar capturas Slint");
        return Ok(());
    }
    ui.run()
}
