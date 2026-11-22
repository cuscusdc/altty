//! Testes da lógica do app sem janela: `Controller` + `Store` reais, motor de áudio com `NullSink`, tela falsa.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kazzu_app::accent::Accent;
use kazzu_app::art::Pixels;
use kazzu_app::config::{AccentPref, Config, ThemePref};
use kazzu_app::controller::{Controller, Msg, NavTab, SystemPrefs};
use kazzu_app::view::View;
use kazzu_app::vm::*;
use kazzu_app::catalog::{Catalog, CatResult, ExploreFeed, Providers, Resolved};
use kazzu_app::covers::{CoverFetcher, ImageSource};
use kazzu_app::remote::RemoteWorker;
use kazzu_app::worker::{Job, JobResult, JobRunner, QueueMode, Store};
use kazzu_audio::{Engine, EngineConfig, NullSink};
use kazzu_core::{OutputInfo, PlaybackEvent, QueueItem, StreamSource, Track as CoreTrack, TrackSource};
use lofty::config::WriteOptions;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt, TagType};

// ---------------------------------------------------------------------------------------------
// Tela falsa: só grava o que o controller pediu.
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
struct Rec {
    playback: PlaybackVm,
    progress: (f32, String, String),
    loading: Option<bool>,
    recents: Vec<String>,
    home_albums: Vec<String>,
    collections: Vec<(String, String)>,
    playlists: Vec<String>,
    queue: Vec<String>,
    queue_summary: String,
    queue_more: String,
    queue_covers: Vec<bool>,
    search: Option<(SearchStateVm, Vec<String>, Vec<String>)>,
    detail: Option<(String, Vec<String>)>,
    detail_open: bool,
    scan: Option<ScanVm>,
    scan_seen: usize,
    toasts: Vec<(String, bool)>,
    toast_actions: Vec<String>,
    home: Option<HomeVm>,
    explore: Option<ExploreVm>,
    online: Vec<bool>,
    perf_offer: bool,
    search_vm: Option<SearchVm>,
    detail_vm: Option<DetailVm>,
    queue_titles: Vec<String>,
    queue_ids: Vec<String>,
    search_sets: usize,
    home_sets: usize,
    playing_marks: Vec<Option<String>>,
    liked_marks: Vec<(String, bool)>,
    prefs: Option<PrefsVm>,
    accent: Option<(Accent, bool)>,
    now_art: bool,
    ambient: bool,
    settings: Option<(SettingsVm, Vec<ProviderVm>)>,
    volume: f32,
}

#[derive(Clone, Default)]
struct FakeView(Rc<RefCell<Rec>>);

impl View for FakeView {
    fn set_playback(&self, p: &PlaybackVm) {
        self.0.borrow_mut().playback = p.clone();
    }
    fn set_progress(&self, progress: f32, position: &str, remaining: &str) {
        self.0.borrow_mut().progress = (progress, position.into(), remaining.into());
    }
    fn set_now_art(&self, art: &NowArtVm) {
        self.0.borrow_mut().now_art = art.cover.is_some();
    }
    fn set_ambient(&self, backdrop: Option<&Pixels>) {
        self.0.borrow_mut().ambient = backdrop.is_some();
    }
    fn set_accent(&self, accent: &Accent, dynamic: bool) {
        self.0.borrow_mut().accent = Some((*accent, dynamic));
    }
    fn set_home(&self, home: &HomeVm) {
        let mut r = self.0.borrow_mut();
        r.recents = home.recents.iter().map(|t| t.title.clone()).collect();
        r.home_albums = home.albums.iter().map(|a| a.title.clone()).collect();
        r.home = Some(home.clone());
        r.home_sets += 1;
    }
    fn set_explore(&self, e: &ExploreVm) {
        self.0.borrow_mut().explore = Some(e.clone());
    }
    fn set_online(&self, online: bool) {
        self.0.borrow_mut().online.push(online);
    }
    fn set_perf_offer(&self, offer: bool) {
        self.0.borrow_mut().perf_offer = offer;
    }
    fn close_sheets(&self) {}
    fn set_library(&self, l: &LibraryVm) {
        let mut r = self.0.borrow_mut();
        r.collections = l.collections.iter().map(|c| (c.id.clone(), c.count_label.clone())).collect();
        r.playlists = l.playlists.iter().map(|p| p.name.clone()).collect();
    }
    fn set_settings(&self, s: &SettingsVm, providers: &[ProviderVm]) {
        self.0.borrow_mut().settings = Some((s.clone(), providers.to_vec()));
    }
    fn set_loading(&self, loading: bool) {
        self.0.borrow_mut().loading = Some(loading);
    }
    fn set_queue(&self, q: &QueueVm) {
        let mut r = self.0.borrow_mut();
        r.queue = q.tracks.iter().map(|t| t.title.clone()).collect();
        r.queue_titles = r.queue.clone();
        r.queue_ids = q.tracks.iter().map(|t| t.id.clone()).collect();
        r.queue_summary = q.summary.clone();
        r.queue_more = q.more_label.clone();
        r.queue_covers = q.tracks.iter().map(|t| t.cover.is_some()).collect();
    }
    fn set_search(&self, s: &SearchVm) {
        let mut r = self.0.borrow_mut();
        r.search = Some((s.state, s.songs.iter().map(|t| t.title.clone()).collect(), s.recent.clone()));
        r.search_vm = Some(s.clone());
        r.search_sets += 1;
    }
    fn set_detail(&self, d: &DetailVm) {
        let mut r = self.0.borrow_mut();
        r.detail = Some((d.title.clone(), d.tracks.iter().map(|t| t.title.clone()).collect()));
        r.detail_vm = Some(d.clone());
    }
    fn open_detail(&self, open: bool) {
        self.0.borrow_mut().detail_open = open;
    }
    fn set_scan(&self, scan: Option<&ScanVm>) {
        let mut r = self.0.borrow_mut();
        if scan.is_some() {
            r.scan_seen += 1;
        }
        r.scan = scan.cloned();
    }
    fn toast(&self, text: &str, error: bool, action: &str) {
        let mut r = self.0.borrow_mut();
        r.toasts.push((text.into(), error));
        r.toast_actions.push(action.into());
    }
    fn mark_playing(&self, id: Option<&str>) {
        self.0.borrow_mut().playing_marks.push(id.map(str::to_owned));
    }
    fn mark_liked(&self, id: &str, liked: bool) {
        self.0.borrow_mut().liked_marks.push((id.into(), liked));
    }
    fn set_prefs(&self, prefs: &PrefsVm) {
        self.0.borrow_mut().prefs = Some(*prefs);
    }
    fn set_volume(&self, volume: f32) {
        self.0.borrow_mut().volume = volume;
    }
}

// ---------------------------------------------------------------------------------------------
// Execução de jobs na hora, com os resultados numa fila que o teste esvazia.
// ---------------------------------------------------------------------------------------------

type Results = Arc<Mutex<VecDeque<JobResult>>>;

struct InlineRunner {
    store: Mutex<Store>,
    out: Results,
    remote: Option<Arc<RemoteWorker>>,
}

impl JobRunner for InlineRunner {
    fn submit(&self, job: Job) {
        let out = self.out.clone();
        if let Job::Remote(r) = job {
            if let Some(w) = &self.remote {
                w.run(r, &move |res| {
                    if let Ok(mut q) = out.lock() {
                        q.push_back(res);
                    }
                });
            }
            return;
        }
        if let Ok(mut store) = self.store.lock() {
            store.run(job, &move |r| {
                if let Ok(mut q) = out.lock() {
                    q.push_back(r);
                }
            });
        }
    }
}

struct Harness {
    ctrl: Controller<FakeView>,
    rec: Rc<RefCell<Rec>>,
    results: Results,
    _dir: tempfile::TempDir,
}

