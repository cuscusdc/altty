//! Imagens: decodificação de capas e fundo derivado (DESIGN.md §3.5). Tudo aqui roda fora da thread de UI.

use std::path::Path;

use image::{Rgba, RgbaImage, imageops::FilterType};

/// Buffer de pixels que pode cruzar threads (vira `slint::Image` na thread de UI).
pub type Pixels = slint::SharedPixelBuffer<slint::Rgba8Pixel>;

pub fn to_pixels(img: &RgbaImage) -> Pixels {
    Pixels::clone_from_slice(img.as_raw(), img.width(), img.height())
}

/// Lê e decodifica uma imagem (JPEG/PNG). Erros viram `None`: capa ausente não é falha.
pub fn load(path: &Path) -> Option<RgbaImage> {
    match image::open(path) {
        Ok(i) => Some(i.to_rgba8()),
        Err(e) => {
            log::warn!("imagem ilegível {}: {e}", path.display());
            None
        }
    }
}

/// Lado do fundo largo (mini-player e player em janelas largas).
pub const BACKDROP_SIDE: u32 = 96;
/// `backdrop` do player (SLINT_SPEC v4 §6): capa 192x192, a capa espelhada logo abaixo e os 80 px do topo de novo.
pub const BACKDROP_W: u32 = 192;
pub const BACKDROP_H: u32 = 464;

/// Recorta o quadrado central.
fn center_square(cover: &RgbaImage) -> RgbaImage {
    let side = cover.width().min(cover.height()).max(1);
    let (x, y) = ((cover.width() - side) / 2, (cover.height() - side) / 2);
    image::imageops::crop_imm(cover, x, y, side, side).to_image()
}

/// Saturação como a do `ImageEnhance.Color` do PIL (mistura com o cinza de luma 601): `gray + (c - gray) * k`.
fn saturate(img: &mut RgbaImage, k: f32) {
    for p in img.pixels_mut() {
        let Rgba([r, g, b, a]) = *p;
        let gray = 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
        let adj = |c: u8| (gray + (f32::from(c) - gray) * k).round().clamp(0.0, 255.0) as u8;
        *p = Rgba([adj(r), adj(g), adj(b), a]);
    }
}

/// Fundo do player (192x464): a capa, o reflexo dela (espelhada na vertical) e os 80 px de cima de novo, com desfoque leve
/// (σ3) e saturação ×1,2. Quem desfoca de verdade é o vidro do console, ao vivo; aqui a cor só precisa estar lá.
pub fn backdrop(cover: &RgbaImage) -> RgbaImage {
    let s192 = image::imageops::resize(&center_square(cover), BACKDROP_W, BACKDROP_W, FilterType::Lanczos3);
    let mut tall = RgbaImage::new(BACKDROP_W, BACKDROP_H);
    image::imageops::replace(&mut tall, &s192, 0, 0);
    image::imageops::replace(&mut tall, &image::imageops::flip_vertical(&s192), 0, i64::from(BACKDROP_W));
    image::imageops::replace(&mut tall, &image::imageops::crop_imm(&s192, 0, 0, BACKDROP_W, 80).to_image(), 0, 384);
    let mut out = image::imageops::blur(&tall, 3.0);
    saturate(&mut out, 1.2);
    for p in out.pixels_mut() {
        p[3] = 255;
    }
    out
}

/// Capa 96x96 com desfoque forte (σ9) e saturação ×1,35: tinge o mini-player e fica atrás do player em janelas largas.
pub fn backdrop_wide(cover: &RgbaImage) -> RgbaImage {
    let small = image::imageops::resize(&center_square(cover), BACKDROP_SIDE, BACKDROP_SIDE, FilterType::Lanczos3);
    let mut out = image::imageops::blur(&small, 9.0);
    saturate(&mut out, 1.35);
    for p in out.pixels_mut() {
        p[3] = 255;
    }
    out
}

