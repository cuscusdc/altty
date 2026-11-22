#!/usr/bin/env python3
"""Captura todas as telas dos protótipos (390, 360, 768, 1280; escuro e claro) em design/ux/plan/img/*.webp (usado pelo artifact do plano de UX).
Uso: python3 design/tools/shoot_all.py [--png]   (--png mantém também PNG em design/prototypes/shots/ para revisão)
"""
import subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "ux" / "plan" / "img"
OUT.mkdir(parents=True, exist_ok=True)
SCREENS = ["home", "explore", "library", "album", "search", "player", "queue", "sheets", "settings", "states"]
VIEWS = {"390": (390, 844), "360": (360, 740), "768": (768, 1024), "1280": (1280, 800)}
HASH = {"search": "", "states": ""}


def job(a):
    s, vk, th = a
    w, h = VIEWS[vk]
    png = OUT / f"{s}-{vk}-{th}.png"
    for _ in range(4):
        r = subprocess.run(["node", str(ROOT / "tools" / "capture.mjs"), s, str(w), str(h), str(png), f"theme={th}"], capture_output=True, timeout=90)
        if r.returncode == 0 and png.exists():
            break
    else:
        raise RuntimeError(f"falhou: {a}")
    webp = png.with_suffix(".webp")
    subprocess.run(["magick", str(png), "-quality", "84", str(webp)], check=True)
    png.unlink()
    return webp.name


with ThreadPoolExecutor(max_workers=4) as ex:
    for n in ex.map(job, [(s, v, t) for s in SCREENS for v in VIEWS for t in ("dark", "light")]):
        pass
print("ok:", len(list(OUT.glob("*.webp"))), "capturas em", OUT)