fn write_track(dir: &Path, file: &str, title: &str, artist: &str, album: &str, genre: &str, no: u32) {
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(file);
    let spec = hound::WavSpec { channels: 2, sample_rate: 44_100, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..(44_100 * 3) {
        let s = ((i as f32 * 0.03).sin() * 6000.0) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title(title.into());
    tag.set_artist(artist.into());
    tag.set_album(album.into());
    tag.set_genre(genre.into());
    tag.set_track(no);
    tag.insert_text(ItemKey::AlbumArtist, artist.into());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
}

fn cover(dir: &Path, rgb: [u8; 3]) {
    image::RgbImage::from_pixel(300, 300, image::Rgb(rgb)).save(dir.join("cover.jpg")).unwrap();
}

/// Duas pastas de álbum: "Aurora" (3 faixas) e "Brasa" (1 faixa).
fn make_music(root: &Path) -> PathBuf {
    let music = root.join("music");
    let a = music.join("aurora");
    write_track(&a, "01.wav", "Maré Alta", "Aurora Vento", "Aurora", "Eletrônica", 1);
    write_track(&a, "02.wav", "Linha d'Água", "Aurora Vento", "Aurora", "Eletrônica", 2);
    write_track(&a, "03.wav", "Horizonte", "Aurora Vento", "Aurora", "Eletrônica", 3);
    cover(&a, [30, 120, 140]);
    let b = music.join("brasa");
    write_track(&b, "01.wav", "Calor de Março", "Nina Sabiá", "Brasa", "MPB", 1);
    cover(&b, [220, 90, 60]);
    music
}

fn harness() -> Harness {
    harness_with(None)
}

fn harness_with(remote: Option<Arc<RemoteWorker>>) -> Harness {
    let dir = tempfile::tempdir().unwrap();
    let music = make_music(dir.path());
    let lib = kazzu_library::Library::open(dir.path().join("data")).unwrap();
    let results: Results = Arc::default();
    let runner = InlineRunner { store: Mutex::new(Store::new(lib)), out: results.clone(), remote: remote.clone() };
    let engine = Engine::builder()
        .sink(NullSink::new().with_speed(8.0))
        .config(EngineConfig { volume: 1.0, ..EngineConfig::default() })
        .spawn()
        .unwrap();
    let view = FakeView::default();
    let rec = view.0.clone();
    let sink = Arc::new(|_msg: Msg| {});
    let cfg_path = dir.path().join("config.toml");
    let mut ctrl = Controller::new(view, engine, Box::new(runner), sink, Config::default(), Some(cfg_path));
    if remote.is_some() {
        ctrl.enable_remote();
    }
    ctrl.init(SystemPrefs::default());
    ctrl.add_folder_path(music);
    let mut h = Harness { ctrl, rec, results, _dir: dir };
    h.pump_until("biblioteca carregada", |r| r.loading == Some(false) && !r.recents.is_empty() && r.scan_seen > 0);
    h.settle(150); // a segunda atualização (depois do scan) também chega
    h
}

impl Harness {
    /// Entrega resultados de jobs e eventos do motor ao controller até `cond` ficar verdadeira (ou 15 s).
    fn pump_until(&mut self, what: &str, cond: impl Fn(&Rec) -> bool) {
        let start = Instant::now();
        loop {
            self.pump_once();
            if cond(&self.rec.borrow()) {
                return;
            }
            assert!(start.elapsed() < Duration::from_secs(15), "tempo esgotado esperando: {what}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn pump_once(&mut self) {
        let jobs: Vec<JobResult> = self.results.lock().unwrap().drain(..).collect();
        for r in jobs {
            self.ctrl.handle(Msg::Job(r));
        }
        let events: Vec<PlaybackEvent> = self.ctrl.engine().events().try_iter().collect();
        for e in events {
            self.ctrl.handle(Msg::Engine(e));
        }
    }

    /// Como `pump_until`, mas só entrega eventos do motor (jobs parados).
    fn pump_engine_until(&mut self, what: &str, cond: impl Fn(&Rec) -> bool) {
        let start = Instant::now();
        loop {
            self.pump_engine_only();
            if cond(&self.rec.borrow()) {
                return;
            }
            assert!(start.elapsed() < Duration::from_secs(15), "tempo esgotado esperando: {what}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn pump_until_engine(&mut self, what: &str, cond: impl Fn(&Rec) -> bool) {
        self.pump_engine_until(what, cond);
    }

    /// Só os eventos do motor: os resultados de jobs ficam parados (simula rede lenta).
    fn pump_engine_only(&mut self) {
        let events: Vec<PlaybackEvent> = self.ctrl.engine().events().try_iter().collect();
        for e in events {
            self.ctrl.handle(Msg::Engine(e));
        }
    }

    /// Entrega os resultados pendentes até (e incluindo) o primeiro de resolução de faixa; o resto fica parado.
    fn deliver_one_job(&mut self) -> bool {
        loop {
            let r = self.results.lock().unwrap().pop_front();
            let Some(r) = r else { return false };
            let is_resolve = matches!(&r, JobResult::Remote(kazzu_app::remote::RemoteResult::Resolved { .. } | kazzu_app::remote::RemoteResult::ResolveFailed { .. }));
            self.ctrl.handle(Msg::Job(r));
            if is_resolve {
                return true;
            }
        }
    }

    fn settle(&mut self, ms: u64) {
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end {
            self.pump_once();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn first_recent_id(&self) -> String {
        let title = self.rec.borrow().recents.first().cloned().unwrap();
        track_id_by_title(self, &title)
    }
}

fn track_id_by_title(h: &Harness, title: &str) -> String {
    let lib = kazzu_library::Library::open(h._dir.path().join("data")).unwrap();
    lib.tracks().unwrap().into_iter().find(|t| t.title == title).map(|t| t.id.0.to_string()).unwrap()
}

// ---------------------------------------------------------------------------------------------
// Testes
// ---------------------------------------------------------------------------------------------

#[test]
fn first_run_scans_and_fills_home_library_and_settings() {
    let h = harness();
    let r = h.rec.borrow();
    assert_eq!(r.recents.len(), 4);
    assert_eq!(r.collections.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["downloads", "liked", "albums", "playlists"]);
    assert_eq!(r.collections[2].1, "2 álbuns");
    assert_eq!(r.collections[1].1, "0 músicas");
    let (settings, providers) = r.settings.clone().unwrap();
    assert_eq!(settings.summary, "1 pasta, 4 músicas");
    assert!(settings.has_library);
    assert_eq!(providers.len(), 4);
    assert!(providers.iter().any(|p| p.id == "local" && p.connected), "a biblioteca local está ativa");
    assert!(providers.iter().filter(|p| p.id != "local" && p.id != "ytmusic").all(|p| !p.available), "outros provedores: em breve");
    assert!(r.scan_seen > 0, "o scan reportou progresso");
    assert!(r.scan.is_none(), "o banner some quando o scan termina");
}

#[test]
fn clicking_a_track_loads_the_list_as_queue_and_plays() {
    let mut h = harness();
    let id = track_id_by_title(&h, "Linha d'Água");
    h.ctrl.play_track(&id);
    h.pump_until("faixa tocando", |r| r.playback.playing && r.playback.now.as_ref().is_some_and(|t| t.title == "Linha d'Água"));
    let r = h.rec.borrow();
    let now = r.playback.now.as_ref().unwrap();
    assert_eq!((now.artist.as_str(), now.album.as_str()), ("Aurora Vento", "Aurora"));
    assert_eq!(r.playback.origin, "Recentes");
    assert!(now.playing);
    assert!(r.playing_marks.contains(&Some(id.clone())));
    assert_eq!(now.codec, "WAV");
    assert_eq!(now.detail, "16/44,1");
    drop(r);
    // as demais faixas dos recentes viram "a seguir" (3 depois da atual, na ordem da lista)
    h.pump_until("fila", |r| !r.queue.is_empty());
    assert!(h.rec.borrow().queue_summary.starts_with("A seguir, "));
}

#[test]
fn progress_labels_follow_position_events_and_seek_works() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("progresso", |r| r.progress.0 > 0.05);
    let (p, pos, rem) = h.rec.borrow().progress.clone();
    assert!(p > 0.0 && p <= 1.0);
    assert!(pos.starts_with("0:") && rem.starts_with('−'), "{pos} / {rem}");
    h.ctrl.seek(0.9);
    h.pump_until("pulou", |r| r.progress.0 >= 0.88);
}

#[test]
fn pause_resume_and_keyboard_shortcuts() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("tocando", |r| r.playback.playing);
    h.ctrl.toggle_play();
    h.pump_until("pausado", |r| !r.playback.playing && r.playback.has_now);
    h.ctrl.toggle_play();
    h.pump_until("retomou", |r| r.playback.playing);
    h.ctrl.volume_step(-1);
    assert!((h.ctrl.playback().volume - 0.95).abs() < 1e-3);
    h.ctrl.seek_relative(-5);
    h.ctrl.seek_relative(5);
    h.ctrl.toggle_shuffle();
    h.pump_until("shuffle", |r| r.playback.shuffle);
    h.ctrl.cycle_repeat();
    h.pump_until("repeat", |r| r.playback.repeat == RepeatVm::All);
    h.ctrl.cycle_repeat();
    h.pump_until("repeat um", |r| r.playback.repeat == RepeatVm::One);
}

#[test]
fn output_info_drives_the_bit_perfect_note_and_the_badge() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("saída informada", |r| r.playback.bit_perfect && r.playback.output_note == "Original, sem equalizador");
    // volume abaixo de 1,0 tira o selo bit-perfect (regra do motor)
    h.ctrl.set_volume(0.5);
    h.pump_until("nota de volume", |r| !r.playback.bit_perfect);
    assert!(h.rec.borrow().playback.output_note.starts_with("Volume ajustado"));
    // reamostragem: o texto diz de quanto para quanto
    let info = OutputInfo { device: "default".into(), sample_rate: 48_000, channels: 2, source: None, bit_perfect: false, resampled_from: Some(96_000) };
    h.ctrl.on_engine(PlaybackEvent::OutputChanged { info });
    assert_eq!(h.rec.borrow().playback.output_note, "Reamostrado de 96 para 48 kHz");
    assert_eq!(h.rec.borrow().playback.output_name, "Padrão do sistema");
}

#[test]
fn now_playing_art_arrives_with_accent_and_backdrop() {
    let mut h = harness();
    h.ctrl.play_track(&track_id_by_title(&h, "Calor de Março"));
    h.pump_until("capa", |r| r.now_art && r.playback.now.as_ref().is_some_and(|t| t.cover.is_some()));
    let (accent, dynamic) = h.rec.borrow().accent.unwrap();
    assert!(dynamic);
    // capa vermelho-alaranjada: o acento não pode ser o âmbar da marca
    assert_ne!(accent, Accent::BRAND);
    assert!(h.rec.borrow().ambient);
}

#[test]
fn search_states_idle_results_empty() {
    let mut h = harness();
    assert_eq!(h.rec.borrow().search.as_ref().unwrap().0, SearchStateVm::Idle);
    h.ctrl.search("hori");
    h.pump_until("resultados", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Results));
    assert_eq!(h.rec.borrow().search.as_ref().unwrap().1, ["Horizonte"]);
    h.ctrl.search("zzzzqq");
    h.pump_until("vazio", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Empty));
    h.ctrl.search("   ");
    assert_eq!(h.rec.borrow().search.as_ref().unwrap().0, SearchStateVm::Idle);
    // filtro "Artistas": o acervo local não tem artistas listáveis, então não aparece nada dele
    h.ctrl.search("aurora");
    h.pump_until("resultados 2", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Results));
    h.ctrl.search_kind_changed(3);
    h.pump_until("vazio 2", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Empty));
}

#[test]
fn stale_search_results_are_ignored() {
    let mut h = harness();
    h.ctrl.search("hori");
    h.ctrl.search("calor");
    // os dois resultados chegam; só o da última busca vale
    h.pump_until("calor", |r| r.search.as_ref().is_some_and(|s| s.1 == ["Calor de Março"]));
}

#[test]
fn playing_from_search_remembers_the_query_and_uses_results_as_queue() {
    let mut h = harness();
    h.ctrl.navigate(NavTab::Search);
    h.ctrl.search("aurora");
    h.pump_until("resultados", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Results));
    let id = track_id_by_title(&h, "Horizonte");
    h.ctrl.play_track(&id);
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Horizonte"));
    assert_eq!(h.rec.borrow().playback.origin, "Busca");
    assert_eq!(h.rec.borrow().search.as_ref().unwrap().2, ["aurora"], "busca recente gravada");
    assert_eq!(h.ctrl.config().recent_searches, ["aurora"]);
    h.ctrl.clear_recent_searches();
    assert!(h.rec.borrow().search.as_ref().unwrap().2.is_empty());
}

