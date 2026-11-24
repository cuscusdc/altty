#!/usr/bin/env bash
# Folha de contato dos protótipos: tools/sheet.sh <saida.png> <hash> <largura> <altura> tela1 tela2 ...
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"; out="$1"; hash="$2"; w="$3"; h="$4"; shift 4
tmp="$(mktemp -d)"; i=0
for s in "$@"; do i=$((i+1)); name="${s%%#*}"; extra=""; [[ "$s" == *"#"* ]] && extra="&${s#*#}"
  node "$HERE/tools/capture.mjs" "$HERE/prototypes/$name.html" "$w" "$h" "$tmp/$(printf %02d $i).png" "$hash$extra" >/dev/null; done
magick montage "$tmp"/*.png -tile "${TILE:-6x}" -geometry +10+10 -background '#333' "$out"; rm -rf "$tmp"
