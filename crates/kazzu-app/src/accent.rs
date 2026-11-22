//! Cor de destaque derivada da capa (DESIGN.md §3.4), em OKLCH.
//!
//! Espelha `design/tools/make_assets.py` (`derive_accent` / `accent_for`).
//!
//! Passos: capa reduzida para 32x32; ignora croma < 0,04, L < 0,18 ou L > 0,97; peso `C^1,5 · (1 − |L − 0,62|)` em
//! 12 baldes de matiz; peso total < 0,6 vira o âmbar da marca; balde vencedor (com metade dos vizinhos);
//! L e C presos à faixa do tema; tinta do acento = a de maior contraste.

use image::{RgbaImage, imageops::FilterType};

/// Cor sRGB 8 bits.
pub type Rgb = [u8; 3];

/// Acento + tinta sobre ele (para texto/ícone em botões de acento).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccentColor {
    pub accent: Rgb,
    pub ink: Rgb,
}

/// Acento para os dois temas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accent {
    pub dark: AccentColor,
    pub light: AccentColor,
    /// Matiz (graus, 0..360) que alimenta o palco; 58° (âmbar) quando a capa não tem cor.
    pub hue: u16,
}

/// Matiz do âmbar da marca.
pub const BRAND_HUE: u16 = 58;

impl Accent {
    /// Âmbar da marca (`tokens.json` > brand).
    pub const BRAND: Accent = Accent {
        dark: AccentColor { accent: [0xff, 0xa1, 0x54], ink: [0x19, 0x0c, 0x04] },
        light: AccentColor { accent: [0xab, 0x4b, 0x01], ink: [0xfe, 0xfb, 0xf8] },
        hue: BRAND_HUE,
    };

    pub fn for_theme(&self, dark: bool) -> AccentColor {
        if dark { self.dark } else { self.light }
    }
}

fn gamma_to_lin(x: f64) -> f64 {
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn lin_to_gamma(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }
}

/// sRGB 0..255 -> (L, C, h em graus).
pub fn srgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let [r, g, b] = [r, g, b].map(|c| gamma_to_lin(f64::from(c) / 255.0));
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_702_5 * b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());
    let ll = 0.210_454_255_3 * l_ + 0.793_617_785 * m_ - 0.004_072_046_8 * s_;
    let a = 1.977_998_495_1 * l_ - 2.428_592_205 * m_ + 0.450_593_709_9 * s_;
    let bb = 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766 * s_;
    (ll, a.hypot(bb), bb.atan2(a).to_degrees().rem_euclid(360.0))
}

/// OKLCH -> sRGB linear (pode sair do gamut).
fn oklch_to_linear(l: f64, c: f64, h: f64) -> [f64; 3] {
    let (a, b) = (c * h.to_radians().cos(), c * h.to_radians().sin());
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548 * b;
    let (l, m, s) = (l_.powi(3), m_.powi(3), s_.powi(3));
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
    ]
}

fn in_gamut(rgb: [f64; 3]) -> bool {
    rgb.iter().all(|c| (-1e-4..=1.0 + 1e-4).contains(c))
}

fn to_u8(rgb: [f64; 3]) -> Rgb {
    rgb.map(|c| (lin_to_gamma(c) * 255.0).round() as u8)
}