#[test]
fn like_updates_rows_collection_count_and_library() {
    let mut h = harness();
    let id = track_id_by_title(&h, "Maré Alta");
    h.ctrl.toggle_like(&id);
    assert_eq!(h.rec.borrow().liked_marks, [(id.clone(), true)]);
    assert_eq!(h.rec.borrow().collections[1], ("liked".into(), "1 música".into()));
    h.pump_until("gravado", |_| true);
    let lib = kazzu_library::Library::open(h._dir.path().join("data")).unwrap();
    assert!(lib.is_liked(kazzu_library::TrackId(id.parse().unwrap())).unwrap());
    h.ctrl.toggle_like(&id);
    assert_eq!(h.rec.borrow().collections[1].1, "0 músicas");
}

#[test]
fn detail_for_album_and_liked_collection() {
    let mut h = harness();
    let album_id = kazzu_library::Library::open(h._dir.path().join("data")).unwrap().albums().unwrap().into_iter().find(|a| a.title == "Aurora").unwrap().id.0.to_string();
    h.ctrl.open_album(&album_id);
    h.pump_until("detalhe", |r| r.detail_open);
    let (title, tracks) = h.rec.borrow().detail.clone().unwrap();
    assert_eq!(title, "Aurora");
    assert_eq!(tracks, ["Maré Alta", "Linha d'Água", "Horizonte"]);
    // tocar o detalhe usa a lista exibida como fila
    h.ctrl.play_all(false);
    h.pump_until("tocando o álbum", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Maré Alta"));
    assert_eq!(h.rec.borrow().playback.origin, "Aurora");
    // destaque "album:<id>": o rótulo vira o título do álbum
    h.ctrl.play_track(&format!("album:{album_id}"));
    h.pump_until("rótulo do álbum", |r| r.playback.origin == "Aurora");
    // coleção vazia
    h.ctrl.open_collection("liked");
    h.pump_until("curtidos", |r| r.detail.as_ref().is_some_and(|d| d.0 == "Curtidos"));
    assert!(h.rec.borrow().detail.as_ref().unwrap().1.is_empty());
}

#[test]
fn queue_remove_jump_and_clear() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("fila cheia", |r| r.queue.len() >= 3);
    let before = h.rec.borrow().queue.clone();
    let ids = h.rec.borrow().queue_ids.clone();
    h.ctrl.queue_remove(&ids[0]);
    h.pump_until("removeu", |r| r.queue.len() == before.len() - 1);
    assert_eq!(h.rec.borrow().queue.first(), before.get(1));
    h.ctrl.queue_jump(&ids[1]);
    h.pump_until("pulou na fila", |r| r.playback.now.as_ref().is_some_and(|t| Some(&t.title) == before.get(1)));
    h.ctrl.queue_clear();
    h.pump_until("limpa", |r| r.queue.is_empty());
}

#[test]
fn play_next_and_enqueue_show_a_toast() {
    let mut h = harness();
    h.ctrl.play_track(&track_id_by_title(&h, "Maré Alta"));
    h.pump_until("tocando", |r| r.playback.playing);
    let id = track_id_by_title(&h, "Calor de Março");
    h.ctrl.queue_track(&id, QueueMode::PlayNext);
    h.pump_until("toast", |r| r.toasts.iter().any(|t| t.0 == "Toca em seguida"));
    h.pump_until("fila com a faixa", |r| r.queue.first().is_some_and(|t| t == "Calor de Março"));
    h.ctrl.queue_track(&id, QueueMode::Enqueue);
    h.pump_until("toast 2", |r| r.toasts.iter().any(|t| t.0 == "Adicionada à fila"));
}

#[test]
fn playlists_can_be_created_and_empty_names_are_refused() {
    let mut h = harness();
    h.ctrl.create_playlist("Foco", "");
    h.pump_until("playlist", |r| r.playlists == ["Foco"]);
    assert_eq!(h.rec.borrow().collections[3].1, "1 playlist");
    h.ctrl.create_playlist("   ", "");
    h.pump_until("aviso", |r| r.toasts.iter().any(|t| t.0 == "Dê um nome à playlist." && t.1));
}

