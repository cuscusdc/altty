#![allow(clippy::excessive_precision)] // constantes publicadas do OKLab
//! Capas procedurais (sem arquivos), cor do palco/acento a partir da capa (OKLCH) e o `backdrop` pré-desfocado.

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Sunset,
    Waves,
    Radar,
    Pale,
    Gray,
    Stripes,
    Yellow,
    Teal,
    Rose,
    Night,
}

pub const ALL_KINDS: [Kind; 10] = [Kind::Sunset, Kind::Waves, Kind::Radar, Kind::Pale, Kind::Gray, Kind::Stripes, Kind::Yellow, Kind::Teal, Kind::Rose, Kind::Night];

type Rgb = [f32; 3];

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn dist(u: f32, v: f32, cx: f32, cy: f32) -> f32 {
    ((u - cx).powi(2) + (v - cy).powi(2)).sqrt()
}

/// Cor da capa no ponto (u, v) ∈ [0, 1]².
fn shade(kind: Kind, u: f32, v: f32) -> Rgb {
    match kind {
        Kind::Sunset => {
            let mut c = mix([0.96, 0.68, 0.38], [0.93, 0.58, 0.34], v);
            let r = dist(u, v, 0.5, 1.08);
            let bands: [(f32, Rgb); 6] = [
                (0.20, [0.98, 0.86, 0.62]),
                (0.27, [0.96, 0.62, 0.36]),
                (0.34, [0.86, 0.33, 0.27]),
                (0.40, [0.97, 0.66, 0.38]),
                (0.47, [0.98, 0.86, 0.62]),
                (0.56, [0.86, 0.33, 0.27]),
            ];
            for (limit, col) in bands {
                if r < limit {
                    c = col;
                    break;
                }
            }
            if dist(u, v, 0.78, 0.2) < 0.11 {
                c = [0.99, 0.9, 0.72];
            }
            c
        }
        Kind::Waves => {
            let mut c = mix([0.14, 0.38, 0.80], [0.05, 0.2, 0.55], v);
            for k in 0..5 {
                let y = 0.22 + 0.14 * k as f32 + 0.05 * (std::f32::consts::TAU * (u * 1.4 + k as f32 * 0.21)).sin();
                if (v - y).abs() < 0.02 {
                    c = [0.72, 0.88, 1.0];
                }
            }
            c
        }
        Kind::Radar => {
            let mut c = mix([0.03, 0.12, 0.09], [0.02, 0.07, 0.06], v);
            let r = dist(u, v, 0.5, 0.5);
            for k in 1..5 {
                if (r - 0.11 * k as f32).abs() < 0.006 {
                    c = [0.18, 0.74, 0.48];
                }
            }
            if (u - 0.5).abs() < 0.003 || (v - 0.5).abs() < 0.003 {
                c = [0.14, 0.55, 0.38];
            }
            if r < 0.03 {
                c = [0.55, 0.95, 0.7];
            }
            c
        }
        Kind::Pale => {
            // capa clara: pior caso de contraste para o texto sobre o palco
            let mut c = mix([0.98, 0.95, 0.90], [0.96, 0.90, 0.86], v);
            let hill = 0.62 + 0.08 * (std::f32::consts::TAU * u * 0.9).sin();
            if v > hill {
                c = [0.94, 0.74, 0.70];
            }
            if v > hill + 0.16 {
                c = [0.96, 0.84, 0.78];
            }
            if dist(u, v, 0.5, 0.4) < 0.15 {
                c = [0.97, 0.80, 0.72];
            }
            c
        }
        Kind::Gray => {
            let mut c = mix([0.62, 0.62, 0.63], [0.44, 0.44, 0.46], v);
            let r = dist(u, v, 0.5, 0.5);
            if r < 0.27 {
                c = [0.30, 0.30, 0.32];
            }
            if (r - 0.34).abs() < 0.008 {
                c = [0.78, 0.78, 0.8];
            }
            c
        }
        Kind::Stripes => {
            let mut c = [0.11, 0.07, 0.08];
            for k in 0..5 {
                let y0 = 0.14 + 0.16 * k as f32;
                if v > y0 && v < y0 + 0.09 && u > 0.1 && u < 0.9 {
                    c = mix([0.98, 0.62, 0.35], [0.85, 0.28, 0.30], k as f32 / 4.0);
                }
            }
            c
        }
        Kind::Yellow => {
            let mut c = [0.97, 0.80, 0.16];
            if v > 0.72 {
                c = [0.20, 0.40, 0.86];
            }
            if dist(u, v, 0.5, 0.45) < 0.23 {
                c = [0.09, 0.08, 0.08];
            }
            if (dist(u, v, 0.82, 0.2) - 0.07).abs() < 0.012 {
                c = [0.09, 0.08, 0.08];
            }
            c
        }
        Kind::Teal => {
            let mut c = mix([0.10, 0.48, 0.52], [0.04, 0.24, 0.30], v);
            if dist(u, v, 0.5, 0.58) < 0.2 && v < 0.64 {
                c = [0.97, 0.66, 0.38];
            }
            if v > 0.64 && (v * 60.0).fract() < 0.28 {
                c = mix(c, [0.03, 0.16, 0.2], 0.7);
            }
            c
        }
        Kind::Rose => {
            let mut c = mix([0.96, 0.52, 0.62], [0.68, 0.30, 0.56], v);
            if dist(u, v, 0.35, 0.4) < 0.18 {
                c = [0.99, 0.82, 0.80];
            }
            if dist(u, v, 0.66, 0.62) < 0.13 {
                c = [0.99, 0.65, 0.55];
            }
            c
        }
        Kind::Night => {
            let mut c = mix([0.08, 0.09, 0.28], [0.30, 0.15, 0.42], v);
            if dist(u, v, 0.7, 0.3) < 0.12 {
                c = [0.98, 0.93, 0.78];
            }
            if dist(u, v, 0.76, 0.27) < 0.1 {
                c = mix([0.08, 0.09, 0.28], [0.30, 0.15, 0.42], 0.35);
            }
            c
        }
    }
}

