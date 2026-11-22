//! Local library for YouTube Music items (no account): liked tracks and saved
//! albums, artists and playlists, kept in one JSON file on this device.
//!
//! Nothing is sent to YouTube. Sync between the user's devices is a Kazzu
//! feature (stage 4), not a YouTube one.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kazzu_core::{Album, AlbumId, Artist, ArtistId, LibrarySnapshot, Playlist, PlaylistId, Track, TrackId};
use serde::{Deserialize, Serialize};

use crate::error::{Result, YtmError};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Data {
    #[serde(default)]
    version: u32,
    /// Most recent first.
    #[serde(default)]
    liked: Vec<Track>,
    #[serde(default)]
    albums: Vec<Album>,
    #[serde(default)]
    artists: Vec<Artist>,
    #[serde(default)]
    playlists: Vec<Playlist>,
}

pub struct LocalLibrary {
    path: Option<PathBuf>,
    data: Mutex<Data>,
}

impl LocalLibrary {
    /// Library stored at `path` (loaded now if it exists).
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let data = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw)
                .map_err(|e| YtmError::Storage(format!("{}: {e}", path.display())))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Data::default(),
            Err(e) => return Err(e.into()),
        };
        Ok(Self {
            path: Some(path),
            data: Mutex::new(data),
        })
    }

    /// Library kept in memory only.
    pub fn in_memory() -> Self {
        Self {
            path: None,
            data: Mutex::new(Data::default()),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn with<R>(&self, change: impl FnOnce(&mut Data) -> R) -> Result<R> {
        let mut data = self.data.lock().unwrap_or_else(|p| p.into_inner());
        let before = data.clone();
        let out = change(&mut data);
        if *data != before
            && let Some(path) = &self.path
        {
            data.version = 1;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let tmp = path.with_extension("tmp");
            fs::write(&tmp, serde_json::to_vec_pretty(&*data).map_err(|e| YtmError::Storage(e.to_string()))?)?;
            fs::rename(&tmp, path)?;
        }
        Ok(out)
    }

    fn read<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        f(&self.data.lock().unwrap_or_else(|p| p.into_inner()))
    }

    /// Likes a track (moves it to the top if already liked).
    pub fn like(&self, track: &Track) -> Result<()> {
        self.with(|d| {
            d.liked.retain(|t| t.id != track.id);
            d.liked.insert(0, track.clone());
        })
    }

    /// Returns whether the track was liked.
    pub fn unlike(&self, id: &TrackId) -> Result<bool> {
        self.with(|d| {
            let before = d.liked.len();
            d.liked.retain(|t| &t.id != id);
            before != d.liked.len()
        })
    }

    pub fn is_liked(&self, id: &TrackId) -> bool {
        self.read(|d| d.liked.iter().any(|t| &t.id == id))
    }

    pub fn liked(&self) -> Vec<Track> {
        self.read(|d| d.liked.clone())
    }

    pub fn save_album(&self, album: &Album) -> Result<()> {
        self.with(|d| {
            d.albums.retain(|a| a.id != album.id);
            d.albums.insert(0, album.clone());
        })
    }

    pub fn remove_album(&self, id: &AlbumId) -> Result<bool> {
        self.with(|d| {
            let n = d.albums.len();
            d.albums.retain(|a| &a.id != id);
            n != d.albums.len()
        })
    }

    pub fn save_artist(&self, artist: &Artist) -> Result<()> {
        self.with(|d| {
            d.artists.retain(|a| a.id != artist.id);
            d.artists.insert(0, artist.clone());
        })
    }

    pub fn remove_artist(&self, id: &ArtistId) -> Result<bool> {
        self.with(|d| {
            let n = d.artists.len();
            d.artists.retain(|a| &a.id != id);
            n != d.artists.len()
        })
    }

    pub fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        self.with(|d| {
            d.playlists.retain(|p| p.id != playlist.id);
            d.playlists.insert(0, playlist.clone());
        })
    }

    pub fn remove_playlist(&self, id: &PlaylistId) -> Result<bool> {
        self.with(|d| {
            let n = d.playlists.len();
            d.playlists.retain(|p| &p.id != id);
            n != d.playlists.len()
        })
    }

    pub fn snapshot(&self) -> LibrarySnapshot {
        self.read(|d| LibrarySnapshot {
            albums: d.albums.clone(),
            artists: d.artists.clone(),
            playlists: d.playlists.clone(),
            liked: d.liked.iter().map(|t| t.id.clone()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazzu_core::{ProviderKind, TrackSource};

    fn track(id: &str) -> Track {
        Track::new(
            id,
            "t",
            TrackSource::Remote {
                provider: ProviderKind::YtMusic,
                id: id.into(),
            },
        )
    }

    #[test]
    fn likes_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lib/ytm.json");
        let lib = LocalLibrary::open(&path).unwrap();
        lib.like(&track("a")).unwrap();
        lib.like(&track("b")).unwrap();
        lib.like(&track("a")).unwrap();
        assert_eq!(lib.liked().iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        let again = LocalLibrary::open(&path).unwrap();
        assert!(again.is_liked(&TrackId::new("b")));
        assert!(again.unlike(&TrackId::new("b")).unwrap());
        assert!(!again.unlike(&TrackId::new("b")).unwrap());
        assert_eq!(again.snapshot().liked, vec![TrackId::new("a")]);
    }
}
