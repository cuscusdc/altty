# kazzu-app

App do Kazzu (Slint + Rust), mobile-first: Android, depois desktop (e iOS). YouTube Music é o provedor padrão, sem login. Binário: `kazzu`.

```
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo   # YouTube Music + música de demonstração gerada por programa
cargo run -p kazzu-app --release                                                # YouTube Music + sua biblioteca
cargo run -p kazzu-app --release -- --no-youtube                                # só arquivos locais (nenhuma chamada de rede)
cargo run -p kazzu-app -- --help
```

Estrutura:

| Caminho | O que é |
|---|---|
| `ui/` | Telas Slint v2. `theme/` = tokens, átomos e contrato (cópias do design, ver `docs/UI_NOTES.md`); `components/`, `screens/`; `assets/` = ícones, logo, fontes (OFL) |
| `src/controller/` | Lógica: motor de áudio + biblioteca + YouTube Music + tela. `mod.rs` (estado, motor, comandos), `remote.rs` (Início, busca, fila sob demanda com rádio, URL vencida). Sem Slint, testável |
| `src/catalog.rs`, `src/remote.rs`, `src/ytm_vm.rs`, `src/covers.rs` | Provedores remotos: trait `Catalog` + registro, trabalho de rede fora da thread de UI, view-models, capas |
| `src/view.rs`, `src/vm.rs` | Fronteira com a tela (`View`) e dados prontos para ela |
| `src/slint_view.rs`, `src/app.rs` | Ligação com o Slint e montagem do app (threads, eventos do motor e do streaming, callbacks, medição de quadros) |
| `src/worker.rs` | Biblioteca, scan, playlists e despacho dos jobs de rede |
| `src/accent.rs`, `src/art.rs` | Acento da capa e cores do palco (OKLCH), fundo desfocado |
| `src/demo.rs`, `src/snapshot.rs` | Biblioteca de demonstração (feature `demo`); `--snapshot <dir>` captura todas as telas |
| `tests/` | `controller.rs` (sem janela, provedor falso), `worker.rs`, `ui_render.rs`, `design_sync.rs`, `live_ytm.rs` (ao vivo, ignorado) |

A galeria de UI com dados falsos (todos os estados, capturas, prova de vidro) é o crate `kazzu-ui-gallery`.

Notas de decisão, desvios do design e lacunas de API: `docs/UI_NOTES.md`.
