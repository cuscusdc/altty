# kazzu-app

App desktop do Kazzu (Slint + Rust). Binário: `kazzu`.

```
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo   # música de demonstração gerada por programa
cargo run -p kazzu-app --release                                                # sua biblioteca (estado vazio na primeira vez)
cargo run -p kazzu-app -- --help
```

Estrutura:

| Caminho | O que é |
|---|---|
| `ui/` | Telas Slint. `theme/` = tokens, átomos e contrato do design (cópias, ver `docs/UI_NOTES.md`); `components/`, `screens/`; `assets/` = ícones, logo, fontes (OFL) |
| `src/controller.rs` | Lógica: motor de áudio + biblioteca + tela. Sem Slint, testável |
| `src/view.rs`, `src/vm.rs` | Fronteira com a tela (`View`) e dados prontos para ela |
| `src/slint_view.rs`, `src/app.rs` | Ligação com o Slint e montagem do app (threads, eventos do motor, callbacks) |
| `src/worker.rs` | Biblioteca, scan e capas fora da thread de UI |
| `src/accent.rs`, `src/art.rs` | Acento da capa (OKLCH) e fundo desfocado do player |
| `src/demo.rs` | Biblioteca de demonstração (feature `demo`, ligada por padrão) |
| `src/snapshot.rs` | `--snapshot <dir>`: captura todas as telas |
| `tests/` | `controller.rs` (sem janela) e `ui_render.rs` (todas as telas, renderer de software) |

Notas de decisão, desvios do design e lacunas de API: `docs/UI_NOTES.md`.
