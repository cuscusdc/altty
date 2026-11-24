#!/usr/bin/env python3
"""Gera: logos/ícone de app/favicons (design/assets), conjunto de ícones (design/assets/icons + sprite),
capas fictícias (design/prototypes/covers) e a cor de destaque derivada de cada capa
(design/prototypes/assets/covers.css + covers.json), usando o MESMO algoritmo descrito em DESIGN.md §3.4.

Requer: rsvg-convert, Pillow, ImageMagick (`magick`). Uso: python3 design/tools/make_assets.py
"""
import tempfile, json, math, subprocess, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_tokens as G  # noqa: E402
from PIL import Image  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
ICONS = ASSETS / "icons"
COVERS = ROOT / "prototypes" / "covers"
for d in (ASSETS, ICONS, COVERS, ROOT / "prototypes" / "assets"):
    d.mkdir(parents=True, exist_ok=True)

DK = {k: G.to_hex(G.parse(v)) for k, v in G.colors("dark").items()}
LT = {k: G.to_hex(G.parse(v)) for k, v in G.colors("light").items()}
EMBER = G.to_hex(G.brand["ember"])
EMBER_LIGHT = G.to_hex(G.brand["ember-on-light"])

# =============================================================================================
# 1. MARCA — "K de barras": haste + divisa destacada (chevron "<"), traço 8/64, pontas redondas.
# =============================================================================================
SW = 8  # espessura do traço na grade 64


def mark_group(stem, chev, sw=SW, extra=""):
    return (f'<g fill="none" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round"{extra}>'
            f'<path d="M20 14V50" stroke="{stem}"/>'
            f'<path d="M45 14 31.5 32 45 50" stroke="{chev}"/></g>')


# Wordmark "kazzu": letras monolinha desenhadas à mão (traço 5, x-height 26 numa caixa de 40).
WM_SW = 5
WM_PARTS = [
    ("k", 0, 'M3 2.5V37.5', 'M23 16.5 11 27 23 37.5'),
    ("a", 33, None, None),
    ("z", 66, 'M4 16.5H22L4 37.5H22', None),
    ("z", 96, 'M4 16.5H22L4 37.5H22', None),
    ("u", 126, 'M3 16.5V28A9.5 9.5 0 0 0 22 28V16.5', 'M22 16.5V37.5'),
]
WM_W, WM_H = 152, 40


def wordmark_inner(color, chev_color=None):
    chev_color = chev_color or color
    out = [f'<g fill="none" stroke-width="{WM_SW}" stroke-linecap="round" stroke-linejoin="round">']
    for name, x, p1, p2 in WM_PARTS:
        out.append(f'<g transform="translate({x} 0)">')
        if name == "k":
            out.append(f'<path d="{p1}" stroke="{color}"/><path d="{p2}" stroke="{chev_color}"/>')
        elif name == "a":
            out.append(f'<circle cx="13" cy="27" r="10.5" stroke="{color}"/><path d="M23.5 16.5V37.5" stroke="{color}"/>')
        else:
            out.append(f'<path d="{p1}" stroke="{color}"/>')
            if p2:
                out.append(f'<path d="{p2}" stroke="{color}"/>')
        out.append("</g>")
    out.append("</g>")
    return "".join(out)


def svg(w, h, inner, vb=None, extra=""):
    vb = vb or f"0 0 {w} {h}"
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{vb}"{extra}>{inner}</svg>\n'


def write(path, text):
    Path(path).write_text(text, encoding="utf-8")


ink_d, ink_l = DK["ink"], LT["ink"]
# marca isolada (64x64) — tema escuro / claro / mono
write(ASSETS / "kazzu-mark-dark.svg", svg(64, 64, mark_group(ink_d, EMBER)))
write(ASSETS / "kazzu-mark-light.svg", svg(64, 64, mark_group(ink_l, EMBER_LIGHT)))
write(ASSETS / "kazzu-mark-mono.svg", svg(64, 64, mark_group("currentColor", "currentColor")))
# wordmark isolado
write(ASSETS / "kazzu-wordmark-dark.svg", svg(WM_W, WM_H, wordmark_inner(ink_d, EMBER), vb=f"-1 0 {WM_W+1} {WM_H}"))
write(ASSETS / "kazzu-wordmark-light.svg", svg(WM_W, WM_H, wordmark_inner(ink_l, EMBER_LIGHT), vb=f"-1 0 {WM_W+1} {WM_H}"))
write(ASSETS / "kazzu-wordmark-mono.svg", svg(WM_W, WM_H, wordmark_inner("currentColor"), vb=f"-1 0 {WM_W+1} {WM_H}"))


