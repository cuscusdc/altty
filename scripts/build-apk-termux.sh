#!/usr/bin/env bash
# Build do Kazzu APK nativamente no Termux (dispositivo ARM/aarch64).
# Usa: NDK toolchain (clang + sysroot) para linkar c++_static, aapt2/zipalign do Termux, jarsigner do Java.
#
# Uso:
#   scripts/build-apk-termux.sh [--debug-sign] [--version X.Y.Z]

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

DEBUG_SIGN=0
VERSION=""
OUT="$REPO/target/android/apk"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug-sign) DEBUG_SIGN=1; shift ;;
    --version) VERSION="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "opção desconhecida: $1" >&2; exit 2 ;;
  esac
done

MIN_SDK=26
TRIPLE=aarch64-linux-android

# NDK toolchain (prebuilt linux-x86_64 roda no Termux via qemu-user ou rosetta, mas clang é binary wrapper)
# Usamos o sysroot do NDK para achar libc++_static.a e libc++abi.a
NDK_HOME="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/25.0.8775105}"
TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
SYSROOT="$TOOLCHAIN/sysroot"
CLANG="$TOOLCHAIN/bin/${TRIPLE}${MIN_SDK}-clang"
CLANGPP="$TOOLCHAIN/bin/${TRIPLE}${MIN_SDK}-clang++"
AR="$TOOLCHAIN/bin/llvm-ar"
AAPT2="/data/data/com.termux/files/usr/bin/aapt2"
ZIPALIGN="/data/data/com.termux/files/usr/bin/zipalign"
JAVA_HOME="/data/data/com.termux/files/usr/lib/jvm/java-17-openjdk"
JARSIGNER="$JAVA_HOME/bin/jarsigner"
KEYTOOL="$JAVA_HOME/bin/keytool"

# Verifica NDK
[[ -f "$CLANG" ]] || { echo "NDK clang não encontrado: $CLANG" >&2; exit 1; }

if [[ -z "$VERSION" ]]; then
  VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
fi

IFS='.-' read -r V_MAJ V_MIN V_PAT V_PRE V_PRE_N <<<"$VERSION"
if [[ -n "${V_PRE:-}" ]]; then BUILD=${V_PRE_N:-1}; else BUILD=99; fi
VERSION_CODE=$((V_MAJ * 1000000 + V_MIN * 10000 + V_PAT * 100 + BUILD))

echo "==> Versão: $VERSION (code: $VERSION_CODE)"

# --- assinatura ---
mkdir -p "$OUT"
KEY_ALIAS="kazzu"
if [[ $DEBUG_SIGN -eq 1 ]]; then
  KS="$REPO/target/android/debug.jks"; PASS_FILE="$REPO/target/android/debug.pass"
  if [[ ! -f "$KS" ]]; then
    umask 077; mkdir -p "$(dirname "$KS")"
    head -c 24 /dev/urandom | base64 > "$PASS_FILE"
    "$KEYTOOL" -genkeypair -keystore "$KS" -storetype PKCS12 -storepass:file "$PASS_FILE" \
      -keypass:file "$PASS_FILE" -alias "$KEY_ALIAS" -keyalg RSA -keysize 2048 -validity 3650 \
      -dname "CN=Kazzu debug" >/dev/null
  fi
else
  KS="${KAZZU_KEYSTORE:-$HOME/.android/kazzu-release.jks}"
  PASS_FILE="${KAZZU_KEYSTORE_PASS_FILE:-$HOME/.android/kazzu-release.pass}"
  [[ -f "$KS" && -f "$PASS_FILE" ]] || { echo "keystore/release não encontrados. Use --debug-sign." >&2; exit 1; }
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO/target/android}"
export CC_aarch64_unknown_linux_android="$CLANG"
export CXX_aarch64_unknown_linux_android="$CLANGPP"
export AR_aarch64_unknown_linux_android="$AR"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_ANDROID_LINKER="$CLANG"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_ANDROID_AR="$AR"

# NDK sysroot para achar libc++_static.a, libc++abi.a
# O clang wrapper do NDK adiciona automaticamente --sysroot e -L do sysroot
# Mas precisamos garantir que o linker (ld.lld) encontre as libs
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_ANDROID_RUSTFLAGS="-Clink-arg=-L$SYSROOT/usr/lib/$TRIPLE -Clink-arg=-lc++_static -Clink-arg=-lc++abi -Clink-arg=--sysroot=$SYSROOT"

# --- ícones/recursos ---
WORK="$REPO/target/android/work"
rm -rf "$WORK"; mkdir -p "$WORK"
"$AAPT2" compile --dir android/res -o "$WORK/res.zip"

# --- cargo build ---
echo "==> cargo: kazzu_app ($TRIPLE, release)"
cargo rustc -p kazzu-app --lib --release --no-default-features --target "$TRIPLE" \
  --config 'profile.release.strip="symbols"' --crate-type cdylib

SO="$CARGO_TARGET_DIR/$TRIPLE/release/libkazzu_app.so"
[[ -f "$SO" ]] || { echo "libkazzu_app.so não gerada" >&2; exit 1; }

# --- empacotar APK ---
PLATFORM_JAR="$(ls -d $HOME/Android/Sdk/platforms/android-*/ | sort -V | tail -1)android.jar"
BASE="$WORK/base-$TRIPLE.apk"
"$AAPT2" link -o "$BASE" -I "$PLATFORM_JAR" --manifest android/AndroidManifest.xml \
  --min-sdk-version "$MIN_SDK" --target-sdk-version 34 --version-code "$VERSION_CODE" --version-name "$VERSION" \
  -R "$WORK/res.zip" --auto-add-overlay

python3 -I - "$BASE" "$SO" "$TRIPLE" <<'PY'
import sys, zipfile
apk, so, abi = sys.argv[1:4]
with zipfile.ZipFile(apk, "a") as z:
    z.write(so, f"lib/{abi}/libkazzu_app.so", compress_type=zipfile.ZIP_STORED)
PY

ALIGNED="$WORK/aligned.apk"
"$ZIPALIGN" -f -p 4 "$BASE" "$ALIGNED"

APK="$OUT/kazzu-$VERSION-$TRIPLE.apk"
"$JARSIGNER" -keystore "$KS" -storepass:file "$PASS_FILE" -keypass:file "$PASS_FILE" \
  -signedjar "$APK" "$ALIGNED" "$KEY_ALIAS" >/dev/null 2>&1

echo "==> $APK ($(du -h "$APK" | cut -f1))"
