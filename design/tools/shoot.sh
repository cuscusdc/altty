#!/usr/bin/env bash
# Uso: tools/shoot.sh <tela> <largura> <altura> <saida.png> [hash]   (ex.: tools/shoot.sh home 390 844 /tmp/h.png "theme=light&rt=1")
# Renderiza o protótipo com o Chromium headless (backdrop-filter real).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
chromium --headless=new --no-sandbox --disable-gpu --hide-scrollbars --force-device-scale-factor="${DPR:-1}" \
  --window-size="$2,$3" --virtual-time-budget=2500 --screenshot="$4" "file://$HERE/prototypes/$1.html#${5:-}" >/dev/null 2>&1
