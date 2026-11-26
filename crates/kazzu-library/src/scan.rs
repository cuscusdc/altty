//! Scan incremental: descoberta → leitura paralela de tags → upsert → capas.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use rusqlite::{OptionalExtension, Transaction, params};

use crate::artwork;
use crate::error::{Error, Result};
use crate::model::{Rgb, ScanProgress, ScanReport};
use crate::scanner::{self, FileEntry, ParsedTrack};
use crate::{Library, now};

/// Faixas gravadas por transação durante o scan (lotes fecham em fronteira de pasta, então um álbum nunca é dividido).
pub(crate) const SCAN_BATCH: usize = 100;

/// Agrupa `parsed` (já ordenado por pasta) em lotes de pelo menos `size` faixas, sem separar uma pasta.
fn batches_by_dir(parsed: &[ParsedTrack], size: usize) -> Vec<&[ParsedTrack]> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..=parsed.len() {
        let boundary = i == parsed.len() || parsed[i].entry.path.parent() != parsed[i - 1].entry.path.parent();
        if boundary && i - start >= size.max(1) {
            out.push(&parsed[start..i]);
            start = i;
        }
    }
    if start < parsed.len() {
        out.push(&parsed[start..]);
    }
    out
}

fn folders_in(tx: &Transaction<'_>) -> Result<Vec<PathBuf>> {
    let mut st = tx.prepare("SELECT path FROM music_folders ORDER BY path")?;
    let rows = st.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?.into_iter().map(PathBuf::from).collect())
}

/// Artista de álbuns com faixas de artistas diferentes e sem tag de artista do álbum.
pub const VARIOUS_ARTISTS: &str = "Various Artists";

fn path_str(p: &Path) -> Option<&str> {
    p.to_str()
}

