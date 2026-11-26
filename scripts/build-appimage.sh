#!/usr/bin/env bash
# Native Slint AppImage. ARM builds require an external cross-toolchain/sysroot.
# Usage: scripts/build-appimage.sh x86_64|aarch64
# APPIMAGETOOL: host executable; APPIMAGE_RUNTIME: matching target runtime.
# KAZZU_ARM_SYSROOT: target libraries; CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: cross linker.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"
ARCH="${1:-x86_64}"
case "$ARCH" in x86_64|aarch64) ;; *) echo 'Architecture must be x86_64 or aarch64' >&2; exit 2;; esac
TRIPLE="$ARCH-unknown-linux-gnu"
VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
OUT="${KAZZU_DIST_DIR:-$REPO/dist}"
STAGE="$REPO/target/appimage/$ARCH/Kazzu.AppDir"
TOOL="${APPIMAGETOOL:-$REPO/target/toolchains/appimagetool-x86_64.AppImage}"
RUNTIME="${APPIMAGE_RUNTIME:-$REPO/target/toolchains/runtime-$ARCH}"
[[ -x "$TOOL" && -f "$RUNTIME" ]] || { echo 'Provide APPIMAGETOOL and APPIMAGE_RUNTIME' >&2; exit 1; }
TARGET_ARGS=(); [[ "$ARCH" == x86_64 ]] || TARGET_ARGS=(--target "$TRIPLE")
cargo build --config "profile.release.package.kazzu-app.opt-level=${KAZZU_APP_OPT_LEVEL:-1}" -j "${KAZZU_BUILD_JOBS:-1}" -p kazzu-app --bin kazzu --release --no-default-features "${TARGET_ARGS[@]}"
BIN="${CARGO_TARGET_DIR:-$REPO/target}/${TRIPLE}/release/kazzu"
[[ "$ARCH" != x86_64 ]] || BIN="${CARGO_TARGET_DIR:-$REPO/target}/release/kazzu"
[[ -x "$BIN" ]] || { echo "Missing $BIN" >&2; exit 1; }
# Re-link only the executable against a compatible libm when the local compatibility toolchain is present.
# Rust dependencies stay optimized and cached; host libm symbol-version bumps do not constrain the AppImage.
X64_RUNTIME="${KAZZU_X64_LIB_ROOT:-$REPO/target/toolchains/linux-x64}"
if [[ "$ARCH" == x86_64 && -x "$X64_RUNTIME/bin/kazzu-linker" ]]; then
  cargo rustc --config "profile.release.package.kazzu-app.opt-level=${KAZZU_APP_OPT_LEVEL:-1}" -j "${KAZZU_BUILD_JOBS:-1}" -p kazzu-app --bin kazzu --release --no-default-features -- -C "linker=$X64_RUNTIME/bin/kazzu-linker"
fi
# Stage only this architecture's build outputs.
mkdir -p "$STAGE/usr/bin" "$STAGE/usr/lib" "$STAGE/usr/share/icons/hicolor/256x256/apps" "$OUT"
cp "$BIN" "$STAGE/usr/bin/kazzu"
llvm-strip --strip-unneeded "$STAGE/usr/bin/kazzu"
mkdir -p "$STAGE/usr/share/doc/kazzu"
cp docs/DEPENDENCIES.md docs/PACKAGING.md "$STAGE/usr/share/doc/kazzu/"
if [[ "$ARCH" == x86_64 ]]; then
  while read -r lib; do
    case "$(basename "$lib")" in libc.so.*|libm.so.*|libpthread.so.*|libdl.so.*|librt.so.*|ld-linux*) continue;; esac
    source_lib="$lib"
    if [[ -d "$X64_RUNTIME" ]]; then
      source_lib=""
      for libdir in "$X64_RUNTIME/lib/x86_64-linux-gnu" "$X64_RUNTIME/usr/lib/x86_64-linux-gnu"; do
        if [[ -e "$libdir/$(basename "$lib")" ]]; then source_lib="$libdir/$(basename "$lib")"; break; fi
      done
      [[ -n "$source_lib" ]] || { echo "Missing compatible x64 library: $lib" >&2; exit 1; }
    fi
    cp -L "$source_lib" "$STAGE/usr/lib/"
  done < <(ldd "$BIN" | awk '/=> \// { print $3 }')
else
  : "${KAZZU_ARM_SYSROOT:?Set KAZZU_ARM_SYSROOT}"
  # Resolve the ELF dependency closure statically; never run a foreign binary with ldd.
  python3 - "$BIN" "$KAZZU_ARM_SYSROOT" "$STAGE/usr/lib" <<'PY'
import pathlib,subprocess,re,shutil,sys
binary,root,dest=map(pathlib.Path,sys.argv[1:])
skip={'libc.so.6','libm.so.6','libpthread.so.0','libdl.so.2','librt.so.1','ld-linux-aarch64.so.1'}
queue=[binary];seen=set()
while queue:
 obj=queue.pop()
 for name in re.findall(r'\(NEEDED\).*?\[(.*?)\]',subprocess.check_output(['readelf','-d',str(obj)],text=True)):
  if name in skip or name in seen:continue
  seen.add(name)
  candidates=list(root.rglob(name))
  if not candidates:raise SystemExit('Missing target library: '+name)
  source=candidates[0].resolve();shutil.copyfile(source,dest/name);queue.append(source)
PY
fi
cp android/res/mipmap-xxxhdpi/ic_launcher.png "$STAGE/kazzu.png"
cp "$STAGE/kazzu.png" "$STAGE/usr/share/icons/hicolor/256x256/apps/kazzu.png"
cat > "$STAGE/kazzu.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Kazzu
Comment=Your music, in a native player
Exec=kazzu
Icon=kazzu
Terminal=false
Categories=AudioVideo;Audio;Player;
DESKTOP
mkdir -p "$STAGE/usr/share/fontconfig"
cat > "$STAGE/usr/share/fontconfig/fonts.conf" <<'FONTS'
<?xml version="1.0"?>
<fontconfig>
  <dir>/usr/share/fonts</dir>
  <dir>/usr/local/share/fonts</dir>
  <dir prefix="xdg">fonts</dir>
  <dir>~/.fonts</dir>
  <cachedir prefix="xdg">fontconfig</cachedir>
  <alias><family>sans-serif</family><prefer><family>Noto Sans</family><family>DejaVu Sans</family></prefer></alias>
  <alias><family>serif</family><prefer><family>Noto Serif</family><family>DejaVu Serif</family></prefer></alias>
  <alias><family>monospace</family><prefer><family>Noto Sans Mono</family><family>DejaVu Sans Mono</family></prefer></alias>
</fontconfig>
FONTS
cat > "$STAGE/AppRun" <<'RUN'
#!/bin/sh
APPDIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export LD_LIBRARY_PATH="$APPDIR/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export FONTCONFIG_FILE="${FONTCONFIG_FILE:-$APPDIR/usr/share/fontconfig/fonts.conf}"
exec "$APPDIR/usr/bin/kazzu" "$@"
RUN
chmod +x "$STAGE/AppRun"
APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$ARCH" "$TOOL" --runtime-file "$RUNTIME" "$STAGE" "$OUT/kazzu-$VERSION-$ARCH.AppImage"
(cd "$OUT" && sha256sum "kazzu-$VERSION-$ARCH.AppImage" > "kazzu-$VERSION-$ARCH.AppImage.sha256")
echo "$OUT/kazzu-$VERSION-$ARCH.AppImage"
