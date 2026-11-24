# design/v4: UX e UI do Kazzu, do zero

Estado: **proposta, aguardando o autor.** Nada aqui foi implementado no `kazzu-app` e `crates/` não foi tocado.
O material anterior (`design/DESIGN.md`, `design/SLINT_SPEC.md`, `design/ux/`, `design/prototypes/`, `design/slint/`) é a v2/v3 reprovada: fica como estudo e não foi movido.

## Por onde começar

| Quero | Abra |
|---|---|
| Ver o resultado em um minuto | `shots/key-screens.webp` e `shots/before-after.webp` |
| Ver cada tela no Slint de verdade | `shots/slint/*.webp` (44 renders: claro, escuro, sem transparência, 360, 390, 768, renderer software) |
| Clicar nas telas | `prototypes/index.html` (HTML, o mesmo desenho em 360, 390, 768 e 1280) |
| Entender o que mudou e por quê | `ux/README.md` e `DIRECTION.md` |
| Implementar | `SLINT_SPEC.md` |
| Conferir que o vidro é vidro | `shots/glass-proof.md` |
| Conferir o contraste | `contrast-report.md` (medido) e `contrast-tokens.md` (calculado) |
| Saber como as duas skills foram usadas | `SKILLS_REPORT.md` |

## Conteúdo

| Pasta ou arquivo | O que é |
|---|---|
| `ux/` | Plano de UX em 7 arquivos (diagnóstico, objetivos, navegação, fluxos, estados e textos, acessibilidade, backlog e aceite). Versão navegável: artifact `kazzu-ux-v4` no Traycer |
| `DIRECTION.md` | Moodboard escrito, três direções comparadas, a escolhida e por quê |
| `DESIGN.md` | Design system: cor, tipo, espaço, vidro, player, componentes, telas, movimento |
| `SLINT_SPEC.md` | Contrato para o implementador: árvore, medidas, dados novos do Rust, limites do Slint 1.19 |
| `SKILLS_REPORT.md` | O que foi rodado das skills `hallmark` e `design-taste-frontend`, com as saídas |
| `tokens.json` | Fonte única dos tokens |
| `slint/` | `tokens.slint` e `data.slint` (gerados), `atoms.slint`, `chrome.slint`, `screens.slint`, `preview.slint`: compilam no Slint 1.19 |
| `prototypes/` | 22 páginas HTML geradas + `assets/` (CSS, tokens, JS) |
| `assets/` | Marca (`kazzu-*.svg`), 57 ícones de traço uniforme, 22 capas de demonstração (fotos CC0 e de domínio público; créditos em `assets/covers/CREDITS.md`) |
| `fonts/` | Inter e Inter Display (OFL) |
| `shots/` | Capturas: `slint/`, `proto/`, `ux/`, prova de vidro, painel antes × depois |
| `skills-logs/` | Saídas brutas dos verificadores das skills |
| `tools/` | Geradores e verificadores (abaixo) |

## Regenerar e verificar

```
python3 tools/make_covers.py           # capas (rede só na primeira vez; guarda os originais em assets/covers/.src, fora do git)
python3 tools/gen_tokens.py            # tokens.css + tokens.slint + contrast-tokens.md; sai com 1 se algum par reprovar
python3 tools/gen_slint_data.py        # slint/data.slint
python3 tools/build_prototypes.py      # prototypes/*.html
python3 tools/check_render.py --out contrast-report.md     # contraste medido em 188 capturas (Chromium + Node 22+ + Pillow)
tools/shoot_proto.sh                   # shots/proto e shots/ux
python3 tools/build_ux_artifact.py     # artifact HTML do plano de UX

# Slint de verdade (FemtoVG, desfoque real). Precisa de Xvfb. ~1 GB em target/design-v4: apague depois.
df -h /home | tail -1                  # pare se houver menos de 20 GB livres
(cd tools/slint-shot && CARGO_TARGET_DIR=../../../../target/design-v4 cargo build --offline)
tools/shoot_slint.sh                   # shots/slint
python3 tools/glass_proof.py           # shots/glass-proof.md e .webp
rm -rf ../../target/design-v4
```

Opções dos protótipos pela URL: `#theme=dark|light`, `#rt=1` (reduzir transparência), `#rm=1` (reduzir movimento), `#scroll=N`, `#c=NN` (capa do player ou do álbum, 01 a 22).

## O que foi verificado

| Verificação | Resultado |
|---|---|
| Slint 1.19 compila `slint/preview.slint` (e tudo o que ele importa) | Sim, sem avisos |
| Render no Slint real, FemtoVG com `backdrop-blur` | 44 cenas |
| Prova de vidro por pixels | 11 de 11 casos passam |
| Contraste dos tokens (calculado, com o pior caso do vidro) | 47 pares, 0 reprovados |
| Contraste medido nos protótipos | 188 capturas, 0 textos abaixo de AA (ver `contrast-report.md`) |
| hallmark `sloplint --scope app --render` | 0 FAIL, 0 REFLEX, 4 respondidos |
| taste-skill `copy-lint` | 0 FAIL em 22 páginas |
| taste-skill `gate` (feito para sites) | OPEN 34: todos itens de estrutura de site; detalhado no relatório |
| Revisão de contexto limpo (Sonnet 5.5, só capturas e brief) | "fix" com 8 achados; corrigidos e pontuados no relatório |

## O que NÃO foi verificado

- Desempenho do desfoque em Android real e no renderer Skia (o Android usa Skia; aqui só FemtoVG e software).
- TalkBack e teste com pessoas.
- `--hit-targets` e `--selftest` da galeria com os componentes novos (a galeria compila a UI do app, que não foi tocada).
- A faixa larga (desktop) no Slint: só existe no protótipo HTML.
- Gestos, teclado virtual sobre os campos, transições animadas (a prévia é estática).
- A segunda rodada do revisor: as correções foram pontuadas por mim, como a skill manda.
