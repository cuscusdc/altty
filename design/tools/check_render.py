#!/usr/bin/env python3
"""Mede o contraste REAL dos textos dos protótipos renderizados (Chromium headless + Pillow).
Para cada tela/tamanho/tema: 1) captura o DOM com a lista de textos (cor, caixa) em `#probe=1`;
2) captura a tela com os textos escondidos (`#probe=2`); 3) para cada texto, compara a cor do texto com os pixels de fundo
atrás dele (pior caso = percentil 5 do contraste; o texto semitransparente é composto sobre cada pixel).
Limites WCAG 2.2 AA: 4,5:1; 3:1 para texto grande (>= 24 px, ou >= 18,66 px em negrito).
Uso: python3 design/tools/check_render.py [--quick] [--out design/ux/contrast-report.md]
"""
import json, re, subprocess, sys, tempfile, itertools
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
PROTO = ROOT / "prototypes"
SCREENS = ["home", "explore", "library", "album", "search", "player", "queue", "sheets", "settings", "states"]
VIEWS = {"390": (390, 844), "360": (360, 740), "768": (768, 1024), "1280": (1280, 800)}


def parse_color(s):
    m = re.match(r"rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)(?:,\s*([\d.]+))?\)", s)
    if m:
        return float(m[1]), float(m[2]), float(m[3]), float(m[4] if m[4] is not None else 1)
    m = re.match(r"rgba?\(([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*/\s*([\d.%]+))?\)", s)
    if m:
        a = m[4] or "1"
        a = float(a[:-1]) / 100 if a.endswith("%") else float(a)
        return float(m[1]), float(m[2]), float(m[3]), a
    m = re.match(r"color\(srgb ([\d.e-]+) ([\d.e-]+) ([\d.e-]+)(?:\s*/\s*([\d.%]+))?\)", s)
    if m:
        a = m[4] or "1"
        a = float(a[:-1]) / 100 if a.endswith("%") else float(a)
        return float(m[1]) * 255, float(m[2]) * 255, float(m[3]) * 255, a
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
    screen, vk, theme, rt, extra = case
    w, h = VIEWS[vk]
    hv = [f"theme={theme}"] + (["rt=1"] if rt else []) + (extra.split("&") if extra else [])
    base = f"file://{PROTO}/{screen}.html#" + "&".join(hv)
    with tempfile.TemporaryDirectory() as td:
        png = Path(td) / "s.png"
        r = None
        for _ in range(3):
            try:
                r = subprocess.run(["node", str(ROOT / "tools" / "capture.mjs"), screen, str(w), str(h), str(png), "&".join(hv), "--probe"], capture_output=True, text=True, timeout=70)
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
        if it.get("o", 1) < 0.05:  # texto invisível (opacidade 0, ex.: título pequeno do cabeçalho antes de rolar)
            continue
        try:
            fg = parse_color(it["c"])
            fg = (fg[0], fg[1], fg[2], fg[3] * it.get("o", 1))
        except ValueError:
            continue
        x0, y0 = max(0, int(it["x"])), max(0, int(it["y"]))
        x1, y1 = min(W, int(it["x"] + it["w"]) + 1), min(H, int(it["y"] + it["h"]) + 1)
        if x1 - x0 < 2 or y1 - y0 < 2:
            continue
        reg = img.crop((x0, y0, x1, y1))
        px = list(reg.get_flattened_data() if hasattr(reg, 'get_flattened_data') else reg.getdata())
        step = max(1, len(px) // 400)
        cs = sorted(contrast(fg, p) for p in px[::step])
        worst = cs[max(0, int(len(cs) * 0.05))]
        large = it["fs"] >= 24 or (it["fs"] >= 18.66 and it["fw"] >= 700)
        need = 3.0 if large else 4.5
        res.append((worst, need, it["t"], it["fs"]))
    return case, res, None


def main():
    quick = "--quick" in sys.argv
    out = None
    if "--out" in sys.argv:
        out = Path(sys.argv[sys.argv.index("--out") + 1])
    cases = []
    for s in SCREENS:
        for vk, theme in (("390", "dark"), ("390", "light"), ("360", "dark"), ("768", "dark"), ("768", "light"), ("1280", "dark"), ("1280", "light")):
            if quick and vk in ("768",):
                continue
            cases.append((s, vk, theme, False, ""))
        cases.append((s, "390", "dark", True, ""))
        cases.append((s, "390", "light", True, ""))
    for s in ("home", "library", "explore"):
        for theme in ("dark", "light"):
            cases.append((s, "390", theme, False, "scroll=420"))
    for c in ("01", "03", "05", "06", "08", "09", "10"):
        cases.append(("player", "390", "dark", False, f"c={c}"))
        cases.append(("home", "390", "dark", False, f"c={c}"))
        cases.append(("home", "390", "light", False, f"c={c}"))
    with ThreadPoolExecutor(max_workers=4) as ex:
        results = list(ex.map(run_case, cases))
    lines, fails, total, minimum = [], 0, 0, 99.0
    worst_rows = []
    for case, res, err in results:
        if res is None:
            lines.append(f"| {case[0]} | {case[1]} | {case[2]} | {'rt' if case[3] else ''} {case[4]} | erro: {err} | | |")
            continue
        n_fail = [r for r in res if r[0] < r[1]]
        total += len(res)
        fails += len(n_fail)
        if res:
            minimum = min(minimum, min(r[0] for r in res))
        w = min(res, key=lambda r: r[0] - r[1]) if res else None
        lines.append(f"| {case[0]} | {case[1]} | {case[2]} | {'sem transparência ' if case[3] else ''}{case[4]} | {len(res)} | {len(n_fail)} | {('%.2f:1 “%s”' % (w[0], w[2])) if w else ''} |")
        for r in n_fail:
            worst_rows.append((r[0], case, r))
    head = ["# Contraste medido nos protótipos renderizados", "",
            "Gerado por `design/tools/check_render.py` (Chromium headless + Pillow). Para cada texto visível: contraste da cor do texto contra o **pior** pixel (percentil 5) do fundo atrás dele, já com o vidro, o palco e o desfoque aplicados. Limite WCAG 2.2 AA: 4,5:1 (3:1 para texto grande).", "",
            f"**{total} textos medidos em {len(cases)} capturas; {fails} abaixo do limite; menor contraste {minimum:.2f}:1.**", "",
            "| Tela | Largura | Tema | Variante | Textos | Reprovados | Pior caso |", "|---|---|---|---|---|---|---|"]
    rep = "\n".join(head + lines)
    if worst_rows:
        rep += "\n\n## Reprovados\n\n| Contraste | Mín. | Tela | Largura | Tema | Variante | Texto |\n|---|---|---|---|---|---|---|\n"
        for c, case, r in sorted(worst_rows, key=lambda x: x[0])[:80]:
            rep += f"| {c:.2f}:1 | {r[1]} | {case[0]} | {case[1]} | {case[2]} | {'rt ' if case[3] else ''}{case[4]} | {r[2]} |\n"
    print(rep[:6000])
    if out:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(rep + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
