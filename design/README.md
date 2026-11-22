# design/

Design system e planejamento de UX do Kazzu (**v2: mobile-first, vidro real**). Comece por `ux/UX_PLAN.md` (o quê e por quê) e `DESIGN.md` (como é); o implementador de UI segue `SLINT_SPEC.md`.

| Pasta/arquivo | Conteúdo |
|---|---|
| `ux/UX_PLAN.md` | planejamento de UX: personas, JTBD, heurísticas, arquitetura de informação, navegação mobile, fluxos, estados, gestos, acessibilidade, desempenho, responsivo, validação sem telemetria, backlog MoSCoW |
| `ux/plan/` | cópia versionável do artifact HTML navegável do plano (`index.html` + `img/`) |
| `ux/contrast-report.md` | contraste de texto **medido em capturas reais** dos protótipos |
| `DESIGN.md` | direção de arte, palco vivo, vidro em 3 níveis, cor, tipografia, movimento, acessibilidade, decisões a aprovar |
| `SLINT_SPEC.md` | árvore de componentes, props, callbacks, medidas e estados por tela |
| `tokens.json` | fonte única dos tokens |
| `slint/` | `tokens.slint` (gerado), `atoms.slint` (`Stage`, `GlassPanel` 3 níveis, `Chip`, …), `app-contract.slint`, `preview.slint` + renders `preview-femtovg-*.png` (blur real) e `preview-software-*.png` (fallback sólido) |
| `prototypes/` | HTML navegável **mobile-first** (390 e 360 primeiro): abra `prototypes/index.html` |
| `assets/` | logo, ícone do app, favicons, 54 ícones (`icons/`) |
| `fonts/` | Hanken Grotesk e Geist Mono (OFL 1.1, com as licenças) |
| `tools/` | geradores e verificadores (abaixo) |

## Regenerar e verificar

```
python3 tools/gen_tokens.py        # tokens.slint + tokens.css + conferência AA dos pares
python3 tools/make_assets.py       # logos, ícones, capas fictícias (rsvg-convert, Pillow, magick)
python3 tools/build_prototypes.py  # protótipos + stage.css (palco por capa)
python3 tools/check_render.py --out ux/contrast-report.md   # contraste real em ~120 capturas (Chromium + Node >= 22 + Pillow)
python3 tools/shoot_all.py         # capturas webp (para o artifact do plano)
python3 tools/build_ux_plan.py     # artifact HTML do plano (baixa mermaid@11.4.1 para tools/.cache, não versionado)
node tools/capture.mjs home 390 844 /tmp/home.png "theme=dark"   # uma captura (viewport exato, via CDP)
```

Opções dos protótipos (hash na URL): `#theme=light|dark`, `#rt=1` (reduzir transparência), `#sat=0` (sem `saturate`, como no Slint), `#c=01..10` (capa), `#s=` (estado da tela).

O app **Kazzu** precisa receber o diff de `slint/atoms.slint` e `slint/tokens.slint` (novos: `Stage`, `Chip`, `GlassPanel.level`, tokens de palco e vidro). Ver `SLINT_SPEC.md` §0 e §7.