# lockup horizontal: tile do ícone (grafite) + wordmark. O tile é igual nos dois temas; só o wordmark muda.
TILE_TOP = G.to_hex(G.parse("oklch(21% 0.014 55)"))
TILE_BOT = G.to_hex(G.parse("oklch(13% 0.010 55)"))


def lockup(word, word_chev, uid):
    tile = (f'<defs><linearGradient id="t{uid}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{TILE_TOP}"/><stop offset="1" stop-color="{TILE_BOT}"/></linearGradient></defs>'
            f'<g transform="translate(0 2)"><rect width="44" height="44" rx="12" fill="url(#t{uid})"/>'
            f'<g transform="translate(4.2 4.4) scale(.56)">{mark_group(ink_d, EMBER)}</g></g>')
    w = f'<g transform="translate(58 8) scale(.8)">{wordmark_inner(word, word_chev)}</g>'
    return svg(184, 48, tile + w, vb="-1 0 186 48")


write(ASSETS / "kazzu-logo-dark.svg", lockup(ink_d, EMBER, "d"))
write(ASSETS / "kazzu-logo-light.svg", lockup(ink_l, EMBER_LIGHT, "l"))
write(ASSETS / "kazzu-logo-mono.svg", lockup("currentColor", "currentColor", "m"))

# ícone de app (1024): squircle grafite, haste clara + divisa âmbar. Sem glow; só um fio de luz no topo.
BG_TOP = G.to_hex(G.parse("oklch(21% 0.014 55)"))
BG_BOT = G.to_hex(G.parse("oklch(13% 0.010 55)"))
def _squircle(n=5.0, steps=160):
    pts = []
    for i in range(steps):
        t = 2 * math.pi * i / steps
        c, s_ = math.cos(t), math.sin(t)
        x = 512 + 512 * math.copysign(abs(c) ** (2 / n), c)
        y = 512 + 512 * math.copysign(abs(s_) ** (2 / n), s_)
        pts.append(f"{x:.1f} {y:.1f}")
    return "M" + "L".join(pts) + "Z"


SQ = _squircle()
icon_inner = (
    f'<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{BG_TOP}"/><stop offset="1" stop-color="{BG_BOT}"/></linearGradient>'
    f'<linearGradient id="rim" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".22"/><stop offset=".35" stop-color="#fff" stop-opacity="0"/></linearGradient>'
    f'<clipPath id="c"><path d="{SQ}"/></clipPath></defs>'
    f'<path d="{SQ}" fill="url(#bg)"/>'
    f'<path d="{SQ}" fill="none" stroke="url(#rim)" stroke-width="6" clip-path="url(#c)" transform="translate(0 0)"/>'
    f'<g transform="translate(154.5 160) scale(11)">{mark_group(ink_d, EMBER)}</g>'
)
write(ASSETS / "kazzu-icon.svg", svg(1024, 1024, icon_inner))
# favicon: marca sem fundo, legível em 16px (traço mais grosso)
fav = svg(64, 64, f'<rect width="64" height="64" rx="14" fill="{BG_BOT}"/>' + mark_group(ink_d, EMBER, sw=9, extra=' transform="translate(2 0) scale(.96) translate(1.5 1.3)"'))
write(ASSETS / "favicon.svg", fav)

for size in (16, 32, 48, 64, 128, 180, 256, 512, 1024):
    subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), str(ASSETS / ("favicon.svg" if size <= 48 else "kazzu-icon.svg")),
                    "-o", str(ASSETS / f"icon-{size}.png")], check=True)
subprocess.run(["magick", str(ASSETS / "icon-16.png"), str(ASSETS / "icon-32.png"), str(ASSETS / "icon-48.png"), str(ASSETS / "favicon.ico")], check=True)
for size in (64, 128, 180):
    (ASSETS / f"icon-{size}.png").unlink()  # só 16/32/48 (favicon) e 256/512/1024 (app) ficam