fn luminance(rgb: Rgb) -> f64 {
    let [r, g, b] = rgb.map(|c| gamma_to_lin(f64::from(c) / 255.0));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (l1, l2) = (luminance(a), luminance(b));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Tinta do acento: a de maior contraste entre `#120f0c` e `#fcfbf8`.
fn ink_for(accent: Rgb) -> Rgb {
    const DARK: Rgb = [0x12, 0x0f, 0x0c];
    const WHITE: Rgb = [0xfc, 0xfb, 0xf8];
    if contrast(DARK, accent) >= contrast(WHITE, accent) { DARK } else { WHITE }
}

/// Prende (L, C, h) à faixa do tema e reduz o croma até caber no sRGB.
fn clamp_to_theme(l: f64, c: f64, h: f64, dark: bool) -> AccentColor {
    let (lo, hi) = if dark { (0.74, 0.86) } else { (0.44, 0.52) };
    let (l2, mut c2) = (l.clamp(lo, hi), c.clamp(0.10, 0.17));
    let mut rgb = oklch_to_linear(l2, c2, h);
    while !in_gamut(rgb) && c2 > 0.0 {
        c2 -= 0.002;
        rgb = oklch_to_linear(l2, c2.max(0.0), h);
    }
    let accent = to_u8(rgb);
    AccentColor { accent, ink: ink_for(accent) }
}

#[derive(Clone, Copy, Default)]
struct Bin {
    w: f64,
    l: f64,
    c: f64,
    x: f64,
    y: f64,
}

/// Cor base (L, C, h) da capa, ou `None` se for quase monocromática.
pub fn base_color(img: &RgbaImage) -> Option<(f64, f64, f64)> {
    let small = image::imageops::resize(img, 32, 32, FilterType::Triangle);
    let mut bins = [Bin::default(); 12];
    let mut total = 0.0;
    for p in small.pixels() {
        let (l, c, h) = srgb_to_oklch(p[0], p[1], p[2]);
        if c < 0.04 || !(0.18..=0.97).contains(&l) {
            continue;
        }
        let w = c.powf(1.5) * (1.0 - (l - 0.62).abs());
        let k = ((h / 30.0).floor() as usize) % 12;
        let b = &mut bins[k];
        b.w += w;
        b.l += w * l;
        b.c += w * c;
        b.x += w * h.to_radians().cos();
        b.y += w * h.to_radians().sin();
        total += w;
    }
    if total < 0.6 {
        return None;
    }
    let score = |k: usize| bins[k].w + 0.5 * (bins[(k + 11) % 12].w + bins[(k + 1) % 12].w);
    let best_k = (0..12).max_by(|&a, &b| score(a).total_cmp(&score(b)))?;
    let best = bins[best_k];
    if best.w <= 0.0 {
        return None;
    }
    Some((best.l / best.w, best.c / best.w, best.y.atan2(best.x).to_degrees().rem_euclid(360.0)))
}

/// Acento da capa para os dois temas (âmbar da marca se a capa não tiver cor).
pub fn derive(img: &RgbaImage) -> Accent {
    match base_color(img) {
        None => Accent::BRAND,
        Some((l, c, h)) => Accent { dark: clamp_to_theme(l, c, h, true), light: clamp_to_theme(l, c, h, false), hue: (h.round() as u16) % 360 },
    }
}

/// Cores do palco (`Theme.stage-*`, `Theme.glass-tint-*`) para um matiz: fórmula de `DESIGN.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagePalette {
    pub dark: [Rgb; 4],
    pub light: [Rgb; 4],
    pub base_dark: Rgb,
    pub base_light: Rgb,
    pub tint_dark: Rgb,
    pub tint_light: Rgb,
}

/// `[L, C, dh]` dos quatro manchões (tokens.json > stage).
const STAGE_DARK: [[f64; 3]; 4] = [[0.50, 0.19, 0.0], [0.45, 0.18, -36.0], [0.41, 0.14, 147.0], [0.44, 0.15, 37.0]];
const STAGE_LIGHT: [[f64; 3]; 4] = [[0.86, 0.09, 0.0], [0.86, 0.085, -36.0], [0.89, 0.06, 147.0], [0.90, 0.08, 37.0]];

/// OKLCH -> sRGB 8 bits, reduzindo o croma até caber no gamut.
pub fn oklch_to_srgb(l: f64, c: f64, h: f64) -> Rgb {
    let mut c2 = c;
    let mut rgb = oklch_to_linear(l, c2, h);
    while !in_gamut(rgb) && c2 > 0.0 {
        c2 -= 0.002;
        rgb = oklch_to_linear(l, c2.max(0.0), h);
    }
    to_u8(rgb)
}