#[test]
fn errors_become_toasts_and_current_track_error_is_shown_in_the_player() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("tocando", |r| r.playback.playing);
    let current = h.ctrl.engine().state().unwrap();
    let id = current.current_item().map(|i| i.id).unwrap();
    h.ctrl.on_engine(PlaybackEvent::Error { id: Some(id), message: "unsupported codec".into() });
    let r = h.rec.borrow();
    assert!(r.toasts.last().unwrap().1, "toast de erro");
    assert!(r.toasts.last().unwrap().0.ends_with("O formato deste arquivo não é suportado."), "{:?}", r.toasts.last());
    assert!(r.playback.error.contains("não é suportado"));
    drop(r);
    h.ctrl.on_engine(PlaybackEvent::Error { id: None, message: "audio output error: ALSA function 'snd_pcm_open' failed".into() });
    let last = h.rec.borrow().toasts.last().unwrap().0.clone();
    assert!(last.starts_with("Erro de reprodução") && last.contains("saída de áudio"), "{last}");
}

#[test]
fn adding_a_missing_folder_is_reported_not_fatal() {
    let mut h = harness();
    h.ctrl.add_folder_path(PathBuf::from("/nao/existe/mesmo"));
    h.pump_until("aviso", |r| r.toasts.iter().any(|t| t.1 && t.0.contains("não existe")));
}

#[test]
fn preferences_are_applied_and_persisted() {
    let mut h = harness();
    h.ctrl.set_theme(ThemePref::Light);
    h.ctrl.set_accent_pref(AccentPref::Brand);
    h.ctrl.set_reduce_transparency(true);
    h.ctrl.set_reduce_motion(true);
    let prefs = h.rec.borrow().prefs.unwrap();
    assert_eq!((prefs.theme, prefs.accent, prefs.reduce_transparency, prefs.reduce_motion), (ThemePref::Light, AccentPref::Brand, true, true));
    assert!(!h.rec.borrow().accent.unwrap().1, "acento da marca: sem acento dinâmico");
    let saved = Config::load(&h._dir.path().join("config.toml")).unwrap();
    assert_eq!(saved.theme, ThemePref::Light);
    assert_eq!(saved.reduce_motion, Some(true));
    // o sistema só vale enquanto o usuário não escolheu
    h.ctrl.handle(Msg::System(SystemPrefs { reduce_motion: Some(false) }));
    assert!(h.rec.borrow().prefs.unwrap().reduce_motion);
}

#[test]
fn system_reduce_motion_applies_until_the_user_decides() {
    let mut h = harness();
    h.ctrl.handle(Msg::System(SystemPrefs { reduce_motion: Some(true) }));
    assert!(h.rec.borrow().prefs.unwrap().reduce_motion);
    h.ctrl.set_reduce_motion(false);
    h.ctrl.handle(Msg::System(SystemPrefs { reduce_motion: Some(true) }));
    assert!(!h.rec.borrow().prefs.unwrap().reduce_motion);
}

#[test]
fn toggle_play_with_nothing_loaded_starts_the_first_recent() {
    let mut h = harness();
    assert!(!h.rec.borrow().playback.has_now);
    h.ctrl.toggle_play();
    h.pump_until("começou", |r| r.playback.playing);
}

#[test]
fn volume_is_remembered_in_the_config() {
    let mut h = harness();
    h.ctrl.set_volume(0.4);
    assert!((h.ctrl.config().volume - 0.4).abs() < 1e-6);
}

#[test]
fn loading_a_new_list_while_playing_replaces_the_queue() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("fila", |r| r.queue.len() >= 3);
    let id = track_id_by_title(&h, "Horizonte");
    h.ctrl.play_track(&id);
    h.pump_until("trocou de faixa", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Horizonte"));
    h.settle(300);
    // "Horizonte" está no meio da lista dos recentes: o que vem depois dela continua na fila
    assert!(!h.rec.borrow().queue.is_empty(), "fila vazia após recarregar: {:?}", h.rec.borrow().queue);
    h.ctrl.play_track(&h.first_recent_id());
    h.settle(400);
    assert!(!h.rec.borrow().queue.is_empty());
}

fn synthetic_items(n: usize) -> Vec<QueueItem> {
    (0..n)
        .map(|i| {
            let path = PathBuf::from(format!("/nao/existe/{i}.flac"));
            let mut t = CoreTrack::new(format!("syn{i}"), format!("Faixa {i}"), TrackSource::Local { path: path.clone() });
            t.duration = Some(Duration::from_secs(180));
            QueueItem::new(t, StreamSource::file(path))
        })
        .collect()
}

#[test]
fn huge_queue_shows_only_the_next_hundred_with_an_and_more_label() {
    let mut h = harness();
    h.ctrl.on_engine(PlaybackEvent::QueueChanged { items: synthetic_items(2_000), current: Some(0) });
    let r = h.rec.borrow();
    assert_eq!(r.queue.len(), 100);
    assert_eq!(r.queue[0], "Faixa 1");
    assert_eq!(r.queue_more, "e mais 1.899 músicas");
    assert!(r.queue_summary.starts_with("A seguir, 1.999 músicas"), "{}", r.queue_summary);
    drop(r);
    // fila curta: sem "e mais"
    h.ctrl.on_engine(PlaybackEvent::QueueChanged { items: synthetic_items(5), current: Some(0) });
    assert!(h.rec.borrow().queue_more.is_empty());
    assert_eq!(h.rec.borrow().queue.len(), 4);
}

#[test]
fn late_track_vms_still_fill_the_cache() {
    let mut h = harness();
    let items = synthetic_items(3);
    // resposta de uma fila antiga (seq 0 != seq atual) chega: o cache a aproveita mesmo assim
    let stale = TrackVm { id: "syn1".into(), title: "Faixa 1".into(), cover: Some(Pixels::new(4, 4)), ..Default::default() };
    h.ctrl.handle(Msg::Job(JobResult::TrackVms { seq: 0, tracks: vec![stale] }));
    h.ctrl.on_engine(PlaybackEvent::QueueChanged { items, current: Some(0) });
    let covers = h.rec.borrow().queue_covers.clone();
    assert_eq!(covers, [true, false], "a faixa 1 tem capa vinda do cache; a 2 não");
}

#[test]
fn a_late_detail_does_not_open_after_changing_tab() {
    let mut h = harness();
    let album_id = kazzu_library::Library::open(h._dir.path().join("data")).unwrap().albums().unwrap().into_iter().next().unwrap().id.0.to_string();
    h.ctrl.open_album(&album_id);
    h.ctrl.navigate(NavTab::Explore); // o usuário saiu antes da resposta chegar
    h.settle(100);
    assert!(!h.rec.borrow().detail_open, "detalhe atrasado abriu por cima da aba");
    // o pedido seguinte (atual) funciona
    h.ctrl.open_album(&album_id);
    h.pump_until("detalhe", |r| r.detail_open);
}

#[test]
fn nan_and_infinite_values_never_panic_or_stick() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("tocando", |r| r.playback.playing && r.progress.0 >= 0.0);
    h.settle(150);
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 9.0] {
        h.ctrl.seek(bad);
        h.ctrl.set_volume(bad);
        h.ctrl.on_engine(PlaybackEvent::VolumeChanged { volume: bad });
    }
    let v = h.ctrl.playback().volume;
    assert!(v.is_finite() && (0.0..=1.0).contains(&v), "volume {v}");
    assert!(h.ctrl.config().volume.is_finite());
    h.ctrl.on_engine(PlaybackEvent::Position { position: Duration::from_secs(1), duration: Some(Duration::ZERO) });
    let p = h.rec.borrow().progress.0;
    assert!(p.is_finite() && (0.0..=1.0).contains(&p));
}

