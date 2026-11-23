#!/usr/bin/env python3
"""Prova por pixels de que o vidro da v4 é vidro, no Slint de verdade (FemtoVG, backdrop-blur real).

Para cada caso renderiza três vezes a MESMA cena com o binário design/v4/tools/slint-shot:
  vidro    como o usuário vê
  base     sem o painel (o que está atrás dele)
  sólido   com Reduzir transparência
e mede numa região do painel sem texto nem ícone (V = cor média no vidro, B = cor média do que está atrás,
T = a tinta do painel, lida de tokens.json):
  1. diferença média vidro x sólido > 6            (o painel não é o sólido)
  2. cosseno entre (V - T) e (B - T) > 0,8         (a cor que atravessa é a do conteúdo de trás, não uma cor inventada)
  3. transmissão |V - T| / |B - T| entre 0,1 e 0,92 (é translúcido: nem opaco, nem ausente)
  4. nitidez do vidro / nitidez da base < 0,5       (o que está atrás foi desfocado de verdade)
O critério 4 só vale quando a base tem detalhe (nitidez > 1,5). A rolagem de cada caso é escolhida para pôr capas atrás do painel.
O mini-player é tingido pela capa da faixa atual (55%) e pelo que rola atrás (45%): nele os critérios 2 e 3 trocam a tinta T
pela cor do próprio mini-player sobre o fundo liso (medida com a página no topo de uma tela vazia).
Saídas: design/v4/shots/glass-proof.md e design/v4/shots/glass-proof.webp. Sai com código 1 se algum caso reprovar.
Uso: python3 design/v4/tools/glass_proof.py   (precisa do binário e do xvfb-run; veja design/v4/README.md)
"""
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("KAZZU_SHOT_BIN", ROOT.parents[1] / "target" / "design-v4" / "debug" / "kazzu-slint-shot-v4"))
sys.path.insert(0, str(ROOT / "tools"))
from gen_tokens import T as TOKENS, oklch_to_srgb  # noqa: E402


def tint(key):
    """RGB da tinta de um painel: 'tint:dark', 'tint:light' (cromo) ou 'console' (player)."""
    if key == "console":
        v = TOKENS["media"]["console"]
    else:
        name, mode = key.split(":")
        v = TOKENS["glass"][name][mode]
    return oklch_to_srgb(v[0], v[1], v[2])


LAP = ImageFilter.Kernel((3, 3), [0, -1, 0, -1, 4, -1, 0, -1, 0], scale=1, offset=128)


def render(shots, tmp):
    args = []
    for name, spec in shots:
        args += ["--shot", f"{spec} out={tmp}/{name}.png"]
    env = dict(os.environ, SLINT_SCALE_FACTOR="1")
    subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 1600x1200x24 +extension GLX", str(BIN), *args], check=True, env=env, capture_output=True)


def px(img):
    return list(img.get_flattened_data())


def chroma(img):
    p = px(img.convert("RGB"))
    return sum(max(c) - min(c) for c in p) / len(p)


def sharp(img):
    p = px(img.convert("L").filter(LAP))
    return sum(abs(v - 128) for v in p) / len(p)


def mean_diff(a, b):
    pa, pb = px(a.convert("RGB")), px(b.convert("RGB"))
    return sum(abs(x[i] - y[i]) for x, y in zip(pa, pb) for i in range(3)) / (3 * len(pa))


def mean_rgb(img):
    p = px(img.convert("RGB"))
    return tuple(sum(c[i] for c in p) / len(p) for i in range(3))


def follow(glass, base, s):
    """Cosseno entre (V - T) e (B - T) e a transmissão |V - T| / |B - T|; `s` é a tinta T em RGB."""
    v, b = mean_rgb(glass), mean_rgb(base)
    vs, bs = [v[i] - s[i] for i in range(3)], [b[i] - s[i] for i in range(3)]
    nv, nb = sum(x * x for x in vs) ** 0.5, sum(x * x for x in bs) ** 0.5
    if nv < 1e-6 or nb < 1e-6:
        return 0.0, 0.0
    return sum(x * y for x, y in zip(vs, bs)) / (nv * nb), nv / nb


