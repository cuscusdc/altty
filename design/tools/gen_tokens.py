#!/usr/bin/env python3
"""Gera design/slint/tokens.slint e design/prototypes/assets/tokens.css a partir de design/tokens.json
e confere os contrastes WCAG AA dos pares de cores.

Uso:  python3 design/tools/gen_tokens.py [--check]
Sem dependências além da biblioteca padrão.
"""
import json, math, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
T = json.loads((ROOT / "tokens.json").read_text(encoding="utf-8"))

# ---------- cor ----------------------------------------------------------------------------------


def oklch_to_srgb(L, C, h):
    a = C * math.cos(math.radians(h))
    b = C * math.sin(math.radians(h))
    l_ = L + 0.3963377774 * a + 0.2158037573 * b
    m_ = L - 0.1055613458 * a - 0.0638541728 * b
    s_ = L - 0.0894841775 * a - 1.2914855480 * b
    l, m, s = l_ ** 3, m_ ** 3, s_ ** 3
    r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s
    g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s
    bb = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    return r, g, bb


def lin_to_gamma(x):
    x = max(0.0, min(1.0, x))
    return 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055


def gamma_to_lin(x):
    return x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4


def in_gamut(rgb):
    return all(-1e-4 <= c <= 1 + 1e-4 for c in rgb)


def parse(value):
    """-> (r, g, b, a) em 0..1 sRGB gama. Reduz croma até caber no gamut."""
    v = value.strip()
    if v.startswith("#"):
        v = v[1:]
        r, g, b = (int(v[i:i + 2], 16) / 255 for i in (0, 2, 4))
        a = int(v[6:8], 16) / 255 if len(v) == 8 else 1.0
        return r, g, b, a
    m = re.fullmatch(r"oklch\(\s*([\d.]+)%\s+([\d.]+)\s+([\d.]+)\s*(?:/\s*([\d.]+)\s*)?\)", v)
    if not m:
        raise ValueError(f"cor inválida: {value}")
    L, C, h = float(m[1]) / 100, float(m[2]), float(m[3])
    a = float(m[4]) if m[4] else 1.0
    lin = oklch_to_srgb(L, C, h)
    while not in_gamut(lin) and C > 0:
        C -= 0.002
        lin = oklch_to_srgb(L, C, h)
    r, g, b = (lin_to_gamma(c) for c in lin)
    return r, g, b, a


def to_hex(rgba, force_alpha=False):
    r, g, b, a = rgba
    s = "#%02x%02x%02x" % tuple(round(c * 255) for c in (r, g, b))
    if a < 0.999 or force_alpha:
        s += "%02x" % round(a * 255)
    return s


def luminance(rgb):
    r, g, b = (gamma_to_lin(c) for c in rgb[:3])
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def over(fg, bg):
    r, g, b, a = fg
    return tuple(fg[i] * a + bg[i] * (1 - a) for i in range(3)) + (1.0,)


def contrast(fg, bg):
    bg = (bg[0], bg[1], bg[2], 1.0)
    fg = over(fg, bg) if fg[3] < 1 else fg
    l1, l2 = luminance(fg), luminance(bg)
    if l1 < l2:
        l1, l2 = l2, l1
    return (l1 + 0.05) / (l2 + 0.05)


def kebab(s):
    return s.replace("_", "-")


def colors(group):
    return {k: v for k, v in T["color"][group].items() if isinstance(v, str) and not k.startswith("_")}


brand = {k: parse(v) for k, v in T["color"]["brand"].items()}

# ---------- verificação de contraste -------------------------------------------------------------

failures = []


def check(name, fg, bg, minimum):
    c = contrast(fg, bg)
    ok = c >= minimum
    print(f"  {'ok ' if ok else 'FALHA'} {name:44s} {c:5.2f}:1  (mín {minimum})")
    if not ok:
        failures.append(name)