#[test]
fn user_shuffle_survives_loading_a_new_list_and_explicit_shuffle_turns_it_on() {
    let mut h = harness();
    h.ctrl.play_track(&h.first_recent_id());
    h.pump_until("tocando", |r| r.playback.playing);
    h.ctrl.toggle_shuffle();
    h.pump_until("shuffle ligado", |r| r.playback.shuffle);
    h.ctrl.play_track(&track_id_by_title(&h, "Horizonte"));
    h.pump_until("trocou", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Horizonte"));
    h.settle(200);
    assert!(h.rec.borrow().playback.shuffle, "tocar uma lista nova desligou o embaralhar do usuário");
    // desligado pelo usuário, o "aleatório" explícito liga
    h.ctrl.toggle_shuffle();
    h.pump_until("shuffle desligado", |r| !r.playback.shuffle);
    h.ctrl.play_track("shuffle-all");
    h.pump_until("shuffle explícito", |r| r.playback.shuffle);
}

#[test]
fn shutdown_saves_the_config_even_without_a_debounce_tick() {
    let mut h = harness();
    h.ctrl.set_volume(0.4);
    let path = h._dir.path().join("config.toml");
    assert!(Config::load(&path).unwrap().volume > 0.99, "ainda não gravado");
    h.ctrl.flush_config();
    assert!((Config::load(&path).unwrap().volume - 0.4).abs() < 1e-6, "flush grava");
    h.ctrl.set_volume(0.25);
    h.ctrl.shutdown();
    assert!((Config::load(&path).unwrap().volume - 0.25).abs() < 1e-6, "shutdown grava o que faltava");
}


// ---------------------------------------------------------------------------------------------
// Provedor remoto falso (sem rede): o mesmo controller, o mesmo motor, um `Catalog` de mentira.
// ---------------------------------------------------------------------------------------------

use kazzu_core::{Album, AlbumId, AlbumRef, ArtistId, ArtistRef, CoverRef, ProviderKind, StreamQuality, TrackId};
use kazzu_ytm::{AlbumPage, ArtistPage, BrowseRef, Discover, ErrorKind, Item, PlaylistPage, Radio, SearchFilter, Shelf, ShelfPage, Suggestions, YtmError};

#[derive(Default)]
struct FakeState {
    calls: Vec<String>,
    /// erro para todas as chamadas de metadados
    fail_meta: Option<ErrorKind>,
    /// ids cuja resolução falha, com o tipo de erro
    fail_resolve: Vec<(String, ErrorKind)>,
    /// erro só para o rádio
    fail_radio: Option<ErrorKind>,
    liked: Vec<CoreTrack>,
    invalidated: Vec<String>,
    audio: PathBuf,
}

#[derive(Clone)]
struct FakeCatalog(Arc<Mutex<FakeState>>);

fn rtrack(id: &str, title: &str) -> CoreTrack {
    let mut t = CoreTrack::new(id, title, TrackSource::Remote { provider: ProviderKind::YtMusic, id: id.into() });
    t.artists = vec![ArtistRef { id: Some(ArtistId::new("UCnina")), name: "Nina Sabiá".into() }];
    t.album = Some(AlbumRef { id: Some(AlbumId::new("MPREb_calor")), title: "Calor".into() });
    t.cover = Some(CoverRef::Url { url: format!("https://i.ytimg.com/vi/{id}/hq.jpg") });
    t.duration = Some(Duration::from_secs(3));
    t
}

fn ralbum() -> Album {
    Album { id: AlbumId::new("MPREb_calor"), provider: ProviderKind::YtMusic, title: "Calor".into(), artists: vec![ArtistRef::named("Nina Sabiá")], year: Some(2024), track_count: Some(3), cover: Some(CoverRef::Url { url: "https://i.ytimg.com/vi/al/hq.jpg".into() }) }
}

impl FakeCatalog {
    fn new(audio: PathBuf) -> Self {
        Self(Arc::new(Mutex::new(FakeState { audio, ..Default::default() })))
    }

    fn calls(&self) -> Vec<String> {
        self.0.lock().unwrap().calls.clone()
    }

    fn rec(&self, what: &str) -> CatResult<()> {
        let mut s = self.0.lock().unwrap();
        s.calls.push(what.to_owned());
        match s.fail_meta {
            Some(k) => Err(match k {
                ErrorKind::Network => YtmError::Network("fora do ar".into()),
                _ => YtmError::Blocked("recusado".into()),
            }),
            None => Ok(()),
        }
    }
}

impl Catalog for FakeCatalog {
    fn kind(&self) -> ProviderKind {
        ProviderKind::YtMusic
    }
    fn peek_home(&self) -> Option<Vec<Shelf>> {
        self.0.lock().unwrap().calls.push("peek_home".into());
        None
    }
    fn discover(&self) -> CatResult<Discover> {
        self.rec("discover")?;
        let tracks = (0..5).map(|i| Item::Track(rtrack(&format!("v{i}"), &format!("Faixa {i}")))).collect();
        Ok(Discover {
            shelves: vec![
                Shelf { title: "Novos lançamentos".into(), subtitle: None, items: vec![Item::Album(ralbum())], more: None },
                Shelf { title: "Em alta hoje".into(), subtitle: None, items: tracks, more: None },
            ],
            partial_errors: vec![],
        })
    }
    fn explore(&self) -> CatResult<ExploreFeed> {
        self.rec("explore")?;
        Ok(ExploreFeed::default())
    }
    fn browse(&self, _t: &BrowseRef) -> CatResult<ShelfPage> {
        self.rec("browse")?;
        Ok(ShelfPage::default())
    }
    fn search(&self, text: &str, _f: Option<SearchFilter>, _l: usize) -> CatResult<Vec<Item>> {
        self.rec(&format!("search:{text}"))?;
        Ok(vec![Item::Track(rtrack("s1", "Quase Verão")), Item::Album(ralbum())])
    }
    fn suggestions(&self, prefix: &str) -> CatResult<Suggestions> {
        self.rec(&format!("suggest:{prefix}"))?;
        Ok(Suggestions { queries: vec![format!("{prefix} sabiá")], items: vec![] })
    }
    fn album(&self, id: &str) -> CatResult<AlbumPage> {
        self.rec(&format!("album:{id}"))?;
        let tracks = (0..3).map(|i| rtrack(&format!("al{i}"), &format!("Álbum faixa {i}"))).collect();
        Ok(AlbumPage { album: ralbum(), tracks, description: None, summary: Some("3 músicas • 9 minutos".into()), album_type: None, audio_playlist_id: None, related: vec![] })
    }
    fn artist(&self, id: &str) -> CatResult<ArtistPage> {
        self.rec(&format!("artist:{id}"))?;
        Err(YtmError::NotFound("sem artista".into()))
    }
    fn playlist(&self, id: &str) -> CatResult<PlaylistPage> {
        self.rec(&format!("playlist:{id}"))?;
        Err(YtmError::NotFound("sem playlist".into()))
    }
    fn radio(&self, video_id: &str) -> CatResult<Radio> {
        self.rec(&format!("radio:{video_id}"))?;
        if let Some(k) = self.0.lock().unwrap().fail_radio {
            return Err(match k {
                ErrorKind::Network => YtmError::Network("sem rede".into()),
                _ => YtmError::NotFound("sem rádio".into()),
            });
        }
        Ok(Radio { playlist_id: "RD".into(), tracks: (0..4).map(|i| rtrack(&format!("r{i}"), &format!("Parecida {i}"))).collect(), continuation: None })
    }
    fn radio_continuation(&self, r: &Radio) -> CatResult<Radio> {
        self.rec("radio_continuation")?;
        Ok(r.clone())
    }
    fn resolve(&self, track: &CoreTrack, _q: StreamQuality) -> CatResult<Resolved> {
        let id = track.id.as_str().to_owned();
        self.rec(&format!("resolve:{id}")).ok();
        let (audio, fail) = {
            let s = self.0.lock().unwrap();
            (s.audio.clone(), s.fail_resolve.iter().find(|(i, _)| *i == id).map(|(_, k)| *k))
        };
        match fail {
            Some(ErrorKind::Network) => Err(YtmError::Network("sem rede".into())),
            Some(ErrorKind::RegionBlocked) => Err(YtmError::RegionBlocked("região".into())),
            Some(_) => Err(YtmError::Unavailable("removida".into())),
            None => Ok(Resolved { source: StreamSource::file(audio), track: track.clone(), quality_label: "OPUS 160 kbps".into() }),
        }
    }
    fn invalidate(&self, track: &CoreTrack) {
        self.0.lock().unwrap().invalidated.push(track.id.as_str().to_owned());
    }
    fn liked(&self) -> Vec<CoreTrack> {
        self.0.lock().unwrap().liked.clone()
    }
    fn like(&self, track: &CoreTrack) -> CatResult<()> {
        self.0.lock().unwrap().liked.insert(0, track.clone());
        Ok(())
    }
    fn unlike(&self, id: &TrackId) -> CatResult<bool> {
        let mut s = self.0.lock().unwrap();
        let n = s.liked.len();
        s.liked.retain(|t| &t.id != id);
        Ok(n != s.liked.len())
    }
    fn saved_albums(&self) -> Vec<Album> {
        vec![]
    }
    fn save_album(&self, _a: &Album) -> CatResult<()> {
        self.rec("save_album")
    }
    fn remove_album(&self, _id: &str) -> CatResult<bool> {
        Ok(true)
    }
}

struct FakeImages;

impl ImageSource for FakeImages {
    fn get(&self, _url: &str) -> Option<Vec<u8>> {
        let img = image::RgbaImage::from_pixel(64, 64, image::Rgba([220, 90, 60, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).ok()?;
        Some(out.into_inner())
    }
}

/// Imagens que falham nas primeiras `n` tentativas de cada URL (rede ruim).
struct FlakyImages {
    fail_first: usize,
    seen: Mutex<std::collections::HashMap<String, usize>>,
}

impl ImageSource for FlakyImages {
    fn get(&self, url: &str) -> Option<Vec<u8>> {
        let mut seen = self.seen.lock().unwrap();
        let n = seen.entry(url.to_owned()).or_insert(0);
        *n += 1;
        if *n <= self.fail_first {
            return None;
        }
        FakeImages.get(url)
    }
}

fn remote_harness() -> (Harness, FakeCatalog) {
    let audio_dir = tempfile::tempdir().unwrap();
    let audio = audio_dir.path().join("fake.wav");
    write_track(audio_dir.path(), "fake.wav", "x", "x", "x", "x", 1);
    std::mem::forget(audio_dir); // o arquivo precisa durar o teste inteiro (limpo pelo SO)
    let cat = FakeCatalog::new(audio);
    let worker = RemoteWorker::new(Providers::new().with(Arc::new(cat.clone())), CoverFetcher::with_source(None, Box::new(FakeImages)));
    let mut h = harness_with(Some(worker));
    h.pump_until("início carregado", |r| r.home.as_ref().is_some_and(|h| h.state != LoadStateVm::Loading));
    (h, cat)
}

#[test]
fn nothing_touches_the_network_until_the_remote_provider_is_enabled_and_asked() {
    // sem YouTube Music: o catálogo falso nunca é chamado
    let audio = PathBuf::from("/nao/importa.wav");
    let cat = FakeCatalog::new(audio);
    let _worker = RemoteWorker::new(Providers::new().with(Arc::new(cat.clone())), CoverFetcher::with_source(None, Box::new(FakeImages)));
    let mut h = harness();
    h.settle(200);
    assert!(cat.calls().is_empty());
    assert!(h.rec.borrow().home.as_ref().is_some_and(|x| x.state == LoadStateVm::Ready), "sem rede, o Início fica pronto só com o local");
    // ligado: só o Início (cache + feed) é pedido na abertura; nada de busca, rádio ou resolução
    let (h2, cat2) = remote_harness();
    assert_eq!(cat2.calls(), ["peek_home", "discover"]);
    drop(h2);
}

#[test]
fn home_shows_the_feed_with_the_first_run_banner_that_never_comes_back() {
    let (mut h, _cat) = remote_harness();
    {
        let r = h.rec.borrow();
        let home = r.home.as_ref().unwrap();
        assert_eq!(home.state, LoadStateVm::Ready);
        assert!(home.banner, "aviso de primeiro uso");
        assert_eq!(home.source_label, "Do YouTube Music");
        assert_eq!(home.quick_picks.len(), 5);
        assert_eq!(home.quick_picks[0].why, "Em alta hoje");
        assert_eq!(home.highlights[0].kind, "album");
    }
    h.ctrl.dismiss_first_run_banner();
    assert!(!h.rec.borrow().home.as_ref().unwrap().banner);
    assert!(h.ctrl.config().first_run_dismissed);
    let saved = Config::load(&h._dir.path().join("config.toml")).unwrap();
    assert!(saved.first_run_dismissed, "gravado: nunca volta");
}

#[test]
fn remote_covers_arrive_after_the_text() {
    let (mut h, _cat) = remote_harness();
    h.pump_until("capas", |r| r.home.as_ref().is_some_and(|h| h.quick_picks.iter().all(|t| t.cover.is_some())));
}

#[test]
fn offline_home_without_cache_is_an_offline_state_and_recovers_when_the_connection_returns() {
    let audio = PathBuf::from("/x.wav");
    let cat = FakeCatalog::new(audio);
    cat.0.lock().unwrap().fail_meta = Some(ErrorKind::Network);
    let worker = RemoteWorker::new(Providers::new().with(Arc::new(cat.clone())), CoverFetcher::with_source(None, Box::new(FakeImages)));
    let mut h = harness_with(Some(worker));
    h.pump_until("offline", |r| r.home.as_ref().is_some_and(|x| x.state == LoadStateVm::Offline));
    {
        let r = h.rec.borrow();
        let home = r.home.as_ref().unwrap();
        assert_eq!(home.error_title, "Sem conexão");
        assert_eq!(home.error_code, "ytm:network");
        assert_eq!(r.online.last(), Some(&false));
    }
    cat.0.lock().unwrap().fail_meta = None;
    h.ctrl.retry_home();
    h.pump_until("de volta", |r| r.home.as_ref().is_some_and(|x| x.state == LoadStateVm::Ready));
    assert_eq!(h.rec.borrow().online.last(), Some(&true));
    assert!(h.rec.borrow().toasts.iter().any(|t| t.0 == "Conexão de volta"));
}

#[test]
fn a_failed_refresh_keeps_the_content_with_a_stale_warning() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_meta = Some(ErrorKind::Blocked);
    // um erro com conteúdo em cache mantém a tela: simulamos o cache já tendo mostrado o feed
    h.ctrl.handle(Msg::Job(JobResult::Remote(kazzu_app::remote::RemoteResult::HomeFailed { seq: 1, kind: ErrorKind::Blocked, message: "x".into() })));
    let r = h.rec.borrow();
    let home = r.home.as_ref().unwrap();
    assert_eq!(home.state, LoadStateVm::Ready);
    assert!(home.stale && !home.quick_picks.is_empty());
    assert_eq!(home.error_title, "O YouTube Music não respondeu");
}

#[test]
fn playing_a_remote_pick_resolves_off_the_ui_thread_and_continues_with_the_radio() {
    let (mut h, cat) = remote_harness();
    let id = h.rec.borrow().home.as_ref().unwrap().quick_picks[1].id.clone();
    assert_eq!(id, "yt:v1");
    h.ctrl.play_track(&id);
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Faixa 1"));
    {
        let r = h.rec.borrow();
        assert_eq!(r.playback.origin, "Início");
        assert_eq!(r.playback.now.as_ref().unwrap().source, "YouTube Music");
    }
    // lista de uma só faixa: o controller pede o rádio e mantém duas faixas resolvidas à frente
    h.pump_until("rádio na fila", |r| r.queue_titles.len() >= 2);
    assert!(cat.calls().iter().any(|c| c == "radio:v1"), "{:?}", cat.calls());
    let resolves = cat.calls().iter().filter(|c| c.starts_with("resolve:")).count();
    assert!(resolves <= 4, "não resolve a lista toda: {resolves}");
    assert_eq!(h.rec.borrow().playback.quality_label, "OPUS 160 kbps");
}

#[test]
fn an_album_plays_in_order_and_only_two_tracks_are_resolved_ahead() {
    let (mut h, cat) = remote_harness();
    h.ctrl.open_item("album", "yt:MPREb_calor");
    h.pump_until("álbum", |r| r.detail_vm.as_ref().is_some_and(|d| d.state == LoadStateVm::Ready && d.tracks.len() == 3));
    assert_eq!(h.rec.borrow().detail_vm.as_ref().unwrap().subtitle, "Nina Sabiá, 2024, 3 músicas, 9 minutos");
    h.ctrl.play_all(false);
    h.pump_until("primeira", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 0"));
    h.settle(300);
    let resolved: Vec<String> = cat.calls().into_iter().filter(|c| c.starts_with("resolve:")).collect();
    assert_eq!(resolved, ["resolve:al0", "resolve:al1", "resolve:al2"], "faixa atual + 2 à frente, na ordem");
    assert_eq!(h.rec.borrow().playback.origin, "Calor");
}

#[test]
fn unavailable_tracks_are_skipped_with_a_calm_message_and_network_failures_wait_for_the_user() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("al1".into(), ErrorKind::RegionBlocked)];
    h.ctrl.open_item("album", "yt:MPREb_calor");
    h.pump_until("álbum", |r| r.detail_vm.as_ref().is_some_and(|d| d.tracks.len() == 3));
    h.ctrl.play_all(false);
    h.pump_until("pulou", |r| r.toasts.iter().any(|t| t.0.starts_with("Pulei “Álbum faixa 1”")));
    h.pump_until("a terceira entrou", |r| r.queue_titles.iter().any(|t| t == "Álbum faixa 2") || r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 2"));
    // rede caiu ao resolver a próxima: não martela o servidor, avisa e oferece tentar de novo
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("al1".into(), ErrorKind::Network)];
    h.ctrl.open_item("album", "yt:MPREb_calor");
    h.pump_until("álbum", |r| r.detail_vm.as_ref().is_some_and(|d| d.tracks.len() == 3));
    h.ctrl.play_all(false);
    h.pump_until("aviso de rede", |r| r.toast_actions.iter().any(|a| a == "Tentar de novo"));
    let before = cat.calls().iter().filter(|c| *c == "resolve:al1").count();
    h.settle(300);
    assert_eq!(cat.calls().iter().filter(|c| *c == "resolve:al1").count(), before, "sem martelar");
    cat.0.lock().unwrap().fail_resolve.clear();
    h.ctrl.toast_action();
    h.pump_until("retomou", |r| r.queue_titles.iter().any(|t| t == "Álbum faixa 1"));
}

#[test]
fn a_failed_start_offers_try_again() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("v0".into(), ErrorKind::Network)];
    h.ctrl.play_track("yt:v0");
    h.pump_until("aviso", |r| r.toast_actions.iter().any(|a| a == "Tentar de novo"));
    assert!(h.rec.borrow().playback.now.is_none());
    cat.0.lock().unwrap().fail_resolve.clear();
    h.ctrl.toast_action();
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Faixa 0"));
}