impl Library {
    /// Pastas de música configuradas.
    pub fn folders(&self) -> Result<Vec<PathBuf>> {
        let mut st = self
            .conn
            .prepare("SELECT path FROM music_folders ORDER BY path")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(PathBuf::from)
            .collect())
    }

    /// Registra uma pasta de música (idempotente). Não escaneia.
    pub fn add_folder(&self, path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        let canon = std::fs::canonicalize(p).map_err(|e| Error::io(p, e))?;
        if !canon.is_dir() {
            return Err(Error::NotFound("diretório de músicas"));
        }
        let s = canon.to_str().ok_or(Error::NotFound("caminho UTF-8"))?;
        self.conn
            .execute("INSERT OR IGNORE INTO music_folders(path) VALUES (?1)", [s])?;
        Ok(())
    }

    /// Remove a pasta e as faixas que estavam sob ela.
    pub fn remove_folder(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        let canon = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        let s = canon.to_string_lossy().into_owned();
        self.conn
            .execute("DELETE FROM music_folders WHERE path = ?1", [&s])?;
        // O próximo scan remove as faixas órfãs; remove já para refletir na UI.
        self.delete_tracks_where_outside_folders()?;
        Ok(())
    }

    fn delete_tracks_where_outside_folders(&mut self) -> Result<()> {
        let roots = self.folders()?;
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let rows: Vec<(i64, String)> = {
            let mut st = tx.prepare("SELECT id, path FROM tracks")?;
            st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<std::result::Result<_, _>>()?
        };
        let ids: Vec<i64> = rows
            .into_iter()
            .filter(|(_, p)| !roots.iter().any(|r| Path::new(p).starts_with(r)))
            .map(|(id, _)| id)
            .collect();
        delete_tracks(&tx, &ids)?;
        cleanup_orphans(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// Escaneia todas as pastas configuradas, de forma incremental
    /// (mtime + tamanho). `progress` pode ser chamado de várias threads;
    /// para usar um canal, passe `&move |p| { let _ = tx.send(p); }`.
    pub fn scan(&mut self, progress: &(dyn Fn(ScanProgress) + Sync)) -> Result<ScanReport> {
        let roots = self.folders()?;
        let mut report = ScanReport::default();

        // 1. Descoberta.
        let mut files: Vec<FileEntry> = Vec::new();
        let mut live_roots: Vec<PathBuf> = Vec::new();
        for root in &roots {
            if !root.is_dir() || std::fs::read_dir(root).is_err() {
                continue; // volume desmontado: não apaga as faixas dele
            }
            live_roots.push(root.clone());
            let base = files.len();
            let found = scanner::discover(
                root,
                &mut |n| progress(ScanProgress::Discovering { files_found: n }),
                base,
            );
            files.extend(found);
        }
        progress(ScanProgress::Discovering {
            files_found: files.len(),
        });


        // 2. Diferença com o banco.
        let existing: HashMap<String, (i64, i64, i64)> = {
            let mut st = self
                .conn
                .prepare("SELECT path, id, mtime, size FROM tracks")?;
            st.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?)))
            })?
            .collect::<std::result::Result<_, _>>()?
        };
        let mut seen: HashSet<&str> = HashSet::new();
        let mut to_read: Vec<&FileEntry> = Vec::new();
        for f in &files {
            let Some(p) = path_str(&f.path) else {
                report
                    .failed
                    .push((f.path.clone(), "caminho não é UTF-8".into()));
                continue;
            };
            if !seen.insert(p) {
                continue; // pastas cadastradas podem se sobrepor
            }
            match existing.get(p) {
                Some(&(_, mtime, size)) if mtime == f.mtime && size == f.size => {
                    report.unchanged += 1
                }
                _ => to_read.push(f),
            }
        }

        report.scanned = seen.len();

        // 3. Leitura paralela.
        let total = to_read.len();
        let done = AtomicUsize::new(0);
        progress(ScanProgress::Reading { done: 0, total });
        let results: Vec<std::result::Result<ParsedTrack, (PathBuf, String)>> = to_read
            .par_iter()
            .map(|e| {
                let r = scanner::read_track(e).map_err(|m| (e.path.clone(), m));
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                if n.is_multiple_of(8) || n == total {
                    progress(ScanProgress::Reading { done: n, total });
                }
                r
            })
            .collect();
        let mut parsed: Vec<ParsedTrack> = Vec::with_capacity(results.len());
        for r in results {
            match r {
                Ok(p) => parsed.push(p),
                Err(f) => report.failed.push(f),
            }
        }

        // 4. Banco. Remoções numa transação; upserts em lotes (por pasta, para não separar um álbum), cada um
        //    numa transação curta: o scan não segura a escrita do banco por muito tempo (curtir, histórico, outra
        //    conexão do app). As pastas são relidas DENTRO de cada transação: pasta removida durante o scan não
        //    tem faixas reinseridas.
        let mut touched_albums = HashSet::new();
        {
            let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let roots_now = folders_in(&tx)?;
            let gone: Vec<i64> = existing
                .iter()
                .filter(|(p, _)| {
                    let path = Path::new(p.as_str());
                    let under_live = live_roots.iter().any(|r| path.starts_with(r));
                    let under_any = roots_now.iter().any(|r| path.starts_with(r));
                    (under_live && !seen.contains(p.as_str())) || !under_any
                })
                .map(|(_, (id, _, _))| *id)
                .collect();
            report.removed = gone.len();
            delete_tracks(&tx, &gone)?;
            cleanup_orphans(&tx)?;
            tx.commit()?;
        }
        parsed.sort_by(|a, b| a.entry.path.parent().cmp(&b.entry.path.parent()).then_with(|| a.entry.path.cmp(&b.entry.path)));
        for chunk in batches_by_dir(&parsed, SCAN_BATCH) {
            let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let roots_now = folders_in(&tx)?;
            let chunk: Vec<ParsedTrack> = chunk.iter().filter(|p| roots_now.iter().any(|r| p.entry.path.starts_with(r))).cloned().collect();
            let resolved = resolve_albums(&tx, &chunk)?;
            for (p, album) in chunk.iter().zip(resolved) {
                let path = path_str(&p.entry.path).unwrap_or_default();
                let is_new = !existing.contains_key(path);
                let album_id = match &album {
                    Some((title, artist)) => Some(upsert_album(&tx, title, artist, p.year)?),
                    None => None,
                };
                if let Some(a) = album_id {
                    touched_albums.insert(a);
                }
                upsert_track(&tx, p, album_id)?;
                if is_new {
                    report.added += 1
                } else {
                    report.updated += 1
                }
            }
            cleanup_orphans(&tx)?;
            tx.commit()?;
        }

        // 5. Capas (apenas álbuns sem arte processada / com cache perdido).
        self.process_artwork(&touched_albums, progress)?;

        progress(ScanProgress::Finished(report.clone()));
        Ok(report)
    }

    fn process_artwork(
        &mut self,
        touched: &HashSet<i64>,
        progress: &(dyn Fn(ScanProgress) + Sync),
    ) -> Result<()> {
        // Reavalia: álbuns tocados sem arte ("") e álbuns cujo cache sumiu.
        {
            let rows: Vec<(i64, String)> = {
                let mut st = self
                    .conn
                    .prepare("SELECT id, art_key FROM albums WHERE art_key IS NOT NULL")?;
                st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<std::result::Result<_, _>>()?
            };
            let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            for (id, key) in rows {
                let retry = if key.is_empty() {
                    touched.contains(&id)
                } else {
                    !artwork::covers_dir(&self.cache_dir)
                        .join(format!("{key}.jpg"))
                        .exists()
                };
                if retry {
                    tx.execute("UPDATE albums SET art_key = NULL WHERE id = ?1", [id])?;
                }
            }
            tx.commit()?;
        }

        let pending: Vec<(i64, Vec<(PathBuf, bool)>)> = {
            let ids: Vec<i64> = {
                let mut st = self
                    .conn
                    .prepare("SELECT id FROM albums WHERE art_key IS NULL")?;
                st.query_map([], |r| r.get(0))?
                    .collect::<std::result::Result<_, _>>()?
            };
            let mut st = self.conn.prepare(
                "SELECT path, has_picture FROM tracks WHERE album_id = ?1
                 ORDER BY disc_no, track_no, path",
            )?;
            let mut out = Vec::with_capacity(ids.len());
            for id in ids {
                let tracks = st
                    .query_map([id], |r| {
                        Ok((PathBuf::from(r.get::<_, String>(0)?), r.get::<_, bool>(1)?))
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                out.push((id, tracks));
            }
            out
        };
        let total = pending.len();
        if total == 0 {
            return Ok(());
        }
        let done = AtomicUsize::new(0);
        progress(ScanProgress::Artwork { done: 0, total });
        let cache = self.cache_dir.clone();
        let results: Vec<(i64, Option<(String, Rgb)>)> = pending
            .par_iter()
            .map(|(id, tracks)| {
                let r = album_art(tracks, &cache);
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                progress(ScanProgress::Artwork { done: n, total });
                (*id, r)
            })
            .collect();
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        for (id, r) in results {
            match r {
                Some((key, accent)) => tx.execute(
                    "UPDATE albums SET art_key = ?2, accent = ?3 WHERE id = ?1",
                    params![id, key, accent.pack()],
                )?,
                None => tx.execute(
                    "UPDATE albums SET art_key = '', accent = NULL WHERE id = ?1",
                    [id],
                )?,
            };
        }
        tx.commit()?;
        Ok(())
    }
}

/// Capa embutida (primeira faixa que tiver) ou de pasta.
fn album_art(tracks: &[(PathBuf, bool)], cache: &Path) -> Option<(String, Rgb)> {
    for (path, has_pic) in tracks {
        if *has_pic
            && let Some(bytes) = scanner::read_embedded_picture(path)
            && let Some(r) = artwork::process(&bytes, cache)
        {
            return Some(r);
        }
    }
    let mut dirs: Vec<&Path> = tracks.iter().filter_map(|(p, _)| p.parent()).collect();
    dirs.dedup();
    for dir in dirs {
        if let Some(img) = scanner::find_folder_cover(dir)
            && let Ok(bytes) = std::fs::read(&img)
            && let Some(r) = artwork::process(&bytes, cache)
        {
            return Some(r);
        }
    }
    None
}

fn delete_tracks(tx: &Transaction<'_>, ids: &[i64]) -> Result<()> {
    for id in ids {
        tx.execute("DELETE FROM tracks_fts WHERE rowid = ?1", [id])?;
        tx.execute("DELETE FROM tracks WHERE id = ?1", [id])?;
    }
    Ok(())
}

fn cleanup_orphans(tx: &Transaction<'_>) -> Result<()> {
    tx.execute("DELETE FROM albums WHERE id NOT IN (SELECT album_id FROM tracks WHERE album_id IS NOT NULL)", [])?;
    tx.execute(
        "DELETE FROM artists WHERE id NOT IN (SELECT artist_id FROM track_artists)",
        [],
    )?;
    Ok(())
}

/// Decide (título, artista) do álbum de cada faixa lida.
///
/// * Com tag de artista do álbum: usa-a.
/// * Sem ela: agrupa por (título, pasta); artista comum ou "Various Artists".
///   Irmãos já no banco (mesma pasta/álbum) entram na conta, para que um scan
///   incremental não separe uma coletânea.
fn resolve_albums(
    tx: &Transaction<'_>,
    parsed: &[ParsedTrack],
) -> Result<Vec<Option<(String, String)>>> {
    type Key = (String, PathBuf);
    let mut groups: HashMap<Key, Vec<Option<&str>>> = HashMap::new();
    let key = |p: &ParsedTrack| -> Option<Key> {
        let album = p.album.as_ref()?;
        Some((album.to_lowercase(), p.entry.path.parent()?.to_path_buf()))
    };
    for p in parsed {
        if p.album.is_some()
            && p.album_artist.is_none()
            && let Some(k) = key(p)
        {
            groups
                .entry(k)
                .or_default()
                .push(p.artists.first().map(String::as_str));
        }
    }
    let changed: HashSet<&Path> = parsed.iter().map(|p| p.entry.path.as_path()).collect();
    let mut group_artist: HashMap<Key, String> = HashMap::new();
    for ((album_lc, dir), mut firsts) in groups.clone() {
        // Irmãos persistidos.
        let prefix = format!("{}/", dir.to_string_lossy());
        let mut st = tx.prepare(
            "SELECT t.path, (SELECT a.name FROM track_artists ta JOIN artists a ON a.id = ta.artist_id
                             WHERE ta.track_id = t.id AND ta.position = 0)
             FROM tracks t JOIN albums al ON al.id = t.album_id
             WHERE lower(al.title) = ?1 AND substr(t.path, 1, ?2) = ?3",
        )?;
        let sib: Vec<(String, Option<String>)> = st
            .query_map(
                params![album_lc, prefix.chars().count() as i64, prefix],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?
            .collect::<std::result::Result<_, _>>()?;
        let sib: Vec<(String, Option<String>)> = sib
            .into_iter()
            .filter(|(p, _)| !changed.contains(Path::new(p)))
            .collect();
        let mut all: Vec<Option<String>> = firsts.drain(..).map(|o| o.map(str::to_owned)).collect();
        all.extend(sib.into_iter().map(|(_, a)| a));
        let first = all.first().cloned().flatten();
        let artist = match first {
            Some(f)
                if all
                    .iter()
                    .all(|a| a.as_deref().is_some_and(|a| a.eq_ignore_ascii_case(&f))) =>
            {
                f
            }
            None if all.iter().all(Option::is_none) => String::new(),
            _ => VARIOUS_ARTISTS.to_owned(),
        };
        group_artist.insert((album_lc, dir), artist);
    }
    Ok(parsed
        .iter()
        .map(|p| {
            let album = p.album.clone()?;
            let artist = match &p.album_artist {
                Some(a) => a.clone(),
                None => key(p)
                    .and_then(|k| group_artist.get(&k).cloned())
                    .unwrap_or_default(),
            };
            Some((album, artist))
        })
        .collect())
}

fn upsert_album(tx: &Transaction<'_>, title: &str, artist: &str, year: Option<i32>) -> Result<i64> {
    let existing: Option<i64> = tx
        .query_row(
            "SELECT id FROM albums WHERE title = ?1 AND artist = ?2",
            params![title, artist],
            |r| r.get(0),
        )
        .optional()?;
    match existing {
        Some(id) => {
            if let Some(y) = year {
                tx.execute(
                    "UPDATE albums SET year = COALESCE(year, ?2) WHERE id = ?1",
                    params![id, y],
                )?;
            }
            Ok(id)
        }
        None => {
            tx.execute(
                "INSERT INTO albums(title, artist, year) VALUES (?1, ?2, ?3)",
                params![title, artist, year],
            )?;
            Ok(tx.last_insert_rowid())
        }
    }
}

fn upsert_track(tx: &Transaction<'_>, p: &ParsedTrack, album_id: Option<i64>) -> Result<()> {
    let path = path_str(&p.entry.path).unwrap_or_default();
    let id: i64 = tx.query_row(
        "INSERT INTO tracks(path, mtime, size, title, album_id, year, track_no, disc_no, genre,
                            duration_ms, sample_rate, bit_depth, bitrate, channels, codec,
                            has_picture, added_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
         ON CONFLICT(path) DO UPDATE SET
            mtime=excluded.mtime, size=excluded.size, title=excluded.title,
            album_id=excluded.album_id, year=excluded.year, track_no=excluded.track_no,
            disc_no=excluded.disc_no, genre=excluded.genre, duration_ms=excluded.duration_ms,
            sample_rate=excluded.sample_rate, bit_depth=excluded.bit_depth,
            bitrate=excluded.bitrate, channels=excluded.channels, codec=excluded.codec,
            has_picture=excluded.has_picture
         RETURNING id",
        params![
            path,
            p.entry.mtime,
            p.entry.size,
            p.title,
            album_id,
            p.year,
            p.track_no,
            p.disc_no,
            p.genre,
            p.duration_ms as i64,
            p.sample_rate,
            p.bit_depth,
            p.bitrate,
            p.channels,
            p.codec,
            p.has_picture,
            now()
        ],
        |r| r.get(0),
    )?;
    tx.execute("DELETE FROM track_artists WHERE track_id = ?1", [id])?;
    let mut seen = HashSet::new();
    let mut pos = 0i64;
    for name in &p.artists {
        if !seen.insert(name.to_lowercase()) {
            continue;
        }
        tx.execute("INSERT OR IGNORE INTO artists(name) VALUES (?1)", [name])?;
        let aid: i64 = tx.query_row("SELECT id FROM artists WHERE name = ?1", [name], |r| {
            r.get(0)
        })?;
        tx.execute(
            "INSERT INTO track_artists(track_id, artist_id, position) VALUES (?1,?2,?3)",
            params![id, aid, pos],
        )?;
        pos += 1;
    }
    tx.execute("DELETE FROM tracks_fts WHERE rowid = ?1", [id])?;
    tx.execute(
        "INSERT INTO tracks_fts(rowid, title, artists, album) VALUES (?1,?2,?3,?4)",
        params![
            id,
            p.title,
            p.artists.join(" "),
            p.album.clone().unwrap_or_default()
        ],
    )?;
    Ok(())
}
