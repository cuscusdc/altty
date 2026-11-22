//! Abertura do banco SQLite e migrações versionadas (`PRAGMA user_version`).

use rusqlite::Connection;

use crate::error::{Error, Result};

/// Cada item é uma migração; o índice + 1 é a versão resultante.
/// Nunca edite uma migração já publicada: acrescente uma nova.
const MIGRATIONS: &[&str] = &[
    // v1 — esquema inicial
    r#"
    CREATE TABLE music_folders (
        path TEXT PRIMARY KEY
    );

    CREATE TABLE artists (
        id   INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE COLLATE NOCASE
    );

    CREATE TABLE albums (
        id        INTEGER PRIMARY KEY,
        title     TEXT NOT NULL COLLATE NOCASE,
        artist    TEXT NOT NULL COLLATE NOCASE,
        year      INTEGER,
        -- NULL: arte ainda não processada; '': processada, sem arte; senão chave do cache.
        art_key   TEXT,
        accent    INTEGER,
        UNIQUE (title, artist)
    );

    CREATE TABLE tracks (
        id          INTEGER PRIMARY KEY,
        path        TEXT NOT NULL UNIQUE,
        mtime       INTEGER NOT NULL,
        size        INTEGER NOT NULL,
        title       TEXT NOT NULL,
        album_id    INTEGER REFERENCES albums(id) ON DELETE SET NULL,
        year        INTEGER,
        track_no    INTEGER,
        disc_no     INTEGER,
        genre       TEXT,
        duration_ms INTEGER NOT NULL,
        sample_rate INTEGER,
        bit_depth   INTEGER,
        bitrate     INTEGER,
        channels    INTEGER,
        codec       TEXT NOT NULL,
        has_picture INTEGER NOT NULL DEFAULT 0,
        added_at    INTEGER NOT NULL
    );
    CREATE INDEX idx_tracks_album ON tracks(album_id, disc_no, track_no);
    CREATE INDEX idx_tracks_added ON tracks(added_at);

    CREATE TABLE track_artists (
        track_id  INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
        artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
        position  INTEGER NOT NULL,
        PRIMARY KEY (track_id, position)
    );
    CREATE INDEX idx_track_artists_artist ON track_artists(artist_id);

    CREATE TABLE playlists (
        id         INTEGER PRIMARY KEY,
        name       TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );

    CREATE TABLE playlist_items (
        id          INTEGER PRIMARY KEY,
        playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
        track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
        position    INTEGER NOT NULL,
        added_at    INTEGER NOT NULL
    );
    CREATE INDEX idx_playlist_items ON playlist_items(playlist_id, position);

    CREATE TABLE likes (
        track_id INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
        liked_at INTEGER NOT NULL
    );

    -- Placeholder para o modo offline (etapa 4): faixas de provedores baixadas.
    CREATE TABLE downloads (
        id         INTEGER PRIMARY KEY,
        provider   TEXT NOT NULL,
        remote_id  TEXT NOT NULL,
        track_id   INTEGER REFERENCES tracks(id) ON DELETE SET NULL,
        file_path  TEXT,
        status     TEXT NOT NULL DEFAULT 'queued',
        bytes      INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        UNIQUE (provider, remote_id)
    );

    -- Histórico de reprodução: SOMENTE local. Nunca é enviado à rede.
    CREATE TABLE play_history (
        id          INTEGER PRIMARY KEY,
        track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
        played_at   INTEGER NOT NULL,
        listened_ms INTEGER NOT NULL
    );
    CREATE INDEX idx_history_time ON play_history(played_at);

    CREATE VIRTUAL TABLE tracks_fts USING fts5(
        title, artists, album,
        tokenize = 'unicode61 remove_diacritics 2'
    );
    "#,
];

pub(crate) const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

pub(crate) fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    // Ignora erro em bancos em memória (WAL não se aplica).
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

pub(crate) fn migrate(conn: &mut Connection) -> Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if current > SCHEMA_VERSION {
        return Err(Error::SchemaTooNew {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    for (idx, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", idx as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}
