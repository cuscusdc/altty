//! Consultas: faixas, álbuns, artistas, busca, playlists, curtidos, histórico.

use std::path::PathBuf;

use rusqlite::{OptionalExtension, Row, params};

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


/// Chave textual do provedor (nome serde, ex.: `"yt_music"`).
fn provider_key(p: kazzu_core::ProviderKind) -> Result<String> {
    match serde_json::to_value(p) {
        Ok(serde_json::Value::String(s)) => Ok(s),
        _ => Err(Error::InvalidRemote("provedor não serializável".into())),
    }
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

/// Resultado da leitura de um item de playlist.
enum ScannedKind {
    /// Faixa local existente (id da faixa).
    Local(i64),
    /// Faixa remota com JSON válido.
    Remote(Box<kazzu_core::Track>),
    /// Órfão ou inválido: preservado no banco, fora dos índices visíveis.
    Hidden,
}

struct ScannedItem {
    item_id: i64,
    kind: ScannedKind,
}

/// Lê todos os itens da playlist em ordem (`position, id`) e classifica cada
/// um como visível ou oculto. É a única fonte da noção de "índice unificado":
/// listar, remover e reordenar usam esta função (na mesma transação, quando
/// há escrita).
fn scan_playlist_items(conn: &rusqlite::Connection, id: PlaylistId) -> Result<Vec<ScannedItem>> {
    let mut st = conn.prepare(
        "SELECT i.id, i.track_id, i.remote_json, t.id IS NOT NULL
         FROM playlist_items i LEFT JOIN tracks t ON t.id = i.track_id
         WHERE i.playlist_id = ?1 ORDER BY i.position, i.id",
    )?;
    let rows: Vec<(i64, Option<i64>, Option<String>, bool)> = st
        .query_map([id.0], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<std::result::Result<_, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (item_id, tid, json, track_exists) in rows {
        let kind = match (tid, json) {
            (Some(tid), _) if track_exists => ScannedKind::Local(tid),
            (Some(_), _) => {
                log::warn!("playlist {}: item {item_id} local órfão ignorado", id.0);
                ScannedKind::Hidden
            }
            (None, Some(json)) => match serde_json::from_str(&json) {
                Ok(t) => ScannedKind::Remote(Box::new(t)),
                Err(e) => {
                    log::warn!(
                        "playlist {}: item {item_id} com remote_json inválido ignorado: {e}",
                        id.0
                    );
                    ScannedKind::Hidden
                }
            },
            (None, None) => ScannedKind::Hidden,
        };
        out.push(ScannedItem { item_id, kind });
    }
    Ok(out)
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

    /// Minutos e número de reproduções registrados no histórico local.
    pub fn listening_totals(&self) -> Result<(u64, u64)> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(SUM(listened_ms), 0) / 60000, COUNT(*) FROM play_history",
            [], |row| Ok((row.get::<_, i64>(0)?.max(0) as u64, row.get::<_, i64>(1)?.max(0) as u64)),
        )?)
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

    /// Faixas **locais** da playlist na ordem definida pelo usuário
    /// (repetições permitidas). Entradas remotas são ignoradas aqui; use
    /// [`playlist_entries`](Self::playlist_entries) para a lista unificada.
    pub fn playlist_tracks(&self, id: PlaylistId) -> Result<Vec<Track>> {
        let mut out = Vec::new();
        for it in scan_playlist_items(&self.conn, id)? {
            if let ScannedKind::Local(tid) = it.kind
                && let Some(t) = self.track(TrackId(tid))?
            {
                out.push(t);
            }
        }
        Ok(out)
    }

    /// Lista unificada (locais + remotas) na ordem de `position`.
    ///
    /// Esta é a **definição única de índice** usada também por
    /// [`remove_from_playlist`](Self::remove_from_playlist) e
    /// [`reorder_playlist`](Self::reorder_playlist). Itens inválidos (JSON
    /// remoto corrompido) e órfãos (faixa local inexistente) ficam de fora,
    /// com `log::warn!`; permanecem no banco e não entram na contagem de índices.
    pub fn playlist_entries(&self, id: PlaylistId) -> Result<Vec<PlaylistEntry>> {
        let mut out = Vec::new();
        for it in scan_playlist_items(&self.conn, id)? {
            match it.kind {
                ScannedKind::Local(tid) => {
                    if let Some(t) = self.track(TrackId(tid))? {
                        out.push(PlaylistEntry::Local(t));
                    }
                }
                ScannedKind::Remote(t) => out.push(PlaylistEntry::Remote(*t)),
                ScannedKind::Hidden => {}
            }
        }
        Ok(out)
    }

    /// Acrescenta faixas remotas ao fim da playlist, guardando os metadados
    /// serializados (exibíveis offline). Faixas que não são
    /// `TrackSource::Remote` são ignoradas, e faixas já presentes na playlist
    /// (mesmo `(provider, id)`) ou repetidas em `tracks` são puladas, sem erro.
    pub fn add_remote_to_playlist(
        &mut self,
        id: PlaylistId,
        tracks: &[kazzu_core::Track],
    ) -> Result<()> {
        self.require_playlist(id)?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        insert_remote_tracks(&tx, id, tracks)?;
        tx.commit()?;
        self.touch_playlist(id)
    }

    /// Imports by stable service identifier. Re-import adds missing entries
    /// without deleting tracks added locally to the playlist.
    pub fn import_remote_playlist(&mut self, key: &str, name: &str, tracks: &[kazzu_core::Track]) -> Result<()> {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute_batch("CREATE TABLE IF NOT EXISTS imported_playlists (
            source_key TEXT PRIMARY KEY,
            playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE
        )")?;
        let existing = tx.query_row("SELECT playlist_id FROM imported_playlists WHERE source_key = ?1", [key], |row| row.get::<_, i64>(0)).optional()?;
        let id = if let Some(id) = existing {
            PlaylistId(id)
        } else {
            tx.execute("INSERT INTO playlists(name, created_at, updated_at) VALUES (?1, ?2, ?2)", params![name.trim(), now()])?;
            let id = PlaylistId(tx.last_insert_rowid());
            tx.execute("INSERT INTO imported_playlists(source_key, playlist_id) VALUES (?1, ?2)", params![key, id.0])?;
            id
        };
        insert_remote_tracks(&tx, id, tracks)?;
        tx.execute("UPDATE playlists SET updated_at = ?2 WHERE id = ?1", params![id.0, now()])?;
        tx.commit()?;
        Ok(())
    }

    /// Se a playlist já contém a faixa remota `(provider, remote_id)`;
    /// `provider` é o nome serde do `ProviderKind` (ex.: `"yt_music"`).
    pub fn playlist_contains_remote(
        &self,
        id: PlaylistId,
        provider: &str,
        remote_id: &str,
    ) -> Result<bool> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM playlist_items
                 WHERE playlist_id = ?1 AND remote_provider = ?2 AND remote_id = ?3",
                params![id.0, provider, remote_id],
                |_| Ok(()),
            )
            .is_ok())
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
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
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

    /// Remove o item na posição `index` (0-based) da lista unificada
    /// ([`playlist_entries`](Self::playlist_entries)): só itens visíveis contam.
    /// Itens ocultos (órfãos/inválidos) são preservados. Delete e renumeração
    /// ocorrem numa única transação; um índice inválido não altera nada.
    pub fn remove_from_playlist(&mut self, id: PlaylistId, index: usize) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let items = scan_playlist_items(&tx, id)?;
        let visible: Vec<usize> = (0..items.len())
            .filter(|&i| !matches!(items[i].kind, ScannedKind::Hidden))
            .collect();
        let Some(&slot) = visible.get(index) else {
            return Err(Error::InvalidPosition(index));
        };
        tx.execute(
            "DELETE FROM playlist_items WHERE id = ?1",
            [items[slot].item_id],
        )?;
        let rest: Vec<i64> = items
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != slot)
            .map(|(_, it)| it.item_id)
            .collect();
        rewrite_positions(&tx, &rest)?;
        tx.commit()?;
        self.touch_playlist(id)
    }

    /// Move o item visível de `from` para `to` na lista unificada (índices
    /// 0-based, como `Vec::remove` + `insert`). Os itens ocultos mantêm suas
    /// posições absolutas; só os slots visíveis são permutados. Atômico.
    pub fn reorder_playlist(&mut self, id: PlaylistId, from: usize, to: usize) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let items = scan_playlist_items(&tx, id)?;
        let slots: Vec<usize> = (0..items.len())
            .filter(|&i| !matches!(items[i].kind, ScannedKind::Hidden))
            .collect();
        if from >= slots.len() {
            return Err(Error::InvalidPosition(from));
        }
        if to >= slots.len() {
            return Err(Error::InvalidPosition(to));
        }
        let mut vis_ids: Vec<i64> = slots.iter().map(|&i| items[i].item_id).collect();
        let it = vis_ids.remove(from);
        vis_ids.insert(to, it);
        let mut all: Vec<i64> = items.iter().map(|i| i.item_id).collect();
        for (slot, vid) in slots.iter().zip(vis_ids) {
            all[*slot] = vid;
        }
        rewrite_positions(&tx, &all)?;
        tx.commit()?;
        self.touch_playlist(id)
    }
}

