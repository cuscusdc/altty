#!/usr/bin/env bash
# Renderiza a prévia v4 no Slint de verdade (FemtoVG, desfoque real, via Xvfb) e grava design/v4/shots/slint/*.webp.
# Uso: design/v4/tools/shoot_slint.sh [pasta-dos-png]   (precisa do binário: veja design/v4/README.md)
# Os PNG ficam na pasta dada (padrão: temporária); o repositório guarda só os WebP.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${KAZZU_SHOT_BIN:-$HERE/../../target/design-v4/debug/kazzu-slint-shot-v4}"
RAW="${1:-$(mktemp -d)}"; mkdir -p "$RAW" "$HERE/shots/slint"
SCENES=(
  "home|scene=home"
  "home-scrolled|scene=home scroll=420"
  "home-first-run|scene=home.first-run now=0"
  "home-loading|scene=home.loading now=0"
  "home-offline|scene=home.offline"
  "home-error|scene=home.error now=0"
  "explore|scene=explore"
  "search|scene=search"
  "search-results|scene=search.results"
  "search-empty|scene=search.empty"
  "library|scene=library"
  "library-empty|scene=library.empty now=0"
  "album|scene=album cover=3"
  "album-scrolled|scene=album cover=3 scroll=360"
  "player|scene=player cover=5"
  "player-delta|scene=player cover=3"
  "player-white|scene=player cover=13"
  "player-dark|scene=player cover=9"
  "queue|scene=player+queue cover=5"
  "menu|scene=album+menu cover=3"
  "new-playlist|scene=library+new-playlist"
  "settings|scene=settings"
  "home-light|scene=home theme=light"
  "home-scrolled-light|scene=home scroll=420 theme=light"
  "library-light|scene=library theme=light"
  "album-light|scene=album cover=3 theme=light"
  "search-light|scene=search theme=light"
  "settings-light|scene=settings theme=light"
  "queue-light|scene=player+queue cover=5 theme=light"
  "menu-light|scene=album+menu cover=3 theme=light"
  "home-rt|scene=home rt=1"
  "home-rt-light|scene=home rt=1 theme=light"
  "player-rt|scene=player cover=5 rt=1"
  "queue-rt|scene=player+queue cover=5 rt=1"
  "home-safe|scene=home safe=32,24"
  "player-safe|scene=player cover=3 safe=32,24"
  "home-360|scene=home size=360x740"
  "player-360|scene=player cover=5 size=360x740"
  "library-360|scene=library size=360x740"
  "home-768|scene=home size=768x1024"
  "library-768|scene=library size=768x1024"
  "player-768|scene=player cover=3 size=768x1024"
)
args=()
for s in "${SCENES[@]}"; do args+=(--shot "${s#*|} out=$RAW/${s%%|*}.png"); done
SLINT_SCALE_FACTOR="${SCALE:-2}" xvfb-run -a -s "-screen 0 2800x2400x24 +extension GLX" "$BIN" "${args[@]}"
# fallback sólido: renderer software (sem backdrop-blur)
SLINT_SCALE_FACTOR="${SCALE:-2}" xvfb-run -a -s "-screen 0 2800x2400x24 +extension GLX" "$BIN" --renderer software \
  --shot "scene=home out=$RAW/home-software.png" --shot "scene=player cover=5 out=$RAW/player-software.png"
for f in "$RAW"/*.png; do magick "$f" -quality 88 -define webp:method=6 "$HERE/shots/slint/$(basename "${f%.png}").webp"; done
echo "PNG em $RAW; WebP em $HERE/shots/slint ($(ls "$HERE/shots/slint" | wc -l) arquivos, $(du -sh "$HERE/shots/slint" | cut -f1))"
