use std::path::{Path, PathBuf};
use std::sync::Mutex;

use image::{ImageBuffer, Rgb as ImgRgb};
use kazzu_library::{Library, ScanProgress, TrackId};
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};

fn write_wav(path: &Path, rate: u32, ms: u32) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for i in 0..(rate * ms / 1000) {
        let s = ((i as f32 * 0.05).sin() * 8000.0) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
}

fn png(color: [u8; 3]) -> Vec<u8> {
    let img: ImageBuffer<ImgRgb<u8>, _> = ImageBuffer::from_pixel(300, 300, ImgRgb(color));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

struct Spec<'a> {
    file: &'a str,
    title: &'a str,
    artist: &'a str,
    album: &'a str,
    album_artist: Option<&'a str>,
    track: u32,
    cover: Option<[u8; 3]>,
}

fn make(dir: &Path, s: &Spec) -> PathBuf {
    let p = dir.join(s.file);
    std::fs::create_dir_all(dir).unwrap();
    write_wav(&p, 44100, 300);
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(s.title.into());
    tag.set_artist(s.artist.into());
    tag.set_album(s.album.into());
    tag.set_track(s.track);
    tag.insert_text(ItemKey::RecordingDate, "2021".into());
    if let Some(a) = s.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, a.into());
    }
    if let Some(c) = s.cover {
        tag.push_picture(
            Picture::unchecked(png(c))
                .pic_type(PictureType::CoverFront)
                .mime_type(MimeType::Png)
                .build(),
        );
    }
    tag.save_to_path(&p, WriteOptions::default()).unwrap();
    p
}

fn setup() -> (tempfile::TempDir, Library, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let music = tmp.path().join("music");
    std::fs::create_dir_all(&music).unwrap();
    let lib = Library::open_in_memory(tmp.path().join("cache")).unwrap();
    lib.add_folder(&music).unwrap();
    (tmp, lib, std::fs::canonicalize(music).unwrap())
}

fn quiet(_: ScanProgress) {}

#[test]
fn scan_reads_tags_and_properties() {
    let (_t, mut lib, music) = setup();
    make(
        &music.join("A"),
        &Spec {
            file: "01.wav",
            title: "Aurora",
            artist: "Nina",
            album: "Dawn",
            album_artist: Some("Nina"),
            track: 1,
            cover: None,
        },
    );
    let rep = lib.scan(&quiet).unwrap();
    assert_eq!((rep.added, rep.updated, rep.removed), (1, 0, 0));
    let t = &lib.tracks().unwrap()[0];
    assert_eq!(t.title, "Aurora");
    assert_eq!(t.artists, vec!["Nina"]);
    assert_eq!(t.album.as_deref(), Some("Dawn"));
    assert_eq!(t.year, Some(2021));
    assert_eq!(t.track_no, Some(1));
    assert_eq!(t.sample_rate, Some(44100));
    assert_eq!(t.bit_depth, Some(16));
    assert_eq!(t.channels, Some(2));
    assert_eq!(t.codec, "WAV");
    assert!((250..=350).contains(&t.duration_ms), "{}", t.duration_ms);
    let core = t.to_core(None);
    assert_eq!(core.artist_line(), "Nina");
    assert_eq!(TrackId::from_core(&core.id), Some(t.id));
}

#[test]
fn incremental_scan_detects_changes_and_removals() {
    let (_t, mut lib, music) = setup();
    let spec = |f, t| Spec {
        file: f,
        title: t,
        artist: "X",
        album: "Al",
        album_artist: Some("X"),
        track: 1,
        cover: None,
    };
    let p1 = make(&music, &spec("a.wav", "One"));
    make(&music, &spec("b.wav", "Two"));
    assert_eq!(lib.scan(&quiet).unwrap().added, 2);

    let rep = lib.scan(&quiet).unwrap();
    assert_eq!((rep.added, rep.updated, rep.unchanged), (0, 0, 2));

    // Modifica a: tamanho muda.
    make(
        &music,
        &Spec {
            title: "One (remaster)",
            ..spec("a.wav", "")
        },
    );
    std::fs::remove_file(music.join("b.wav")).unwrap();
    let rep = lib.scan(&quiet).unwrap();
    assert_eq!((rep.updated, rep.removed), (1, 1));
    let tracks = lib.tracks().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].title, "One (remaster)");
    assert_eq!(tracks[0].path, p1);

    std::fs::remove_file(&p1).unwrap();
    lib.scan(&quiet).unwrap();
    assert!(lib.albums().unwrap().is_empty());
    assert!(lib.artists().unwrap().is_empty());
}

