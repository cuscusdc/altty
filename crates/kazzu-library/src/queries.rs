//! Consultas: faixas, álbuns, artistas, busca, playlists, curtidos, histórico.

use std::path::PathBuf;

use rusqlite::{Row, params};

use crate::artwork;
use crate::error::{Error, Result};
use crate::model::*;
use crate::{Library, now};

const TRACK_SELECT: &str = "SELECT t.id, t.path, t.title, t.album_id, al.title, al.artist, t.year,
    t.track_no, t.disc_no, t.genre, t.duration_ms, t.sample_rate, t.bit_depth, t.bitrate,
    t.channels, t.codec, t.added_at
    FROM tracks t LEFT JOIN albums al ON al.id = t.album_id";

const ALBUM_SELECT: &str = "SELECT al.id, al.title, al.artist, al.year, al.art_key, al.accent,
    COUNT(t.id), COALESCE(SUM(t.duration_ms), 0)
    FROM albums al LEFT JOIN tracks t ON t.album_id = al.id";

const ALBUM_ORDER: &str = "GROUP BY al.id ORDER BY al.artist COLLATE NOCASE,
    al.year, al.title COLLATE NOCASE";

fn row_to_track(r: &Row<'_>) -> rusqlite::Result<Track> {
    Ok(Track {
        id: TrackId(r.get(0)?),
        path: PathBuf::from(r.get::<_, String>(1)?),
        title: r.get(2)?,
        artists: Vec::new(),
        album_id: r.get::<_, Option<i64>>(3)?.map(AlbumId),
        album: r.get(4)?,
        album_artist: r.get(5)?,
        year: r.get(6)?,
        track_no: r.get(7)?,
        disc_no: r.get(8)?,
        genre: r.get(9)?,
        duration_ms: r.get::<_, i64>(10)? as u64,
        sample_rate: r.get(11)?,
        bit_depth: r.get(12)?,
        bitrate: r.get(13)?,
        channels: r.get(14)?,
        codec: r.get(15)?,
        added_at: r.get(16)?,
    })
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// `foo bar` → `"foo"* "bar"*` (prefixos; sem sintaxe FTS vinda do usuário).
fn fts_query(q: &str) -> Option<String> {
    let toks: Vec<String> = q
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{t}\"*"))
        .collect();
    (!toks.is_empty()).then(|| toks.join(" "))
}

