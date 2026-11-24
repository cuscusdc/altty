#!/usr/bin/env python3
"""Gera design/v4/slint/data.slint: capas de demonstração e textos fictícios para a prévia (mesmos dados dos protótipos)."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
C = json.loads((ROOT / "assets" / "covers" / "covers.json").read_text(encoding="utf-8"))


def arr(suffix):
    return "[" + ", ".join(f'@image-url("../assets/covers/{c["id"]}{suffix}.jpg")' for c in C) + "]"


def strs(key):
    return "[" + ", ".join(json.dumps(c[key], ensure_ascii=False) for c in C) + "]"


ICONS = sorted(p.stem for p in (ROOT / "assets" / "icons").glob("*.svg"))
out = ["// GERADO por design/v4/tools/gen_slint_data.py: não edite à mão.",
       "// Dados FICTÍCIOS da prévia do design (as mesmas capas CC0 e os mesmos textos dos protótipos HTML). O app real recebe tudo do Rust.",
       "", "export global Icons {"]
for i in ICONS:
    out.append(f'    out property <image> {i}: @image-url("../assets/icons/{i}.svg");')
out += ["}", "", "export global Demo {",
        "    // índice = número da capa - 1",
        f"    out property <[image]> cover: {arr('')};",
        f"    out property <[image]> thumb: {arr('-s')};",
        f"    out property <[image]> back: {arr('-back')};     // capa + reflexo, pré-desfocada (fundo do player no celular)",
        f"    out property <[image]> blur: {arr('-blur')};     // capa pré-desfocada (fundo do player em telas largas)",
        f"    out property <[string]> title: {strs('title')};",
        f"    out property <[string]> artist: {strs('artist')};",
        f"    out property <[string]> kind: {strs('kind')};",
        "    out property <[float]> dim: [" + ", ".join(str(c["dim"]) for c in C) + "];   // escurecimento do console por capa (regra em make_covers.py)",
        "    out property <[float]> mini-dim: [" + ", ".join(str(c["mini"]) for c in C) + "];   // idem para o mini-player",
        "}", ""]
(ROOT / "slint" / "data.slint").write_text("\n".join(out), encoding="utf-8")
print("data.slint:", len(ICONS), "ícones,", len(C), "capas")