#[test]
fn embedded_cover_blur_and_accent() {
    let (_t, mut lib, music) = setup();
    for n in 1..=2 {
        make(
            &music.join("Red"),
            &Spec {
                file: &format!("{n}.wav"),
                title: "T",
                artist: "Ana",
                album: "Red",
                album_artist: Some("Ana"),
                track: n,
                cover: Some([220, 20, 30]),
            },
        );
    }
    let progress = Mutex::new(Vec::new());
    lib.scan(&|p| progress.lock().unwrap().push(p)).unwrap();
    let p = progress.into_inner().unwrap();
    assert!(
        p.iter()
            .any(|x| matches!(x, ScanProgress::Reading { done: 2, total: 2 }))
    );
    assert!(matches!(p.last(), Some(ScanProgress::Finished(_))));

    let album = &lib.albums().unwrap()[0];
    assert_eq!(album.track_count, 2);
    let art = album.artwork.as_ref().expect("capa");
    assert!(art.cover.exists() && art.thumb.exists() && art.blurred.exists());
    let blur = image::open(&art.blurred).unwrap();
    assert!(blur.width() <= 64);
    assert!(
        art.accent.0 > 180 && art.accent.1 < 80 && art.accent.2 < 90,
        "{:?}",
        art.accent
    );
}

#[test]
fn folder_cover_fallback() {
    let (_t, mut lib, music) = setup();
    let dir = music.join("B");
    make(
        &dir,
        &Spec {
            file: "1.wav",
            title: "T",
            artist: "Bo",
            album: "Blue",
            album_artist: Some("Bo"),
            track: 1,
            cover: None,
        },
    );
    std::fs::write(dir.join("Cover.JPG"), {
        let img: ImageBuffer<ImgRgb<u8>, _> =
            ImageBuffer::from_pixel(100, 100, ImgRgb([10u8, 40, 220]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
        out.into_inner()
    })
    .unwrap();
    lib.scan(&quiet).unwrap();
    let art = lib
        .albums()
        .unwrap()
        .remove(0)
        .artwork
        .expect("capa da pasta");
    assert!(art.accent.2 > 150);
}

#[test]
fn compilation_without_album_artist_is_grouped() {
    let (_t, mut lib, music) = setup();
    let dir = music.join("Hits");
    for (i, a) in ["P", "Q", "R"].iter().enumerate() {
        make(
            &dir,
            &Spec {
                file: &format!("{i}.wav"),
                title: "S",
                artist: a,
                album: "Hits",
                album_artist: None,
                track: i as u32 + 1,
                cover: None,
            },
        );
    }
    lib.scan(&quiet).unwrap();
    let albums = lib.albums().unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].artist, "Various Artists");
    // Mudança incremental em uma faixa mantém o álbum único.
    make(
        &dir,
        &Spec {
            file: "0.wav",
            title: "S2",
            artist: "P",
            album: "Hits",
            album_artist: None,
            track: 1,
            cover: None,
        },
    );
    lib.scan(&quiet).unwrap();
    assert_eq!(lib.albums().unwrap().len(), 1);
}

#[test]
fn search_accents_prefix_and_albums() {
    let (_t, mut lib, music) = setup();
    make(
        &music,
        &Spec {
            file: "1.wav",
            title: "Coração Selvagem",
            artist: "Zé Ramalho",
            album: "Paêbirú",
            album_artist: Some("Zé Ramalho"),
            track: 1,
            cover: None,
        },
    );
    make(
        &music,
        &Spec {
            file: "2.wav",
            title: "Other",
            artist: "Someone",
            album: "Else",
            album_artist: Some("Someone"),
            track: 1,
            cover: None,
        },
    );
    lib.scan(&quiet).unwrap();
    assert_eq!(lib.search("coracao", 10).unwrap().tracks.len(), 1);
    assert_eq!(lib.search("ze ram", 10).unwrap().tracks.len(), 1);
    assert_eq!(lib.search("Paêb", 10).unwrap().albums.len(), 1);
    assert_eq!(lib.search("Some", 10).unwrap().artists.len(), 1);
    assert!(lib.search("   ", 10).unwrap().tracks.is_empty());
    assert!(lib.search("\"*(", 10).is_ok());
}

