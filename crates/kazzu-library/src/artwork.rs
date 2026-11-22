//! Processamento e cache em disco das capas.
//!
//! Para cada imagem de origem gera, em `<cache>/covers/`:
//! * `<chave>.jpg`       — capa (lado maior ≤ 640 px);
//! * `<chave>_thumb.jpg` — miniatura (≤ 160 px);
//! * `<chave>_blur.jpg`  — versão minúscula já DESFOCADA (fundo glass; a UI
//!   Slint não tem backdrop blur, então o desfoque é pré-computado aqui).
//!
//! A chave é o hash FNV-1a 64 dos bytes originais: a mesma capa em vários
//! álbuns é gravada uma única vez. (Não é uso criptográfico.)

use std::path::{Path, PathBuf};

use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder};

use crate::model::{Artwork, Rgb};

const COVER_MAX: u32 = 640;
const THUMB_MAX: u32 = 160;
const BLUR_SIZE: u32 = 48;
const BLUR_SIGMA: f32 = 4.0;

pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub(crate) fn covers_dir(cache: &Path) -> PathBuf {
    cache.join("covers")
}

pub(crate) fn paths_for(cache: &Path, key: &str, accent: Rgb) -> Artwork {
    let d = covers_dir(cache);
    Artwork {
        cover: d.join(format!("{key}.jpg")),
        thumb: d.join(format!("{key}_thumb.jpg")),
        blurred: d.join(format!("{key}_blur.jpg")),
        accent,
    }
}

fn save_jpeg(img: &DynamicImage, path: &Path, quality: u8) -> Option<()> {
    let rgb = img.to_rgb8();
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, quality)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .ok()?;
    std::fs::write(path, buf).ok()
}

/// Decodifica, gera as três versões e devolve `(chave, accent)`.
pub(crate) fn process(source: &[u8], cache: &Path) -> Option<(String, Rgb)> {
    let img = image::load_from_memory(source).ok()?;
    let key = format!("{:016x}", fnv1a(source));
    let accent = dominant_color(&img);
    let dir = covers_dir(cache);
    std::fs::create_dir_all(&dir).ok()?;
    let art = paths_for(cache, &key, accent);

    if !art.cover.exists() {
        let cover = if img.width().max(img.height()) > COVER_MAX {
            img.resize(COVER_MAX, COVER_MAX, FilterType::Lanczos3)
        } else {
            img.clone()
        };
        save_jpeg(&cover, &art.cover, 88)?;
    }
    if !art.thumb.exists() {
        save_jpeg(
            &img.resize(THUMB_MAX, THUMB_MAX, FilterType::Triangle),
            &art.thumb,
            82,
        )?;
    }
    if !art.blurred.exists() {
        // Corta ao quadrado central e reduz antes de desfocar (barato e suave).
        let small = img
            .resize_to_fill(BLUR_SIZE, BLUR_SIZE, FilterType::Triangle)
            .to_rgb8();
        let blurred = imageops::blur(&small, BLUR_SIGMA);
        save_jpeg(&DynamicImage::ImageRgb8(blurred), &art.blurred, 80)?;
    }
    Some((key, accent))
}

/// Cor dominante: histograma 4 bits/canal em 32×32, pontuado por contagem ×
/// saturação (evita cinzas de fundo vencerem cores vivas).
pub(crate) fn dominant_color(img: &DynamicImage) -> Rgb {
    let small: RgbImage = img.resize(32, 32, FilterType::Triangle).to_rgb8();
    let mut buckets = vec![(0u32, 0u32, 0u32, 0u32); 4096];
    for p in small.pixels() {
        let [r, g, b] = p.0;
        let i = (usize::from(r >> 4) << 8) | (usize::from(g >> 4) << 4) | usize::from(b >> 4);
        let e = &mut buckets[i];
        e.0 += 1;
        e.1 += u32::from(r);
        e.2 += u32::from(g);
        e.3 += u32::from(b);
    }
    let mut best = (f32::MIN, Rgb(128, 128, 128));
    for (n, r, g, b) in buckets.into_iter().filter(|b| b.0 > 0) {
        let (r, g, b) = ((r / n) as u8, (g / n) as u8, (b / n) as u8);
        let max = f32::from(r.max(g).max(b));
        let min = f32::from(r.min(g).min(b));
        let sat = if max == 0.0 { 0.0 } else { (max - min) / max };
        let lum = max / 255.0;
        // Penaliza quase-preto/quase-branco, que são péssimos accents.
        let usable = (1.0 - (lum - 0.55).abs() * 1.2).clamp(0.15, 1.0);
        let score = n as f32 * (0.25 + sat) * usable;
        if score > best.0 {
            best = (score, Rgb(r, g, b));
        }
    }
    best.1
}
