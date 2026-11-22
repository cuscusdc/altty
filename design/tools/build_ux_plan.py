#!/usr/bin/env python3
"""Gera o artifact HTML navegável do planejamento de UX a partir de design/ux/UX_PLAN.md.
Saída: design/ux/plan/index.html (+ img/ com as capturas dos protótipos) e, com --artifact <pasta>, uma cópia para a pasta do artifact do Traycer.
Diagramas: blocos ```mermaid são renderizados em SVG por tools/render_mermaid.mjs (Chromium). Precisa de tools/.cache/mermaid.min.js.
Uso: python3 design/tools/build_ux_plan.py [--artifact /caminho/artifacts/kazzu-ux-plan] [--no-diagrams]
"""
import html, json, re, shutil, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "ux" / "UX_PLAN.md"
OUT = ROOT / "ux" / "plan"


def inline(s):
    s = html.escape(s, quote=False)
    s = re.sub(r"`([^`]+)`", lambda m: f"<code>{m[1]}</code>", s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"(?<![\w*])\*([^*\n]+)\*(?![\w*])", r"<em>\1</em>", s)
    s = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r'<a href="\2">\1</a>', s)
    return s


def slug(t):
    t = re.sub(r"[`*]", "", t).lower()
    t = t.translate(str.maketrans("áàâãäéèêëíìîïóòôõöúùûüç", "aaaaaeeeeiiiiooooouuuuc"))
    return re.sub(r"[^a-z0-9]+", "-", t).strip("-")


def parse(md):
    lines = md.split("\n")
    i, out, toc, mer = 0, [], [], []
    para = []

    def flush():
        if para:
            out.append("<p>" + inline(" ".join(para)) + "</p>")
            para.clear()

    while i < len(lines):
        ln = lines[i]
        if ln.startswith("```"):
            flush()
            lang = ln[3:].strip()
            j = i + 1
            while j < len(lines) and not lines[j].startswith("```"):
                j += 1
            code = "\n".join(lines[i + 1:j])
            if lang == "mermaid":
                mid = f"m{len(mer)}"
                mer.append({"id": mid, "code": code})
                out.append(f'<figure class="diagram" data-mid="{mid}"><div class="dg" id="{mid}">@@{mid}@@</div></figure>')
            else:
                out.append(f'<pre class="code"><code>{html.escape(code)}</code></pre>')
            i = j + 1
            continue
        m = re.match(r"^(#{1,4}) (.*)$", ln)
        if m:
            flush()
            lvl, text = len(m[1]), m[2]
            sid = slug(text)
            if lvl in (2, 3):
                toc.append((lvl, sid, re.sub(r"[`*]", "", text)))
            if lvl == 1:
                out.append(f'<h1 id="{sid}">{inline(text)}</h1>')
            else:
                out.append(f'<h{lvl} id="{sid}"><a class="anchor" href="#{sid}" aria-label="Link para esta seção">#</a>{inline(text)}</h{lvl}>')
            i += 1
            continue
        if ln.strip() == "---":
            flush(); out.append("<hr>"); i += 1; continue
        if ln.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|$", lines[i + 1].strip()):
            flush()
            hdr = [c.strip() for c in ln.strip().strip("|").split("|")]
            rows = []
            j = i + 2
            while j < len(lines) and lines[j].startswith("|"):
                rows.append([c.strip() for c in re.split(r"(?<!\\)\|", lines[j].strip().strip("|"))])
                j += 1
            t = '<div class="tw"><table><thead><tr>' + "".join(f"<th>{inline(h)}</th>" for h in hdr) + "</tr></thead><tbody>"
            is_backlog = "MoSCoW" in hdr
            for r in rows:
                attrs = ""
                if is_backlog:
                    attrs = f' data-m="{re.sub(r"[^A-Z]", "", r[hdr.index("MoSCoW")])}" data-e="{html.escape(r[hdr.index("Etapa")])}"'
                t += f"<tr{attrs}>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>"
            t += "</tbody></table></div>"
            out.append(t + ("" if not is_backlog else ""))
            if is_backlog:
                out[-1] = '<div class="filters" role="group" aria-label="Filtrar backlog por prioridade">' + "".join(f'<button type="button" class="chip" data-f="{k}" aria-pressed="false">{v}</button>' for k, v in (("M", "Must"), ("S", "Should"), ("C", "Could"), ("W", "Won’t"))) + '<button type="button" class="chip" data-f="" aria-pressed="true">Tudo</button></div>' + out[-1]
            i = j
            continue
        if ln.startswith("> "):
            flush()
            q = []
            while i < len(lines) and lines[i].startswith(">"):
                q.append(lines[i][1:].strip()); i += 1
            out.append("<blockquote>" + inline(" ".join(q)) + "</blockquote>")
            continue
        if re.match(r"^\s*([-*]|\d+\.) ", ln):
            flush()
            ordered = bool(re.match(r"^\s*\d+\.", ln))
            items = []
            while i < len(lines) and re.match(r"^\s*([-*]|\d+\.) ", lines[i]):
                items.append(re.sub(r"^\s*([-*]|\d+\.) ", "", lines[i]))
                i += 1
                while i < len(lines) and lines[i].startswith("   ") and lines[i].strip():
                    items[-1] += " " + lines[i].strip(); i += 1
            tag = "ol" if ordered else "ul"
            out.append(f"<{tag}>" + "".join(f"<li>{inline(x)}</li>" for x in items) + f"</{tag}>")
            continue
        if not ln.strip():
            flush(); i += 1; continue
        para.append(ln.strip()); i += 1
    flush()
    return "\n".join(out), toc, mer