def run_checks():
    for mode, acc, acc_ink in (("dark", brand["ember"], brand["ember-ink-on-dark"]),
                               ("light", brand["ember-on-light"], brand["ember-ink-on-light"])):
        c = {k: parse(v) for k, v in colors(mode).items()}
        print(f"[{mode}]")
        for s in ("bg", "surface", "surface-2", "surface-3"):
            check(f"ink / {s}", c["ink"], c[s], 7.0 if s == "bg" else 4.5)
        for s in ("bg", "surface", "surface-2"):
            check(f"ink-2 / {s}", c["ink-2"], c[s], 4.5)
        for s in ("bg", "surface"):
            check(f"ink-3 / {s}", c["ink-3"], c[s], 4.5)
        check("accent-ink / accent", acc_ink, acc, 4.5)
        check("accent / bg (texto de destaque 4.5:1)", acc, c["bg"], 4.5)
        check("accent / surface-2 (ícones/foco 3:1)", acc, c["surface-2"], 3.0)
        for k in ("success", "warning", "danger"):
            check(f"{k} / bg", c[k], c["bg"], 4.5)
        check("line-strong / bg (borda de campo 1.4:1)", c["line-strong"], c["bg"], 1.4)
        # texto sobre vidro: composição do fill sobre bg
        gf = over(c["glass-fill-strong"], c["bg"])
        check("ink / glass-fill-strong sobre bg", c["ink"], gf, 7.0)
        check("ink-2 / glass-fill-strong sobre bg", c["ink-2"], gf, 4.5)
        check("ink / glass-solid (fallback sem blur)", c["ink"], c["glass-solid"], 7.0)
        check("ink-3 / glass-solid (fallback sem blur)", c["ink-3"], c["glass-solid"], 4.5)
        # pior caso do vidro SEM blur: fill translúcido sobre texto/capa clara atrás (conteúdo branco) — por isso o fallback é sólido
        worst = (1.0, 1.0, 1.0, 1.0) if mode == "dark" else (0.0, 0.0, 0.0, 1.0)
        check("ink / glass-fill sobre pior fundo (c/ blur dilui)", c["ink"], over(c["glass-fill-strong"], worst), 3.0)
    # imersivo: pior caso = capa branca
    im = {k: parse(v) for k, v in colors("immersive").items()}
    base = T["color"]["immersive"]["base-darken"]
    sb = parse(T["color"]["immersive"]["scrim-bottom"])[3]
    print("[immersive, pior caso: capa branca]")
    for frac, nome in ((0.60, "linha do artista"), (0.70, "tempos/progresso"), (0.90, "botões inferiores")):
        a = 0.0 if frac < 0.30 else sb * (frac - 0.30) / 0.70
        g = 1.0 * base
        bg = (g * (1 - a),) * 3 + (1.0,)
        check(f"ink / fundo @ {nome} ({int(frac*100)}%)", im["ink"], bg, 7.0)
        check(f"ink-2 / fundo @ {nome}", im["ink-2"], bg, 4.5)
        check(f"ink-3 / fundo @ {nome}", im["ink-3"], bg, 4.5)
    print("FALHAS:" if failures else "Todos os pares passaram.", ", ".join(failures))


# ---------- saída Slint ---------------------------------------------------------------------------