# =============================================================================================
# 2. ÍCONES — grade 24, traço 1.75, pontas redondas. (filled=True → preenchido)
# =============================================================================================
ICON_DEFS = {
    # navegação
    "home": ("stroke", '<path d="M4 11.2 12 4l8 7.2V19a1.5 1.5 0 0 1-1.5 1.5H15V15H9v5.5H5.5A1.5 1.5 0 0 1 4 19z"/>'),
    "explore": ("stroke", '<circle cx="12" cy="12" r="8.75"/><path d="m15.6 8.4-1.9 5.3-5.3 1.9 1.9-5.3z"/>'),
    "library": ("stroke", '<rect x="4" y="9.5" width="16" height="10.5" rx="2.5"/><path d="M7 6h10M9.5 2.75h5"/>'),
    "home-fill": ("fillstroke", '<path d="M4 11.2 12 4l8 7.2V19a1.5 1.5 0 0 1-1.5 1.5H15V15H9v5.5H5.5A1.5 1.5 0 0 1 4 19z"/>'),
    "explore-fill": ("fill", '<path fill-rule="evenodd" d="M12 3.25a8.75 8.75 0 1 0 0 17.5 8.75 8.75 0 0 0 0-17.5zM15.6 8.4l-1.9 5.3-5.3 1.9 1.9-5.3z"/>'),
    "library-fill": ("fillstroke", '<rect x="4" y="9.5" width="16" height="10.5" rx="2.5"/><path d="M7 6h10M9.5 2.75h5"/>'),
    "search": ("stroke", '<circle cx="10.5" cy="10.5" r="6.25"/><path d="m15.2 15.2 4.8 4.8"/>'),
    "back": ("stroke", '<path d="M19 12H5M11 6l-6 6 6 6"/>'),
    "chevron-right": ("stroke", '<path d="m9.5 6 6 6-6 6"/>'),
    "chevron-left": ("stroke", '<path d="m14.5 6-6 6 6 6"/>'),
    "chevron-down": ("stroke", '<path d="m6 9.5 6 6 6-6"/>'),
    "close": ("stroke", '<path d="m6 6 12 12M18 6 6 18"/>'),
    "check": ("stroke", '<path d="m5 12.5 4.5 4.5L19 7.5"/>'),
    "plus": ("stroke", '<path d="M12 5v14M5 12h14"/>'),
    "more": ("fill", '<circle cx="5.5" cy="12" r="1.6"/><circle cx="12" cy="12" r="1.6"/><circle cx="18.5" cy="12" r="1.6"/>'),
    "more-v": ("fill", '<circle cx="12" cy="5.5" r="1.6"/><circle cx="12" cy="12" r="1.6"/><circle cx="12" cy="18.5" r="1.6"/>'),
    # transporte (preenchidos)
    "play": ("fill", '<path d="M8.2 5.9v12.2a1 1 0 0 0 1.52.85l9.9-6.1a1 1 0 0 0 0-1.7l-9.9-6.1A1 1 0 0 0 8.2 5.9z"/>'),
    "pause": ("fill", '<rect x="6.25" y="5" width="4.1" height="14" rx="1.6"/><rect x="13.65" y="5" width="4.1" height="14" rx="1.6"/>'),
    "prev": ("fill", '<rect x="4.6" y="5" width="2.6" height="14" rx="1.3"/><path d="M19.2 6.4v11.2a.9.9 0 0 1-1.4.75l-8.1-5.6a.9.9 0 0 1 0-1.5l8.1-5.6a.9.9 0 0 1 1.4.75z"/>'),
    "next": ("fill", '<rect x="16.8" y="5" width="2.6" height="14" rx="1.3"/><path d="M4.8 6.4v11.2a.9.9 0 0 0 1.4.75l8.1-5.6a.9.9 0 0 0 0-1.5L6.2 5.65a.9.9 0 0 0-1.4.75z"/>'),
    "shuffle": ("stroke", '<path d="M3.5 7H6.8a4 4 0 0 1 3.3 1.8l4 6.4a4 4 0 0 0 3.3 1.8H20.5M3.5 17H6.8a4 4 0 0 0 3.3-1.8M14 8.8A4 4 0 0 1 17.3 7H20.5M17.8 4.3 20.5 7l-2.7 2.7M17.8 14.3l2.7 2.7-2.7 2.7"/>'),
    "repeat": ("stroke", '<path d="M3.5 11V9.5A3.5 3.5 0 0 1 7 6h13.5M17.6 3l3 3-3 3M20.5 13v1.5A3.5 3.5 0 0 1 17 18H3.5M6.4 15l-3 3 3 3"/>'),
    "heart": ("stroke", '<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.3a4.3 4.3 0 0 1 7.5 2.5C19.5 15.4 12 20 12 20z"/>'),
    "heart-fill": ("fillstroke", '<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.3a4.3 4.3 0 0 1 7.5 2.5C19.5 15.4 12 20 12 20z"/>'),
    "vol-low": ("stroke", '<path d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5z"/><path d="M15.5 9.5a3.5 3.5 0 0 1 0 5"/>'),
    "vol-high": ("stroke", '<path d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5z"/><path d="M15.5 9.5a3.5 3.5 0 0 1 0 5M18.2 6.8a7 7 0 0 1 0 10.4"/>'),
    "vol-mute": ("stroke", '<path d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5z"/><path d="m16 9.5 4.5 5M20.5 9.5l-4.5 5"/>'),
    "queue": ("stroke", '<path d="M4 7h10.5M4 12h10.5M4 17h5.5"/><circle cx="16.5" cy="17" r="2.25"/><path d="M18.75 17V8.5c1.2.1 2.1.7 2.5 1.8"/>'),
    "playlist-add": ("stroke", '<path d="M4 7h11M4 12h7M4 17h7M17.5 13.5v7M14 17h7"/>'),
    "lyrics": ("stroke", '<path d="M5.5 4.75h13A1.75 1.75 0 0 1 20.25 6.5v8A1.75 1.75 0 0 1 18.5 16.25H11.4L7 19.75v-3.5H5.5A1.75 1.75 0 0 1 3.75 14.5v-8A1.75 1.75 0 0 1 5.5 4.75z"/><path d="M8 9.25h8M8 12.25h5"/>'),
    "output": ("stroke", '<path d="M6 17.5H4.75A1.75 1.75 0 0 1 3 15.75V6.5a1.75 1.75 0 0 1 1.75-1.75h14.5A1.75 1.75 0 0 1 21 6.5v9.25a1.75 1.75 0 0 1-1.75 1.75H18"/><path d="m12 14 5 6.25H7z"/>'),
    # biblioteca
    "download": ("stroke", '<path d="M12 4v11M7.5 11 12 15.5 16.5 11M5 19.5h14"/>'),
    "disc": ("stroke", '<circle cx="12" cy="12" r="8.75"/><circle cx="12" cy="12" r="2.5"/><path d="M12 3.25a8.75 8.75 0 0 1 8.75 8.75"/>'),
    "folder": ("stroke", '<path d="M3.75 7.5A1.75 1.75 0 0 1 5.5 5.75h3.9l2.1 2.25h7A1.75 1.75 0 0 1 20.25 9.75v7.75a1.75 1.75 0 0 1-1.75 1.75h-13A1.75 1.75 0 0 1 3.75 17.5z"/>'),
    "device": ("stroke", '<rect x="6.75" y="3" width="10.5" height="18" rx="2.75"/><path d="M10.5 17.75h3"/>'),
    "clock": ("stroke", '<circle cx="12" cy="12" r="8.75"/><path d="M12 7.25V12l3.2 2"/>'),
    "wave": ("stroke", '<path d="M12 4.5v15M8 8v8M4 10.5v3M16 8v8M20 10.5v3"/>'),
    "trash": ("stroke", '<path d="M4.5 7h15M9.5 7V4.75h5V7M6.75 7l.85 11.7a1.75 1.75 0 0 0 1.75 1.55h5.3a1.75 1.75 0 0 0 1.75-1.55L17.25 7"/>'),
    "grip": ("fill", '<circle cx="9" cy="6.5" r="1.4"/><circle cx="15" cy="6.5" r="1.4"/><circle cx="9" cy="12" r="1.4"/><circle cx="15" cy="12" r="1.4"/><circle cx="9" cy="17.5" r="1.4"/><circle cx="15" cy="17.5" r="1.4"/>'),
    # configurações
    "sliders": ("stroke", '<path d="M4 7h8M17 7h3M4 17h3M12 17h8"/><circle cx="14.5" cy="7" r="2.25"/><circle cx="9.5" cy="17" r="2.25"/>'),
    "eq": ("stroke", '<path d="M6 4v16M12 4v16M18 4v16"/><circle cx="6" cy="15" r="2.25" class="knob"/><circle cx="12" cy="8.5" r="2.25" class="knob"/><circle cx="18" cy="13" r="2.25" class="knob"/>'),
    "user": ("stroke", '<circle cx="12" cy="8.5" r="3.75"/><path d="M5 20c.6-3.7 3.5-5.75 7-5.75S18.4 16.3 19 20"/>'),
    "shield": ("stroke", '<path d="M12 3.25 5 6v5.5c0 4.4 2.9 7.4 7 9.25 4.1-1.85 7-4.85 7-9.25V6z"/><path d="m9 12 2.2 2.2L15.2 10"/>'),
    "lock": ("stroke", '<rect x="5" y="10.5" width="14" height="9.75" rx="2.5"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"/>'),
    "moon": ("stroke", '<path d="M19.5 14.2A7.75 7.75 0 0 1 9.8 4.5a7.75 7.75 0 1 0 9.7 9.7z"/>'),
    "sun": ("stroke", '<circle cx="12" cy="12" r="4"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6 7 7M17 17l1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4"/>'),
    "info": ("stroke", '<circle cx="12" cy="12" r="8.75"/><path d="M12 11v5.25"/><circle cx="12" cy="7.9" r=".6" fill="currentColor"/>'),
    "gear": ("stroke", '<circle cx="12" cy="12" r="3.1"/><path d="M12 3.5l1.3 2.1 2.4-.6.9 2.3 2.3.9-.6 2.4 2.1 1.3-2.1 1.3.6 2.4-2.3.9-.9 2.3-2.4-.6L12 20.5l-1.3-2.1-2.4.6-.9-2.3-2.3-.9.6-2.4L3.5 12l2.1-1.3-.6-2.4 2.3-.9.9-2.3 2.4.6z"/>'),
    "refresh": ("stroke", '<path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3M19.5 4.5v4h-4"/>'),
    "cloud": ("stroke", '<path d="M7 18.5a4.25 4.25 0 0 1-.6-8.45 5.75 5.75 0 0 1 11.1 1.2 3.65 3.65 0 0 1-.4 7.25z"/>'),
    "stats": ("stroke", '<path d="M5 20V12M12 20V5M19 20v-5"/>'),
    "timer": ("stroke", '<circle cx="12" cy="13.5" r="7"/><path d="M12 10v3.5l2.2 1.3M9.5 3.5h5"/>'),
    "arrow-up": ("stroke", '<path d="M12 19V5M6 11l6-6 6 6"/>'),
    "arrow-down": ("stroke", '<path d="M12 5v14M6 13l6 6 6-6"/>'),
    "minus": ("stroke", '<path d="M5 12h14"/>'),
    "badge-check": ("stroke", '<circle cx="12" cy="12" r="8.75"/><path d="m8.2 12.3 2.6 2.6 5-5.4"/>'),
    "cloud-off": ("stroke", '<path d="M4 4l16 16M8.6 8.2A5 5 0 0 0 7 18h10.6M17 10.2a4 4 0 0 1 3 3.8"/>'),
}