#[test]
fn an_expired_stream_is_resolved_again_and_a_network_failure_asks_to_retry() {
    let (mut h, cat) = remote_harness();
    h.ctrl.play_track("yt:v2");
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Faixa 2"));
    let item = h.ctrl.engine().state().unwrap().current_item().unwrap().clone();
    h.ctrl.handle(Msg::Stream(kazzu_audio::StreamEvent::Expired { id: item.id, position: Duration::from_secs(1) }));
    h.settle(200);
    assert_eq!(cat.0.lock().unwrap().invalidated, ["v2"]);
    assert!(cat.calls().iter().filter(|c| *c == "resolve:v2").count() >= 2, "resolveu de novo");

    h.ctrl.handle(Msg::Stream(kazzu_audio::StreamEvent::NetworkFailed { id: item.id, position: Duration::from_secs(1), message: "caiu".into() }));
    let r = h.rec.borrow();
    assert_eq!(r.online.last(), Some(&false));
    assert_eq!(r.playback.error, "Sem conexão. Toque para tentar de novo.");
    assert_eq!(r.toast_actions.last().map(String::as_str), Some("Tentar de novo"));
}

#[test]
fn remote_likes_are_kept_on_the_device_and_can_be_undone() {
    let (mut h, cat) = remote_harness();
    h.ctrl.toggle_like("yt:v3");
    h.settle(100);
    assert_eq!(cat.0.lock().unwrap().liked.len(), 1);
    assert_eq!(h.rec.borrow().toasts.last().unwrap().0, "Adicionada às Curtidas");
    assert_eq!(h.rec.borrow().toast_actions.last().map(String::as_str), Some("Desfazer"));
    h.ctrl.toast_action();
    h.settle(100);
    assert!(cat.0.lock().unwrap().liked.is_empty(), "desfeito");
}