pub fn rgba(kind: Kind, size: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let c = shade(kind, (x as f32 + 0.5) / size as f32, (y as f32 + 0.5) / size as f32);
            out.extend_from_slice(&[(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255]);
        }
    }
    out
}

pub fn image(kind: Kind, size: u32) -> Image {
    to_image(&rgba(kind, size), size)
}

pub fn to_image(rgba: &[u8], size: u32) -> Image {
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(size, size);
    buf.make_mut_bytes().copy_from_slice(rgba);
    Image::from_rgba8(buf)
}

/// Capa 96×96 pré-desfocada para o palco (σ≈4, saturação ×1,35, brilho ×0,72): a mesma receita do Rust do app.
pub fn backdrop(kind: Kind) -> Image {
    const N: usize = 96;
    let src = rgba(kind, N as u32);
    let mut px: Vec<[f32; 3]> = src.chunks(4).map(|p| [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]).collect();
    for _ in 0..3 {
        // desfoque de caixa 9×9 separável, três passadas ≈ gaussiana de σ 4
        for horizontal in [true, false] {
            let mut next = px.clone();
            for y in 0..N {
                for x in 0..N {
                    let mut acc = [0.0f32; 3];
                    let mut n = 0.0;
                    for d in -4i32..=4 {
                        let (xx, yy) = if horizontal { ((x as i32 + d).clamp(0, N as i32 - 1) as usize, y) } else { (x, (y as i32 + d).clamp(0, N as i32 - 1) as usize) };
                        let p = px[yy * N + xx];
                        acc[0] += p[0];
                        acc[1] += p[1];
                        acc[2] += p[2];
                        n += 1.0;
                    }
                    next[y * N + x] = [acc[0] / n, acc[1] / n, acc[2] / n];
                }
            }
            px = next;
        }
    }
    let mut out = Vec::with_capacity(N * N * 4);
    for p in px {
        let l = 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2];
        let sat = |c: f32| ((l + (c - l) * 1.35) * 0.72).clamp(0.0, 1.0);
        out.extend_from_slice(&[(sat(p[0]) * 255.0) as u8, (sat(p[1]) * 255.0) as u8, (sat(p[2]) * 255.0) as u8, 255]);
    }
    to_image(&out, N as u32)
}

// ---- OKLCH ---------------------------------------------------------------------------------------------------------

fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn from_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// (L, C, h°) de uma cor sRGB.
pub fn to_oklch(rgb: Rgb) -> (f32, f32, f32) {
    let (r, g, b) = (to_linear(rgb[0]), to_linear(rgb[1]), to_linear(rgb[2]));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    let ll = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s;
    let a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
    let bb = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
    (ll, (a * a + bb * bb).sqrt(), bb.atan2(a).to_degrees().rem_euclid(360.0))
}

pub fn oklch(l: f32, c: f32, h_deg: f32) -> slint::Color {
    let h = h_deg.to_radians();
    let (a, b) = (c * h.cos(), c * h.sin());
    let l_ = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m_ = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s_ = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    let r = 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_;
    let g = -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_;
    let bl = -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_;
    slint::Color::from_rgb_u8((from_linear(r) * 255.0) as u8, (from_linear(g) * 255.0) as u8, (from_linear(bl) * 255.0) as u8)
}

/// Matiz da capa (média ponderada pela saturação); capa pálida ou cinza cai no âmbar da marca (58°).
pub fn cover_hue(kind: Kind) -> f32 {
    let src = rgba(kind, 32);
    let (mut sx, mut sy, mut w) = (0.0f32, 0.0f32, 0.0f32);
    for p in src.chunks(4) {
        let (_, c, h) = to_oklch([p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]);
        sx += c * h.to_radians().cos();
        sy += c * h.to_radians().sin();
        w += c;
    }
    let mean_c = w / (src.len() / 4) as f32;
    if mean_c < 0.05 {
        return 58.0;
    }
    sy.atan2(sx).to_degrees().rem_euclid(360.0)
}

/// Cores do palco, do vidro e do acento para um matiz (SLINT_SPEC §7).
pub struct StageColors {
    pub dark: [slint::Color; 4],
    pub light: [slint::Color; 4],
    pub tint_dark: slint::Color,
    pub tint_light: slint::Color,
    pub accent_dark: slint::Color,
    pub accent_dark_ink: slint::Color,
    pub accent_light: slint::Color,
    pub accent_light_ink: slint::Color,
}

pub fn stage_colors(h: f32) -> StageColors {
    let d = |l, c, dh: f32| oklch(l, c, h + dh);
    StageColors {
        dark: [d(0.50, 0.19, 0.0), d(0.45, 0.18, -36.0), d(0.41, 0.14, 147.0), d(0.44, 0.15, 37.0)],
        light: [d(0.86, 0.09, 0.0), d(0.86, 0.085, -36.0), d(0.89, 0.06, 147.0), d(0.90, 0.08, 37.0)],
        tint_dark: oklch(0.24, 0.055, h),
        tint_light: oklch(0.99, 0.012, h),
        accent_dark: oklch(0.80, 0.14, h),
        accent_dark_ink: oklch(0.20, 0.03, h),
        accent_light: oklch(0.50, 0.15, h),
        accent_light_ink: oklch(0.98, 0.01, h),
    }
}
