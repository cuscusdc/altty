//! Descoberta de arquivos de áudio e leitura de tags/propriedades.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use lofty::config::{ParseOptions, ParsingMode};
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::mp4::{Mp4Codec, Mp4File};
use lofty::tag::{Accessor, ItemKey};
use walkdir::WalkDir;

pub const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "wav", "opus", "ogg", "oga", "mp3", "m4a", "aac", "aiff", "aif",
];

#[derive(Debug, Clone)]
pub(crate) struct FileEntry {
    pub path: PathBuf,
    /// Segundos desde a época Unix.
    pub mtime: i64,
    pub size: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedTrack {
    pub entry: FileEntry,
    pub title: String,
    pub artists: Vec<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub track_no: Option<u32>,
    pub disc_no: Option<u32>,
    pub genre: Option<String>,
    pub duration_ms: u64,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub codec: String,
    pub has_picture: bool,
}

pub(crate) fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| AUDIO_EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
}

/// Percorre a raiz e devolve os arquivos de áudio com mtime/tamanho.
pub(crate) fn discover(
    root: &Path,
    on_found: &mut dyn FnMut(usize),
    base: usize,
) -> Vec<FileEntry> {
    let mut out = Vec::new();
    for item in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !item.file_type().is_file() || !is_audio(item.path()) {
            continue;
        }
        let Ok(meta) = item.metadata() else { continue };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs() as i64);
        out.push(FileEntry {
            path: item.into_path(),
            mtime,
            size: meta.len() as i64,
        });
        if out.len() % 64 == 0 {
            on_found(base + out.len());
        }
    }
    out
}

fn codec_name(ft: FileType) -> &'static str {
    match ft {
        FileType::Aac => "AAC",
        FileType::Aiff => "AIFF",
        FileType::Ape => "APE",
        FileType::Flac => "FLAC",
        FileType::Mpeg => "MP3",
        FileType::Mp4 => "AAC",
        FileType::Mpc => "MPC",
        FileType::Opus => "Opus",
        FileType::Vorbis => "Vorbis",
        FileType::Speex => "Speex",
        FileType::Wav => "WAV",
        FileType::WavPack => "WavPack",
        _ => "Unknown",
    }
}

fn parse_year(s: &str) -> Option<i32> {
    let digits: String = s.trim().chars().take_while(char::is_ascii_digit).collect();
    if digits.len() == 4 {
        digits.parse().ok()
    } else {
        None
    }
}

fn clean(s: Option<impl AsRef<str>>) -> Option<String> {
    s.map(|s| s.as_ref().trim().to_owned())
        .filter(|s| !s.is_empty())
}

pub(crate) fn read_track(entry: &FileEntry) -> Result<ParsedTrack, String> {
    let opts = ParseOptions::new().parsing_mode(ParsingMode::Relaxed);
    let tagged = lofty::probe::Probe::open(&entry.path)
        .map_err(|e| e.to_string())?
        .options(opts)
        .guess_file_type()
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let props = tagged.properties();
    let file_type = tagged.file_type();
    let mut codec = codec_name(file_type).to_owned();
    if file_type == FileType::Mp4
        && let Ok(mut f) = File::open(&entry.path)
        && let Ok(mp4) = Mp4File::read_from(
            &mut f,
            ParseOptions::new().read_tags(false).read_cover_art(false),
        )
    {
        codec = match mp4.properties().codec() {
            Some(Mp4Codec::ALAC) => "ALAC",
            Some(Mp4Codec::MP3) => "MP3",
            Some(Mp4Codec::FLAC) => "FLAC",
            _ => "AAC",
        }
        .to_owned();
    }

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let stem = entry
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_owned();

    let (title, artists, album_artist, album, year, track_no, disc_no, genre, has_picture) =
        match tag {
            Some(t) => {
                let artists: Vec<String> = t
                    .get_strings(ItemKey::TrackArtist)
                    .filter_map(|s| clean(Some(s)))
                    .collect();
                (
                    clean(t.title()).unwrap_or_else(|| stem.clone()),
                    artists,
                    clean(t.get_string(ItemKey::AlbumArtist)),
                    clean(t.album()),
                    t.date()
                        .map(|d| i32::from(d.year))
                        .or_else(|| t.get_string(ItemKey::Year).and_then(parse_year)),
                    t.track(),
                    t.disk(),
                    clean(t.genre()),
                    !t.pictures().is_empty(),
                )
            }
            None => (stem, vec![], None, None, None, None, None, None, false),
        };

    Ok(ParsedTrack {
        entry: entry.clone(),
        title,
        artists,
        album_artist,
        album,
        year,
        track_no,
        disc_no,
        genre,
        duration_ms: props.duration().as_millis() as u64,
        sample_rate: props.sample_rate(),
        bit_depth: props.bit_depth(),
        bitrate: props.audio_bitrate().or(props.overall_bitrate()),
        channels: props.channels(),
        codec,
        has_picture,
    })
}

/// Lê a primeira imagem embutida (preferindo "capa frontal").
pub(crate) fn read_embedded_picture(path: &Path) -> Option<Vec<u8>> {
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let pics = tag.pictures();
    let front = pics
        .iter()
        .find(|p| p.pic_type() == lofty::picture::PictureType::CoverFront);
    front.or_else(|| pics.first()).map(|p| p.data().to_vec())
}

/// Procura cover/folder/front/album .jpg/.png/.webp na pasta.
pub(crate) fn find_folder_cover(dir: &Path) -> Option<PathBuf> {
    const NAMES: &[&str] = &["cover", "folder", "front", "album", "albumart"];
    const EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];
    let mut best: Option<(usize, PathBuf)> = None;
    for e in std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok()) {
        let p = e.path();
        let (Some(stem), Some(ext)) = (
            p.file_stem().and_then(|s| s.to_str()),
            p.extension().and_then(|s| s.to_str()),
        ) else {
            continue;
        };
        if !EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
            continue;
        }
        if let Some(rank) = NAMES.iter().position(|n| n.eq_ignore_ascii_case(stem))
            && best.as_ref().is_none_or(|(r, _)| rank < *r)
        {
            best = Some((rank, p));
        }
    }
    best.map(|(_, p)| p)
}