pub fn stage_palette(hue: u16) -> StagePalette {
    let h = f64::from(hue);
    let mk = |t: &[[f64; 3]; 4]| t.map(|[l, c, dh]| oklch_to_srgb(l, c, (h + dh).rem_euclid(360.0)));
    StagePalette {
        dark: mk(&STAGE_DARK),
        light: mk(&STAGE_LIGHT),
        base_dark: oklch_to_srgb(0.20, 0.05, h),
        base_light: oklch_to_srgb(0.94, 0.03, h),
        tint_dark: oklch_to_srgb(0.24, 0.055, h),
        tint_light: oklch_to_srgb(0.99, 0.012, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(rgb: Rgb) -> RgbaImage {
        RgbaImage::from_pixel(64, 64, Rgba([rgb[0], rgb[1], rgb[2], 255]))
    }

    fn hex(c: Rgb) -> String {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    }

    #[test]
    fn gray_and_black_covers_use_the_brand_amber() {
        assert_eq!(derive(&solid([128, 128, 128])), Accent::BRAND);
        assert_eq!(derive(&solid([0, 0, 0])), Accent::BRAND);
        assert_eq!(derive(&solid([255, 255, 255])), Accent::BRAND);
    }

    #[test]
    fn teal_cover_gives_a_teal_accent_inside_the_theme_lightness_band() {
        let a = derive(&solid([0x1d, 0x7a, 0x86]));
        let (l_d, _, h_d) = srgb_to_oklch(a.dark.accent[0], a.dark.accent[1], a.dark.accent[2]);
        let (l_l, _, _) = srgb_to_oklch(a.light.accent[0], a.light.accent[1], a.light.accent[2]);
        assert!((0.72..=0.88).contains(&l_d), "L escuro {l_d}");
        assert!((0.42..=0.54).contains(&l_l), "L claro {l_l}");
        assert!((190.0..=230.0).contains(&h_d), "matiz {h_d} deveria ser azul-esverdeado");
    }

    #[test]
    fn every_derived_accent_reads_on_both_backgrounds() {
        // fundo escuro #0c0806 e claro #f6f3ed: o acento precisa de >= 4,5:1 (a faixa de L garante).
        for rgb in [[200, 60, 40], [60, 160, 90], [230, 190, 50], [90, 120, 220], [220, 120, 160]] {
            let a = derive(&solid(rgb));
            assert!(contrast(a.dark.accent, [0x0c, 0x08, 0x06]) >= 4.5, "{rgb:?} escuro {}", hex(a.dark.accent));
            assert!(contrast(a.light.accent, [0xf6, 0xf3, 0xed]) >= 4.5, "{rgb:?} claro {}", hex(a.light.accent));
            assert!(contrast(a.dark.ink, a.dark.accent) >= 4.5);
            assert!(contrast(a.light.ink, a.light.accent) >= 4.5);
        }
    }

    #[test]
    fn stage_for_the_brand_hue_matches_the_design_tokens_closely() {
        // tokens.slint: stage-a-dark #944f04, stage-b-dark #a20222, stage-c-dark #00555c (matiz 58°)
        let p = stage_palette(BRAND_HUE);
        let close = |a: Rgb, b: Rgb| a.iter().zip(b).all(|(x, y)| (i32::from(*x) - i32::from(y)).abs() <= 12);
        assert!(close(p.dark[0], [0x94, 0x4f, 0x04]), "a {}", hex(p.dark[0]));
        assert!(close(p.dark[1], [0xa2, 0x02, 0x22]), "b {}", hex(p.dark[1]));
        assert!(close(p.dark[2], [0x00, 0x55, 0x5c]), "c {}", hex(p.dark[2]));
        assert!(close(p.tint_dark, [0x33, 0x17, 0x02]), "tint {}", hex(p.tint_dark));
    }

    #[test]
    fn dark_stage_stays_dark_enough_for_the_text_on_glass() {
        for hue in [40, 205, 150, 58, 300] {
            for c in stage_palette(hue).dark {
                let (l, _, _) = srgb_to_oklch(c[0], c[1], c[2]);
                assert!(l <= 0.51, "hue {hue}: L {l}");
            }
        }
    }

    #[test]
    fn brand_values_match_the_design_tokens() {
        assert_eq!(hex(Accent::BRAND.dark.accent), "#ffa154");
        assert_eq!(hex(Accent::BRAND.light.accent), "#ab4b01");
        assert_eq!(hex(Accent::BRAND.dark.ink), "#190c04");
    }
}
