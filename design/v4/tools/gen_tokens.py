#!/usr/bin/env python3
"""Gera os tokens da v4 a partir de design/v4/tokens.json e confere os pares de contraste (WCAG 2.2).

Saídas:
  design/v4/prototypes/assets/tokens.css   (nomes da skill hallmark: --color-*, --space-*, --text-*, ...)
  design/v4/slint/tokens.slint             (global Theme)
  design/v4/contrast-tokens.md             (tabela dos pares, com o pior caso do vidro)
Uso: python3 design/v4/tools/gen_tokens.py   (sai com código 1 se algum par obrigatório reprovar)
"""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
T = json.loads((ROOT / "tokens.json").read_text(encoding="utf-8"))


# ---------- cor ----------
def oklch_to_srgb(L, C, H):
    L /= 100.0
    a, b = C * math.cos(math.radians(H)), C * math.sin(math.radians(H))
    l_ = L + 0.3963377774 * a + 0.2158037573 * b
    m_ = L - 0.1055613458 * a - 0.0638541728 * b
    s_ = L - 0.0894841775 * a - 1.2914855480 * b
    l, m, s = l_ ** 3, m_ ** 3, s_ ** 3
    rgb = (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
           -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
           -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)

    def enc(c):
        c = min(1.0, max(0.0, c))
        return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
    return tuple(round(enc(c) * 255) for c in rgb)


def rgba(v):
    r, g, b = oklch_to_srgb(v[0], v[1], v[2])
    return (r, g, b, v[3] if len(v) > 3 else 1.0)


def css(v):
    a = f" / {v[3]}" if len(v) > 3 else ""
    return f"oklch({v[0]}% {v[1]} {v[2]}{a})"


def hexa(v):
    r, g, b, a = rgba(v)
    return f"#{r:02x}{g:02x}{b:02x}" + (f"{round(a * 255):02x}" if a < 1 else "")


def over(fg, bg):
    """Compõe fg (r,g,b,a) sobre bg (r,g,b) em sRGB, como o navegador e o Slint."""
    a = fg[3]
    return tuple(fg[i] * a + bg[i] * (1 - a) for i in range(3))


def lum(c):
    def lin(x):
        x /= 255
        return x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4
    return 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])


def ratio(fg, bg):
    a, b = lum(fg), lum(bg)
    if a < b:
        a, b = b, a
    return (a + 0.05) / (b + 0.05)


WHITE, BLACK = (255, 255, 255), (0, 0, 0)


def checks():
    rows, ok = [], True
    C, G, M = T["color"], T["glass"], T["media"]
    for mode in ("dark", "light"):
        c = {k: rgba(v[mode]) for k, v in C.items()}
        g = {k: rgba(v[mode]) for k, v in G.items() if isinstance(v, dict)}

        def add(name, fg, bg, need):
            nonlocal ok
            r = ratio(fg[:3], bg[:3])
            good = r >= need
            ok = ok and good
            rows.append((mode, name, r, need, good))
        for p in ("paper", "paper-2", "paper-3"):
            add(f"ink sobre {p}", c["ink"], c[p], 7.0)
            add(f"ink-2 sobre {p}", c["ink-2"], c[p], 4.5)
        add("muted sobre paper", c["muted"], c["paper"], 3.0)
        add("muted sobre paper-2", c["muted"], c["paper-2"], 3.0)
        add("accent sobre paper", c["accent"], c["paper"], 4.5)
        add("accent sobre paper-2", c["accent"], c["paper-2"], 4.5)
        add("accent-ink sobre accent", c["accent-ink"], c["accent"], 4.5)
        add("primary-ink sobre primary", c["primary-ink"], c["primary"], 7.0)
        add("error sobre paper", c["error"], c["paper"], 4.5)
        add("focus sobre paper (anel)", c["focus"], c["paper"], 3.0)
        add("ink sobre bubble", c["ink"], c["bubble"], 4.5)
        worst = WHITE if mode == "dark" else BLACK   # o pior conteúdo atrás do vidro
        best = BLACK if mode == "dark" else WHITE
        for tint in ("tint", "tint-sheet"):
            add(f"ink sobre vidro ({tint}) com {'branco' if mode == 'dark' else 'preto'} puro atrás", c["ink"], over(g[tint], worst), 4.5)
            add(f"ink sobre vidro ({tint}) com fundo liso atrás", c["ink"], over(g[tint], c["paper"][:3]), 7.0)
        add("ink sobre vidro sólido (reduzir transparência)", c["ink"], g["solid"], 7.0)
        add("ink-2 sobre vidro sólido", c["ink-2"], g["solid"], 4.5)
        _ = best
    m_ink, plate = rgba(M["ink"]), rgba(M["plate"])
    con = rgba(M["console"])
    for name, bg in (("branco puro", WHITE), ("capa média #808080", (128, 128, 128)), ("preto", BLACK)):
        r = ratio(m_ink[:3], over(plate, bg))
        good = r >= 4.5
        ok = ok and good
        rows.append(("capas", f"texto branco sobre placa de capa com {name} atrás", r, 4.5, good))
    # console do player: alfa = dim calculado por capa (make_covers.py: V*(1-dim) + 12*dim <= 112); texto sempre branco puro
    for dim, bg, nm in ((M["dim-max"], WHITE, "dim máx., branco puro atrás"), (M["dim-min"], (112, 112, 112), "dim mín., cinza 112 atrás (limite da regra)")):
        b = over((con[0], con[1], con[2], dim), bg)
        r = ratio(m_ink[:3], b)
        good = r >= 4.5
        ok = ok and good
        rows.append(("player", f"texto branco no console, {nm}", r, 4.5, good))
    return rows, ok