def slint():
    L = []
    w = L.append
    w("// GERADO por design/tools/gen_tokens.py a partir de design/tokens.json. NÃO edite à mão.")
    w("// Uso: `import { Theme } from \"tokens.slint\";`  ->  `background: Theme.bg;`")
    w("")
    w("export global Theme {")
    w("    // ---- estado de tema (o kazzu-app escreve) ------------------------------------------------")
    w("    in-out property <bool> dark: true;")
    w("    in-out property <bool> reduce-motion: false;")
    w("    /// largura da janela; o AppWindow atualiza em `changed width`.")
    w("    in property <length> window-w: 390px;")
    w("    out property <bool> compact: window-w < bp-regular;")
    w("    out property <bool> wide: window-w >= bp-wide;")
    bp = T["breakpoint"]
    w(f"    out property <length> bp-regular: {bp['regular-min']}px;")
    w(f"    out property <length> bp-wide: {bp['wide-min']}px;")
    w("")
    w("    // ---- transparência / blur (Slint 1.19 backdrop-blur: Skia e FemtoVG) --------------------------")
    w("    /// preferência do usuário (Configurações > Aparência > Reduzir transparência).")
    w("    in-out property <bool> reduce-transparency: false;")
    w("    /// o Rust escreve false quando o renderer é o software (sem backdrop-blur).")
    w("    in property <bool> blur-supported: true;")
    w("    out property <bool> glass-live: blur-supported && !reduce-transparency;")
    w("")
    w("    // ---- cor de destaque dinâmica (derivada da capa; calculada em Rust, ver DESIGN.md §3.4) ----")
    w("    in-out property <bool> use-dynamic-accent: false;")
    w("    in-out property <color> dynamic-accent: " + to_hex(brand["ember"]) + ";")
    w("    in-out property <color> dynamic-accent-ink: " + to_hex(brand["ember-ink-on-dark"]) + ";")
    w("    out property <color> accent: use-dynamic-accent ? dynamic-accent : (dark ? "
      + to_hex(brand["ember"]) + " : " + to_hex(brand["ember-on-light"]) + ");")
    w("    out property <color> accent-ink: use-dynamic-accent ? dynamic-accent-ink : (dark ? "
      + to_hex(brand["ember-ink-on-dark"]) + " : " + to_hex(brand["ember-ink-on-light"]) + ");")
    w(f"    out property <color> accent-soft: accent.with-alpha(dark ? {T['color']['dark']['accent-soft-alpha']} : {T['color']['light']['accent-soft-alpha']});")
    w("    out property <color> focus-ring: accent;")
    w("")
    w("    // ---- paleta (claro/escuro) -----------------------------------------------------------------")
    dk, lt = colors("dark"), colors("light")
    for k in dk:
        if k.startswith("_"):
            continue
        w(f"    out property <color> {kebab(k)}: dark ? {to_hex(parse(dk[k]))} : {to_hex(parse(lt[k]))};")
    w(f"    out property <length> glass-blur-radius: glass-live ? {T['glass']['blur']}px : 0px;")
    w(f"    out property <length> glass-blur-strong-radius: glass-live ? {T['glass']['blur-strong']}px : 0px;")
    w(f"    out property <length> glass-blur-light-radius: glass-live ? {T['glass']['blur-light']}px : 0px;")
    w("    /// fundo do painel: translúcido com blur ao vivo; sólido sem blur (renderer software / reduzir transparência).")
    w("    out property <color> glass-bg: glass-live ? glass-fill : glass-solid;")
    w("    out property <color> glass-bg-strong: glass-live ? glass-fill-strong : glass-solid-strong;")
    w(f"    out property <float> ambient-opacity: dark ? {T['color']['dark']['ambient-opacity']} : {T['color']['light']['ambient-opacity']};")
    w("")
    w("    // ---- imersivo (player em tela cheia): independe do tema --------------------------------------")
    for k, v in colors("immersive").items():
        w(f"    out property <color> im-{kebab(k)}: {to_hex(parse(v))};")
    w(f"    out property <float> im-base-darken: {T['color']['immersive']['base-darken']};")
    w("")
    w("    // ---- tipografia ----------------------------------------------------------------------------")
    w(f"    out property <string> font-sans: \"{T['font']['sans']}\";")
    w(f"    out property <string> font-mono: \"{T['font']['mono']}\";")
    for name, t in T["type"].items():
        if name.startswith("_"):
            continue
        k = kebab(name)
        w(f"    out property <length> fs-{k}: {t['size']}px;")
        w(f"    out property <length> lh-{k}: {t['line']}px;")
        w(f"    out property <int> fw-{k}: {t['weight']};")
        w(f"    out property <length> ls-{k}: {t['tracking']}px;")
    w("")
    w("    // ---- espaçamento / raios / tamanhos --------------------------------------------------------")
    for k, v in T["space"].items():
        w(f"    out property <length> sp-{k}: {v}px;")
    for k, v in T["radius"].items():
        w(f"    out property <length> r-{k}: {v}px;")
    for k, v in T["size"].items():
        w(f"    out property <length> {kebab(k)}: {v}px;")
    w("")
    w("    // ---- elevação (drop-shadow-blur / -offset-y; a cor é `shadow` × strength) ------------------")
    for k, v in T["elevation"].items():
        if k.startswith("_"):
            continue
        w(f"    out property <length> {k}-blur: {v['blur']}px;")
        w(f"    out property <length> {k}-y: {v['y']}px;")
        w(f"    out property <color> {k}-color: shadow.with-alpha(dark ? {v['strength'] * 0.5:.2f} : {v['strength'] * 0.28:.2f});")
    w("")
    w("    // ---- movimento (reduzir movimento zera as durações) ----------------------------------------")
    for k in ("micro", "short", "medium", "long"):
        w(f"    out property <duration> dur-{k}: reduce-motion ? 0ms : {T['motion'][k]}ms;")
    for k in ("ease-out", "ease-in", "ease-in-out"):
        a = T["motion"][k]
        w(f"    // {k}: cubic-bezier({a[0]}, {a[1]}, {a[2]}, {a[3]})  (Slint exige literal em `easing:`; copie daqui)")
    w("}")
    return "\n".join(L) + "\n"


