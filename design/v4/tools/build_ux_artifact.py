#!/usr/bin/env python3
"""Gera o artifact HTML navegável do plano de UX v4 a partir de design/v4/ux/*.md.

Uso: python3 design/v4/tools/build_ux_artifact.py [pasta-de-saída]
Saída padrão: /root/.traycer/epics/3948db01-f336-4094-8c9f-c2b8cfbd4b3f/artifacts/kazzu-ux-v4
Também copia para a saída as capturas de design/v4/shots/ux/*.webp, se existirem (seção "Telas").
O conversor cobre só o subconjunto de Markdown usado nos arquivos: títulos, tabelas, listas, blocos de código e Mermaid.
"""
import html
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UX = ROOT / "ux"
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/root/.traycer/epics/3948db01-f336-4094-8c9f-c2b8cfbd4b3f/artifacts/kazzu-ux-v4")

FILES = [
    ("resumo", "Resumo", "README.md"),
    ("diagnostico", "Diagnóstico", "01-diagnostico-e-benchmark.md"),
    ("objetivos", "Objetivos e pessoas", "02-objetivos-pessoas-jtbd.md"),
    ("navegacao", "Arquitetura e navegação", "03-arquitetura-e-navegacao.md"),
    ("fluxos", "Fluxos", "04-fluxos.md"),
    ("estados", "Estados e textos", "05-estados-e-microcopy.md"),
    ("acesso", "Gestos e acessibilidade", "06-gestos-acessibilidade-responsivo.md"),
    ("backlog", "Backlog e aceite", "07-backlog-e-aceite.md"),
]


def inline(t: str) -> str:
    t = html.escape(t, quote=False)
    t = re.sub(r"`([^`]+)`", r"<code>\1</code>", t)
    t = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", t)
    return t


def slug(t: str) -> str:
    t = re.sub(r"[^\w\s-]", "", t.lower(), flags=re.UNICODE)
    return re.sub(r"\s+", "-", t.strip())


def convert(md: str, prefix: str) -> str:
    out, lines, i = [], md.splitlines(), 0
    first_h1 = True
    while i < len(lines):
        ln = lines[i]
        if ln.startswith("```"):
            lang = ln[3:].strip()
            j = i + 1
            buf = []
            while j < len(lines) and not lines[j].startswith("```"):
                buf.append(lines[j])
                j += 1
            code = html.escape("\n".join(buf), quote=False)
            if lang == "mermaid":
                out.append(f'<div class="scroll"><pre class="mermaid">{code}</pre></div>')
            else:
                out.append(f'<div class="scroll"><pre class="code"><code>{code}</code></pre></div>')
            i = j + 1
            continue
        m = re.match(r"(#{1,4})\s+(.*)", ln)
        if m:
            lvl, txt = len(m[1]), m[2]
            if lvl == 1:
                if first_h1:
                    out.append(f'<h2 class="sec" id="{prefix}">{inline(txt)}</h2>')
                    first_h1 = False
                else:
                    out.append(f"<h3>{inline(txt)}</h3>")
            else:
                tag = {2: "h3", 3: "h4", 4: "h5"}[lvl]
                out.append(f'<{tag} id="{prefix}-{slug(txt)}">{inline(txt)}</{tag}>')
            i += 1
            continue
        if ln.startswith("|") and i + 1 < len(lines) and re.match(r"\|\s*[-:]+", lines[i + 1]):
            head = [c.strip() for c in ln.strip().strip("|").split("|")]
            rows = []
            j = i + 2
            while j < len(lines) and lines[j].startswith("|"):
                rows.append([c.strip() for c in lines[j].strip().strip("|").split("|")])
                j += 1
            th = "".join(f"<th>{inline(c)}</th>" for c in head)
            body = "".join("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>" for r in rows)
            out.append(f'<div class="scroll"><table><thead><tr>{th}</tr></thead><tbody>{body}</tbody></table></div>')
            i = j
            continue
        if re.match(r"\s*([-*]|\d+\.)\s+", ln):
            ordered = bool(re.match(r"\s*\d+\.", ln))
            items = []
            while i < len(lines) and re.match(r"\s*([-*]|\d+\.)\s+", lines[i]):
                items.append(re.sub(r"^\s*([-*]|\d+\.)\s+", "", lines[i]))
                i += 1
            tag = "ol" if ordered else "ul"
            out.append(f"<{tag}>" + "".join(f"<li>{inline(x)}</li>" for x in items) + f"</{tag}>")
            continue
        if not ln.strip():
            i += 1
            continue
        buf = [ln]
        i += 1
        while i < len(lines) and lines[i].strip() and not re.match(r"(#|\||```|\s*([-*]|\d+\.)\s)", lines[i]):
            buf.append(lines[i])
            i += 1
        out.append(f"<p>{inline(' '.join(buf))}</p>")
    return "\n".join(out)