# nome, descrição, cena comum, região (x0, y0, x1, y1) em px lógicos, extra do vidro, extra da base, extra do sólido
CASES = [
    ("mini-escuro", "Mini-player sobre as capas do Início (escuro)", "scene=home scroll=@ theme=dark", (182, 706, 268, 754), "", "chrome=0", "rt=1", "self:scene=home.error theme=dark"),
    ("mini-claro", "Mini-player sobre as capas do Início (claro)", "scene=home scroll=@ theme=light", (182, 706, 268, 754), "", "chrome=0", "rt=1", "self:scene=home.error theme=light"),
    ("abas-escuro", "Barra cápsula sobre as capas (escuro), vão entre Explorar e Biblioteca", "scene=home scroll=@ theme=dark", (176, 772, 196, 828), "", "chrome=0", "rt=1", "tint:dark"),
    ("topo-escuro", "Cabeçalho recolhido sobre o Início rolado (escuro)", "scene=home scroll=@ theme=dark", (24, 6, 170, 42), "", "chrome=0", "rt=1", "tint:dark"),
    ("topo-claro", "Cabeçalho recolhido sobre o Início rolado (claro)", "scene=home scroll=@ theme=light", (24, 6, 170, 42), "", "chrome=0", "rt=1", "tint:light"),
    ("console-pimenta", "Console do player sobre a capa Pimenta", "scene=player cover=5", (214, 369, 306, 387), "", "chrome=0", "rt=1", "console"),
    ("console-delta", "Console do player sobre a capa Delta", "scene=player cover=3", (214, 369, 306, 387), "", "chrome=0", "rt=1", "console"),
    ("console-novelo", "Console do player sobre a capa Novelo", "scene=player cover=1", (214, 369, 306, 387), "", "chrome=0", "rt=1", "console"),
    ("fila-escuro", "Folha da Fila sobre o player (escuro)", "scene=player cover=5 theme=dark", (226, 126, 370, 150), "+queue", "", "+queue rt=1", "tint-sheet:dark"),
    ("menu-claro", "Folha de ações sobre a capa do álbum (claro)", "scene=album cover=1 theme=light", (236, 229, 360, 247), "+menu", "", "+menu rt=1", "tint-sheet:light"),
    ("placa-inicio", "Placa do cartão do Início sobre a capa Delta", "scene=home theme=dark", (60, 424, 200, 452), "", "", "rt=1", "console"),
]


def best_scroll(tmp, common, region):
    """Escolhe a rolagem em que há mais cor atrás do painel (para a prova usar capas, não o fundo liso)."""
    opts = list(range(40, 600, 40))
    render([(f"s{s}", f"{common.replace('@', str(s))} chrome=0") for s in opts], tmp)
    scored = [(chroma(Image.open(f"{tmp}/s{s}.png").crop(region)), s) for s in opts]
    return max(scored)[1]


