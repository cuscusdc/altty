//! Teste AO VIVO contra o YouTube Music real (rede). Ignorado por padrão:
//!
//! ```text
//! cargo test -p kazzu-app --test live_ytm -- --ignored --nocapture
//! ```
//!
//! Exercita o caminho do app: `YtmCatalog` → Início (view-model) → busca → resolve → motor de áudio (saída simulada).

use std::collections::HashSet;
use std::time::{Duration, Instant};

use kazzu_app::catalog::{Catalog, YtmCatalog};
use kazzu_app::vm::{HomeSectionVm, LoadStateVm};
use kazzu_app::ytm_vm;
use kazzu_audio::{Engine, EngineConfig, NullSink};
use kazzu_core::{PlaybackEvent, PlaybackStatus, QueueItem, StreamQuality};
use kazzu_ytm::Item;

#[test]
#[ignore = "usa a rede real (YouTube Music)"]
fn live_home_search_resolve_and_play() {
    let dir = tempfile::tempdir().unwrap();
    let cat = YtmCatalog::open(dir.path()).unwrap();

    // Início
    let discover = cat.discover().expect("início");
    let mut tracks = vec![];
    let home = ytm_vm::build_home(&discover.shelves, HomeSectionVm::ForYou, &HashSet::new(), &mut tracks);
    assert_eq!(home.state, LoadStateVm::Ready, "o feed do Início veio vazio");
    println!("Início: {} destaques, {} escolhas, {} mixes, {} álbuns", home.highlights.len(), home.quick_picks.len(), home.mixes.len(), home.albums.len());

    // Busca → primeira música → stream
    let items = cat.search("nirvana", None, 20).expect("busca");
    let track = items.iter().find_map(|i| if let Item::Track(t) = i { Some(t.clone()) } else { None }).expect("alguma música");
    let resolved = cat.resolve(&track, StreamQuality::Original).expect("resolve");
    println!("{} → {}", track.title, resolved.quality_label);
    assert!(!resolved.quality_label.is_empty());

    // Motor (sem placa de som): abre o stream HTTP e chega a Playing
    let engine = Engine::builder().sink(NullSink::new()).config(EngineConfig::default()).spawn().unwrap();
    engine.load(vec![QueueItem::new(resolved.track, resolved.source)], 0);
    let t0 = Instant::now();
    let mut playing = false;
    while t0.elapsed() < Duration::from_secs(30) && !playing {
        if let Ok(ev) = engine.events().recv_timeout(Duration::from_millis(200)) {
            playing = matches!(ev, PlaybackEvent::StatusChanged { status: PlaybackStatus::Playing });
        }
    }
    assert!(playing, "o stream remoto não chegou a tocar em 30 s");
    println!("tocando depois de {:?}", t0.elapsed());
}