CSS = r"""
:root { --c-bg: var(--color-background-primary, #fbf9f6); --c-bg2: var(--color-background-secondary, #f2eee8); --c-bg3: var(--color-background-tertiary, #e8e2d9);
  --c-tx: var(--color-text-primary, #1d1713); --c-tx2: var(--color-text-secondary, #4b4139); --c-tx3: var(--color-text-tertiary, #6b6057);
  --c-bd: var(--color-border-primary, #d9d1c6); --c-bd2: var(--color-border-secondary, #e6dfd5); --c-ring: var(--color-ring-primary, #b4530a);
  --c-acc: #b4530a; --c-ok: var(--color-text-success, #1c6b3f); --c-no: var(--color-text-danger, #a52a2a);
  --f-sans: var(--font-sans, "Hanken Grotesk", system-ui, sans-serif); --f-mono: var(--font-mono, ui-monospace, monospace); --r: var(--border-radius-md, 10px); }
* { box-sizing: border-box; } html { scroll-behavior: smooth; scroll-padding-top: 20px; } body { margin: 0; background: var(--c-bg); color: var(--c-tx); font: 400 16px/1.6 var(--f-sans); }
a { color: inherit; } :focus-visible { outline: 2px solid var(--c-ring); outline-offset: 2px; }
.layout { display: grid; grid-template-columns: minmax(0, 1fr); gap: 0 40px; max-width: 1280px; margin-inline: auto; padding: 0 20px 80px; }
.toc { position: sticky; top: 0; z-index: 5; background: var(--c-bg); border-bottom: 1px solid var(--c-bd2); margin-inline: -20px; padding: 8px 20px; }
.toc summary { cursor: pointer; min-height: 44px; display: flex; align-items: center; font-weight: 600; }
.toc ol { list-style: none; padding: 0; margin: 4px 0 8px; display: grid; gap: 2px; font-size: 14px; } .toc li li { padding-left: 14px; }
.toc a { display: block; padding: 6px 8px; border-radius: 6px; text-decoration: none; color: var(--c-tx2); } .toc a:hover { background: var(--c-bg2); color: var(--c-tx); } .toc a[aria-current="true"] { background: var(--c-bg3); color: var(--c-tx); font-weight: 600; }
main { min-width: 0; padding-top: 24px; } h1 { font-size: clamp(28px, 5vw, 40px); line-height: 1.15; letter-spacing: -.02em; margin: 0 0 12px; }
h2 { font-size: 26px; line-height: 1.25; margin: 56px 0 12px; letter-spacing: -.01em; } h3 { font-size: 19px; line-height: 1.3; margin: 32px 0 8px; } h4 { font-size: 16px; margin: 20px 0 6px; }
.anchor { margin-right: 8px; text-decoration: none; color: var(--c-tx3); font-weight: 400; opacity: 0; } h2:hover .anchor, h3:hover .anchor, .anchor:focus-visible { opacity: 1; }
p, li { max-width: 74ch; } p { margin: 10px 0; } ul, ol { padding-left: 22px; margin: 10px 0; } li { margin: 4px 0; } hr { border: 0; border-top: 1px solid var(--c-bd2); margin: 36px 0 0; }
code { font: 500 .88em var(--f-mono); background: var(--c-bg2); padding: 1px 5px; border-radius: 5px; } pre.code { background: var(--c-bg2); border: 1px solid var(--c-bd2); border-radius: var(--r); padding: 14px 16px; overflow-x: auto; line-height: 1.45; } pre.code code { background: none; padding: 0; font-size: 13px; }
blockquote { margin: 16px 0; padding: 4px 18px; border-left: 3px solid var(--c-acc); font-size: 19px; color: var(--c-tx); max-width: 60ch; }
.tw { overflow-x: auto; margin: 14px 0; border: 1px solid var(--c-bd2); border-radius: var(--r); } table { border-collapse: collapse; width: 100%; font-size: 14.5px; min-width: 560px; }
th { text-align: left; background: var(--c-bg2); font-weight: 600; } th, td { padding: 9px 12px; vertical-align: top; border-bottom: 1px solid var(--c-bd2); } tr:last-child td { border-bottom: 0; }
td:first-child { font-weight: 500; }
.diagram { margin: 16px 0; border: 1px solid var(--c-bd2); border-radius: var(--r); background: #faf6f1; padding: 14px; overflow-x: auto; color: #2a1608; } .diagram svg { max-width: 100%; height: auto; margin-inline: auto; display: block; }
.diagram .edgeLabel, .diagram .edgeLabel p { color: #2a1608 !important; background: #faf6f1 !important; }
.chip { min-height: 40px; padding: 0 16px; border-radius: 999px; border: 1px solid var(--c-bd); background: var(--c-bg); color: var(--c-tx); font: 600 14px var(--f-sans); cursor: pointer; }
.chip[aria-pressed="true"] { background: var(--c-tx); color: var(--c-bg); border-color: var(--c-tx); }
.filters { display: flex; gap: 8px; flex-wrap: wrap; margin: 12px 0 4px; }
.gal { border: 1px solid var(--c-bd2); border-radius: var(--r); padding: 16px; background: var(--c-bg2); display: grid; gap: 14px; margin: 16px 0; }
.gal .row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; } .gal .lab { font-size: 13px; color: var(--c-tx2); min-width: 68px; }
.gal .shot { display: grid; place-items: start center; overflow: auto; border-radius: var(--r); background: var(--c-bg3); padding: 16px; min-height: 200px; } .gal img { max-width: 100%; height: auto; border-radius: 22px; border: 1px solid var(--c-bd); display: block; }
.gal .cap { font: 500 12px var(--f-mono); color: var(--c-tx2); }
.meta { color: var(--c-tx2); font-size: 14px; margin-bottom: 20px; }
@media (min-width: 980px) {
  .layout { grid-template-columns: 250px minmax(0, 1fr); padding-inline: 28px; }
  .toc { position: sticky; top: 0; align-self: start; max-height: 100vh; overflow-y: auto; border: 0; margin: 0; padding: 24px 0; background: none; } .toc summary { display: none; } .toc details { display: block; } .toc details > * { display: block; }
}
"""