CSS = """
*{box-sizing:border-box}
html{scroll-behavior:smooth;scroll-padding-top:64px}
body{margin:0;padding:0;font-family:var(--font-sans,system-ui,sans-serif);color:var(--color-text-primary);background:var(--color-background-primary);font-size:15px;line-height:1.55}
main{max-width:1040px;margin:0 auto;padding:16px 16px 72px;display:flex;flex-direction:column;gap:14px}
header.top{display:flex;flex-direction:column;gap:6px;padding:20px 0 4px}
header.top h1{font-size:30px;line-height:1.15;margin:0;letter-spacing:-0.02em}
header.top p{margin:0;color:var(--color-text-secondary);max-width:70ch}
nav.tabs{position:sticky;top:0;z-index:5;display:flex;gap:6px;overflow-x:auto;padding:10px 0;background:var(--color-background-primary);border-bottom:1px solid var(--color-border-secondary)}
nav.tabs a{flex:none;padding:8px 12px;border-radius:var(--border-radius-md,8px);color:var(--color-text-secondary);text-decoration:none;font-size:14px;white-space:nowrap;border:1px solid transparent}
nav.tabs a:hover{background:var(--color-background-secondary);color:var(--color-text-primary)}
nav.tabs a:focus-visible{outline:2px solid var(--color-ring-primary);outline-offset:1px}
nav.tabs a.on{background:var(--color-background-secondary);color:var(--color-text-primary);border-color:var(--color-border-secondary)}
section.doc{display:flex;flex-direction:column;gap:12px;padding-top:18px;min-width:0}
h2.sec{font-size:24px;line-height:1.2;margin:10px 0 0;letter-spacing:-0.015em;padding-top:14px;border-top:1px solid var(--color-border-secondary)}
h3{font-size:18px;margin:14px 0 0;line-height:1.25}
h4{font-size:15px;margin:10px 0 0}
h5{font-size:14px;margin:8px 0 0;color:var(--color-text-secondary)}
p{margin:0;max-width:78ch}
ul,ol{margin:0;padding-left:22px;display:flex;flex-direction:column;gap:4px;max-width:78ch}
code{font-family:var(--font-mono,monospace);font-size:.88em;background:var(--color-background-secondary);padding:1px 5px;border-radius:4px}
.scroll{overflow-x:auto;min-width:0;border:1px solid var(--color-border-secondary);border-radius:var(--border-radius-md,8px)}
table{border-collapse:collapse;width:100%;font-size:14px}
th,td{text-align:left;vertical-align:top;padding:8px 12px;border-bottom:1px solid var(--color-border-secondary)}
th{background:var(--color-background-secondary);font-weight:600;white-space:nowrap}
tbody tr:last-child td{border-bottom:0}
pre.code{margin:0;padding:12px 14px;font-family:var(--font-mono,monospace);font-size:12.5px;line-height:1.45}
pre.code code{background:none;padding:0}
pre.mermaid{margin:0;padding:14px;font-family:var(--font-mono,monospace);font-size:12px;min-width:560px;text-align:center;white-space:pre-wrap}
.shots{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:14px}
.shots figure{margin:0;display:flex;flex-direction:column;gap:6px;min-width:0}
.shots img{max-width:100%;height:auto;border-radius:14px;border:1px solid var(--color-border-secondary);display:block}
.shots figcaption{font-size:13px;color:var(--color-text-secondary)}
.note{padding:10px 14px;border-radius:var(--border-radius-md,8px);background:var(--color-background-info);color:var(--color-text-info);max-width:78ch}
"""

