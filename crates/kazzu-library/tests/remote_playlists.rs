use std::time::Duration;

use kazzu_core::{AlbumRef, ArtistRef, CoverRef, ProviderKind, TrackSource};
use kazzu_library::{Library, PlaylistEntry, PlaylistId, TrackId};
use rusqlite::Connection;

fn remote(id: &str, title: &str) -> kazzu_core::Track {
    let mut t = kazzu_core::Track::new(
        format!("yt:{id}"),
        title,
        TrackSource::Remote {
            provider: ProviderKind::YtMusic,
            id: id.to_owned(),
        },
    );
    t.artists = vec![ArtistRef::named("Artista X"), ArtistRef::named("Y")];
    t.album = Some(AlbumRef {
        id: None,
        title: "Álbum".into(),
    });
    t.duration = Some(Duration::from_millis(215_500));
    t.cover = Some(CoverRef::Url {
        url: "https://example.com/c.jpg".into(),
    });
    t
}

fn local_core() -> kazzu_core::Track {
    kazzu_core::Track::new(
        "1",
        "Local",
        TrackSource::Local {
            path: "/a.flac".into(),
        },
    )
}

/// Biblioteca em arquivo com 3 faixas locais (ids 1..=3) inseridas direto.
fn setup() -> (tempfile::TempDir, Library, PlaylistId) {
    let tmp = tempfile::tempdir().unwrap();
    let lib = Library::open(tmp.path()).unwrap();
    {
        let conn = Connection::open(tmp.path().join("library.sqlite")).unwrap();
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO tracks(id,path,mtime,size,title,duration_ms,codec,added_at)
                 VALUES (?1, ?2, 0, 0, ?3, 1000, 'FLAC', 0)",
                rusqlite::params![i, format!("/t{i}.flac"), format!("L{i}")],
            )
            .unwrap();
        }
    }
    let pl = lib.create_playlist("Mista").unwrap();
    (tmp, lib, pl)
}

fn titles(lib: &Library, pl: PlaylistId) -> Vec<String> {
    lib.playlist_entries(pl)
        .unwrap()
        .into_iter()
        .map(|e| match e {
            PlaylistEntry::Local(t) => t.title,
            PlaylistEntry::Remote(t) => t.title,
        })
        .collect()
}

#[test]
fn mixed_playlist_keeps_order_count_and_local_view() {
    let (_tmp, mut lib, pl) = setup();
    lib.add_to_playlist(pl, &[TrackId(1)]).unwrap();
    lib.add_remote_to_playlist(pl, &[remote("a", "R-a")])
        .unwrap();
    lib.add_to_playlist(pl, &[TrackId(2)]).unwrap();
    lib.add_remote_to_playlist(pl, &[remote("b", "R-b")])
        .unwrap();

    assert_eq!(titles(&lib, pl), ["L1", "R-a", "L2", "R-b"]);
    assert_eq!(lib.playlists().unwrap()[0].track_count, 4);
    let locals: Vec<_> = lib
        .playlist_tracks(pl)
        .unwrap()
        .into_iter()
        .map(|t| t.title)
        .collect();
    assert_eq!(locals, ["L1", "L2"]);
    assert!(lib.playlist_contains_remote(pl, "yt_music", "a").unwrap());
    assert!(!lib.playlist_contains_remote(pl, "yt_music", "zzz").unwrap());
    assert!(!lib.playlist_contains_remote(pl, "spotify", "a").unwrap());
}

#[test]
fn duplicate_remote_and_non_remote_are_ignored() {
    let (_tmp, mut lib, pl) = setup();
    lib.add_remote_to_playlist(pl, &[remote("a", "R-a"), remote("a", "dup"), local_core()])
        .unwrap();
    lib.add_remote_to_playlist(pl, &[remote("a", "again"), remote("b", "R-b")])
        .unwrap();
    assert_eq!(titles(&lib, pl), ["R-a", "R-b"]);
    assert_eq!(lib.playlists().unwrap()[0].track_count, 2);
}

#[test]
fn remove_and_reorder_use_unified_index() {
    let (_tmp, mut lib, pl) = setup();
    lib.add_to_playlist(pl, &[TrackId(1), TrackId(2)]).unwrap();
    lib.add_remote_to_playlist(pl, &[remote("a", "R-a"), remote("b", "R-b")])
        .unwrap();
    assert_eq!(titles(&lib, pl), ["L1", "L2", "R-a", "R-b"]);

    lib.reorder_playlist(pl, 3, 0).unwrap();
    assert_eq!(titles(&lib, pl), ["R-b", "L1", "L2", "R-a"]);
    lib.remove_from_playlist(pl, 2).unwrap();
    assert_eq!(titles(&lib, pl), ["R-b", "L1", "R-a"]);
    lib.remove_from_playlist(pl, 0).unwrap();
    assert_eq!(titles(&lib, pl), ["L1", "R-a"]);
    assert!(lib.remove_from_playlist(pl, 2).is_err());
    assert!(lib.reorder_playlist(pl, 0, 2).is_err());
    // Após remover, a faixa remota pode ser adicionada de novo.
    lib.add_remote_to_playlist(pl, &[remote("b", "R-b")])
        .unwrap();
    assert_eq!(titles(&lib, pl), ["L1", "R-a", "R-b"]);
}

