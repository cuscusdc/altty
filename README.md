# Kazzu

Player/cliente de música em **Rust + Slint**: visual Apple-like com glassmorphism, áudio em qualidade original (FLAC/WAV/Opus), sem telemetria.

Repositório: `cuscusdc/altty`. Mapa mental do autor: [`docs/mindmap/Kazzu.canvas`](docs/mindmap/Kazzu.canvas) (abra no Obsidian). Requisitos e regras: [`docs/BRIEF.md`](docs/BRIEF.md).

## Estado

Etapa 1 (Fundação) concluída: motor de áudio bit-perfect, biblioteca local, design system e UI Slint tocando música local. Próxima: Etapa 2 (provedores). Ver [`docs/BRIEF.md`](docs/BRIEF.md).

## Build

```sh
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo
cargo test --workspace
```

Veja `crates/kazzu-app/README.md` para flags (`--null-audio`, `--renderer`, `--snapshot`).

## Material de referência (não versionado, `.refs/`)

Para reconstruir a pasta `.refs/` (somente leitura, inspiração; nada é copiado para o app):

- https://github.com/Leonxlnx/taste-skill (MIT)
- https://github.com/Nutlope/hallmark (MIT)
- https://github.com/metrolistgroup/metrolist (GPL-3.0, só inspiração conceitual)
- Imagens de referência de UI: ver `docs/mindmap/Kazzu.canvas` (screenshots de Bitchord, Umihi Music e Intertune, ficam no vault Obsidian do autor).
