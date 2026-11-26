//! Testes do trabalho em segundo plano: fila de jobs (debounce da busca), scan repetido e cargas concorrentes.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kazzu_app::worker::{Job, JobQueue, JobResult, JobRunner, Scope, Store, ThreadedRunner};
use lofty::config::WriteOptions;
use lofty::tag::{Accessor, Tag, TagExt, TagType};

fn write_wav(dir: &Path, name: &str, title: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(name);
    let spec = hound::WavSpec { channels: 2, sample_rate: 44_100, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..4_410 {
        let s = ((i as f32 * 0.03).sin() * 3000.0) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(title.into());
    tag.set_artist("Artista".into());
    tag.set_album(dir.file_name().unwrap().to_string_lossy().into_owned());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
}

// ---- fila de jobs --------------------------------------------------------------------------------

fn search(q: &str) -> Job {
    Job::Search { seq: 0, query: q.into() }
}

fn describe(j: &Job) -> String {
    match j {
        Job::Search { query, .. } => format!("search:{query}"),
        Job::NowArt { seq, .. } => format!("nowart:{seq}"),
        Job::Refresh => "refresh".into(),
        other => format!("{other:?}").chars().take(12).collect(),
    }
}

#[test]
fn only_the_last_search_of_a_burst_runs_and_other_jobs_do_not_wait_for_it() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let mut q = JobQueue::new(rx, Duration::from_millis(150));
    tx.send(search("a")).unwrap();
    tx.send(search("au")).unwrap();
    tx.send(Job::NowArt { seq: 7, cover: None }).unwrap();
    tx.send(search("aur")).unwrap();
    tx.send(Job::Refresh).unwrap();
    let t0 = Instant::now();
    // os outros jobs saem na hora, na ordem de chegada, sem esperar o debounce da busca
    assert_eq!(describe(&q.next_job().unwrap()), "nowart:7");
    assert_eq!(describe(&q.next_job().unwrap()), "refresh");
    assert!(t0.elapsed() < Duration::from_millis(100), "outros jobs esperaram a busca: {:?}", t0.elapsed());
    // e só então a busca mais nova, depois do prazo
    assert_eq!(describe(&q.next_job().unwrap()), "search:aur");
    assert!(t0.elapsed() >= Duration::from_millis(140), "a busca não esperou o debounce");
    drop(tx);
    assert!(q.next_job().is_none());
}

#[test]
fn a_steady_stream_of_other_jobs_cannot_starve_a_pending_search() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let mut q = JobQueue::new(rx, Duration::from_millis(120));
    tx.send(search("x")).unwrap();
    let feeder = std::thread::spawn(move || {
        for i in 0..8 {
            std::thread::sleep(Duration::from_millis(30));
            if tx.send(Job::NowArt { seq: i, cover: None }).is_err() {
                return;
            }
        }
    });
    let t0 = Instant::now();
    let mut got_search_at = None;
    while let Some(j) = q.next_job() {
        if matches!(j, Job::Search { .. }) {
            got_search_at = Some(t0.elapsed());
            break;
        }
    }
    feeder.join().unwrap();
    let at = got_search_at.expect("a busca precisa rodar");
    assert!(at < Duration::from_millis(400), "busca atrasada por outros jobs: {at:?}");
}

// ---- scan repetido ----------------------------------------------------------------------------------

#[test]
fn a_folder_added_during_a_scan_is_indexed_by_a_follow_up_scan() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
    for i in 0..40 {
        write_wav(&a, &format!("{i:02}.wav"), &format!("A{i}"));
    }
    for i in 0..5 {
        write_wav(&b, &format!("{i:02}.wav"), &format!("B{i}"));
    }
    let results: Arc<Mutex<Vec<&'static str>>> = Arc::default();
    let sink = results.clone();
    let runner = ThreadedRunner::spawn(
        data.clone(),
        Arc::new(move |r| {
            let tag = match r {
                JobResult::ScanDone(Ok(_)) => "scan-done",
                JobResult::ScanDone(Err(_)) => "scan-error",
                JobResult::FoldersChanged => "folders",
                _ => return,
            };
            sink.lock().unwrap().push(tag);
        }),
        None,
    )
    .unwrap();
    runner.submit(Job::AddFolder(a));
    wait_for(&results, "folders", 1);
    runner.submit(Job::Scan);
    // sem esperar o primeiro scan acabar: outra pasta + novo pedido de scan
    runner.submit(Job::AddFolder(b));
    runner.submit(Job::Scan);
    runner.submit(Job::Scan);
    // 2 scans (o pedido repetido durante o primeiro é agrupado em UM scan seguinte)
    wait_for(&results, "scan-done", 2);
    std::thread::sleep(Duration::from_millis(300));
    let lib = kazzu_library::Library::open(&data).unwrap();
    assert_eq!(lib.track_count().unwrap(), 45, "a pasta adicionada durante o scan precisa ser indexada: {:?}", results.lock().unwrap());
    assert!(!results.lock().unwrap().contains(&"scan-error"));
}

fn wait_for(results: &Arc<Mutex<Vec<&'static str>>>, tag: &str, count: usize) {
    let start = Instant::now();
    while results.lock().unwrap().iter().filter(|t| **t == tag).count() < count {
        assert!(start.elapsed() < Duration::from_secs(30), "esperando {tag} x{count}: {:?}", results.lock().unwrap());
        std::thread::sleep(Duration::from_millis(10));
    }
}