fn gamma_to_lin(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Escurecimentos do vidro sobre a capa (`player-dim`, `mini-dim`): regra de `design/v4/tools/make_covers.py`.
/// Com a capa em 12x12 (média de caixa), `V` é o cinza sRGB equivalente à célula mais clara; o texto branco precisa de um
/// fundo composto de no máximo 112 (5:1), e a tinta do vidro é quase preta (12) com alfa `dim`.
pub fn glass_dims(cover: &RgbaImage) -> (f32, f32) {
    let cells = image::imageops::resize(&center_square(cover), 12, 12, FilterType::Triangle);
    let worst = cells.pixels().map(|p| 0.2126 * gamma_to_lin(f32::from(p[0])) + 0.7152 * gamma_to_lin(f32::from(p[1])) + 0.0722 * gamma_to_lin(f32::from(p[2]))).fold(0.0_f32, f32::max);
    let v = 255.0 * if worst <= 0.003_130_8 { 12.92 * worst } else { 1.055 * worst.powf(1.0 / 2.4) - 0.055 };
    let need = |x: f32| if x <= 112.0 { 0.0 } else { (x - 112.0) / (x - 12.0) };
    let player = (need(v) + 0.02).clamp(0.24, 0.64);
    let mini = (need(0.45 * 255.0 + 0.55 * v) + 0.02).clamp(0.30, 0.70);
    (player, mini)
}

/// Redimensiona mantendo a proporção para que o maior lado seja `max` (sem ampliar).
pub fn fit_within(img: RgbaImage, max: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w.max(h) <= max {
        return img;
    }
    let scale = f64::from(max) / f64::from(w.max(h));
    let (nw, nh) = (((f64::from(w) * scale).round() as u32).max(1), ((f64::from(h) * scale).round() as u32).max(1));
    image::imageops::resize(&img, nw, nh, FilterType::Triangle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_is_192x464_opaque_and_mirrors_the_cover_below_it() {
        // topo vermelho, base azul: o reflexo (logo abaixo da capa) tem as cores trocadas de lado
        let cover = RgbaImage::from_fn(200, 200, |_, y| if y < 100 { Rgba([240, 20, 20, 255]) } else { Rgba([20, 20, 240, 255]) });
        let b = backdrop(&cover);
        assert_eq!(b.dimensions(), (192, 464));
        assert!(b.pixels().all(|p| p[3] == 255));
        assert!(b.get_pixel(96, 20)[0] > 150, "topo da capa é vermelho");
        assert!(b.get_pixel(96, 170)[2] > 150, "base da capa é azul");
        assert!(b.get_pixel(96, 200)[2] > 150, "logo abaixo da capa, o reflexo começa pela base (azul)");
        assert!(b.get_pixel(96, 370)[0] > 150, "o reflexo termina pelo topo (vermelho)");
    }

    #[test]
    fn backdrop_wide_is_96px_and_boosts_saturation() {
        let teal = RgbaImage::from_pixel(100, 100, Rgba([30, 120, 140, 255]));
        let b = backdrop_wide(&teal);
        assert_eq!(b.dimensions(), (96, 96));
        let p = b.get_pixel(48, 48);
        assert!(i32::from(p[2]) - i32::from(p[0]) > 110, "mais saturada que a capa (110): {p:?}");
    }

    #[test]
    fn glass_dims_follow_the_design_rule() {
        let white = glass_dims(&RgbaImage::from_pixel(64, 64, Rgba([255, 255, 255, 255])));
        let black = glass_dims(&RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 255])));
        // capa branca: (255 - 112) / (255 - 12) + 0,02 = 0,61 nos dois; capa preta: o piso de cada um
        assert!((white.0 - 0.61).abs() < 0.01 && (white.1 - 0.61).abs() < 0.01, "{white:?}");
        assert_eq!(black, (0.24, 0.30));
        let mid = glass_dims(&RgbaImage::from_pixel(64, 64, Rgba([160, 70, 30, 255])));
        assert!((0.24..=0.64).contains(&mid.0) && (0.30..=0.70).contains(&mid.1) && mid.1 >= mid.0, "{mid:?}");
    }

    #[test]
    fn fit_within_keeps_aspect_and_never_upscales() {
        let img = RgbaImage::new(1000, 500);
        assert_eq!(fit_within(img, 640).dimensions(), (640, 320));
        let small = RgbaImage::new(100, 50);
        assert_eq!(fit_within(small, 640).dimensions(), (100, 50));
    }

    #[test]
    fn missing_cover_is_none_not_a_panic() {
        assert!(load(Path::new("/nonexistent/cover.jpg")).is_none());
    }
}
