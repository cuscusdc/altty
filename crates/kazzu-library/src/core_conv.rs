//! Conversões para os tipos de domínio de `kazzu-core`.
//!
//! Ids do SQLite viram strings opacas (`"42"`).

use std::time::Duration;

use kazzu_core as core;

use crate::model::*;

macro_rules! id_conv {
    ($mine:ident, $core:ident) => {
        impl From<$mine> for core::$core {
            fn from(v: $mine) -> Self {
                core::$core::new(v.0.to_string())
            }
        }
        impl $mine {
            /// Converte um id do core de volta (`None` se não for um id desta biblioteca).
            pub fn from_core(id: &core::$core) -> Option<Self> {
                id.as_str().parse().ok().map($mine)
            }
        }
    };
}
id_conv!(TrackId, TrackId);
id_conv!(AlbumId, AlbumId);
id_conv!(ArtistId, ArtistId);
id_conv!(PlaylistId, PlaylistId);

fn core_codec(s: &str) -> core::Codec {
    match s {
        "FLAC" => core::Codec::Flac,
        "ALAC" => core::Codec::Alac,
        "WAV" | "AIFF" => core::Codec::Pcm,
        "MP3" => core::Codec::Mp3,
        "AAC" => core::Codec::Aac,
        "Vorbis" => core::Codec::Vorbis,
        "Opus" => core::Codec::Opus,
        other => core::Codec::Other(other.to_owned()),
    }
}

impl Track {
    /// Converte para o `Track` do core. `cover` costuma vir de `Album::artwork`.
    pub fn to_core(&self, cover: Option<&Artwork>) -> core::Track {
        let mut t = core::Track::new(
            core::TrackId::from(self.id),
            self.title.clone(),
            core::TrackSource::Local {
                path: self.path.clone(),
            },
        );
        t.artists = self.artists.iter().map(core::ArtistRef::named).collect();
        t.album = self.album.as_ref().map(|title| core::AlbumRef {
            id: self.album_id.map(core::AlbumId::from),
            title: title.clone(),
        });
        t.duration = Some(Duration::from_millis(self.duration_ms));
        t.track_number = self.track_no;
        t.disc_number = self.disc_no;
        t.format = Some(core::AudioFormat {
            codec: core_codec(&self.codec),
            sample_rate: self.sample_rate,
            bit_depth: self.bit_depth,
            channels: self.channels.map(u16::from),
            bitrate_kbps: self.bitrate,
        });
        t.cover = cover.map(|a| core::CoverRef::File {
            path: a.cover.clone(),
        });
        t
    }
}

impl Album {
    pub fn to_core(&self) -> core::Album {
        core::Album {
            id: self.id.into(),
            provider: core::ProviderKind::Local,
            title: self.title.clone(),
            artists: if self.artist.is_empty() {
                vec![]
            } else {
                vec![core::ArtistRef::named(&self.artist)]
            },
            year: self.year,
            track_count: Some(self.track_count),
            cover: self.artwork.as_ref().map(|a| core::CoverRef::File {
                path: a.cover.clone(),
            }),
        }
    }
}

impl Artist {
    pub fn to_core(&self) -> core::Artist {
        core::Artist {
            id: self.id.into(),
            provider: core::ProviderKind::Local,
            name: self.name.clone(),
            image: None,
            genres: Vec::new(),
            popularity: 0,
        }
    }
}