JS = """
const links=[...document.querySelectorAll('nav.tabs a')];
const secs=links.map(a=>document.getElementById(a.getAttribute('href').slice(1)));
const io=new IntersectionObserver(es=>{es.forEach(e=>{if(e.isIntersecting){links.forEach(l=>l.classList.toggle('on',l.getAttribute('href')==='#'+e.target.id));}})},{rootMargin:'-20% 0px -70% 0px'});
secs.forEach(s=>s&&io.observe(s));
const cs=getComputedStyle(document.documentElement);const cv=document.createElement('canvas');cv.width=cv.height=1;const cx=cv.getContext('2d',{willReadFrequently:true});
const v=n=>{const raw=cs.getPropertyValue(n).trim();if(!raw||n.startsWith('--font'))return raw;cx.clearRect(0,0,1,1);cx.fillStyle='#808080';cx.fillStyle=raw;cx.fillRect(0,0,1,1);const d=cx.getImageData(0,0,1,1).data;return '#'+[d[0],d[1],d[2]].map(x=>x.toString(16).padStart(2,'0')).join('');};
function draw(){ if(!window.mermaid) return;
  mermaid.initialize({startOnLoad:false,theme:'base',securityLevel:'strict',flowchart:{curve:'basis',htmlLabels:false,nodeSpacing:30,rankSpacing:38},
    themeVariables:{fontFamily:v('--font-sans')||'system-ui',fontSize:'14px',primaryColor:v('--color-background-secondary'),primaryTextColor:v('--color-text-primary'),primaryBorderColor:v('--color-border-primary'),lineColor:v('--color-text-secondary'),secondaryColor:v('--color-background-tertiary'),tertiaryColor:v('--color-background-secondary'),edgeLabelBackground:v('--color-background-primary'),background:v('--color-background-primary'),mainBkg:v('--color-background-secondary'),nodeBorder:v('--color-border-primary'),clusterBkg:v('--color-background-secondary'),labelColor:v('--color-text-primary'),stateLabelColor:v('--color-text-primary'),transitionColor:v('--color-text-secondary'),stateBkg:v('--color-background-secondary')}});
  mermaid.run({querySelector:'pre.mermaid'}); }
const s=document.createElement('script');s.src='https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.min.js';s.onload=draw;document.head.appendChild(s);
"""

SHOTS = [
    ("home", "Início"), ("home-scrolled", "Início, rolado"), ("explore", "Explorar"), ("search", "Buscar"), ("search-results", "Resultados"),
    ("library", "Biblioteca"), ("album", "Álbum"), ("player", "Player"), ("queue", "Fila"), ("menu", "Menu da faixa"),
    ("settings", "Ajustes"), ("home-offline", "Sem conexão"), ("home-loading", "Carregando"), ("home-light", "Início, claro"), ("player-rt", "Player, sem transparência"),
]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    nav = "".join(f'<a href="#{k}">{html.escape(label)}</a>' for k, label, _ in FILES)
    body = []
    for k, _, f in FILES:
        body.append(f'<section class="doc">{convert((UX / f).read_text(encoding="utf-8"), k)}</section>')
    shots_dir = ROOT / "shots" / "ux"
    figs = []
    if shots_dir.exists():
        (OUT / "img").mkdir(exist_ok=True)
        for name, label in SHOTS:
            src = shots_dir / f"{name}.webp"
            if src.exists():
                shutil.copy2(src, OUT / "img" / src.name)
                figs.append(f'<figure><img src="img/{src.name}" alt="Tela {html.escape(label)} do protótipo v4" loading="lazy" width="390" height="844"><figcaption>{html.escape(label)}</figcaption></figure>')
    if figs:
        nav += '<a href="#telas">Telas</a>'
        body.append('<section class="doc"><h2 class="sec" id="telas">Telas da proposta</h2><p>Capturas dos protótipos v4 em 390 × 844. Os renders no Slint real e a prova de vidro estão em <code>design/v4/shots/</code>.</p><div class="shots">' + "".join(figs) + "</div></section>")
    page = f"""<!doctype html>
<html lang="pt-BR">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kazzu UX v4</title>
<style>{CSS}</style>
</head>
<body>
<main>
<header class="top">
<h1>Kazzu v4: plano de UX</h1>
<p>Proposta do zero para o app de música do Kazzu: celular primeiro, convenções da Apple, vidro de verdade sobre as capas. Aguardando a decisão do autor; nada foi implementado no app.</p>
</header>
<nav class="tabs" aria-label="Seções do plano">{nav}</nav>
{''.join(body)}
</main>
<script>{JS}</script>
</body>
</html>
"""
    (OUT / "index.html").write_text(page, encoding="utf-8")
    print("escrito:", OUT / "index.html", len(page), "bytes;", len(figs), "capturas")


if __name__ == "__main__":
    main()