#[test]
fn search_merges_remote_results_with_suggestions_and_the_offline_fallback() {
    let (mut h, cat) = remote_harness();
    h.ctrl.search("hori");
    h.pump_until("resultados", |r| r.search_vm.as_ref().is_some_and(|s| s.state == SearchStateVm::Results && s.songs.iter().any(|t| t.title == "Horizonte") && s.songs.iter().any(|t| t.title == "Quase Verão")));
    let vm = h.rec.borrow().search_vm.clone().unwrap();
    assert_eq!(vm.suggestions, ["hori sabiá"]);
    assert_eq!(vm.best.as_ref().map(|b| b.kind.as_str()), Some("track"));
    // tocar um resultado remoto: lista de uma faixa e rádio
    h.ctrl.play_track("yt:s1");
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Quase Verão"));
    // sem rede: só o local aparece e a tela avisa
    cat.0.lock().unwrap().fail_meta = Some(ErrorKind::Network);
    h.ctrl.search("hori");
    h.pump_until("só local", |r| r.search_vm.as_ref().is_some_and(|s| s.state == SearchStateVm::Results && s.songs.iter().all(|t| t.title == "Horizonte") && !s.songs.is_empty()));
    assert_eq!(h.rec.borrow().online.last(), Some(&false));
    h.ctrl.search("zzzzqq");
    h.pump_until("offline", |r| r.search_vm.as_ref().is_some_and(|s| s.state == SearchStateVm::Offline));
}

#[test]
fn remote_tracks_can_be_added_to_a_local_playlist_and_shown_back() {
    let (mut h, _cat) = remote_harness();
    h.ctrl.create_playlist("Estrada", "yt:v4");
    h.pump_until("playlist", |r| r.playlists.iter().any(|p| p == "Estrada"));
    let pid = {
        let lib = kazzu_library::Library::open(h._dir.path().join("data")).unwrap();
        lib.playlists().unwrap().into_iter().find(|p| p.name == "Estrada").unwrap().id.0.to_string()
    };
    h.ctrl.open_playlist(&pid);
    h.pump_until("detalhe", |r| r.detail_vm.as_ref().is_some_and(|d| d.title == "Estrada" && d.tracks.len() == 1));
    assert_eq!(h.rec.borrow().detail_vm.as_ref().unwrap().tracks[0].title, "Faixa 4");
    // e toca
    h.ctrl.play_all(false);
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Faixa 4"));
}

#[test]
fn stream_quality_and_autoplay_are_persisted_and_used() {
    let (mut h, _cat) = remote_harness();
    h.ctrl.set_stream_quality(kazzu_app::config::StreamQualityPref::Low);
    h.ctrl.set_autoplay_radio(false);
    let saved = Config::load(&h._dir.path().join("config.toml")).unwrap();
    assert_eq!(saved.stream_quality, kazzu_app::config::StreamQualityPref::Low);
    assert!(!saved.autoplay_radio);
    // sem continuar com parecidas: a lista de uma faixa acaba e o rádio não é pedido
    h.ctrl.play_track("yt:v0");
    h.pump_until("tocando", |r| r.playback.now.is_some());
    h.settle(300);
}

#[test]
fn slow_frames_offer_less_transparency_once() {
    let (mut h, _cat) = remote_harness();
    h.ctrl.handle(Msg::SlowFrames);
    assert!(h.rec.borrow().perf_offer);
    h.ctrl.perf_offer_accept();
    assert!(!h.rec.borrow().perf_offer);
    assert!(h.rec.borrow().prefs.unwrap().reduce_transparency);
    h.ctrl.handle(Msg::SlowFrames);
    assert!(!h.rec.borrow().perf_offer, "depois de decidir, não insiste");
}


// ---------------------------------------------------------------------------------------------
// Correções da revisão independente
// ---------------------------------------------------------------------------------------------

fn open_album_and_wait(h: &mut Harness) {
    h.ctrl.open_item("album", "yt:MPREb_calor");
    h.pump_until("álbum", |r| r.detail_vm.as_ref().is_some_and(|d| d.state == LoadStateVm::Ready && d.tracks.len() == 3));
}

#[test]
fn the_engine_stopping_before_the_next_track_arrives_resumes_when_it_does() {
    let (mut h, _cat) = remote_harness();
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    // só a faixa pedida chega; as próximas ficam "a caminho" (rede lenta)
    assert!(h.deliver_one_job());
    h.pump_until_engine("primeira tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 0"));
    // o clipe de 3 s acaba (8× mais rápido) com a próxima ainda não entregue: o motor para no fim da fila
    h.pump_engine_until("fila acabou", |r| !r.playback.playing && r.playback.loading);
    assert!(h.rec.borrow().playback.loading, "mostra carregando, não parado");
    // a próxima chega depois: toca sozinha
    h.pump_until("retomou", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 1") && r.playback.playing);
}