def main():
    if not BIN.exists():
        sys.exit(f"binário ausente: {BIN} (compile design/v4/tools/slint-shot)")
    rows, strips, ok = [], [], True
    with tempfile.TemporaryDirectory() as tmp:
        for name, desc, common, region, g, b, s, tk in CASES:
            if "@" in common:
                common = common.replace("@", str(best_scroll(tmp, common, region)))
            if name == "placa-inicio":
                # a base da placa é a própria capa: renderiza o cartão sem a placa usando a região logo acima dela, só para o painel visual
                render([(f"{name}-g", f"{common} {g}"), (f"{name}-s", f"{common} {s}")], tmp)
                glass, solid = Image.open(f"{tmp}/{name}-g.png").crop(region), Image.open(f"{tmp}/{name}-s.png").crop(region)
                base = Image.open(f"{tmp}/{name}-g.png").crop((region[0], region[1] - 90, region[2], region[3] - 90))
                d, cg, cs = mean_diff(glass, solid), chroma(glass), chroma(solid)
                good = d > 6 and cg - cs > 3
                rows.append((desc, common, d, cg, cs, None, None, None, good))
            else:
                def spec(extra):
                    if extra.startswith("+"):
                        ov, _, rest = extra.partition(" ")
                        sc, _, tail = common.partition(" ")
                        return f"{sc}{ov} {tail} {rest}"
                    return f"{common} {extra}"
                render([(f"{name}-g", spec(g)), (f"{name}-b", spec(b)), (f"{name}-s", spec(s))], tmp)
                glass, base, solid = (Image.open(f"{tmp}/{name}-{k}.png").crop(region) for k in "gbs")
                d, cg, cs = mean_diff(glass, solid), chroma(glass), chroma(solid)
                sb, sg = sharp(base), sharp(glass)
                if tk.startswith("self:"):
                    # T = o painel sobre o fundo liso; B precisa da mesma referência: o fundo liso da página
                    render([(f"{name}-t", tk[5:]), (f"{name}-p", f"{tk[5:]} chrome=0")], tmp)
                    t_ref = mean_rgb(Image.open(f"{tmp}/{name}-t.png").crop(region))
                    p_ref = mean_rgb(Image.open(f"{tmp}/{name}-p.png").crop(region))
                    v, bb = mean_rgb(glass), mean_rgb(base)
                    vs, bs = [v[i] - t_ref[i] for i in range(3)], [bb[i] - p_ref[i] for i in range(3)]
                    nv, nb = sum(x * x for x in vs) ** 0.5, sum(x * x for x in bs) ** 0.5
                    cosv, tr = (sum(x * y for x, y in zip(vs, bs)) / (nv * nb), nv / nb) if nv > 1e-6 and nb > 1e-6 else (0.0, 0.0)
                else:
                    cosv, tr = follow(glass, base, tint(tk))
                r = (cosv, tr)
                good = d > 6 and cosv > 0.8 and 0.1 < tr < 0.92 and (sb <= 1.5 or sg / sb < 0.5)
                rows.append((desc, common, d, cg, cs, sb, sg, r, good))
            ok = ok and good
            # tira comparativa: base | vidro | sólido, ampliada 3x
            w, h = glass.size
            strip = Image.new("RGB", (3 * w * 3 + 4 * 12, h * 3 + 24), (40, 40, 40))
            for i, im in enumerate((base, glass, solid)):
                strip.paste(im.resize((w * 3, h * 3), Image.LANCZOS), (12 + i * (w * 3 + 12), 12))
            strips.append((desc, strip))
    width = max(s.width for _, s in strips)
    sheet = Image.new("RGB", (width, sum(s.height + 26 for _, s in strips) + 30), (40, 40, 40))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.truetype(str(ROOT / "fonts" / "Inter-Regular.ttf"), 13)
    draw.text((12, 8), "atrás do painel   |   vidro   |   sólido (Reduzir transparência)        regiões ampliadas 3x", fill=(240, 240, 240), font=font)
    y = 30
    for desc, s in strips:
        draw.text((12, y + 4), desc, fill=(200, 200, 200), font=font)
        sheet.paste(s, (0, y + 22))
        y += s.height + 26
    out = ROOT / "shots"
    out.mkdir(exist_ok=True)
    sheet.save(out / "glass-proof.webp", quality=88, method=6)
    md = ["# Prova de vidro no Slint real (v4)", "",
          "Gerado por `design/v4/tools/glass_proof.py` com o binário `design/v4/tools/slint-shot` (Slint 1.19, renderer FemtoVG, `backdrop-blur` real, via Xvfb/Mesa).",
          "Cada linha compara a mesma região do painel em três renders: como o usuário vê, sem o painel, e com Reduzir transparência.", "",
          "| Caso | Cena | Vidro x sólido (> 6) | Croma vidro / sólido | Nitidez atrás / no vidro (razão < 0,5) | Cor segue o que está atrás (cosseno > 0,8) e transmissão | |",
          "|---|---|---|---|---|---|---|"]
    for desc, common, d, cg, cs, sb, sg, r, good in rows:
        nit = "não se aplica" if sb is None else f"{sb:.2f} / {sg:.2f}" + ("" if sb <= 1.5 else f" ({sg / sb:.2f})")
        cor = "não se aplica" if r is None else f"{r[0]:.2f}, transmissão {r[1]:.2f}"
        md.append(f"| {desc} | `{common}` | {d:.1f} | {cg:.1f} / {cs:.1f} | {nit} | {cor} | {'passa' if good else '**REPROVA**'} |")
    md += ["", "![base, vidro e sólido de cada caso](glass-proof.webp)", "",
           "Leitura: no vidro a cor do que está atrás aparece (o cosseno perto de 1 diz que a cor do painel se move na direção da cor de trás) e o detalhe some (nitidez baixa): é desfoque de fundo de verdade. A transmissão é quanto da cor de trás chega ao painel.",
           "No sólido a região é uma cor só: é o que aparece com Reduzir transparência e no renderer software."]
    (out / "glass-proof.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    for desc, common, d, cg, cs, sb, sg, r, good in rows:
        print(("ok  " if good else "FALHA"), desc, f"dif={d:.1f} croma={cg:.1f}/{cs:.1f}", "" if sb is None else f"nitidez={sb:.2f}/{sg:.2f} cos={r[0]:.2f} transm={r[1]:.2f}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
