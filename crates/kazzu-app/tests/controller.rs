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
    }
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
        r.queue_summary = q.summary.clone();
        r.queue_more = q.more_label.clone();
        r.queue_covers = q.tracks.iter().map(|t| t.cover.is_some()).collect();
    }
    fn set_search(&self, s: &SearchVm) {
        self.0.borrow_mut().search = Some((s.state, s.songs.iter().map(|t| t.title.clone()).collect(), s.recent.clone()));
    }
    fn set_detail(&self, d: &DetailVm) {
        self.0.borrow_mut().detail = Some((d.title.clone(), d.tracks.iter().map(|t| t.title.clone()).collect()));
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
    fn toast(&self, text: &str, error: bool) {
        self.0.borrow_mut().toasts.push((text.into(), error));
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
}

impl JobRunner for InlineRunner {
    fn submit(&self, job: Job) {
        let out = self.out.clone();
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
    let dir = tempfile::tempdir().unwrap();
    let music = make_music(dir.path());
    let lib = kazzu_library::Library::open(dir.path().join("data")).unwrap();
    let results: Results = Arc::default();
    let runner = InlineRunner { store: Mutex::new(Store::new(lib)), out: results.clone() };
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
    assert!(r.home_albums.contains(&"Aurora".to_owned()) && r.home_albums.contains(&"Brasa".to_owned()));
    assert_eq!(r.collections.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["downloads", "liked", "albums", "playlists"]);
    assert_eq!(r.collections[2].1, "2 álbuns");
    assert_eq!(r.collections[1].1, "0 músicas");
    let (settings, providers) = r.settings.clone().unwrap();
    assert_eq!(settings.summary, "1 pasta, 4 músicas");
    assert!(settings.has_library);
    assert_eq!(providers.len(), 4);
    assert!(providers[0].connected && providers.iter().skip(1).all(|p| !p.available), "só a biblioteca local está ativa");
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
    assert_eq!(r.playback.source_label, "Tocando de Recentes");
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
    // sem a fonte Local marcada, nada a mostrar
    h.ctrl.search("aurora");
    h.pump_until("resultados 2", |r| r.search.as_ref().is_some_and(|s| s.0 == SearchStateVm::Results));
    h.ctrl.search_filters(0, false);
    assert_eq!(h.rec.borrow().search.as_ref().unwrap().0, SearchStateVm::Empty);
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
    assert_eq!(h.rec.borrow().playback.source_label, "Tocando de Busca");
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
    assert_eq!(h.rec.borrow().playback.source_label, "Tocando de Aurora");
    // destaque "album:<id>": o rótulo vira o título do álbum
    h.ctrl.play_track(&format!("album:{album_id}"));
    h.pump_until("rótulo do álbum", |r| r.playback.source_label == "Tocando de Aurora");
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
    h.ctrl.queue_remove(0);
    h.pump_until("removeu", |r| r.queue.len() == before.len() - 1);
    assert_eq!(h.rec.borrow().queue.first(), before.get(1));
    h.ctrl.queue_jump(0);
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
    h.pump_until("toast", |r| r.toasts.iter().any(|t| t.0 == "Música tocará em seguida."));
    h.pump_until("fila com a faixa", |r| r.queue.first().is_some_and(|t| t == "Calor de Março"));
    h.ctrl.queue_track(&id, QueueMode::Enqueue);
    h.pump_until("toast 2", |r| r.toasts.iter().any(|t| t.0 == "Música adicionada à fila."));
}

#[test]
fn playlists_can_be_created_and_empty_names_are_refused() {
    let mut h = harness();
    h.ctrl.create_playlist("Foco");
    h.pump_until("playlist", |r| r.playlists == ["Foco"]);
    assert_eq!(h.rec.borrow().collections[3].1, "1 playlist");
    h.ctrl.create_playlist("   ");
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
