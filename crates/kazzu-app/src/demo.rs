//! Biblioteca de demonstração: arquivos de áudio gerados por programa (nenhum conteúdo de terceiros) com tags e capa
//! fictícias. Serve para `--demo-library`, capturas de tela e testes. Os arquivos ficam fora do git.

use std::path::{Path, PathBuf};

use image::{Rgb, RgbImage};
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};

#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("E/S em {0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("falha ao gerar áudio: {0}")]
    Audio(String),
    #[error("falha ao gravar tags: {0}")]
    Tags(String),
    #[error("falha ao gerar capa: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Clone, Copy)]
enum Fmt {
    Flac { bits: u16, rate: u32 },
    Wav { bits: u16, rate: u32 },
    /// WAV convertido pelo `ffmpeg` se existir (MP3 320 ou Opus 160); sem `ffmpeg`, vira WAV 16/44,1.
    Lossy,
}

struct AlbumSpec {
    title: &'static str,
    artist: &'static str,
    genre: &'static str,
    year: u32,
    colors: ([u8; 3], [u8; 3]),
    shape: u8,
    tracks: &'static [&'static str],
    fmt: Fmt,
}

const ALBUMS: &[AlbumSpec] = &[
    AlbumSpec { title: "Maré Alta", artist: "Aurora Vento", genre: "Eletrônica", year: 2024, colors: ([18, 110, 124], [240, 150, 90]), shape: 0, tracks: &["Maré Alta", "Linha d'Água", "Horizonte Baixo", "Sal na Pele"], fmt: Fmt::Flac { bits: 24, rate: 96_000 } },
    AlbumSpec { title: "Nove Horas", artist: "Doma", genre: "Pop", year: 2023, colors: ([232, 120, 110], [250, 200, 170]), shape: 1, tracks: &["Hora do Café", "Nove Horas", "Fio de Luz", "Última Linha"], fmt: Fmt::Flac { bits: 16, rate: 44_100 } },
    AlbumSpec { title: "Bruma", artist: "Lia Cantos", genre: "Indie", year: 2022, colors: ([160, 190, 225], [245, 190, 170]), shape: 2, tracks: &["Mapa de Neblina", "Bruma", "Janela Aberta"], fmt: Fmt::Flac { bits: 16, rate: 44_100 } },
    AlbumSpec { title: "Quadra 7", artist: "Os Faróis", genre: "Rock", year: 2021, colors: ([238, 190, 50], [40, 80, 170]), shape: 0, tracks: &["Canal Sete", "Quadra 7", "Rádio de Esquina"], fmt: Fmt::Wav { bits: 16, rate: 44_100 } },
    AlbumSpec { title: "Sal e Cinza", artist: "Tomás Reis", genre: "Jazz", year: 2020, colors: ([200, 198, 192], [90, 88, 84]), shape: 2, tracks: &["Escada de Sal", "Cinza Claro", "Sem Pressa"], fmt: Fmt::Flac { bits: 24, rate: 88_200 } },
    AlbumSpec { title: "Linha de Fuga", artist: "Kaito e Vê", genre: "Eletrônica", year: 2024, colors: ([16, 70, 48], [150, 240, 180]), shape: 1, tracks: &["Linha de Fuga", "Ponto Cego", "Radar", "Eco Verde"], fmt: Fmt::Flac { bits: 24, rate: 96_000 } },
    AlbumSpec { title: "Calor de Março", artist: "Nina Sabiá", genre: "MPB", year: 2025, colors: ([240, 160, 90], [210, 80, 70]), shape: 1, tracks: &["Quase Verão", "Calor de Março", "Sol de Quina", "Terra Firme", "Vento Sul"], fmt: Fmt::Flac { bits: 16, rate: 44_100 } },
    AlbumSpec { title: "Vidro Líquido", artist: "Kaito e Vê", genre: "Ambiente", year: 2023, colors: ([30, 100, 200], [110, 170, 240]), shape: 2, tracks: &["Respiração de Vidro", "Vidro Líquido", "Corrente"], fmt: Fmt::Lossy },
    AlbumSpec { title: "Papel de Seda", artist: "Ilha Branca", genre: "Folk", year: 2022, colors: ([246, 228, 216], [217, 119, 106]), shape: 0, tracks: &["Seda e Sal", "Papel de Seda", "Ilha"], fmt: Fmt::Flac { bits: 16, rate: 44_100 } },
    AlbumSpec { title: "Fita Magnética", artist: "Rádio Cobre", genre: "Experimental", year: 2019, colors: ([43, 33, 24], [233, 184, 106]), shape: 1, tracks: &["Fita Magnética", "Cobre", "Loop de Domingo"], fmt: Fmt::Wav { bits: 16, rate: 44_100 } },
];