#[test]
fn playlists_likes_history() {
    let (_t, mut lib, music) = setup();
    for (i, n) in ["a", "b", "c"].iter().enumerate() {
        make(
            &music,
            &Spec {
                file: &format!("{n}.wav"),
                title: n,
                artist: "X",
                album: "Al",
                album_artist: Some("X"),
                track: i as u32 + 1,
                cover: None,
            },
        );
    }
    lib.scan(&quiet).unwrap();
    let ids: Vec<_> = lib
        .album_tracks(lib.albums().unwrap()[0].id)
        .unwrap()
        .iter()
        .map(|t| t.id)
        .collect();
    assert_eq!(ids.len(), 3);

    let pl = lib.create_playlist("Favs").unwrap();
    lib.add_to_playlist(pl, &ids).unwrap();
    lib.reorder_playlist(pl, 0, 2).unwrap();
    let titles = |lib: &Library| {
        lib.playlist_tracks(pl)
            .unwrap()
            .iter()
            .map(|t| t.title.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(titles(&lib), ["b", "c", "a"]);
    lib.remove_from_playlist(pl, 1).unwrap();
    assert_eq!(titles(&lib), ["b", "a"]);
    assert!(lib.remove_from_playlist(pl, 9).is_err());
    assert_eq!(lib.playlists().unwrap()[0].track_count, 2);
    lib.rename_playlist(pl, "Top").unwrap();

    lib.like(ids[0]).unwrap();
    lib.like(ids[0]).unwrap();
    assert_eq!(lib.liked().unwrap().len(), 1);
    assert!(lib.is_liked(ids[0]).unwrap());
    lib.unlike(ids[0]).unwrap();
    assert!(lib.liked().unwrap().is_empty());

    lib.record_play_at(ids[0], 1000, 100).unwrap();
    lib.record_play_at(ids[1], 2000, 200).unwrap();
    lib.record_play_at(ids[0], 3000, 300).unwrap();
    let h = lib.history(10).unwrap();
    assert_eq!(h.len(), 3);
    assert_eq!(h[0].track.id, ids[0]);
    assert_eq!(lib.history_range(150, 250, 10).unwrap().len(), 1);
    let recent = lib.recent(10).unwrap();
    assert_eq!(
        recent.iter().map(|t| t.id).collect::<Vec<_>>(),
        [ids[0], ids[1]]
    );

    // Remover o arquivo remove a faixa das playlists e do histórico (cascata).
    std::fs::remove_file(music.join("a.wav")).unwrap();
    lib.scan(&quiet).unwrap();
    assert_eq!(titles(&lib), ["b"]);
    lib.delete_playlist(pl).unwrap();
    assert!(lib.playlists().unwrap().is_empty());
}

#[test]
fn missing_root_does_not_wipe_tracks_and_removed_folder_does() {
    let (tmp, mut lib, music) = setup();
    make(
        &music,
        &Spec {
            file: "1.wav",
            title: "T",
            artist: "A",
            album: "Al",
            album_artist: Some("A"),
            track: 1,
            cover: None,
        },
    );
    lib.scan(&quiet).unwrap();
    let moved = tmp.path().join("moved");
    std::fs::rename(&music, &moved).unwrap(); // "desmontou"
    lib.scan(&quiet).unwrap();
    assert_eq!(lib.track_count().unwrap(), 1);
    std::fs::rename(&moved, &music).unwrap();
    lib.remove_folder(&music).unwrap();
    assert_eq!(lib.track_count().unwrap(), 0);
}

#[test]
fn migrations_are_versioned_and_persistent() {
    let tmp = tempfile::tempdir().unwrap();
    {
        let lib = Library::open(tmp.path()).unwrap();
        assert_eq!(lib.schema_version().unwrap(), 1);
        lib.create_playlist("P").unwrap();
    }
    let lib = Library::open(tmp.path()).unwrap();
    assert_eq!(lib.playlists().unwrap().len(), 1);
}

#[test]
fn unreadable_file_is_reported_not_fatal() {
    let (_t, mut lib, music) = setup();
    std::fs::write(music.join("broken.flac"), b"not audio").unwrap();
    make(
        &music,
        &Spec {
            file: "ok.wav",
            title: "Ok",
            artist: "A",
            album: "Al",
            album_artist: Some("A"),
            track: 1,
            cover: None,
        },
    );
    let rep = lib.scan(&quiet).unwrap();
    assert_eq!(rep.added, 1);
    assert_eq!(rep.failed.len(), 1);
    let _ = lofty::read_from_path(music.join("ok.wav"))
        .unwrap()
        .primary_tag();
}
