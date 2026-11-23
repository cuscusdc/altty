#!/usr/bin/env python3
"""Mede o contraste REAL dos textos dos protótipos v4 renderizados (Chromium headless + Pillow).

Para cada tela, tamanho e tema: 1) lê do DOM a lista de textos visíveis (cor, caixa); 2) captura a tela com os textos
escondidos; 3) compara a cor de cada texto com os pixels que ficam atrás dele, já com vidro, desfoque e capas
(pior caso = percentil 5 do contraste; texto semitransparente é composto sobre cada pixel).
Limites WCAG 2.2 AA: 4,5:1; 3:1 para texto grande (>= 24 px, ou >= 18,66 px em negrito).
Controles desativados (aria-disabled, .soon, .off) ficam fora, como a WCAG permite; são listados à parte.
Uso: python3 design/v4/tools/check_render.py [--out design/v4/contrast-report.md]
"""
import json
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PROTO = ROOT / "prototypes"
SCREENS = ["home", "explore", "search", "search-results", "library", "album", "playlist", "artist", "player", "queue", "sheet-menu",
           "sheet-playlists", "sheet-new-playlist", "settings", "home-first-run", "home-offline", "home-error", "search-empty", "library-empty", "playlist-empty"]
VIEWS = {"390": (390, 844), "360": (360, 740), "768": (768, 1024), "1280": (1280, 800)}


def parse_color(s):
    m = re.match(r"rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)(?:,\s*([\d.]+))?\)", s)
    if m:
        return float(m[1]), float(m[2]), float(m[3]), float(m[4] if m[4] is not None else 1)
    m = re.match(r"color\(srgb ([\d.e-]+) ([\d.e-]+) ([\d.e-]+)(?:\s*/\s*([\d.%]+))?\)", s)
    if m:
        a = m[4] or "1"
        a = float(a[:-1]) / 100 if a.endswith("%") else float(a)
        return float(m[1]) * 255, float(m[2]) * 255, float(m[3]) * 255, a
    m = re.match(r"oklch\(([\d.]+)(%?) ([\d.]+) ([\d.]+)(?:\s*/\s*([\d.]+))?\)", s)
    if m:
        sys.path.insert(0, str(ROOT / "tools"))
        from gen_tokens import oklch_to_srgb
        L = float(m[1]) if m[2] else float(m[1]) * 100
        r, g, b = oklch_to_srgb(L, float(m[3]), float(m[4]))
        return r, g, b, float(m[5] or 1)
    raise ValueError(s)


def lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lum(rgb):
    return 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])


def contrast(fg, bg):
    a = fg[3]
    c = tuple(fg[i] * a + bg[i] * (1 - a) for i in range(3))
    l1, l2 = lum(c), lum(bg)
    if l1 < l2:
        l1, l2 = l2, l1
    return (l1 + 0.05) / (l2 + 0.05)


def run_case(case):
    """Roda um caso; se a captura sair em branco (quase todo texto com contraste ~1), tenta de novo."""
    for _ in range(3):
        out = run_case_once(case)
        res = out[1]
        if not res or sum(1 for r in res if r[0] < 1.15) < 0.6 * len(res):
            return out
    return out