/// Cria (se faltar) a pasta `dir/music` com os álbuns de demonstração e devolve o caminho dela.
/// Se a pasta já tiver conteúdo, não regera.
pub fn ensure_demo_music(dir: &Path) -> Result<PathBuf, DemoError> {
    let music = dir.join("music");
    let marker = music.join(".kazzu-demo");
    if marker.exists() {
        return Ok(music);
    }
    std::fs::create_dir_all(&music).map_err(|e| DemoError::Io(music.clone(), e))?;
    for (i, album) in ALBUMS.iter().enumerate() {
        write_album(&music, i, album)?;
    }
    std::fs::write(&marker, b"gerado por kazzu --demo-library\n").map_err(|e| DemoError::Io(marker.clone(), e))?;
    Ok(music)
}

fn write_album(music: &Path, index: usize, a: &AlbumSpec) -> Result<(), DemoError> {
    let dir = music.join(format!("{} - {}", a.artist, a.title));
    std::fs::create_dir_all(&dir).map_err(|e| DemoError::Io(dir.clone(), e))?;
    cover(a).save(dir.join("cover.jpg"))?;
    for (n, title) in a.tracks.iter().enumerate() {
        let seed = (index * 16 + n) as u64 + 1;
        let secs = 14 + (seed * 7 % 19) as u32;
        let base = format!("{:02} {}", n + 1, title);
        let (path, tag_type) = match a.fmt {
            Fmt::Flac { bits, rate } => {
                let p = dir.join(format!("{base}.flac"));
                write_flac(&p, rate, bits, secs, seed)?;
                (p, TagType::VorbisComments)
            }
            Fmt::Wav { bits, rate } => {
                let p = dir.join(format!("{base}.wav"));
                write_wav(&p, rate, bits, secs, seed)?;
                (p, TagType::Id3v2)
            }
            Fmt::Lossy => {
                let wav = dir.join(format!("{base}.wav"));
                write_wav(&wav, 44_100, 16, secs, seed)?;
                match lossy_copy(&wav, &dir, &base, n, a, title) {
                    Some(p) => {
                        let _ = std::fs::remove_file(&wav);
                        tag_done(&p);
                        continue;
                    }
                    None => (wav, TagType::Id3v2),
                }
            }
        };
        write_tags(&path, tag_type, a, title, n as u32 + 1)?;
    }
    Ok(())
}

fn tag_done(_: &Path) {}

/// MP3 320 / Opus 160 via `ffmpeg`, se estiver instalado. As tags vão na própria conversão.
fn lossy_copy(wav: &Path, dir: &Path, base: &str, n: usize, a: &AlbumSpec, title: &str) -> Option<PathBuf> {
    let (ext, args): (&str, &[&str]) = if n == 1 { ("opus", &["-c:a", "libopus", "-b:a", "160k"]) } else { ("mp3", &["-c:a", "libmp3lame", "-b:a", "320k"]) };
    let out = dir.join(format!("{base}.{ext}"));
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"])
        .arg(wav)
        .args(args)
        .arg("-metadata")
        .arg(format!("title={title}"))
        .arg("-metadata")
        .arg(format!("artist={}", a.artist))
        .arg("-metadata")
        .arg(format!("album={}", a.title))
        .arg("-metadata")
        .arg(format!("genre={}", a.genre))
        .arg("-metadata")
        .arg(format!("date={}", a.year))
        .arg("-metadata")
        .arg(format!("track={}", n + 1))
        .arg(&out)
        .status()
        .ok()?;
    status.success().then_some(out)
}

fn write_tags(path: &Path, tag_type: TagType, a: &AlbumSpec, title: &str, track: u32) -> Result<(), DemoError> {
    let mut tag = Tag::new(tag_type);
    tag.set_title(title.into());
    tag.set_artist(a.artist.into());
    tag.set_album(a.title.into());
    tag.set_genre(a.genre.into());
    tag.set_track(track);
    tag.insert_text(ItemKey::AlbumArtist, a.artist.into());
    tag.insert_text(ItemKey::RecordingDate, a.year.to_string());
    tag.save_to_path(path, WriteOptions::default()).map_err(|e| DemoError::Tags(e.to_string()))
}

/// Melodia pentatônica com envelope, baixo e leve pan: soa como "música", sem ser de ninguém.
fn synth(frames: usize, rate: u32, seed: u64) -> Vec<(f32, f32)> {
    const SCALE: [f32; 8] = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0];
    let root = 110.0 * 2f32.powf((seed % 7) as f32 / 12.0);
    let note_len = (rate as f32 * 0.375) as usize;
    let tau = std::f32::consts::TAU;
    let mut out = Vec::with_capacity(frames);
    for n in 0..frames {
        let step = n / note_len;
        let t_in = (n % note_len) as f32 / rate as f32;
        let idx = ((step as u64 * 2654435761 + seed * 40503) >> 7) as usize % SCALE.len();
        let f = root * 2f32.powf(SCALE[idx] / 12.0) * 2.0;
        let env = (-t_in * 5.0).exp() * (1.0 - (-t_in * 200.0).exp());
        let lead = (tau * f * n as f32 / rate as f32).sin() * 0.28 * env + (tau * 2.0 * f * n as f32 / rate as f32).sin() * 0.06 * env;
        let bass = (tau * root * 0.5 * n as f32 / rate as f32).sin() * 0.16 * (0.6 + 0.4 * (step.is_multiple_of(4) as u8 as f32));
        let fade = ((n as f32 / rate as f32) / 0.05).min(1.0) * (((frames - n) as f32 / rate as f32) / 0.4).min(1.0);
        let pan = 0.5 + 0.35 * (tau * n as f32 / (rate as f32 * 6.0)).sin();
        out.push(((lead * (1.0 - pan) + bass) * fade, (lead * pan + bass) * fade));
    }
    out
}