JS = r"""
(function(){
  var toc=document.querySelectorAll('.toc a'), map={}; toc.forEach(function(a){map[a.getAttribute('href').slice(1)]=a;});
  var heads=[].slice.call(document.querySelectorAll('main h2, main h3')).filter(function(h){return map[h.id];});
  function spy(){var cur=null;heads.forEach(function(h){if(h.getBoundingClientRect().top<140)cur=h.id;});toc.forEach(function(a){a.removeAttribute('aria-current');});if(cur&&map[cur])map[cur].setAttribute('aria-current','true');}
  addEventListener('scroll',spy,{passive:true});spy();
  document.querySelectorAll('.toc a').forEach(function(a){a.addEventListener('click',function(){var d=document.querySelector('.toc details');if(d&&innerWidth<980)d.open=false;});});
  document.querySelectorAll('.filters').forEach(function(f){var t=f.nextElementSibling.querySelector('table');
    f.addEventListener('click',function(e){var b=e.target.closest('.chip');if(!b)return;f.querySelectorAll('.chip').forEach(function(x){x.setAttribute('aria-pressed',x===b);});var k=b.dataset.f;t.querySelectorAll('tbody tr').forEach(function(r){r.hidden=!!k&&r.dataset.m!==k;});});});
  var g=document.getElementById('gal');
  if(g){var st={s:'home',w:'390',t:'dark'};var img=g.querySelector('img'),cap=g.querySelector('.cap');
    function show(){img.src='img/'+st.s+'-'+st.w+'-'+st.t+'.webp';img.alt='Protótipo '+st.s+' em '+st.w+' px, tema '+(st.t==='dark'?'escuro':'claro');cap.textContent=st.s+'.html  '+st.w+' px  '+st.t;}
    g.addEventListener('click',function(e){var b=e.target.closest('.chip');if(!b)return;var k=b.dataset.k;g.querySelectorAll('.chip[data-k="'+k+'"]').forEach(function(x){x.setAttribute('aria-pressed',x===b);});st[k]=b.dataset.v;show();});show();}
})();
"""