# ---------- CSS ----------
def gen_css():
    C, G, M, ty, sp, ra, sz, mo = T["color"], T["glass"], T["media"], T["type"], T["space"], T["radius"], T["size"], T["motion"]
    bez = lambda v: f"cubic-bezier({v[0]}, {v[1]}, {v[2]}, {v[3]})"

    def block(mode):
        out = [f"--color-{k}: {css(v[mode])};" for k, v in C.items()]
        out += [f"--glass-{k}: {css(v[mode])};" for k, v in G.items() if isinstance(v, dict)]
        return " ".join(out)
    text = " ".join(f"--text-{k}: {v[0] / 16}rem; --lh-{k}: {v[1] / v[0]:.3f};" for k, v in ty["scale"].items())
    lines = [
        "/* Tokens da v4. GERADO por design/v4/tools/gen_tokens.py a partir de design/v4/tokens.json: não edite à mão. */",
        '@font-face { font-family: "Inter"; src: url("../../fonts/Inter-Regular.ttf") format("truetype"); font-weight: 400; font-style: normal; font-display: swap; }',
        '@font-face { font-family: "Inter"; src: url("../../fonts/Inter-SemiBold.ttf") format("truetype"); font-weight: 600; font-style: normal; font-display: swap; }',
        '@font-face { font-family: "Inter"; src: url("../../fonts/Inter-Bold.ttf") format("truetype"); font-weight: 700; font-style: normal; font-display: swap; }',
        '@font-face { font-family: "Inter Display"; src: url("../../fonts/InterDisplay-Bold.ttf") format("truetype"); font-weight: 700; font-style: normal; font-display: swap; }',
        ":root {",
        "  color-scheme: light dark;",
        "  " + " ".join(f"--space-{k}: {v}px;" for k, v in sp.items()),
        "  " + text,
        "  --text-display-s: var(--text-2xl); --text-display: var(--text-3xl); --text-section: var(--text-xl); --text-title: var(--text-lg); --text-lead: var(--text-lg);",
        f"  --ease-out: {bez(mo['ease-out'])}; --ease-in: {bez(mo['ease-in'])}; --ease-in-out: {bez(mo['ease-in-out'])};",
        f"  --dur-micro: {mo['micro']}ms; --dur-short: {mo['short']}ms; --dur-long: {mo['long']}ms;",
        "  --container: 1180px; --gutter: " + f"{sz['gutter']}px; --gap-columns: 16px;",
        "  --z-base: 1; --z-raised: 10; --z-dropdown: 100; --z-sticky: 200; --z-sticky-nav: 300; --z-modal: 400; --z-toast: 500; --z-tooltip: 600;",
        f'  --font-display: "{ty["family-display"]}", "{ty["family"]}", ui-sans-serif, system-ui, sans-serif; --font-body: "{ty["family"]}", ui-sans-serif, system-ui, sans-serif; --font-outlier: var(--font-body);',
        "  " + " ".join(f"--radius-{k}: {v}px;" for k, v in ra.items()) + " --rule-hair: 1px; --shadow-card: none;",
        f"  --tracking-display: {ty['tracking-display']}em;",
        "  " + " ".join(f"--size-{k}: {v}px;" for k, v in sz.items()),
        f"  --glass-blur: {G['blur']}px; --glass-blur-sheet: {G['blur-sheet']}px; --glass-blur-console: {G['blur-console']}px;",
        f"  --media-ink: {css(M['ink'])}; --media-plate: {css(M['plate'])}; --media-scrim: {css(M['scrim'])}; --media-well: {css(M['well'])}; --media-track: {css(M['track'])}; --media-edge-hi: {css(M['edge-hi'])}; --media-edge-lo: {css(M['edge-lo'])};",
        f"  --media-console: {css(M['console'])}; --media-accent: {css(M['accent'])}; --media-shadow: {css(M['shadow'])};",
        "  --safe-top: env(safe-area-inset-top, 0px); --safe-bottom: env(safe-area-inset-bottom, 0px);",
        "  " + block("light"),
        "}",
        '[data-mode="dark"] { ' + block("dark") + " }",
        "@media (prefers-color-scheme: dark) { :root:not([data-mode]) { " + block("dark") + " } }",
        f"@media (max-width: 22.99rem) {{ :root {{ --gutter: {sz['gutter-s']}px; }} }}",
        "@media (min-width: 37.5rem) { :root { --gutter: 32px; } }",
        "",
    ]
    (ROOT / "prototypes" / "assets" / "tokens.css").write_text("\n".join(lines), encoding="utf-8")