// ---- índice inicial da fila --------------------------------------------------------------------------

#[test]
fn queue_start_index_follows_the_playable_list() {
    let tmp = tempfile::tempdir().unwrap();
    let music = tmp.path().join("m");
    for (i, t) in ["Um", "Dois", "Tres"].iter().enumerate() {
        write_wav(&music, &format!("{i}.wav"), t);
    }
    let mut lib = kazzu_library::Library::open(tmp.path().join("data")).unwrap();
    lib.add_folder(&music).unwrap();
    lib.scan(&|_| {}).unwrap();
    let ids: Vec<String> = lib.tracks().unwrap().iter().map(|t| t.id.0.to_string()).collect();
    let mut store = Store::new(lib);
    let out: Mutex<Vec<(usize, usize)>> = Mutex::default();
    // ids inexistentes no meio da lista não deslocam o início
    let mut list = vec!["99999".to_owned()];
    list.extend(ids.clone());
    store.run(
        Job::Play { scope: Scope::Ids(list), start_id: Some(ids[2].clone()), shuffle: false, label: String::new() },
        &|r| {
            if let JobResult::Queue { items, start, .. } = r {
                out.lock().unwrap().push((items.len(), start));
            }
        },
    );
    assert_eq!(*out.lock().unwrap(), [(3, 2)]);
}

#[test]
fn remote_queue_debounces_search_and_suggestions_but_not_other_jobs() {
    use kazzu_app::remote::RemoteJob;
    use kazzu_app::worker::RemoteQueue;
    let (tx, rx) = crossbeam_channel::unbounded();
    let mut q = RemoteQueue::new(rx, Duration::from_millis(60));
    let search = |s: &str| RemoteJob::Search { seq: 1, query: s.into(), kind: 0 };
    tx.send(search("a")).unwrap();
    tx.send(search("ab")).unwrap();
    tx.send(RemoteJob::Suggest { seq: 1, prefix: "ab".into() }).unwrap();
    tx.send(RemoteJob::Covers { urls: vec![] }).unwrap();
    tx.send(search("abc")).unwrap();
    // o que não é busca passa na hora, antes do debounce
    assert!(matches!(q.next_job(), Some(RemoteJob::Covers { .. })));
    let mut got = vec![q.next_job().unwrap(), q.next_job().unwrap()];
    got.sort_by_key(|j| matches!(j, RemoteJob::Suggest { .. }));
    assert!(matches!(&got[0], RemoteJob::Search { query, .. } if query == "abc"), "só a última busca da rajada: {got:?}");
    assert!(matches!(&got[1], RemoteJob::Suggest { prefix, .. } if prefix == "ab"));
    drop(tx);
    assert!(q.next_job().is_none());
}

#[test]
fn obsolete_remote_jobs_are_recognized_by_generation() {
    use kazzu_app::remote::RemoteJob;
    use kazzu_app::vm::HomeSectionVm;
    use kazzu_app::worker::LatestSeq;
    let latest = LatestSeq::default();
    let old = RemoteJob::LoadHome { seq: 1, section: HomeSectionVm::ForYou };
    let new = RemoteJob::LoadHome { seq: 2, section: HomeSectionVm::Mixes };
    latest.note(&old);
    assert!(!latest.is_stale(&old));
    latest.note(&new);
    assert!(latest.is_stale(&old) && !latest.is_stale(&new));
    // tipos diferentes não se atrapalham, e jobs sem geração nunca são descartados
    let covers = RemoteJob::Covers { urls: vec![] };
    latest.note(&covers);
    assert!(!latest.is_stale(&covers));
    assert!(!latest.is_stale(&RemoteJob::LoadExplore { seq: 1 }));
}

#[test]
fn local_songs_and_artist_pages_use_indexed_tracks_and_discography() {
    use kazzu_app::worker::DetailRef;
    let tmp = tempfile::tempdir().unwrap();
    let music = tmp.path().join("album");
    let data = tmp.path().join("data");
    write_wav(&music, "a.wav", "Primeira");
    write_wav(&music, "b.wav", "Segunda");
    let mut lib = kazzu_library::Library::open(&data).unwrap();
    lib.add_folder(&music).unwrap();
    lib.scan(&|_| {}).unwrap();
    let artist = lib.artists().unwrap().remove(0);
    let track = lib.tracks().unwrap().remove(0);
    drop(lib);
    let mut store = Store::open(&data).unwrap();
    store.run(Job::Refresh, &|_| {});
    for what in [DetailRef::Songs, DetailRef::Artist(artist.id.0.to_string()), DetailRef::ArtistOfTrack(track.id.0.to_string())] {
        let result = Mutex::new(None);
        store.run(Job::OpenDetail { seq: 1, what: what.clone() }, &|r| {
            if let JobResult::Detail { detail, ids, remote_tracks, .. } = r { *result.lock().unwrap() = Some((detail, ids, remote_tracks)); }
        });
        let (vm, ids, _) = result.into_inner().unwrap().expect("detail result");
        assert_eq!(ids.len(), 2);
        assert_eq!(vm.tracks.len(), 2);
        if !matches!(what, DetailRef::Songs) {
            assert_eq!(vm.title, "Artista");
            assert_eq!(vm.albums.len(), 1);
        }
    }
}