# ---------- saída CSS -----------------------------------------------------------------------------


def css():
    def block(group, sel, extra=None):
        out = [sel + " {"]
        for k, v in colors(group).items():
            out.append(f"  --{k}: {to_hex(parse(v))};")
        if extra:
            out += extra
        out.append("}")
        return "\n".join(out)

    d, l = T["color"]["dark"], T["color"]["light"]
    dark_extra = [f"  --accent: {to_hex(brand['ember'])};", f"  --accent-ink: {to_hex(brand['ember-ink-on-dark'])};",
                  f"  --accent-soft-alpha: {d['accent-soft-alpha']};", f"  --ambient-opacity: {d['ambient-opacity']};",
                  "  color-scheme: dark;"]
    light_extra = [f"  --accent: {to_hex(brand['ember-on-light'])};", f"  --accent-ink: {to_hex(brand['ember-ink-on-light'])};",
                   f"  --accent-soft-alpha: {l['accent-soft-alpha']};", f"  --ambient-opacity: {l['ambient-opacity']};",
                   "  color-scheme: light;"]
    S = ["/* GERADO por design/tools/gen_tokens.py. NÃO edite à mão. */"]
    S.append(block("dark", ":root", dark_extra))
    S.append(block("light", ':root[data-theme="light"]', light_extra))
    S.append("@media (prefers-color-scheme: light) {\n" + block("light", ':root:not([data-theme="dark"])', light_extra).replace("\n", "\n  ").replace(":root:not", "  :root:not", 1) + "\n}")
    im = ["/* imersivo */", ":root {"]
    for k, v in colors("immersive").items():
        im.append(f"  --im-{k}: {to_hex(parse(v))};")
    im.append(f"  --im-base-darken: {T['color']['immersive']['base-darken']};")
    im.append("}")
    S.append("\n".join(im))
    g = [":root {", f"  --font-sans: \"{T['font']['sans']}\", system-ui, sans-serif;", f"  --font-mono: \"{T['font']['mono']}\", ui-monospace, monospace;"]
    for name, t in T["type"].items():
        if name.startswith("_"):
            continue
        g.append(f"  --fs-{name}: {t['size']}px; --lh-{name}: {t['line']}px; --fw-{name}: {t['weight']}; --ls-{name}: {t['tracking']}px;")
    for k, v in T["space"].items():
        g.append(f"  --sp-{k}: {v}px;")
    for k, v in T["radius"].items():
        g.append(f"  --r-{k}: {v}px;")
    for k, v in T["size"].items():
        g.append(f"  --{k}: {v}px;")
    for k in ("micro", "short", "medium", "long"):
        g.append(f"  --dur-{k}: {T['motion'][k]}ms;")
    for k in ("ease-out", "ease-in", "ease-in-out"):
        a = T["motion"][k]
        g.append(f"  --{k}: cubic-bezier({a[0]}, {a[1]}, {a[2]}, {a[3]});")
    for k, v in T["elevation"].items():
        if k.startswith("_"):
            continue
        g.append(f"  --{k}: 0 {v['y']}px {v['blur']}px var(--shadow);")
    g.append(f"  --glass-blur: {T['glass']['blur']}px; --glass-blur-strong: {T['glass']['blur-strong']}px; --glass-blur-light: {T['glass']['blur-light']}px;")
    g.append("}")
    g.append("@media (prefers-reduce-motion: reduce) { :root { --dur-micro: 0ms; --dur-short: 0ms; --dur-medium: 0ms; --dur-long: 0ms; } }")
    S.append("\n".join(g))
    return "\n\n".join(S) + "\n"


if __name__ == "__main__":
    run_checks()
    if "--check" not in sys.argv:
        (ROOT / "slint").mkdir(exist_ok=True)
        (ROOT / "slint" / "tokens.slint").write_text(slint(), encoding="utf-8")
        (ROOT / "prototypes" / "assets").mkdir(parents=True, exist_ok=True)
        (ROOT / "prototypes" / "assets" / "tokens.css").write_text(css(), encoding="utf-8")
        print("escrito: design/slint/tokens.slint, design/prototypes/assets/tokens.css")
    sys.exit(1 if failures else 0)
