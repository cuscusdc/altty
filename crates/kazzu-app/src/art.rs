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
    image::open(path).ok().map(|i| i.to_rgba8())
}

/// Lado do fundo pré-desfocado.
pub const BACKDROP_SIDE: u32 = 96;
/// Desfoque gaussiano (px, na resolução de 96x96).
const BLUR_SIGMA: f32 = 4.0;
const SATURATION: f32 = 1.25;
/// `im-base-darken` de tokens.json.
const DARKEN: f32 = 0.46;

/// Fundo do player: capa 96x96, desfoque gaussiano σ=4, saturação ×1,25, ×0,46 e 1 LSB de ruído
/// (evita faixas de cor em degradês escuros).
pub fn backdrop(cover: &RgbaImage) -> RgbaImage {
    let side = cover.width().min(cover.height()).max(1);
    let (x, y) = ((cover.width() - side) / 2, (cover.height() - side) / 2);
    let square = image::imageops::crop_imm(cover, x, y, side, side).to_image();
    let small = image::imageops::resize(&square, BACKDROP_SIDE, BACKDROP_SIDE, FilterType::Triangle);
    let mut out = image::imageops::blur(&small, BLUR_SIGMA);
    for (px, py, p) in out.enumerate_pixels_mut() {
        let Rgba([r, g, b, _]) = *p;
        let luma = 0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b);
        // ruído determinístico de 1 LSB (hash espacial barato)
        let noise = ((px.wrapping_mul(73_856_093) ^ py.wrapping_mul(19_349_663)) >> 7) & 1;
        let adj = |c: u8| {
            let sat = luma + (f32::from(c) - luma) * SATURATION;
            ((sat * DARKEN).round().clamp(0.0, 254.0) as u8).saturating_add(noise as u8)
        };
        *p = Rgba([adj(r), adj(g), adj(b), 255]);
    }
    out
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
    fn backdrop_is_96px_dark_and_opaque_even_for_a_white_cover() {
        let white = RgbaImage::from_pixel(640, 480, Rgba([255, 255, 255, 255]));
        let b = backdrop(&white);
        assert_eq!(b.dimensions(), (96, 96));
        assert!(b.pixels().all(|p| p[3] == 255));
        let max = b.pixels().map(|p| p[0].max(p[1]).max(p[2])).max().unwrap();
        // 255 × 0,46 ≈ 117 (+ ruído); nunca claro o bastante para estragar o contraste do texto
        assert!((110..=122).contains(&max), "max {max}");
    }

    #[test]
    fn backdrop_boosts_saturation() {
        let teal = RgbaImage::from_pixel(100, 100, Rgba([30, 120, 140, 255]));
        let b = backdrop(&teal);
        let p = b.get_pixel(48, 48);
        let spread_in = 140 - 30;
        let spread_out = i32::from(p[2]) - i32::from(p[0]);
        // 0,46 × 1,25 ≈ 0,575 do afastamento original
        assert!(spread_out as f32 > spread_in as f32 * 0.5, "spread {spread_out}");
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