#[test]
fn a_stalled_queue_recovers_by_itself_when_the_connection_returns() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("al1".into(), ErrorKind::Network)];
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    h.pump_until("aviso de rede", |r| r.toast_actions.iter().any(|a| a == "Tentar de novo"));
    cat.0.lock().unwrap().fail_resolve.clear();
    // outra chamada dá certo (aqui, recarregar o Início): a conexão voltou e a fila retoma SEM clicar em nada
    h.ctrl.retry_home();
    h.pump_until("retomou sozinha", |r| r.queue_titles.iter().any(|t| t == "Álbum faixa 1"));
    assert_eq!(h.rec.borrow().online.last(), Some(&true));
}

#[test]
fn expired_urls_are_renewed_at_most_twice_a_minute_then_the_user_decides() {
    let (mut h, cat) = remote_harness();
    h.ctrl.play_track("yt:v2");
    h.pump_until("tocando", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Faixa 2"));
    let id = h.ctrl.engine().state().unwrap().current_item().unwrap().id;
    let resolves = |c: &FakeCatalog| c.calls().iter().filter(|x| *x == "resolve:v2").count();
    let before = resolves(&cat);
    for _ in 0..5 {
        h.ctrl.handle(Msg::Stream(kazzu_audio::StreamEvent::Expired { id, position: Duration::from_secs(1) }));
        h.settle(60);
    }
    assert_eq!(resolves(&cat) - before, 2, "no máximo 2 renovações automáticas por minuto");
    assert_eq!(h.rec.borrow().toast_actions.last().map(String::as_str), Some("Tentar de novo"));
    h.ctrl.toast_action();
    h.settle(100);
    assert_eq!(resolves(&cat) - before, 3, "por ação do usuário, mais uma");
}

#[test]
fn jumping_ahead_discards_the_resolutions_of_what_was_skipped() {
    // (a) pula enquanto a faixa pedida ainda resolve: a antiga nunca toca
    let (mut h, _cat) = remote_harness();
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    h.ctrl.queue_jump("yt:al2");
    h.pump_until("a escolhida", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 2"));
    h.settle(200);
    assert!(!h.rec.borrow().playing_marks.iter().any(|m| m.as_deref() == Some("yt:al0")), "a faixa pulada tocou");
    // (b) a "próxima" que estava resolvendo quando o usuário pulou não entra no motor
    let (mut h, _cat) = remote_harness();
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    assert!(h.deliver_one_job()); // al0 chega
    h.pump_engine_only();
    h.pump_engine_only(); // QueueChanged → pede al1 (resultado fica parado)
    h.ctrl.queue_jump("yt:al2");
    h.pump_until("a escolhida", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 2"));
    h.settle(200);
    let queue: Vec<String> = h.ctrl.engine().state().unwrap().queue.iter().map(|i| i.track.title.clone()).collect();
    assert!(!queue.iter().any(|t| t == "Álbum faixa 1"), "a próxima pulada entrou no motor: {queue:?}");
}

#[test]
fn queue_actions_use_the_row_id_even_when_the_tail_is_not_in_the_engine_yet() {
    let (mut h, _cat) = remote_harness();
    h.ctrl.play_track("yt:v1");
    // faixa + rádio (4): o motor tem 2 à frente e as outras 2 ainda esperam
    h.pump_until("fila", |r| r.queue_titles.len() >= 4);
    assert_eq!(h.rec.borrow().queue_titles, ["Parecida 0", "Parecida 1", "Parecida 2", "Parecida 3"]);
    // mover para cima a primeira que ainda não entrou: passa à frente da última que o motor já tem
    h.ctrl.queue_move_up("yt:r2");
    h.pump_until("reordenou", |r| r.queue_titles == ["Parecida 0", "Parecida 2", "Parecida 1", "Parecida 3"]);
    // remover por id uma que ainda não entrou no motor
    h.ctrl.queue_remove("yt:r3");
    h.pump_until("removeu", |r| r.queue_titles == ["Parecida 0", "Parecida 2", "Parecida 1"]);
    // remover por id uma do motor
    h.ctrl.queue_remove("yt:r2");
    h.pump_until("removeu 2", |r| !r.queue_titles.iter().any(|t| t == "Parecida 2"));
}

#[test]
fn covers_that_failed_or_were_evicted_are_requested_again_later() {
    let audio_dir = tempfile::tempdir().unwrap();
    write_track(audio_dir.path(), "fake.wav", "x", "x", "x", "x", 1);
    let cat = FakeCatalog::new(audio_dir.path().join("fake.wav"));
    let flaky = FlakyImages { fail_first: 1, seen: Mutex::new(Default::default()) };
    let worker = RemoteWorker::new(Providers::new().with(Arc::new(cat.clone())), CoverFetcher::with_source(None, Box::new(flaky)));
    let mut h = harness_with(Some(worker));
    h.ctrl.set_cover_retry(Duration::ZERO);
    h.pump_until("início", |r| r.home.as_ref().is_some_and(|x| x.state == LoadStateVm::Ready));
    h.settle(300);
    assert!(h.rec.borrow().home.as_ref().unwrap().quick_picks.iter().all(|t| t.cover.is_none()), "a 1ª tentativa de cada capa falhou");
    // qualquer atualização da tela refaz o pedido das que faltam (e agora dá certo)
    h.ctrl.dismiss_first_run_banner();
    h.pump_until("capas depois", |r| r.home.as_ref().is_some_and(|x| x.quick_picks.iter().all(|t| t.cover.is_some())));
}

#[test]
fn a_new_batch_of_covers_does_not_rebuild_screens_it_does_not_touch() {
    let (mut h, _cat) = remote_harness();
    h.pump_until("capas", |r| r.home.as_ref().is_some_and(|h| h.quick_picks.iter().all(|t| t.cover.is_some())));
    h.settle(200);
    let before = h.rec.borrow().search_sets;
    // mais capas chegam para o Início (um novo feed): a Busca, que não usa essas URLs, não é refeita
    h.ctrl.retry_home();
    h.pump_until("início de novo", |r| r.home.as_ref().is_some_and(|h| h.quick_picks.iter().all(|t| t.cover.is_some())));
    h.settle(200);
    assert_eq!(h.rec.borrow().search_sets, before, "a Busca foi refeita sem necessidade");
}

#[test]
fn a_radio_network_error_does_not_exhaust_the_radio_and_it_comes_back() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_radio = Some(ErrorKind::Network);
    h.ctrl.play_track("yt:v0");
    h.pump_until("tocando", |r| r.playback.now.is_some());
    h.settle(300);
    assert!(h.rec.borrow().queue_titles.is_empty());
    cat.0.lock().unwrap().fail_radio = None;
    h.ctrl.retry_home(); // a conexão volta
    h.pump_until("rádio voltou", |r| r.queue_titles.len() >= 2);
}

#[test]
fn an_unavailable_start_keeps_showing_loading_while_the_next_one_resolves() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("al0".into(), ErrorKind::RegionBlocked)];
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    assert!(h.deliver_one_job()); // a falha da primeira
    assert!(h.rec.borrow().playback.loading, "continua carregando enquanto resolve a seguinte");
    assert!(h.rec.borrow().toasts.iter().any(|t| t.1));
    h.pump_until("a seguinte toca", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 1"));
}

#[test]
fn repeat_all_does_not_loop_over_tracks_that_are_unavailable() {
    let (mut h, cat) = remote_harness();
    cat.0.lock().unwrap().fail_resolve = vec![("al0".into(), ErrorKind::RegionBlocked)];
    h.ctrl.cycle_repeat(); // repetir tudo
    open_album_and_wait(&mut h);
    h.ctrl.play_all(false);
    h.pump_until("a seguinte toca", |r| r.playback.now.as_ref().is_some_and(|t| t.title == "Álbum faixa 1"));
    // dá algumas voltas na lista (clipes de 3 s a 8×): a indisponível não é pedida de novo
    h.settle(2500);
    let tries = cat.calls().iter().filter(|c| *c == "resolve:al0").count();
    assert_eq!(tries, 1, "a faixa indisponível voltou a ser pedida {tries} vezes");
}

#[test]
fn the_player_does_not_call_a_youtube_stream_original() {
    let (mut h, _cat) = remote_harness();
    h.ctrl.play_track("yt:v3");
    h.pump_until("nota de qualidade", |r| r.playback.output_note == "Qualidade do YouTube Music, sem equalizador");
}