def run_case_once(case):
    screen, vk, theme, extra = case
    w, h = VIEWS[vk]
    hv = "&".join([f"theme={theme}"] + ([extra] if extra else []))
    with tempfile.TemporaryDirectory() as td:
        png = Path(td) / "s.png"
        r = None
        for _ in range(3):
            try:
                r = subprocess.run(["node", str(ROOT / "tools" / "capture.mjs"), str(PROTO / f"{screen}.html"), str(w), str(h), str(png), hv, "--probe"], capture_output=True, text=True, timeout=80)
            except subprocess.TimeoutExpired:
                continue
            if r.stdout.strip() and png.exists():
                break
        if r is None or not r.stdout.strip() or not png.exists():
            return case, None, "sem probe"
        items = json.loads(r.stdout)["items"]
        img = Image.open(png).convert("RGB")
    W, H = img.size
    res = []
    for it in items:
        if it.get("o", 1) < 0.05:
            continue
        try:
            fg = parse_color(it["c"])
        except ValueError:
            continue
        disabled = it.get("o", 1) < 0.7   # .soon / .off / aria-disabled esmaecidos por opacidade
        fg = (fg[0], fg[1], fg[2], fg[3] * it.get("o", 1))
        x0, y0 = max(0, int(it["x"])), max(0, int(it["y"]))
        x1, y1 = min(W, int(it["x"] + it["w"]) + 1), min(H, int(it["y"] + it["h"]) + 1)
        if x1 - x0 < 2 or y1 - y0 < 2:
            continue
        reg = img.crop((x0, y0, x1, y1))
        px = list(reg.get_flattened_data())
        step = max(1, len(px) // 400)
        cs = sorted(contrast(fg, p) for p in px[::step])
        worst = cs[max(0, int(len(cs) * 0.05))]
        large = it["fs"] >= 24 or (it["fs"] >= 18.66 and it["fw"] >= 700)
        res.append((worst, 3.0 if large else 4.5, it["t"], it["fs"], disabled))
    return case, res, None


def main():
    out = Path(sys.argv[sys.argv.index("--out") + 1]) if "--out" in sys.argv else None
    cases = []
    for s in SCREENS:
        cases += [(s, "390", "dark", ""), (s, "390", "light", ""), (s, "390", "dark", "rt=1"), (s, "390", "light", "rt=1")]
    for s in ("home", "explore", "library", "album", "player", "settings", "search-results"):
        cases += [(s, "360", "dark", ""), (s, "768", "dark", ""), (s, "1280", "dark", ""), (s, "1280", "light", "")]
    for s in ("home", "explore", "library", "album", "playlist", "settings"):      # texto sob o cabeçalho e a doca de vidro
        for sc in ("120", "260", "420"):
            cases += [(s, "390", "dark", f"scroll={sc}"), (s, "390", "light", f"scroll={sc}")]
    for c in ("01", "03", "06", "07", "08", "09", "13", "16", "17", "20", "22"):   # capas claras, escuras e saturadas
        cases += [("player", "390", "dark", f"c={c}"), ("queue", "390", "dark", f"c={c}"), ("album", "390", "dark", f"c={c}"), ("album", "390", "light", f"c={c}")]
    with ThreadPoolExecutor(max_workers=4) as ex:
        results = list(ex.map(run_case, cases))
    lines, fails, total, minimum, dis, bad = [], 0, 0, 99.0, 0, []
    for case, res, err in results:
        if res is None:
            lines.append(f"| {case[0]} | {case[1]} | {case[2]} | {case[3]} | erro: {err} | | |")
            continue
        active = [r for r in res if not r[4]]
        dis += len(res) - len(active)
        nf = [r for r in active if r[0] < r[1]]
        total += len(active)
        fails += len(nf)
        if active:
            minimum = min(minimum, min(r[0] for r in active))
        w = min(active, key=lambda r: r[0] - r[1]) if active else None
        lines.append(f"| {case[0]} | {case[1]} | {case[2]} | {case[3]} | {len(active)} | {len(nf)} | {('%.2f:1 “%s”' % (w[0], w[2])) if w else ''} |")
        bad += [(r[0], case, r) for r in nf]
    head = ["# Contraste medido nos protótipos v4 renderizados", "",
            "Gerado por `design/v4/tools/check_render.py` (Chromium headless + Pillow). Para cada texto visível: contraste da cor do texto contra o **pior** pixel (percentil 5) do fundo atrás dele, já com vidro, desfoque e capas. Limite WCAG 2.2 AA: 4,5:1 (3:1 para texto grande).", "",
            f"**{total} textos medidos em {len(cases)} capturas; {fails} abaixo do limite; menor contraste {minimum:.2f}:1.** Fora da conta, como a WCAG permite: {dis} textos de controles desativados (itens \"Em breve\").", "",
            "| Tela | Largura | Tema | Variante | Textos | Reprovados | Pior caso |", "|---|---|---|---|---|---|---|"]
    rep = "\n".join(head + lines)
    if bad:
        rep += "\n\n## Reprovados\n\n| Contraste | Mín. | Tela | Largura | Tema | Variante | Texto |\n|---|---|---|---|---|---|---|\n"
        for c, case, r in sorted(bad, key=lambda x: x[0])[:120]:
            rep += f"| {c:.2f}:1 | {r[1]} | {case[0]} | {case[1]} | {case[2]} | {case[3]} | {r[2]} |\n"
    print("\n".join(head[4:5]))
    for c, case, r in sorted(bad, key=lambda x: x[0])[:40]:
        print(f"  {c:.2f} < {r[1]}  {case[0]} {case[1]} {case[2]} {case[3]}  “{r[2]}”")
    if out:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(rep + "\n", encoding="utf-8")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
