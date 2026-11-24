//! Capas procedurais (sem arquivos) e os fundos derivados da capa (`backdrop`, `backdrop-wide`, escurecimentos), como o app os faz.

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

fn rgba_image(kind: Kind, size: u32) -> image::RgbaImage {
    image::RgbaImage::from_raw(size, size, rgba(kind, size)).expect("buffer RGBA")
}

fn saturate(img: &mut image::RgbaImage, k: f32) {
    for p in img.pixels_mut() {
        let gray = 0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2]);
        for c in 0..3 {
            p[c] = (gray + (f32::from(p[c]) - gray) * k).round().clamp(0.0, 255.0) as u8;
        }
        p[3] = 255;
    }
}

fn from_rgba_image(img: &image::RgbaImage) -> Image {
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(img.width(), img.height());
    buf.make_mut_bytes().copy_from_slice(img.as_raw());
    Image::from_rgba8(buf)
}

/// `backdrop` do v4 (192×464): a capa, o reflexo dela e os 80 px do topo, desfoque σ3 e saturação ×1,2. Mesma receita do
/// Rust do app (`kazzu_app::art::backdrop`).
pub fn backdrop(kind: Kind) -> Image {
    let s192 = image::imageops::resize(&rgba_image(kind, 384), 192, 192, image::imageops::FilterType::Lanczos3);
    let mut tall = image::RgbaImage::new(192, 464);
    image::imageops::replace(&mut tall, &s192, 0, 0);
    image::imageops::replace(&mut tall, &image::imageops::flip_vertical(&s192), 0, 192);
    image::imageops::replace(&mut tall, &image::imageops::crop_imm(&s192, 0, 0, 192, 80).to_image(), 0, 384);
    let mut out = image::imageops::blur(&tall, 3.0);
    saturate(&mut out, 1.2);
    from_rgba_image(&out)
}

/// `backdrop-wide` (96×96, σ9, saturação ×1,35): tinge o mini-player.
pub fn backdrop_wide(kind: Kind) -> Image {
    let small = image::imageops::resize(&rgba_image(kind, 192), 96, 96, image::imageops::FilterType::Lanczos3);
    let mut out = image::imageops::blur(&small, 9.0);
    saturate(&mut out, 1.35);
    from_rgba_image(&out)
}

fn lin(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// (`player-dim`, `mini-dim`): regra de `design/v4/tools/make_covers.py`, a mesma do app (`art::glass_dims`).
pub fn glass_dims(kind: Kind) -> (f32, f32) {
    let cells = image::imageops::resize(&rgba_image(kind, 96), 12, 12, image::imageops::FilterType::Triangle);
    let worst = cells.pixels().map(|p| 0.2126 * lin(f32::from(p[0])) + 0.7152 * lin(f32::from(p[1])) + 0.0722 * lin(f32::from(p[2]))).fold(0.0_f32, f32::max);
    let v = 255.0 * if worst <= 0.003_130_8 { 12.92 * worst } else { 1.055 * worst.powf(1.0 / 2.4) - 0.055 };
    let need = |x: f32| if x <= 112.0 { 0.0 } else { (x - 112.0) / (x - 12.0) };
    ((need(v) + 0.02).clamp(0.24, 0.64), (need(0.45 * 255.0 + 0.55 * v) + 0.02).clamp(0.30, 0.70))
}
