#!/usr/bin/env bash
# Gera os ícones do app (res/mipmap-*) a partir de design/assets/. Requer ImageMagick (magick) e rsvg-convert.
# Os PNGs resultantes são versionados; rode de novo só se o design do ícone mudar.
set -euo pipefail
cd "$(dirname "$0")/.."
ASSETS=design/assets
RES=android/res
rm -rf "$RES"
mkdir -p "$RES"/mipmap-{mdpi,hdpi,xhdpi,xxhdpi,xxxhdpi,anydpi-v26} "$RES"/values

# Ícone legado (Android < 8 e launchers sem suporte a adaptativo): icon-1024.png redimensionado; o redondo recorta em círculo.
declare -A LEGACY=([mdpi]=48 [hdpi]=72 [xhdpi]=96 [xxhdpi]=144 [xxxhdpi]=192)
for d in "${!LEGACY[@]}"; do
  s=${LEGACY[$d]}
  magick "$ASSETS/icon-1024.png" -filter Lanczos -resize "${s}x${s}" "$RES/mipmap-$d/ic_launcher.png"
  magick "$ASSETS/icon-1024.png" -filter Lanczos -resize "${s}x${s}" \
    \( -size "${s}x${s}" xc:none -fill white -draw "circle $((s/2)),$((s/2)) $((s/2)),0" \) \
    -compose DstIn -composite "$RES/mipmap-$d/ic_launcher_round.png"
done

# Ícone adaptativo (108 dp; zona segura central de 66%): marca "K" sobre fundo sólido. Densidades: 108 dp * (dpi/160).
declare -A FG=([mdpi]=108 [hdpi]=162 [xhdpi]=216 [xxhdpi]=324 [xxxhdpi]=432)
for d in "${!FG[@]}"; do
  s=${FG[$d]}
  # a marca (viewBox 64) ocupa ~38% da tela: bem dentro da zona segura
  m=$((s * 40 / 100))
  rsvg-convert -w "$m" -h "$m" "$ASSETS/kazzu-mark-dark.svg" -o "$RES/.mark.png"
  magick -size "${s}x${s}" xc:none "$RES/.mark.png" -gravity center -composite "$RES/mipmap-$d/ic_launcher_foreground.png"
  rsvg-convert -w "$m" -h "$m" "$ASSETS/kazzu-mark-mono.svg" -o "$RES/.mark.png"
  magick -size "${s}x${s}" xc:none "$RES/.mark.png" -gravity center -composite "$RES/mipmap-$d/ic_launcher_monochrome.png"
done
rm -f "$RES/.mark.png"

cat > "$RES/values/colors.xml" <<'X'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">#15100C</color>
</resources>
X
for n in ic_launcher ic_launcher_round; do
cat > "$RES/mipmap-anydpi-v26/$n.xml" <<'X'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
    <monochrome android:drawable="@mipmap/ic_launcher_monochrome" />
</adaptive-icon>
X
done
echo "ícones gerados em $RES"
