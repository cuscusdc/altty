# design/

Design system do Kazzu. Comece por `DESIGN.md`; o implementador de UI segue `SLINT_SPEC.md`.

| Pasta/arquivo | Conteúdo |
|---|---|
| `DESIGN.md` | direção de arte, cor, tipografia, vidro (receita Slint), movimento, acessibilidade, decisões a aprovar |
| `SLINT_SPEC.md` | árvore de componentes, props, callbacks, medidas e estados por tela |
| `tokens.json` | fonte única dos tokens |
| `slint/` | `tokens.slint` (gerado), `atoms.slint`, `app-contract.slint`, `preview.slint` |
| `prototypes/` | HTML estático navegável: abra `prototypes/index.html` |
| `assets/` | logo, ícone do app, favicons, 45 ícones (`icons/`) |
| `fonts/` | Hanken Grotesk e Geist Mono (OFL 1.1, com as licenças) |
| `tools/` | geradores: `gen_tokens.py`, `make_assets.py`, `build_prototypes.py` |

Regenerar tudo: `python3 tools/gen_tokens.py && python3 tools/make_assets.py && python3 tools/build_prototypes.py` (precisa de `rsvg-convert`, `magick`, Pillow).
