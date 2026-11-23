#!/usr/bin/env bash
# Captura os protótipos HTML (Chromium headless, viewport exato) e grava design/v4/shots/proto/*.webp e shots/ux/*.webp.
# Uso: design/v4/tools/shoot_proto.sh
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"; RAW="$(mktemp -d)"; mkdir -p "$HERE/shots/proto" "$HERE/shots/ux"
shot() { # nome tela largura altura hash
  node "$HERE/tools/capture.mjs" "$HERE/prototypes/$2.html" "$3" "$4" "$RAW/$1.png" "$5" --dpr="${6:-2}" >/dev/null; }
P=(home explore search search-results library album playlist artist player queue sheet-menu sheet-playlists sheet-new-playlist settings home-first-run home-loading home-offline home-error search-empty library-empty playlist-empty)
for s in "${P[@]}"; do shot "$s-390-dark" "$s" 390 844 "theme=dark"; done
for s in home library album search settings queue sheet-menu explore; do shot "$s-390-light" "$s" 390 844 "theme=light"; done
shot home-scrolled-390-dark home 390 844 "theme=dark&scroll=420"; shot home-scrolled-390-light home 390 844 "theme=light&scroll=420"
shot home-rt-390-dark home 390 844 "theme=dark&rt=1"; shot player-rt-390-dark player 390 844 "theme=dark&rt=1"
for c in 03 09 13 07; do shot "player-c$c-390-dark" player 390 844 "theme=dark&c=$c"; done
for s in home library player album; do shot "$s-360-dark" "$s" 360 740 "theme=dark"; done
for s in home library player album explore; do shot "$s-768-dark" "$s" 768 1024 "theme=dark" 1; shot "$s-1280-dark" "$s" 1280 800 "theme=dark" 1; done
shot home-1280-light home 1280 800 "theme=light" 1; shot library-1280-light library 1280 800 "theme=light" 1
for f in "$RAW"/*.png; do magick "$f" -quality 86 -define webp:method=6 "$HERE/shots/proto/$(basename "${f%.png}").webp"; done
# miniaturas para o artifact de UX (390 de largura)
ux() { magick "$RAW/$2.png" -resize 390x -quality 82 "$HERE/shots/ux/$1.webp"; }
ux home home-390-dark; ux home-scrolled home-scrolled-390-dark; ux explore explore-390-dark; ux search search-390-dark; ux search-results search-results-390-dark
ux library library-390-dark; ux album album-390-dark; ux player player-390-dark; ux queue queue-390-dark; ux menu sheet-menu-390-dark; ux settings settings-390-dark
ux home-offline home-offline-390-dark; ux home-loading home-loading-390-dark; ux home-light home-390-light; ux player-rt player-rt-390-dark
rm -rf "$RAW"; echo "proto: $(ls "$HERE/shots/proto" | wc -l) arquivos, $(du -sh "$HERE/shots/proto" | cut -f1)"