def icon_svg(name):
    kind, body = ICON_DEFS[name]
    if kind == "stroke":
        attrs = 'fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"'
    elif kind == "fill":
        attrs = 'fill="currentColor" stroke="currentColor" stroke-width="0.5" stroke-linejoin="round"'
    else:
        attrs = 'fill="currentColor" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"'
    return attrs, body


sprite = ['<svg xmlns="http://www.w3.org/2000/svg" style="display:none" aria-hidden="true">']
for name in ICON_DEFS:
    attrs, body = icon_svg(name)
    write(ICONS / f"{name}.svg", f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" {attrs}>{body}</svg>\n')
    sprite.append(f'<symbol id="i-{name}" viewBox="0 0 24 24" {attrs}>{body}</symbol>')
sprite.append("</svg>")
write(ROOT / "prototypes" / "assets" / "sprite.svg.html", "\n".join(sprite))

# folha de ícones para revisão visual
_sh = ((len(ICON_DEFS) + 11) // 12) * 88 + 16
sheet = [f'<svg xmlns="http://www.w3.org/2000/svg" width="1056" height="{_sh}" viewBox="0 0 1056 {_sh}">']
sheet.append(f'<rect width="100%" height="100%" fill="{DK["bg"]}"/>')
for i, name in enumerate(ICON_DEFS):
    attrs, body = icon_svg(name)
    x, y = 16 + (i % 12) * 84, 16 + (i // 12) * 88
    sheet.append(f'<g transform="translate({x} {y}) scale(2.4)" color="{DK["ink"]}" {attrs}>{body}</g>')
sheet.append("</svg>")
write(ICONS / "_sheet.svg", "".join(sheet))

# =============================================================================================
# 3. CAPAS FICTÍCIAS (640×640) — abstratas, sem texto, sem arte de terceiros
# =============================================================================================
W = 640


def cover(idx, body, defs=""):
    return svg(W, W, f"<defs>{defs}</defs>{body}", vb=f"0 0 {W} {W}")


COVERS_DEF = []  # (id, titulo, artista, svg)

COVERS_DEF.append(("01", "Cidade Submersa", "Aurora Vento", cover(1, """
<rect width="640" height="640" fill="url(#g)"/>
<circle cx="320" cy="360" r="132" fill="url(#sun)"/>
<g fill="none" stroke="#0a2f38" stroke-opacity=".55" stroke-width="9" stroke-linecap="round">
<path d="M-20 400C90 372 190 428 320 400S540 372 660 400"/><path d="M-20 452C90 424 190 480 320 452S540 424 660 452"/>
<path d="M-20 504C90 476 190 532 320 504S540 476 660 504"/><path d="M-20 556C90 528 190 584 320 556S540 528 660 556"/></g>
<rect y="380" width="640" height="260" fill="#0a2f38" fill-opacity=".38"/>""",
"""<linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1a7a82"/><stop offset="1" stop-color="#0b3640"/></linearGradient>
<linearGradient id="sun" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffc58a"/><stop offset="1" stop-color="#f2803a"/></linearGradient>""")))

COVERS_DEF.append(("02", "Nove Horas", "Doma", cover(2, """
<rect width="640" height="640" fill="#2a1710"/>
<g stroke-linecap="round" fill="none">
<path d="M80 150H560" stroke="#ff9d5c" stroke-width="64"/><path d="M80 236H560" stroke="#ff7a4a" stroke-width="56"/>
<path d="M80 314H560" stroke="#f2574a" stroke-width="48"/><path d="M80 384H560" stroke="#d93f4c" stroke-width="40"/>
<path d="M80 446H560" stroke="#b0304a" stroke-width="32"/><path d="M80 498H560" stroke="#7d2644" stroke-width="24"/>
<path d="M80 540H560" stroke="#4a2036" stroke-width="16"/></g>""")))

COVERS_DEF.append(("03", "Bruma", "Lia Cantos", cover(3, """
<rect width="640" height="640" fill="#cfd6df"/>
<g filter="url(#b)"><circle cx="200" cy="220" r="190" fill="#9db4cf"/><circle cx="470" cy="300" r="170" fill="#f3c5ad"/><circle cx="300" cy="500" r="190" fill="#b9c6d6"/><circle cx="520" cy="540" r="110" fill="#f0a98b"/></g>""",
"""<filter id="b" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="46"/></filter>""")))

COVERS_DEF.append(("04", "Quadra 7", "Os Faróis", cover(4, """
<rect width="640" height="640" fill="#f2c230"/>
<circle cx="214" cy="250" r="150" fill="#16130f"/>
<path d="M640 640V300A340 340 0 0 0 300 640z" fill="#2f5fb4"/>
<rect x="60" y="470" width="190" height="110" rx="10" fill="#e4453a"/>
<rect x="430" y="60" width="150" height="150" rx="75" fill="none" stroke="#16130f" stroke-width="22"/>""")))

COVERS_DEF.append(("05", "Sal e Cinza", "Tomás Reis", cover(5, """
<rect width="640" height="640" fill="url(#g)"/>
<circle cx="320" cy="300" r="170" fill="url(#c)"/>
<circle cx="450" cy="205" r="26" fill="#d8402f"/>
<rect x="150" y="520" width="340" height="8" rx="4" fill="#8e8a82"/>""",
"""<linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f1eee8"/><stop offset="1" stop-color="#dcd8cf"/></linearGradient>
<radialGradient id="c" cx=".35" cy=".3" r=".85"><stop offset="0" stop-color="#d4d1ca"/><stop offset="1" stop-color="#6d6a64"/></radialGradient>""")))

COVERS_DEF.append(("06", "Linha de Fuga", "Meridiano", cover(6, """
<rect width="640" height="640" fill="#0b1510"/>
<g fill="none" stroke="#5fd08a" stroke-width="5">
<circle cx="320" cy="320" r="60"/><circle cx="320" cy="320" r="120" stroke-opacity=".85"/><circle cx="320" cy="320" r="190" stroke-opacity=".65"/>
<circle cx="320" cy="320" r="270" stroke-opacity=".45"/><circle cx="320" cy="320" r="360" stroke-opacity=".28"/></g>
<g stroke="#5fd08a" stroke-width="5" stroke-opacity=".55"><path d="M320 0V640"/><path d="M0 320H640"/></g>
<circle cx="320" cy="320" r="22" fill="#9cf2b5"/>""")))

COVERS_DEF.append(("07", "Calor de Março", "Nina Sabiá", cover(7, """
<rect width="640" height="640" fill="#f0a35a"/>
<g fill="none" stroke-width="64">
<path d="M-40 640A360 360 0 0 1 680 640" stroke="#d9503f"/><path d="M40 640A280 280 0 0 1 600 640" stroke="#f26b4a"/>
<path d="M120 640A200 200 0 0 1 520 640" stroke="#f7894f"/><path d="M200 640A120 120 0 0 1 440 640" stroke="#fbd5a0"/></g>
<circle cx="500" cy="130" r="58" fill="#fbe6bd"/>""")))

COVERS_DEF.append(("08", "Vidro Líquido", "Kaito e Vê", cover(8, """
<rect width="640" height="640" fill="url(#g)"/>
<g fill="none" stroke="#bfe3ff" stroke-linecap="round">
<path d="M-20 200C120 120 240 280 380 200S560 120 660 190" stroke-width="30" stroke-opacity=".9"/>
<path d="M-20 300C120 220 240 380 380 300S560 220 660 290" stroke-width="30" stroke-opacity=".62"/>
<path d="M-20 400C120 320 240 480 380 400S560 320 660 390" stroke-width="30" stroke-opacity=".38"/>
<path d="M-20 500C120 420 240 580 380 500S560 420 660 490" stroke-width="30" stroke-opacity=".2"/></g>""",
"""<linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#2d7fd1"/><stop offset="1" stop-color="#0f2f66"/></linearGradient>""")))

COVERS_DEF.append(("09", "Papel de Seda", "Ilha Branca", cover(9, """
<rect width="640" height="640" fill="#f6e4d8"/>
<path d="M60 560C60 360 200 300 320 300S580 360 580 560z" fill="#e9a58f"/>
<circle cx="330" cy="215" r="96" fill="#f8c9a4"/>
<path d="M-10 640V470C110 420 200 520 330 470S540 420 650 480V640z" fill="#d9776a" fill-opacity=".85"/>""")))

COVERS_DEF.append(("10", "Fita Magnética", "Rádio Cobre", cover(10, """
<rect width="640" height="640" fill="#2b2118"/>
<circle cx="200" cy="300" r="120" fill="#c98f3a"/><circle cx="440" cy="300" r="120" fill="#c98f3a"/>
<circle cx="200" cy="300" r="52" fill="#2b2118"/><circle cx="440" cy="300" r="52" fill="#2b2118"/>
<g stroke="#e9b86a" stroke-width="8" stroke-linecap="round"><path d="M200 248V352M148 300H252"/><path d="M440 248V352M388 300H492"/></g>
<rect x="110" y="470" width="420" height="64" rx="32" fill="#e9b86a"/><rect x="150" y="490" width="340" height="24" rx="12" fill="#2b2118"/>""")))

# ---- algoritmo de cor de destaque (espelha DESIGN.md §3.4) ----


def srgb_to_oklch(r, g, b):
    r, g, b = (G.gamma_to_lin(c / 255) for c in (r, g, b))
    l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
    m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
    s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
    l_, m_, s_ = l ** (1 / 3), m ** (1 / 3), s ** (1 / 3)
    L = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_
    a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_
    bb = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    return L, math.hypot(a, bb), math.degrees(math.atan2(bb, a)) % 360


def derive_accent(png):
    img = Image.open(png).convert("RGB").resize((32, 32), Image.BILINEAR)
    bins = [dict(w=0.0, L=0.0, C=0.0, x=0.0, y=0.0) for _ in range(12)]
    total = 0.0
    for r, g, b in img.getdata():
        L, C, h = srgb_to_oklch(r, g, b)
        if C < 0.04 or L < 0.18 or L > 0.97:
            continue
        w = (C ** 1.5) * (1 - abs(L - 0.62))
        k = int(h // 30) % 12
        d = bins[k]
        d["w"] += w; d["L"] += w * L; d["C"] += w * C
        d["x"] += w * math.cos(math.radians(h)); d["y"] += w * math.sin(math.radians(h))
        total += w
    if total < 0.6:
        return None  # capa quase monocromática → usa o âmbar da marca
    score = [bins[k]["w"] + 0.5 * (bins[(k - 1) % 12]["w"] + bins[(k + 1) % 12]["w"]) for k in range(12)]
    best = bins[max(range(12), key=lambda k: score[k])]
    L, C = best["L"] / best["w"], best["C"] / best["w"]
    h = math.degrees(math.atan2(best["y"], best["x"])) % 360
    return L, C, h


def clamp(x, lo, hi):
    return max(lo, min(hi, x))


def accent_for(mode, base):
    dyn = G.T["color"]["dynamic-accent"]
    if base is None:
        brand = G.brand["ember"] if mode == "dark" else G.brand["ember-on-light"]
        ink = G.brand["ember-ink-on-dark"] if mode == "dark" else G.brand["ember-ink-on-light"]
        return G.to_hex(brand), G.to_hex(ink)
    L, C, h = base
    lo, hi = dyn["dark-lightness" if mode == "dark" else "light-lightness"]
    cl, ch = dyn["chroma"]
    L2, C2 = clamp(L, lo, hi), clamp(C, cl, ch)
    rgb = G.oklch_to_srgb(L2, C2, h)
    while not G.in_gamut(rgb) and C2 > 0:
        C2 -= 0.002
        rgb = G.oklch_to_srgb(L2, C2, h)
    col = tuple(G.lin_to_gamma(c) for c in rgb) + (1.0,)
    dark_ink = (0.07, 0.06, 0.05, 1.0)
    white = (0.99, 0.985, 0.97, 1.0)
    ink = dark_ink if G.contrast(dark_ink, col) >= G.contrast(white, col) else white
    return G.to_hex(col), G.to_hex(ink)


meta = []
css = ["/* GERADO por design/tools/make_assets.py */"]
for cid, title, artist, body in COVERS_DEF:
    svg_path = COVERS / f"cover-{cid}.svg"
    write(svg_path, body)
    png = Path(tempfile.gettempdir()) / f"kazzu-c{cid}.png"
    subprocess.run(["rsvg-convert", "-w", "64", "-h", "64", str(svg_path), "-o", str(png)], check=True)
    base = derive_accent(png)
    ad, ad_ink = accent_for("dark", base)
    al, al_ink = accent_for("light", base)
    # cor média (para o fundo pré-desfocado de fallback)
    avg = Image.open(png).convert("RGB").resize((1, 1), Image.BOX).getpixel((0, 0))
    meta.append(dict(id=cid, title=title, artist=artist, file=f"covers/cover-{cid}.svg", accent_dark=ad, accent_dark_ink=ad_ink,
                     accent_light=al, accent_light_ink=al_ink, avg="#%02x%02x%02x" % avg,
                     derived=None if base is None else dict(L=round(base[0], 3), C=round(base[1], 3), h=round(base[2], 1))))
    css.append(f".cv-{cid} {{ --cv-acc: {ad}; --cv-acc-ink: {ad_ink}; --cv-acc-l: {al}; --cv-acc-ink-l: {al_ink}; --cv-avg: #{('%02x%02x%02x' % avg)}; }}")
(ROOT / "prototypes" / "assets" / "covers.css").write_text("\n".join(css) + "\n", encoding="utf-8")
(ROOT / "prototypes" / "assets" / "covers.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1), encoding="utf-8")
for m in meta:
    print(m["id"], m["title"].ljust(18), m["accent_dark"], m["accent_light"], m["derived"])
print("ok: assets, ícones, capas")