SCREENS = [("home", "Início"), ("explore", "Explorar"), ("library", "Biblioteca"), ("album", "Álbum"), ("search", "Buscar"), ("player", "Player"), ("queue", "Fila"), ("sheets", "Folhas"), ("settings", "Ajustes"), ("states", "Estados")]


def gallery():
    def group(k, label, items, first):
        return (f'<div class="row" role="group" aria-label="{label}"><span class="lab">{label}</span>' +
                "".join(f'<button type="button" class="chip" data-k="{k}" data-v="{v}" aria-pressed="{str(v == first).lower()}">{n}</button>' for v, n in items) + "</div>")
    return ('<section aria-labelledby="telas-h"><h2 id="telas-h"><a class="anchor" href="#telas-h">#</a>Telas dos protótipos</h2>'
            '<p>Capturas reais dos protótipos HTML (<code>design/prototypes/</code>), renderizados no Chromium com vidro de verdade. Escolha tela, largura e tema.</p>'
            '<div class="gal" id="gal">' + group("s", "Tela", SCREENS, "home") + group("w", "Largura", [("390", "390"), ("360", "360"), ("768", "768"), ("1280", "1280")], "390") + group("t", "Tema", [("dark", "Escuro"), ("light", "Claro")], "dark") +
            '<div class="shot"><img src="img/home-390-dark.webp" alt="" width="390"></div><div class="cap"></div></div></section>')


def build(artifact=None, diagrams=True):
    md = SRC.read_text(encoding="utf-8")
    body, toc, mer = parse(md)
    svgs = {}
    if diagrams and mer:
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            outp = Path(td) / "m.json"
            subprocess.run(["node", str(ROOT / "tools" / "render_mermaid.mjs"), str(outp)], input=json.dumps(mer), capture_output=True, text=True, timeout=180)
            svgs = json.loads(outp.read_text(encoding="utf-8"))
        for k, v in svgs.items():
            if v.startswith("ERRO"):
                print("mermaid", k, v[:200], file=sys.stderr)
    for m in mer:
        v = svgs.get(m["id"])
        body = body.replace(f"@@{m['id']}@@", v if v and not v.startswith("ERRO") else f'<pre class="code"><code>{html.escape(m["code"])}</code></pre>')
    # galeria depois do resumo (antes da seção 1)
    marker = '<hr>\n<h2 id="1-visao-e-principios">'
    body = body.replace(marker, "<hr>\n" + gallery() + '\n<h2 id="1-visao-e-principios">', 1) if marker in body else body + gallery()
    tocs = []
    for lvl, sid, text in toc:
        if lvl == 2:
            tocs.append(f'<li><a href="#{sid}">{html.escape(text)}</a></li>')
    toc_html = '<nav class="toc" aria-label="Seções"><details open><summary>Seções</summary><ol>' + "".join(tocs) + '<li><a href="#telas-h">Telas dos protótipos</a></li></ol></details></nav>'
    page = f'''<!doctype html>
<html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kazzu UX v2</title><style>{CSS}</style></head>
<body><div class="layout">{toc_html}<main id="main">
<p class="meta">Planejamento de UX · v0.2 · 2026-11-22 · fonte: <code>design/ux/UX_PLAN.md</code> · protótipos: <code>design/prototypes/index.html</code></p>
{body}
</main></div><script>{JS}</script></body></html>
'''
    page = page.replace('<h1 id="kazzu-planejamento-de-ux-v2-mobile-first">', '<h1 id="kazzu-planejamento-de-ux-v2-mobile-first">')
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "index.html").write_text(page, encoding="utf-8")
    print("escrito:", OUT / "index.html", f"({len(page) // 1024} KB, {len(mer)} diagramas)")
    if artifact:
        a = Path(artifact)
        a.mkdir(parents=True, exist_ok=True)
        shutil.copy(OUT / "index.html", a / "index.html")
        if (OUT / "img").exists():
            shutil.copytree(OUT / "img", a / "img", dirs_exist_ok=True)
        print("copiado para", a)


if __name__ == "__main__":
    art = sys.argv[sys.argv.index("--artifact") + 1] if "--artifact" in sys.argv else None
    build(art, "--no-diagrams" not in sys.argv)
