#!/usr/bin/env bash
# Folhas comparativas: protótipo HTML (Chromium) à esquerda, captura Slint à direita.
# Uso: scripts/compare.sh [diretório das capturas]   (padrão: target/ui-screenshots; precisa de chromium e ImageMagick)
# Gera target/ui-screenshots/compare-<tela>-<WxH>-<tema>.png
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="${1:-$ROOT/target/ui-screenshots}"
SHOOT="$ROOT/design/tools/shoot.sh"
TMP="$(mktemp -d)"
# tela-gallery : protótipo : hash extra
MAP=(
  "home-ready:home:"
  "explore-ready:explore:"
  "library-full:library:"
  "search-idle:search:"
  "search-results:search:s=results"
  "player-orange:player:"
  "queue-open:queue:"
  "detail-album:album:"
  "settings-all:settings:"
  "menu-open:sheets:s=menu"
  "playlist-open:sheets:s=playlist"
  "newplaylist-open:sheets:s=create"
  "library-empty:states:s=empty"
  "home-error:states:s=error"
  "home-offline:states:s=offline"
  "home-loading:states:s=loading"
  "search-empty:states:s=noresult"
)
SIZES=${SIZES:-"390x844 360x740"}
for size in $SIZES; do
  w=${size%x*}; h=${size#*x}
  for theme in dark light; do
    for row in "${MAP[@]}"; do
      IFS=: read -r scene proto extra <<<"$row"
      shot="$OUT/${scene%%-*}-${scene#*-}-${size}-${theme}.png"
      [ -f "$shot" ] || continue
      hash="theme=$theme${extra:+&$extra}"
      "$SHOOT" "$proto" "$w" "$h" "$TMP/p.png" "$hash" || continue
      magick \( "$TMP/p.png" -resize "${w}x${h}!" \) \( "$shot" -resize "${w}x${h}!" \) -background '#888' -splice 6x0 +append \
        "$OUT/compare-${scene}-${size}-${theme}.png"
      echo "comparação: compare-${scene}-${size}-${theme}.png"
    done
  done
done
rm -rf "$TMP"