#[test]
fn remote_json_round_trips_and_survives_reopen() {
    let (tmp, mut lib, pl) = setup();
    let original = remote("a", "Título ç");
    lib.add_remote_to_playlist(pl, std::slice::from_ref(&original))
        .unwrap();
    drop(lib);
    let lib = Library::open(tmp.path()).unwrap();
    let entries = lib.playlist_entries(pl).unwrap();
    let [PlaylistEntry::Remote(back)] = entries.as_slice() else {
        panic!("esperava uma entrada remota");
    };
    assert_eq!(back, &original);
    assert_eq!(back.title, "Título ç");
    assert_eq!(back.artists.len(), 2);
    assert_eq!(back.duration, Some(Duration::from_millis(215_500)));
    assert_eq!(
        back.cover,
        Some(CoverRef::Url {
            url: "https://example.com/c.jpg".into()
        })
    );
    assert!(matches!(
        back.source,
        TrackSource::Remote {
            provider: ProviderKind::YtMusic,
            ..
        }
    ));
    lib.delete_playlist(pl).unwrap();
    assert!(lib.playlist_entries(pl).unwrap().is_empty());
}

/// Playlist [L1, remota inválida, R-ok, local órfão, L3] (itens 1 e 3 ocultos).
fn dirty_playlist() -> (tempfile::TempDir, Library, PlaylistId, Connection) {
    let (tmp, mut lib, pl) = setup();
    lib.add_to_playlist(pl, &[TrackId(1)]).unwrap();
    lib.add_remote_to_playlist(pl, &[remote("bad", "Ruim"), remote("ok", "R-ok")])
        .unwrap();
    lib.add_to_playlist(pl, &[TrackId(2), TrackId(3)]).unwrap();
    let conn = Connection::open(tmp.path().join("library.sqlite")).unwrap();
    conn.execute(
        "UPDATE playlist_items SET remote_json = '{lixo' WHERE remote_id = 'bad'",
        [],
    )
    .unwrap();
    // Órfão: remove a faixa 2 sem cascata.
    conn.execute_batch("PRAGMA foreign_keys = OFF; DELETE FROM tracks WHERE id = 2;")
        .unwrap();
    (tmp, lib, pl, conn)
}

fn item_ids(conn: &Connection) -> Vec<i64> {
    let mut st = conn
        .prepare("SELECT id FROM playlist_items ORDER BY position, id")
        .unwrap();
    st.query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

#[test]
fn invalid_remote_json_skips_only_that_item() {
    let (_t, lib, pl, _c) = dirty_playlist();
    assert_eq!(titles(&lib, pl), ["L1", "R-ok", "L3"]);
}

#[test]
fn remove_uses_visible_index_and_preserves_hidden() {
    let (_t, mut lib, pl, conn) = dirty_playlist();
    let before = item_ids(&conn); // [L1, bad, ok, orfao, L3]
    lib.remove_from_playlist(pl, 1).unwrap(); // R-ok
    assert_eq!(titles(&lib, pl), ["L1", "L3"]);
    assert_eq!(item_ids(&conn), [before[0], before[1], before[3], before[4]]);
    assert!(lib.remove_from_playlist(pl, 2).is_err());
}

#[test]
fn reorder_uses_visible_index_and_keeps_hidden_slots() {
    let (_t, mut lib, pl, conn) = dirty_playlist();
    let before = item_ids(&conn);
    lib.reorder_playlist(pl, 2, 0).unwrap(); // L3 vai para o início
    assert_eq!(titles(&lib, pl), ["L3", "L1", "R-ok"]);
    // ocultos continuam nas posições 1 e 3
    let after = item_ids(&conn);
    assert_eq!(after[1], before[1]);
    assert_eq!(after[3], before[3]);
    assert!(lib.reorder_playlist(pl, 0, 3).is_err());
}

#[test]
fn invalid_index_changes_nothing() {
    let (_t, mut lib, pl, conn) = dirty_playlist();
    let before = item_ids(&conn);
    assert!(lib.remove_from_playlist(pl, 9).is_err());
    assert!(lib.reorder_playlist(pl, 0, 9).is_err());
    assert_eq!(item_ids(&conn), before);
    assert_eq!(titles(&lib, pl), ["L1", "R-ok", "L3"]);
}