fn rewrite_positions(conn: &rusqlite::Connection, items: &[i64]) -> Result<()> {
    for (pos, item) in items.iter().enumerate() {
        conn.execute(
            "UPDATE playlist_items SET position = ?2 WHERE id = ?1",
            params![item, pos as i64],
        )?;
    }
    Ok(())
}

fn insert_remote_tracks(conn: &rusqlite::Connection, id: PlaylistId, tracks: &[kazzu_core::Track]) -> Result<()> {
    let mut next: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_items WHERE playlist_id = ?1",
        [id.0],
        |r| r.get(0),
    )?;
    let t = now();
    for tr in tracks {
        let kazzu_core::TrackSource::Remote { provider, id: rid } = &tr.source else {
            continue;
        };
        let prov = provider_key(*provider)?;
        let exists = conn
            .query_row(
                "SELECT 1 FROM playlist_items
                 WHERE playlist_id = ?1 AND remote_provider = ?2 AND remote_id = ?3",
                params![id.0, prov, rid],
                |_| Ok(()),
            )
            .is_ok();
        if exists {
            continue;
        }
        let json =
            serde_json::to_string(tr).map_err(|e| Error::InvalidRemote(e.to_string()))?;
        conn.execute(
            "INSERT INTO playlist_items(playlist_id, remote_provider, remote_id, remote_json,
                position, added_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id.0, prov, rid, json, next, t],
        )?;
        next += 1;
    }
    Ok(())
}
