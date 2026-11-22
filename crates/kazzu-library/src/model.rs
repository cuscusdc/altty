//! Tipos públicos da biblioteca.
//!
//! Estruturas próprias da biblioteca local. Quando `kazzu-core` estabilizar seus
//! tipos de domínio, `Track`/`Album` daqui ganharão conversões (`From`) para eles.

use std::path::PathBuf;

macro_rules! id_type {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub i64);
    };
}

id_type!(/// Identificador de faixa no banco.
    TrackId);
id_type!(/// Identificador de álbum no banco.
    AlbumId);
id_type!(/// Identificador de artista no banco.
    ArtistId);
id_type!(/// Identificador de playlist no banco.
    PlaylistId);

/// Cor RGB (8 bits por canal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub(crate) fn pack(self) -> i64 {
        (i64::from(self.0) << 16) | (i64::from(self.1) << 8) | i64::from(self.2)
    }
    pub(crate) fn unpack(v: i64) -> Self {
        Rgb(
            ((v >> 16) & 0xff) as u8,
            ((v >> 8) & 0xff) as u8,
            (v & 0xff) as u8,
        )
    }
}

/// Arte de capa já processada e em cache no disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artwork {
    /// Capa redimensionada (JPEG, lado maior ≤ 640 px).
    pub cover: PathBuf,
    /// Miniatura (JPEG, lado maior ≤ 160 px) para listas e grades.
    pub thumb: PathBuf,
    /// Versão pequena PRÉ-DESFOCADA, para fundo glass (a UI deve esticá-la).
    pub blurred: PathBuf,
    /// Cor dominante, usada como accent.
    pub accent: Rgb,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub artists: Vec<String>,
    pub album_id: Option<AlbumId>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<i32>,
    pub track_no: Option<u32>,
    pub disc_no: Option<u32>,
    pub genre: Option<String>,
    pub duration_ms: u64,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    /// kbps.
    pub bitrate: Option<u32>,
    pub channels: Option<u8>,
    /// Ex.: "FLAC", "MP3", "Opus", "ALAC".
    pub codec: String,
    pub added_at: i64,
}

impl Track {
    /// "Artista 1, Artista 2".
    pub fn artist_display(&self) -> String {
        self.artists.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub artist: String,
    pub year: Option<i32>,
    pub track_count: u32,
    pub duration_ms: u64,
    pub artwork: Option<Artwork>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub track_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub track_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub track: Track,
    /// Unix epoch (s).
    pub played_at: i64,
    /// Quanto da faixa foi ouvido.
    pub listened_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

/// Progresso de um scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanProgress {
    /// Percorrendo as pastas.
    Discovering {
        files_found: usize,
    },
    /// Lendo tags/propriedades dos arquivos novos ou modificados.
    Reading {
        done: usize,
        total: usize,
    },
    /// Gerando capas, miniaturas, fundo desfocado e accent.
    Artwork {
        done: usize,
        total: usize,
    },
    Finished(ScanReport),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    pub scanned: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub removed: usize,
    /// Arquivos que não puderam ser lidos (caminho, motivo).
    pub failed: Vec<(PathBuf, String)>,
}