impl Library {
    fn hydrate(&self, mut tracks: Vec<Track>) -> Result<Vec<Track>> {
        let mut st = self.conn.prepare_cached(
            "SELECT a.name FROM track_artists ta JOIN artists a ON a.id = ta.artist_id
             WHERE ta.track_id = ?1 ORDER BY ta.position",
        )?;
        for t in &mut tracks {
            t.artists = st
                .query_map([t.id.0], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<_, _>>()?;
        }
        Ok(tracks)
    }

    fn tracks_query(&self, sql_tail: &str, args: impl rusqlite::Params) -> Result<Vec<Track>> {
        let mut st = self.conn.prepare(&format!("{TRACK_SELECT} {sql_tail}"))?;
        let rows = st
            .query_map(args, row_to_track)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(st);
        self.hydrate(rows)
    }

    fn albums_query(&self, where_clause: &str, args: impl rusqlite::Params) -> Result<Vec<Album>> {
        let mut st = self
            .conn
            .prepare(&format!("{ALBUM_SELECT} {where_clause} {ALBUM_ORDER}"))?;
        let rows = st
            .query_map(args, |r| {
                let key: Option<String> = r.get(4)?;
                let accent: Option<i64> = r.get(5)?;
                Ok(Album {
                    id: AlbumId(r.get(0)?),
                    title: r.get(1)?,
                    artist: r.get(2)?,
                    year: r.get(3)?,
                    track_count: r.get(6)?,
                    duration_ms: r.get::<_, i64>(7)? as u64,
                    artwork: match (key, accent) {
                        (Some(k), Some(a)) if !k.is_empty() => {
                            Some(artwork::paths_for(&self.cache_dir, &k, Rgb::unpack(a)))
                        }
                        _ => None,
                    },
                })
            })?
            .collect::<std::result::Result<_, _>>()?;
        Ok(rows)
    }

    // ---- faixas ----------------------------------------------------------

    /// Todas as faixas (artista, álbum, disco, faixa).
    pub fn tracks(&self) -> Result<Vec<Track>> {
        self.tracks_query("ORDER BY t.title COLLATE NOCASE", [])
    }

    pub fn track(&self, id: TrackId) -> Result<Option<Track>> {
        Ok(self
            .tracks_query("WHERE t.id = ?1", [id.0])?
            .into_iter()
            .next())
    }

    pub fn track_by_path(&self, path: &std::path::Path) -> Result<Option<Track>> {
        let p = path.to_string_lossy().into_owned();
        Ok(self
            .tracks_query("WHERE t.path = ?1", [p])?
            .into_iter()
            .next())
    }

    pub fn track_count(&self) -> Result<u64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get::<_, i64>(0))?
            as u64)
    }

    /// Faixas adicionadas por último.
    pub fn recently_added(&self, limit: usize) -> Result<Vec<Track>> {
        self.tracks_query(
            "ORDER BY t.added_at DESC, t.id DESC LIMIT ?1",
            [limit as i64],
        )
    }

    // ---- álbuns / artistas ----------------------------------------------

    pub fn albums(&self) -> Result<Vec<Album>> {
        self.albums_query("", [])
    }

    pub fn album(&self, id: AlbumId) -> Result<Option<Album>> {
        Ok(self
            .albums_query("WHERE al.id = ?1", [id.0])?
            .into_iter()
            .next())
    }

    /// Faixas do álbum na ordem de disco/faixa.
    pub fn album_tracks(&self, id: AlbumId) -> Result<Vec<Track>> {
        self.tracks_query(
            "WHERE t.album_id = ?1 ORDER BY COALESCE(t.disc_no, 1), COALESCE(t.track_no, 0), t.path",
            [id.0],
        )
    }

    pub fn artists(&self) -> Result<Vec<Artist>> {
        let mut st = self.conn.prepare(
            "SELECT a.id, a.name, COUNT(ta.track_id) FROM artists a
             LEFT JOIN track_artists ta ON ta.artist_id = a.id
             GROUP BY a.id ORDER BY a.name COLLATE NOCASE",
        )?;
        let v = st
            .query_map([], |r| {
                Ok(Artist {
                    id: ArtistId(r.get(0)?),
                    name: r.get(1)?,
                    track_count: r.get(2)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?;
        Ok(v)
    }

    pub fn artist_tracks(&self, id: ArtistId) -> Result<Vec<Track>> {
        self.tracks_query(
            "WHERE t.id IN (SELECT track_id FROM track_artists WHERE artist_id = ?1)
             ORDER BY al.title COLLATE NOCASE, COALESCE(t.disc_no,1), COALESCE(t.track_no,0)",
            [id.0],
        )
    }

    pub fn artist_albums(&self, id: ArtistId) -> Result<Vec<Album>> {
        self.albums_query(
            "WHERE al.id IN (SELECT t.album_id FROM tracks t JOIN track_artists ta ON ta.track_id = t.id
                             WHERE ta.artist_id = ?1)",
            [id.0],
        )
    }

    // ---- busca -----------------------------------------------------------

    /// Busca por prefixo de palavras (FTS5) em título/artista/álbum das faixas,
    /// mais álbuns e artistas por substring. Insensível a acentos e caixa
    /// (nas faixas); álbuns/artistas: caixa só para ASCII.
    pub fn search(&self, query: &str, limit: usize) -> Result<SearchResults> {
        let Some(fts) = fts_query(query) else {
            return Ok(SearchResults::default());
        };
        let tracks = self.tracks_query(
            "JOIN tracks_fts f ON f.rowid = t.id WHERE tracks_fts MATCH ?1
             ORDER BY f.rank LIMIT ?2",
            params![fts, limit as i64],
        )?;
        let like = format!("%{}%", escape_like(query.trim()));
        let albums = {
            let mut a = self.albums_query(
                "WHERE al.title LIKE ?1 ESCAPE '\\' OR al.artist LIKE ?1 ESCAPE '\\'",
                [&like],
            )?;
            a.truncate(limit);
            a
        };
        let artists = {
            let mut st = self.conn.prepare(
                "SELECT a.id, a.name, COUNT(ta.track_id) FROM artists a
                 LEFT JOIN track_artists ta ON ta.artist_id = a.id
                 WHERE a.name LIKE ?1 ESCAPE '\\' GROUP BY a.id
                 ORDER BY a.name COLLATE NOCASE LIMIT ?2",
            )?;
            st.query_map(params![like, limit as i64], |r| {
                Ok(Artist {
                    id: ArtistId(r.get(0)?),
                    name: r.get(1)?,
                    track_count: r.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(SearchResults {
            tracks,
            albums,
            artists,
        })
    }

    // ---- curtidos --------------------------------------------------------

    pub fn like(&self, id: TrackId) -> Result<()> {
        self.require_track(id)?;
        self.conn.execute(
            "INSERT OR IGNORE INTO likes(track_id, liked_at) VALUES (?1, ?2)",
            params![id.0, now()],
        )?;
        Ok(())
    }

    pub fn unlike(&self, id: TrackId) -> Result<()> {
        self.conn
            .execute("DELETE FROM likes WHERE track_id = ?1", [id.0])?;
        Ok(())
    }

    pub fn is_liked(&self, id: TrackId) -> Result<bool> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM likes WHERE track_id = ?1",
                [id.0],
                |_| Ok(()),
            )
            .is_ok())
    }

    /// Curtidas, mais recentes primeiro.
    pub fn liked(&self) -> Result<Vec<Track>> {
        self.tracks_query(
            "JOIN likes l ON l.track_id = t.id ORDER BY l.liked_at DESC, t.id DESC",
            [],
        )
    }

    fn require_track(&self, id: TrackId) -> Result<()> {
        self.conn
            .query_row("SELECT 1 FROM tracks WHERE id = ?1", [id.0], |_| Ok(()))
            .map_err(|_| Error::NotFound("faixa"))
    }

    // ---- histórico (somente local) --------------------------------------

    /// Registra uma reprodução (apenas no banco local).
    pub fn record_play(&self, id: TrackId, listened_ms: u64) -> Result<()> {
        self.record_play_at(id, listened_ms, now())
    }

    /// Como [`record_play`](Self::record_play) com instante explícito (Unix s).
    pub fn record_play_at(&self, id: TrackId, listened_ms: u64, played_at: i64) -> Result<()> {
        self.require_track(id)?;
        self.conn.execute(
            "INSERT INTO play_history(track_id, played_at, listened_ms) VALUES (?1,?2,?3)",
            params![id.0, played_at, listened_ms as i64],
        )?;
        Ok(())
    }

    /// Reproduções, mais recentes primeiro.
    pub fn history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        self.history_range(i64::MIN, i64::MAX, limit)
    }

    /// Reproduções com `from <= played_at < to` (base da retrospectiva).
    pub fn history_range(&self, from: i64, to: i64, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut st = self.conn.prepare(
            "SELECT id, track_id, played_at, listened_ms FROM play_history
             WHERE played_at >= ?1 AND played_at < ?2 ORDER BY played_at DESC, id DESC LIMIT ?3",
        )?;
        let rows: Vec<(i64, i64, i64)> = st
            .query_map(params![from, to, limit as i64], |r| {
                Ok((r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<std::result::Result<_, _>>()?;
        let mut out = Vec::with_capacity(rows.len());
        for (tid, played_at, ms) in rows {
            if let Some(track) = self.track(TrackId(tid))? {
                out.push(HistoryEntry {
                    track,
                    played_at,
                    listened_ms: ms as u64,
                });
            }
        }
        Ok(out)
    }

    /// Faixas tocadas há pouco, sem repetição, mais recente primeiro.
    pub fn recent(&self, limit: usize) -> Result<Vec<Track>> {
        let ids: Vec<i64> = {
            let mut st = self.conn.prepare(
                "SELECT track_id FROM play_history GROUP BY track_id
                 ORDER BY MAX(played_at) DESC, MAX(id) DESC LIMIT ?1",
            )?;
            st.query_map([limit as i64], |r| r.get(0))?
                .collect::<std::result::Result<_, _>>()?
        };
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(t) = self.track(TrackId(id))? {
                out.push(t);
            }
        }
        Ok(out)
    }

    /// Apaga todo o histórico de reprodução.
    pub fn clear_history(&self) -> Result<()> {
        self.conn.execute("DELETE FROM play_history", [])?;
        Ok(())
    }

    // ---- playlists -------------------------------------------------------

    pub fn create_playlist(&self, name: &str) -> Result<PlaylistId> {
        let t = now();
        self.conn.execute(
            "INSERT INTO playlists(name, created_at, updated_at) VALUES (?1,?2,?2)",
            params![name.trim(), t],
        )?;
        Ok(PlaylistId(self.conn.last_insert_rowid()))
    }

    pub fn rename_playlist(&self, id: PlaylistId, name: &str) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id.0, name.trim(), now()],
        )?;
        if n == 0 {
            Err(Error::NotFound("playlist"))
        } else {
            Ok(())
        }
    }

    pub fn delete_playlist(&self, id: PlaylistId) -> Result<()> {
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", [id.0])?;
        Ok(())
    }

    pub fn playlists(&self) -> Result<Vec<Playlist>> {
        let mut st = self.conn.prepare(
            "SELECT p.id, p.name, COUNT(i.id), p.created_at, p.updated_at FROM playlists p
             LEFT JOIN playlist_items i ON i.playlist_id = p.id
             GROUP BY p.id ORDER BY p.name COLLATE NOCASE",
        )?;
        let v = st
            .query_map([], |r| {
                Ok(Playlist {
                    id: PlaylistId(r.get(0)?),
                    name: r.get(1)?,
                    track_count: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?;
        Ok(v)
    }

    /// Faixas da playlist na ordem definida pelo usuário (repetições permitidas).
    pub fn playlist_tracks(&self, id: PlaylistId) -> Result<Vec<Track>> {
        let ids = self.playlist_item_ids(id)?;
        let mut out = Vec::with_capacity(ids.len());
        for (_, tid) in ids {
            if let Some(t) = self.track(TrackId(tid))? {
                out.push(t);
            }
        }
        Ok(out)
    }

    /// `(item_id, track_id)` em ordem.
    fn playlist_item_ids(&self, id: PlaylistId) -> Result<Vec<(i64, i64)>> {
        let mut st = self.conn.prepare(
            "SELECT id, track_id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position, id",
        )?;
        Ok(st
            .query_map([id.0], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<std::result::Result<_, _>>()?)
    }

    fn require_playlist(&self, id: PlaylistId) -> Result<()> {
        self.conn
            .query_row("SELECT 1 FROM playlists WHERE id = ?1", [id.0], |_| Ok(()))
            .map_err(|_| Error::NotFound("playlist"))
    }

    fn touch_playlist(&self, id: PlaylistId) -> Result<()> {
        self.conn.execute(
            "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
            params![id.0, now()],
        )?;
        Ok(())
    }

    /// Acrescenta faixas ao fim.
    pub fn add_to_playlist(&mut self, id: PlaylistId, tracks: &[TrackId]) -> Result<()> {
        self.require_playlist(id)?;
        let tx = self.conn.transaction()?;
        let next: i64 = tx.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_items WHERE playlist_id = ?1",
            [id.0],
            |r| r.get(0),
        )?;
        let t = now();
        for (offset, tr) in tracks.iter().enumerate() {
            tx.query_row("SELECT 1 FROM tracks WHERE id = ?1", [tr.0], |_| Ok(()))
                .map_err(|_| Error::NotFound("faixa"))?;
            tx.execute(
                "INSERT INTO playlist_items(playlist_id, track_id, position, added_at) VALUES (?1,?2,?3,?4)",
                params![id.0, tr.0, next + offset as i64, t],
            )?;
        }
        tx.commit()?;
        self.touch_playlist(id)
    }

    /// Remove o item na posição `index` (0-based).
    pub fn remove_from_playlist(&mut self, id: PlaylistId, index: usize) -> Result<()> {
        let mut items = self.playlist_item_ids(id)?;
        if index >= items.len() {
            return Err(Error::InvalidPosition(index));
        }
        let (item, _) = items.remove(index);
        self.conn
            .execute("DELETE FROM playlist_items WHERE id = ?1", [item])?;
        self.rewrite_positions(&items)?;
        self.touch_playlist(id)
    }

    /// Move o item de `from` para `to` (índices 0-based, como `Vec::remove` + `insert`).
    pub fn reorder_playlist(&mut self, id: PlaylistId, from: usize, to: usize) -> Result<()> {
        let mut items = self.playlist_item_ids(id)?;
        if from >= items.len() {
            return Err(Error::InvalidPosition(from));
        }
        if to >= items.len() {
            return Err(Error::InvalidPosition(to));
        }
        let it = items.remove(from);
        items.insert(to, it);
        self.rewrite_positions(&items)?;
        self.touch_playlist(id)
    }

    fn rewrite_positions(&mut self, items: &[(i64, i64)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for (pos, (item, _)) in items.iter().enumerate() {
            tx.execute(
                "UPDATE playlist_items SET position = ?2 WHERE id = ?1",
                params![item, pos as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