fn to_int(x: f32, bits: u16) -> i32 {
    let max = (1i64 << (bits - 1)) - 1;
    (f64::from(x.clamp(-1.0, 1.0)) * max as f64).round() as i32
}

fn write_wav(path: &Path, rate: u32, bits: u16, secs: u32, seed: u64) -> Result<(), DemoError> {
    let spec = hound::WavSpec { channels: 2, sample_rate: rate, bits_per_sample: bits, sample_format: hound::SampleFormat::Int };
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| DemoError::Audio(e.to_string()))?;
    for (l, r) in synth((rate * secs) as usize, rate, seed) {
        w.write_sample(to_int(l, bits)).map_err(|e| DemoError::Audio(e.to_string()))?;
        w.write_sample(to_int(r, bits)).map_err(|e| DemoError::Audio(e.to_string()))?;
    }
    w.finalize().map_err(|e| DemoError::Audio(e.to_string()))
}

/// flacenc 0.5 só gera arquivos que o symphonia aceita com comprimento múltiplo de 4096 quadros.
fn write_flac(path: &Path, rate: u32, bits: u16, secs: u32, seed: u64) -> Result<(), DemoError> {
    use flacenc::component::BitRepr;
    use flacenc::error::Verify;
    let frames = (rate * secs) as usize / 4096 * 4096;
    let samples: Vec<i32> = synth(frames, rate, seed).into_iter().flat_map(|(l, r)| [to_int(l, bits), to_int(r, bits)]).collect();
    let err = |e: &dyn std::fmt::Debug| DemoError::Audio(format!("{e:?}"));
    let config = flacenc::config::Encoder::default().into_verified().map_err(|(_, e)| err(&e))?;
    let source = flacenc::source::MemSource::from_samples(&samples, 2, usize::from(bits), rate as usize);
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size).map_err(|e| err(&e))?;
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).map_err(|e| err(&e))?;
    std::fs::write(path, sink.as_slice()).map_err(|e| DemoError::Io(path.to_path_buf(), e))
}

/// Capa abstrata 600x600: degradê diagonal, mais uma forma (sol, ondas ou barras).
fn cover(a: &AlbumSpec) -> RgbImage {
    const N: u32 = 600;
    let (c1, c2) = a.colors;
    let mut img = RgbImage::new(N, N);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let t = ((x + y) as f32 / (2.0 * N as f32)).clamp(0.0, 1.0);
        let mut px = [0u8; 3];
        for k in 0..3 {
            px[k] = (f32::from(c1[k]) * (1.0 - t) + f32::from(c2[k]) * t) as u8;
        }
        let (fx, fy) = (x as f32, y as f32);
        let hit = match a.shape {
            // sol
            0 => (fx - 400.0).hypot(fy - 210.0) < 80.0,
            // arcos concêntricos na base
            1 => {
                let d = (fx - 300.0).hypot(fy - 620.0);
                d < 380.0 && ((d / 55.0) as u32).is_multiple_of(2)
            }
            // ondas
            _ => (fy - (380.0 + 40.0 * (fx / 60.0).sin() + 70.0 * ((fy - 380.0) / 70.0).floor())).abs() < 10.0 && fy > 330.0,
        };
        if hit {
            let [r, g, b] = c2;
            px = [r.saturating_add(30), g.saturating_add(30), b.saturating_add(30)];
        }
        *p = Rgb(px);
    }
    img
}

/// Lê de volta as tags de um arquivo gerado (usado nos testes).
pub fn read_title(path: &Path) -> Option<String> {
    let f = lofty::read_from_path(path).ok()?;
    f.primary_tag().or_else(|| f.first_tag())?.title().map(|t| t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_demo_library_is_scannable() {
        let dir = tempfile::tempdir().unwrap();
        let music = ensure_demo_music(dir.path()).unwrap();
        let mut lib = kazzu_library::Library::open(dir.path().join("data")).unwrap();
        lib.add_folder(&music).unwrap();
        let report = lib.scan(&|_| {}).unwrap();
        assert!(report.failed.is_empty(), "falhas: {:?}", report.failed);
        let total: usize = ALBUMS.iter().map(|a| a.tracks.len()).sum();
        assert_eq!(lib.track_count().unwrap() as usize, total);
        assert_eq!(lib.albums().unwrap().len(), ALBUMS.len());
        assert!(lib.albums().unwrap().iter().all(|a| a.artwork.is_some()), "toda capa de pasta deve ser encontrada");
        // segunda chamada reaproveita
        assert_eq!(ensure_demo_music(dir.path()).unwrap(), music);
    }
}