# ---------- Slint ----------
def gen_slint():
    C, G, M, ty, sp, ra, sz, mo = T["color"], T["glass"], T["media"], T["type"], T["space"], T["radius"], T["size"], T["motion"]
    L = ["// Tokens da v4. GERADO por design/v4/tools/gen_tokens.py a partir de design/v4/tokens.json: não edite à mão.",
         "// Slint 1.19. Tudo o que é cor, medida ou tempo na UI vem deste global.",
         'import "../fonts/Inter-Regular.ttf";', 'import "../fonts/Inter-SemiBold.ttf";', 'import "../fonts/Inter-Bold.ttf";', 'import "../fonts/InterDisplay-Bold.ttf";',
         "", "export global Theme {",
         "    // ---- ambiente (o Rust escreve) ----",
         "    in-out property <bool> dark: true;",
         "    in-out property <bool> reduce-transparency: false;   // Ajustes ou sistema",
         "    in-out property <bool> reduce-motion: false;",
         "    in-out property <bool> blur-supported: true;          // false no renderer software",
         "    in-out property <length> window-w: 390px;",
         "    out property <bool> glass-on: blur-supported && !reduce-transparency;",
         "    out property <bool> compact: window-w < 600px;",
         "    out property <bool> wide: window-w >= 1100px;",
         "    out property <length> gutter: window-w < 368px ? %dpx : window-w < 600px ? %dpx : 32px;" % (sz["gutter-s"], sz["gutter"]),
         "", "    // ---- cor ----"]
    for k, v in C.items():
        L.append(f"    out property <color> {k}: dark ? {hexa(v['dark'])} : {hexa(v['light'])};   // {v['use']}")
    L.append("")
    L.append("    // ---- vidro ----")
    for k, v in G.items():
        if isinstance(v, dict):
            L.append(f"    out property <color> glass-{k}: dark ? {hexa(v['dark'])} : {hexa(v['light'])};   // {v['use']}")
    L += [f"    out property <length> glass-blur: {G['blur']}px;", f"    out property <length> glass-blur-sheet: {G['blur-sheet']}px;", f"    out property <length> glass-blur-console: {G['blur-console']}px;", "",
          "    // ---- sobre capas e no player (igual nos dois temas) ----",
          f"    out property <color> media-ink: {hexa(M['ink'])};",
          f"    out property <color> media-plate: {hexa(M['plate'])};", f"    out property <color> media-console: {hexa(M['console'])};",
          f"    out property <color> media-scrim: {hexa(M['scrim'])};", f"    out property <color> media-well: {hexa(M['well'])};", f"    out property <color> media-track: {hexa(M['track'])};",
          f"    out property <color> media-edge-hi: {hexa(M['edge-hi'])};", f"    out property <color> media-edge-lo: {hexa(M['edge-lo'])};",
          f"    out property <color> media-accent: {hexa(M['accent'])};", f"    out property <color> media-shadow: {hexa(M['shadow'])};",
          f"    out property <float> media-dim-min: {M['dim-min']};", f"    out property <float> media-dim-max: {M['dim-max']};", "",
          "    // ---- tipo (Inter; pesos 400, 600, 700) ----",
          f'    out property <string> font: "{ty["family"]}";', f'    out property <string> font-display: "{ty["family-display"]}";']
    for k, v in ty["scale"].items():
        n = k.replace("2xl", "xxl").replace("3xl", "xxxl")
        L.append(f"    out property <length> text-{n}: {v[0]}px;   // {v[3]} (linha {v[1]})")
    L += ["", "    // ---- espaço, raio, tamanho ----"]
    for k, v in sp.items():
        n = "s-" + k.replace("3xs", "xxxs").replace("2xs", "xxs").replace("2xl", "xxl").replace("3xl", "xxxl").replace("4xl", "xxxxl")
        L.append(f"    out property <length> {n}: {v}px;")
    for k, v in ra.items():
        L.append(f"    out property <length> r-{k}: {v}px;")
    for k, v in sz.items():
        if not k.startswith("gutter"):
            L.append(f"    out property <length> {k}-h: {v}px;" if k in ("row", "bar", "mini", "header", "control", "control-s") else f"    out property <length> {k}: {v}px;")
    L += ["", "    // ---- movimento (com Reduzir movimento os deslocamentos somem; fica só o esmaecer curto) ----",
          f"    out property <duration> dur-micro: {mo['micro']}ms;",
          f"    out property <duration> dur-short: reduce-motion ? {mo['micro']}ms : {mo['short']}ms;",
          f"    out property <duration> dur-long: reduce-motion ? {mo['micro']}ms : {mo['long']}ms;",
          f"    // curva de chegada, tipo mola sem ressalto: cubic-bezier({', '.join(str(x) for x in mo['ease-out'])})",
          "}", ""]
    (ROOT / "slint").mkdir(exist_ok=True)
    (ROOT / "slint" / "tokens.slint").write_text("\n".join(L), encoding="utf-8")


def main():
    gen_css()
    gen_slint()
    rows, ok = checks()
    md = ["# Contraste dos tokens (calculado)", "",
          "Gerado por `design/v4/tools/gen_tokens.py`. A composição é feita em sRGB, como no navegador e no Slint.",
          "Os pares do vidro usam o **pior** conteúdo possível atrás do painel (branco puro no tema escuro, preto puro no claro).", "",
          "| Tema | Par | Contraste | Mínimo | |", "|---|---|---|---|---|"]
    for mode, name, r, need, good in rows:
        md.append(f"| {mode} | {name} | {r:.2f}:1 | {need}:1 | {'ok' if good else '**REPROVA**'} |")
    (ROOT / "contrast-tokens.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    bad = [r for r in rows if not r[4]]
    print(f"{len(rows)} pares, {len(bad)} reprovados")
    for mode, name, r, need, _ in bad:
        print(f"  REPROVA {mode}: {name}: {r:.2f} < {need}")
    for k in ("paper", "paper-2", "paper-3", "ink", "ink-2", "muted", "accent", "bubble"):
        print(f"  {k:10s} dark {hexa(T['color'][k]['dark'])}  light {hexa(T['color'][k]['light'])}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
