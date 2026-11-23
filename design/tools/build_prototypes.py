#!/usr/bin/env python3
"""Gera os protótipos HTML v2 (MOBILE-FIRST, vidro real) em design/prototypes/.
Uso: python3 design/tools/build_prototypes.py   (antes: gen_tokens.py e make_assets.py)
Páginas: index (hub), home, explore, library, album, search, player, queue, settings, sheets, states.
"""
import json, math, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
P = ROOT / "prototypes"
sys.path.insert(0, str(ROOT / "tools"))
import gen_tokens as G  # noqa: E402  (só funções de cor; não escreve nada ao importar)

T = G.T
SPRITE = (P / "assets" / "sprite.svg.html").read_text(encoding="utf-8")
COVERS = {c["id"]: c for c in json.loads((P / "assets" / "covers.json").read_text(encoding="utf-8"))}

# ------------------------------------------------------------------ dados fictícios (nada de arte ou dados de terceiros)
YT = "YouTube Music"
TRACKS = {
    "01": dict(song="Maré Alta", artist="Aurora Vento", album="Cidade Submersa", fmt="OPUS", bits="160 kbps", short="160", dur="4:12", src=YT),
    "02": dict(song="Hora do Café", artist="Doma", album="Nove Horas", fmt="AAC", bits="256 kbps", short="256", dur="3:38", src=YT),
    "03": dict(song="Mapa de Neblina", artist="Lia Cantos", album="Bruma", fmt="OPUS", bits="160 kbps", short="160", dur="3:05", src=YT),
    "04": dict(song="Canal Sete", artist="Os Faróis", album="Quadra 7", fmt="OPUS", bits="160 kbps", short="160", dur="2:57", src=YT),
    "05": dict(song="Escada de Sal", artist="Tomás Reis", album="Sal e Cinza", fmt="FLAC", bits="24-bit/192 kHz", short="24/192", dur="5:44", src=""),
    "06": dict(song="Linha de Fuga", artist="Meridiano", album="Linha de Fuga", fmt="OPUS", bits="160 kbps", short="160", dur="6:20", src=YT),
    "07": dict(song="Quase Verão", artist="Nina Sabiá", album="Calor de Março", fmt="OPUS", bits="160 kbps", short="160", dur="3:21", src=YT),
    "08": dict(song="Respiração de Vidro", artist="Kaito e Vê", album="Vidro Líquido", fmt="AAC", bits="256 kbps", short="256", dur="3:49", src=YT),
    "09": dict(song="Seda e Sal", artist="Ilha Branca", album="Papel de Seda", fmt="OPUS", bits="160 kbps", short="160", dur="4:03", src=YT),
    "10": dict(song="Fita sem Fim", artist="Rádio Cobre", album="Fita Magnética", fmt="FLAC", bits="24-bit/96 kHz", short="24/96", dur="7:15", src=""),
}
NOW = "07"
ALBUM = {k: v["album"] for k, v in TRACKS.items()}


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace('"', "&quot;")


def ic(name, cls=""):
    return f'<svg class="ic {cls}" aria-hidden="true"><use href="#i-{name}"/></svg>'


def cov(cid, cls="", size=96, alt=""):
    return f'<span class="cover {cls}"><img src="covers/cover-{cid}.svg" alt="{esc(alt)}" width="{size}" height="{size}" loading="lazy" decoding="async"></span>'


def fmt_badge(t):
    hires = " hires" if t["bits"].startswith("24") else ""
    return f'<span class="fmt{hires}" aria-label="{t["fmt"]} {t["bits"]}"><b>{t["fmt"]}</b><i></i>{t["short"]}</span>'


def wordmark(cls="logo"):
    return (f'<img class="{cls} th-d" src="../assets/kazzu-wordmark-dark.svg" alt="Kazzu" width="108" height="28">'
            f'<img class="{cls} th-l" src="../assets/kazzu-wordmark-light.svg" alt="Kazzu" width="108" height="28">')


def src_tag(text=f"Do {YT}", icon="cloud"):
    return f'<span class="src">{ic(icon)}{text}</span>'


def head(title, cid=NOW, body_cls="", screen="", immersive=False):
    cls = f"{body_cls} cv-{cid}".strip() + (" immersive scope-dark" if immersive else "")
    theme = ' data-theme="dark"' if immersive else ""
    return f'''<!doctype html>
<html lang="pt-BR"{theme or " data-theme=\"dark\""}>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Kazzu: {title}</title>
<meta name="color-scheme" content="dark light">
<link rel="icon" href="../assets/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="assets/tokens.css">
<link rel="stylesheet" href="assets/covers.css">
<link rel="stylesheet" href="assets/kazzu.css">
</head>
<body class="{cls}" data-screen="{screen}" data-cover="{cid}"{' data-immersive="1"' if immersive else ''}>
<a class="skip" href="#main">Ir para o conteúdo</a>
{SPRITE}
'''


def tail():
    return '\n<script src="assets/app.js"></script>\n</body>\n</html>\n'


def stage(cid=NOW, player=False):
    if not player:
        return ""
    return f'<div class="pbg" aria-hidden="true"><img src="covers/cover-{cid}.svg" alt="" width="640" height="640" data-stage></div>\n'


NAV = [("home", "home.html", "Início", "home"), ("explore", "explore.html", "Explorar", "explore"), ("library", "library.html", "Biblioteca", "library")]


FILLED = {"home", "explore", "library"}


def tab(href, icon, label, cur):
    c = ' aria-current="page"' if cur else ""
    name = icon + "-fill" if (cur and icon in FILLED) else icon
    return f'<a class="tab" href="{href}"{c}>{ic(name)}<span>{label}</span></a>'


def rail_nav(active):
    tabs = "".join(tab(h, i, l, k == active) for k, h, l, i in NAV)
    return f'''<aside class="rail-nav glass g1" aria-label="Navegação principal">
  <a class="mk" href="home.html" aria-label="Kazzu, ir para Início"><img class="mark-sm th-d" src="../assets/kazzu-mark-dark.svg" alt="" width="36" height="36"><img class="mark-sm th-l" src="../assets/kazzu-mark-light.svg" alt="" width="36" height="36">{wordmark("logo-w")}</a>
  <nav>{tabs}{tab("search.html", "search", "Buscar", active == "search")}</nav>
  <div class="sp"></div>
  {tab("settings.html", "gear", "Ajustes", active == "settings")}
</aside>'''


def mini_html(cid=NOW):
    t = TRACKS[cid]
    return f'''<div class="mini glass g2" role="region" aria-label="Tocando agora">
    <a class="cover" href="player.html" aria-label="Abrir o player: {esc(t['song'])}, {esc(t['artist'])}" style="display:block">{cov(cid, size=96, cls="")}</a>
    <a class="meta" href="player.html" tabindex="-1" aria-hidden="true"><span class="t-callout ellipsis w6">{esc(t['song'])}</span><span class="t-sub muted2 ellipsis">{esc(t['artist'])}</span></a>
    <span class="extra"><span class="t-mono">1:02</span><div class="slider" role="slider" tabindex="0" aria-label="Progresso" aria-valuemin="0" aria-valuemax="201" aria-valuenow="62" aria-valuetext="1:02 de 3:21" data-kind="seek"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div><span class="t-mono">3:21</span></span>
    <button class="btn-ic" type="button" aria-label="Pausar" data-playtoggle>{ic("pause", "ic-lg")}</button>
    <button class="btn-ic" type="button" aria-label="Próxima faixa">{ic("next")}</button>
    <span class="prog" aria-hidden="true" style="--p:31%"></span>
  </div>'''


def chrome(active):
    cur = ' aria-current="page"' if active == "search" else ""
    tabs = "".join(tab(h, i, l, k == active) for k, h, l, i in NAV)
    return f'''<div class="chrome">
  {mini_html()}
  <div class="dock-row">
    <nav class="dock glass g2" aria-label="Navegação principal">{tabs}</nav>
    <a class="search-btn glass g2" href="search.html" aria-label="Buscar"{cur}>{ic("search", "ic-lg")}</a>
  </div>
</div>'''


def shell(title, active, body, cid=NOW, screen="", mini=True, extra=""):
    return (head(title, cid, screen=screen) + stage(cid) + rail_nav(active) + f'<main class="page" id="main">\n{body}\n</main>\n'
            + (chrome(active) if mini else "") + extra + tail())


def topbar(title="", right="", action="", large=True):
    """cabeçalho de rolagem (vidro ao rolar) + título grande Apple (encolhe para o título pequeno ao rolar)."""
    hdr = f'''<header class="topbar" data-hdr>
    <a class="logo-link" href="home.html" aria-label="Kazzu, ir para Início">{wordmark()}</a>
    <span class="inline-title" aria-hidden="true">{title}</span>
    <div class="actions">{right}<a class="btn-ic glass" href="settings.html" aria-label="Ajustes">{ic("gear")}</a></div>
  </header>'''
    big = f'<div class="lt-row"><h1 class="t-large lt">{title}</h1>{action}</div>' if large and title else ""
    return hdr + "\n  " + big


def topbar_back(title, back="home.html", right="", large=True, back_label="Voltar"):
    hdr = f'''<header class="topbar" data-hdr>
    <a class="btn-ic glass back" href="{back}" aria-label="{back_label}">{ic("back")}</a>
    <span class="inline-title" aria-hidden="true">{title}</span>
    <div class="actions">{right}</div>
  </header>'''
    big = f'<div class="lt-row"><h1 class="t-large lt">{title}</h1></div>' if large and title else ""
    return hdr + "\n  " + big


def row(cid, playing=False, why=None, badge=True, rank=None):
    t = TRACKS[cid]
    sub = esc(t["artist"])
    why_html = f'<span class="why ellipsis">{esc(why)}</span>' if why else ""
    now = '<span class="nowbars" role="img" aria-label="Tocando agora"><i></i><i></i><i></i></span>' if playing else ""
    rk = f'<span class="rank">{rank}</span>' if rank else ""
    lead = rk
    return f'''<li><div class="row{" playing" if playing else ""}">
      <a class="cover" href="player.html" aria-label="Tocar {esc(t['song'])}">{cov(cid, size=104)}</a>
      <a class="meta" href="player.html"><span class="title ellipsis">{esc(t['song'])}</span><span class="sub ellipsis">{sub}</span>{why_html}</a>
      <span class="end">{now}{fmt_badge(t) if badge else ""}<a class="btn-ic" href="sheets.html" aria-label="Mais opções para {esc(t['song'])}">{ic("more")}</a></span>
    </div></li>'''


def tile(c, sub=None):
    t = TRACKS[c]
    return f'<a class="tile" href="album.html">{cov(c, size=352)}<span><span class="title ellipsis" style="display:block">{esc(ALBUM[c])}</span><span class="sub ellipsis" style="display:block">{esc(sub or t["artist"])}</span></span></a>'


# ------------------------------------------------------------------ INÍCIO
def page_home():
    heroes = [("07", "Novo no YouTube Music", "Calor de Março", "Nina Sabiá"), ("06", "Mix para foco", "Linha de Fuga e mais", "Do YouTube Music"),
              ("02", "Para começar o dia", "Hora do Café", "Doma"), ("08", "Descobrir", "Vidro Líquido", "Kaito e Vê")]
    cards = "".join(f'''<a class="hero-card" href="player.html" aria-label="{esc(t)}, {esc(c)}">
      <img src="covers/cover-{cid}.svg" alt="" width="640" height="640" decoding="async">
      <span class="plate glass g1 scope-dark"><span class="txt"><span class="t-caption muted2">{esc(cap)}</span><span class="t-headline ellipsis">{esc(t)}</span><span class="t-sub muted2 ellipsis">{esc(c)}</span></span><span class="go" aria-hidden="true">{ic("play", "ic-lg")}</span></span>
    </a>''' for cid, cap, t, c in heroes)
    quick = "".join([
        row("07", playing=True, why="Em alta hoje"), row("08", why="Novidade para você"), row("02", why="Para a manhã"), row("04", why="Parecido com Os Faróis")])
    tiles = "".join(tile(c) for c in ["01", "06", "09", "10"])
    mixes = "".join(f'<a class="tile" href="album.html">{cov(c, size=352)}<span><span class="title ellipsis" style="display:block">{n}</span><span class="sub ellipsis" style="display:block">{s}</span></span></a>'
                    for c, n, s in [("06", "Mix Foco", "40 músicas"), ("01", "Mix Madrugada", "35 músicas"), ("02", "Mix Energia", "50 músicas"), ("09", "Mix Calmaria", "30 músicas")])
    body = f'''  {topbar("Início")}
  <aside class="banner cell" id="firstrun" aria-label="Aviso de primeiro uso">
    <span class="ico" aria-hidden="true">{ic("badge-check", "ic-lg")}</span>
    <span><span class="t-headline" style="display:block">O YouTube Music já está ativo</span><span class="t-sub muted2">Sem conta e sem configuração. O Kazzu só usa a rede quando você pede algo.</span></span>
    <button class="btn-ic" type="button" aria-label="Dispensar aviso" data-dismiss="firstrun">{ic("close")}</button>
  </aside>
  <div class="chips" role="group" aria-label="Seções do Início">
    <button class="chip" type="button" aria-pressed="true">Para você</button><button class="chip" type="button" aria-pressed="false">Descobrir</button>
    <button class="chip" type="button" aria-pressed="false">Mixes</button><button class="chip" type="button" aria-pressed="false">Foco</button><button class="chip" type="button" aria-pressed="false">Treino</button>
  </div>
  <section class="section" aria-labelledby="h-dest">
    <div class="section-head"><h2 id="h-dest">Continue por aqui</h2>{src_tag()}</div>
    <div class="rail" tabindex="0" role="region" aria-label="Destaques. Role para os lados">{cards}</div>
  </section>
  <section class="section" aria-labelledby="h-q">
    <div class="section-head"><h2 id="h-q">Escolhas rápidas</h2><a class="btn-text" href="explore.html">Ver mais{ic("chevron-right", "ic-sm")}</a></div>
    <ul class="group-list">{quick}</ul>
  </section>
  <section class="section" aria-labelledby="h-mix">
    <div class="section-head"><h2 id="h-mix">Mixes para você</h2>{src_tag()}</div>
    <div class="rail tiles-rail" tabindex="0" role="region" aria-label="Mixes. Role para os lados">{mixes}</div>
  </section>
  <section class="section" aria-labelledby="h-alb">
    <div class="section-head"><h2 id="h-alb">Álbuns em destaque</h2>{src_tag()}</div>
    <div class="tiles">{tiles}</div>
  </section>'''
    return shell("Início", "home", body, screen="home")


# ------------------------------------------------------------------ EXPLORAR
def page_explore():
    moods = [("Foco", "06"), ("Madrugada", "01"), ("Energia", "02"), ("Verão", "07"), ("Calmaria", "09"), ("Estrada", "04")]
    mtiles = "".join(f'''<a class="mood" href="search.html#s=results" style="position:relative;overflow:hidden;aspect-ratio:1.55;border-radius:22px;display:block;box-shadow:0 10px 24px -10px color-mix(in srgb, var(--stage-a) 55%, black), 0 0 0 1px color-mix(in srgb, var(--ink) 12%, transparent)">
      <img src="covers/cover-{c}.svg" alt="" style="position:absolute;inset:0;width:100%;height:100%;object-fit:cover;opacity:.9;z-index:-2">
      <span class="plate glass g1 scope-dark" style="position:absolute;left:8px;right:8px;bottom:8px;padding:10px 14px;--g-r:16px"><span class="t-headline" style="color:#fff">{n}</span></span></a>''' for n, c in moods)
    new = "".join(tile(c, sub=f"{TRACKS[c]['artist']}") for c in ["08", "01", "09", "05", "10"])
    charts = "".join(row(c, rank=i + 1, badge=False) for i, c in enumerate(["03", "07", "02", "06", "04"]))
    body = f'''  {topbar("Explorar")}
  <section class="section" aria-labelledby="h-new">
    <div class="section-head"><h2 id="h-new">Novos lançamentos</h2>{src_tag()}</div>
    <div class="rail tiles-rail" tabindex="0" role="region" aria-label="Lançamentos. Role para os lados">{new.replace('class="tile"', 'class="tile"')}</div>
  </section>
  <section class="section" aria-labelledby="h-mood">
    <div class="section-head"><h2 id="h-mood">Humores e gêneros</h2></div>
    <div class="cols" style="--collection-gap:12px">{mtiles}</div>
  </section>
  <section class="section" aria-labelledby="h-ch">
    <div class="section-head"><h2 id="h-ch">Em alta</h2>{src_tag()}</div>
    <ul class="group-list">{charts}</ul>
  </section>'''
    return shell("Explorar", "explore", body, screen="explore")


# ------------------------------------------------------------------ BIBLIOTECA
def page_library():
    def art_mosaic(ids):
        return '<span class="art mosaic">' + "".join(f'<img src="covers/cover-{i}.svg" alt="" width="120" height="120">' for i in ids) + "</span>"
    cards = f'''<a class="col-card cell" href="album.html"><span class="art dl">{ic("download")}</span><span><span class="t" style="display:block">Baixados</span><span class="n">0 músicas</span></span></a>
    <a class="col-card cell" href="album.html"><span class="art liked">{ic("heart-fill")}</span><span><span class="t" style="display:block">Curtidas</span><span class="n">12 músicas</span></span></a>
    <a class="col-card cell" href="album.html">{art_mosaic(["01", "02", "06", "09"])}<span><span class="t" style="display:block">Álbuns</span><span class="n">10 álbuns</span></span></a>
    <a class="col-card cell" href="album.html">{art_mosaic(["07", "03", "08", "04"])}<span><span class="t" style="display:block">Playlists</span><span class="n">3 playlists</span></span></a>'''
    recents = "".join(row(c) for c in ["07", "05", "10", "02"])
    body = f'''  {topbar("Biblioteca", action=f'<a class="btn btn-primary" href="sheets.html#s=create">{ic("playlist-add")}Criar playlist</a>')}
  <div class="cols">{cards}</div>
  <aside class="group cell" aria-label="Músicas deste aparelho">
    {settings_row("folder", "Neste aparelho", "2 músicas locais", '<a class="btn btn-glass" href="states.html#s=permission">Adicionar pasta</a>')}
  </aside>
  <section class="section" aria-labelledby="h-r">
    <div class="section-head"><h2 id="h-r">Tocadas recentemente</h2></div>
    <ul class="group-list">{recents}</ul>
  </section>'''
    return shell("Biblioteca", "library", body, screen="library")


# ------------------------------------------------------------------ ÁLBUM / COLEÇÃO
def page_album():
    cid = "07"
    tracks = "".join(row(c, playing=(c == "07")) for c in ["07", "01", "09", "04", "06"])
    body = f'''  {topbar_back("Calor de Março", back="library.html", right=f'<a class="btn-ic glass" href="sheets.html" aria-label="Mais opções do álbum">{ic("more")}</a>', large=False)}
  <section class="album-head" aria-labelledby="al-h">
    {cov(cid, size=640, alt="Capa de Calor de Março")}
    <div><h1 class="t-title1" id="al-h">Calor de Março</h1><p class="muted2">Nina Sabiá, álbum</p>
    <p style="margin-top:6px">{src_tag()}</p></div>
    <div class="acts"><button class="btn btn-primary" type="button">{ic("play")}Tocar</button><button class="btn btn-glass" type="button">{ic("shuffle")}Aleatório</button><button class="btn-ic glass" type="button" aria-label="Curtir álbum" aria-pressed="false" data-toggle>{ic("heart")}</button></div>
  </section>
  <ul class="group-list">{tracks}</ul>'''
    return shell("Álbum", "library", body, screen="album")


# ------------------------------------------------------------------ BUSCA
def page_search():
    field = f'''<form class="field" role="search" onsubmit="return false">
      <svg class="ic" aria-hidden="true" style="color:var(--ink-2)"><use href="#i-search"/></svg>
      <input type="search" aria-label="Buscar músicas, artistas e álbuns" placeholder="Músicas, artistas, álbuns" value="" data-q>
      <button class="btn-ic" type="button" aria-label="Reconhecer música pelo som ambiente" title="Reconhecer pelo som">{ic("wave")}</button>
    </form>'''
    chips = '''<div class="chips" role="group" aria-label="Filtrar por tipo"><button class="chip" aria-pressed="true" type="button">Tudo</button><button class="chip" aria-pressed="false" type="button">Músicas</button><button class="chip" aria-pressed="false" type="button">Álbuns</button><button class="chip" aria-pressed="false" type="button">Artistas</button><button class="chip" aria-pressed="false" type="button">Playlists</button></div>'''
    idle = f'''<div data-state="idle">
    <section class="section"><div class="section-head"><h2>Buscas recentes</h2><button class="btn-text" type="button">Limpar</button></div>
    <ul class="group-list">{"".join(f'<li><div class="row" style="grid-template-columns:40px minmax(0,1fr) auto"><span class="lead" style="width:40px;height:40px;display:grid;place-items:center;border-radius:12px;background:var(--chip)">{ic("clock")}</span><a class="meta" href="search.html#s=results"><span class="title">{q}</span></a><span class="end"><button class="btn-ic" type="button" aria-label="Remover {q} das buscas recentes">{ic("close", "ic-sm")}</button></span></div></li>' for q in ["Nina Sabiá", "música para foco", "Meridiano"])}</ul></section>
    <section class="section" style="margin-top:24px"><div class="section-head"><h2>Para experimentar</h2></div><div class="chips" style="flex-wrap:wrap;overflow:visible"><button class="chip" type="button">Calmaria</button><button class="chip" type="button">Estrada</button><button class="chip" type="button">Madrugada</button><button class="chip" type="button">Verão</button></div></section>
  </div>'''
    res_rows = "".join(row(c, badge=True) for c in ["07", "08", "01"])
    results = f'''<div data-state="results" hidden>
    <a class="cell card" href="player.html" style="display:grid;grid-template-columns:96px minmax(0,1fr) auto;gap:14px;align-items:center;--g-r:26px;padding:12px" aria-label="Melhor resultado: Nina Sabiá">
      {cov("07", size=192, cls="")}<span><span class="t-caption muted2" style="display:block">Melhor resultado</span><span class="t-title2" style="display:block">Nina Sabiá</span><span class="t-sub muted2">Artista</span></span><span class="go" style="width:48px;height:48px;border-radius:50%;display:grid;place-items:center;background:var(--accent);color:var(--accent-ink)">{ic("play")}</span></a>
    <section class="section" style="margin-top:20px"><div class="section-head"><h2>Músicas</h2>{src_tag()}</div><ul class="group-list">{res_rows}</ul></section>
    <section class="section" style="margin-top:20px"><div class="section-head"><h2>Álbuns</h2></div><div class="rail tiles-rail">{"".join(tile(c) for c in ["07", "02", "09"])}</div></section>
  </div>'''
    empty = f'''<div data-state="empty" hidden><div class="state cell"><span class="art">{ic("search")}</span><h2>Nada para “nina sabiá verão”</h2><p>Confira a grafia ou busque só o nome do artista.</p><button class="btn btn-glass" type="button">Limpar filtros</button></div></div>'''
    loading = '<div data-state="loading" hidden aria-busy="true" aria-label="Buscando"><div style="display:grid;gap:2px;margin-inline:-8px">' + "".join('<div class="sk-row"><span class="sk"></span><span class="sk-lines"><span class="sk"></span><span class="sk"></span></span></div>' for _ in range(5)) + '</div></div>'
    error = f'''<div data-state="error" hidden><div class="state cell"><span class="art">{ic("cloud-off")}</span><h2>O YouTube Music não respondeu</h2><p>Sua conexão parece ok. O serviço pode estar fora do ar. Tente de novo em instantes.</p><button class="btn btn-primary" type="button">{ic("refresh")}Tentar de novo</button><span class="code">Código: provedor-indisponível</span></div></div>'''
    offline = f'''<div data-state="offline" hidden><div class="state cell"><span class="art">{ic("cloud-off")}</span><h2>Sem conexão</h2><p>A busca no YouTube Music precisa de internet. Suas músicas baixadas e locais continuam tocando.</p><a class="btn btn-glass" href="library.html">Ver baixados</a></div></div>'''
    body = f'''  {topbar("Buscar")}
  {field}
  {chips}
  {idle}{results}{empty}{loading}{error}{offline}'''
    return shell("Buscar", "search", body, screen="search")


# ------------------------------------------------------------------ PLAYER
def page_player():
    cid = NOW
    t = TRACKS[cid]
    body = f'''<main class="player" id="main" data-gesture="sheet">
  <div class="grabber" aria-hidden="true"></div>
  <header class="player-top">
    <a class="btn-ic" href="home.html" aria-label="Fechar o player">{ic("chevron-down", "ic-lg")}</a>
    <span class="from t-sub"><span>Tocando de</span><span class="w6 ellipsis">Mix Foco</span></span>
    <a class="btn-ic" href="sheets.html" aria-label="Mais opções da faixa">{ic("more")}</a>
  </header>
  <div class="p-art"><img src="covers/cover-{cid}.svg" alt="Capa de {esc(t['album'])}" width="640" height="640" data-art></div>
  <div class="p-head">
    <div style="min-width:0"><h1 data-song>{esc(t['song'])}</h1><p class="artist ellipsis" data-artist>{esc(t['artist'])}</p></div>
    <button class="btn-ic glass" type="button" aria-label="Curtir" aria-pressed="false" data-toggle>{ic("heart", "ic-lg")}</button>
  </div>
  <div>
    <div class="slider" role="slider" tabindex="0" aria-label="Progresso da faixa" aria-valuemin="0" aria-valuemax="201" aria-valuenow="62" aria-valuetext="1:02 de 3:21" data-kind="seek"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div>
    <div class="p-times t-mono"><span>1:02</span><span>-2:19</span></div>
  </div>
  <div class="transport glass g2" role="group" aria-label="Controles de reprodução">
    <button class="btn-ic" type="button" aria-label="Aleatório" aria-pressed="false" data-toggle>{ic("shuffle")}</button>
    <button class="btn-ic" type="button" aria-label="Faixa anterior">{ic("prev", "ic-lg")}</button>
    <button class="btn-ic play-main" type="button" aria-label="Pausar" data-playtoggle>{ic("pause")}</button>
    <button class="btn-ic" type="button" aria-label="Próxima faixa">{ic("next", "ic-lg")}</button>
    <button class="btn-ic" type="button" aria-label="Repetir" aria-pressed="false" data-toggle>{ic("repeat")}</button>
  </div>
  <div class="p-vol"><svg class="ic" aria-hidden="true"><use href="#i-vol-low"/></svg><div class="slider" role="slider" tabindex="0" aria-label="Volume" aria-valuemin="0" aria-valuemax="100" aria-valuenow="70" style="--p:70%"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div><svg class="ic" aria-hidden="true"><use href="#i-vol-high"/></svg></div>
  <div class="p-pills">
    <a class="pill glass" href="queue.html">{ic("queue")}Fila</a>
    <button class="pill glass" type="button">{ic("lyrics")}Letra</button>
    <button class="pill glass" type="button">{ic("timer")}Timer</button>
    <button class="pill glass" type="button">{ic("device")}Este celular</button>
  </div>
  <div class="p-quality" data-quality><span class="fmt" data-fmt-badge><b data-fmt>{t['fmt']}</b><i></i><span data-bits>{t['bits']}</span></span><span>Do YouTube Music</span><span>Original, sem equalizador</span></div>
</main>'''
    return head("Player", cid, screen="player", immersive=True) + stage(cid, True) + body + tail()


# ------------------------------------------------------------------ FILA (folha)
def page_queue():
    nxt = "".join(f'''<li class="q-row"><a class="cover" href="player.html" aria-label="Tocar {esc(TRACKS[c]['song'])} agora">{cov(c, size=104)}</a>
      <span style="min-width:0;display:grid"><span class="title ellipsis">{esc(TRACKS[c]['song'])}</span><span class="sub ellipsis">{esc(TRACKS[c]['artist'])}</span></span>
      <span class="end"><button class="btn-ic" type="button" aria-label="Mover {esc(TRACKS[c]['song'])} para cima">{ic("arrow-up", "ic-sm")}</button><button class="btn-ic" type="button" aria-label="Mais opções para {esc(TRACKS[c]['song'])}">{ic("more")}</button></span></li>''' for c in ["01", "09", "04", "06", "02"])
    body = f'''<div class="scrim" aria-hidden="true"></div>
<section class="sheet tall glass g3" role="dialog" aria-modal="true" aria-labelledby="q-h">
  <div class="grabber" aria-hidden="true"></div>
  <div style="display:grid;grid-template-rows:auto 1fr;min-height:0">
    <div class="sheet-title"><h1 class="t-title2" id="q-h">Fila</h1><span style="display:flex;gap:4px;align-items:center"><button class="btn btn-glass" type="button" style="min-height:44px">Limpar fila</button><a class="btn-ic" href="player.html" aria-label="Fechar a fila">{ic("close")}</a></span></div>
    <div class="sheet-body">
      <p class="t-caption muted2" style="margin:4px 0 6px">Tocando agora</p>
      <div class="q-row now">{cov(NOW, size=104)}<span style="min-width:0;display:grid"><span class="title ellipsis" style="font-weight:700">{esc(TRACKS[NOW]['song'])}</span><span class="sub ellipsis">{esc(TRACKS[NOW]['artist'])}</span></span><span class="nowbars" role="img" aria-label="Tocando agora" style="margin-right:16px"><i></i><i></i><i></i></span></div>
      <div class="section-head" style="margin-top:14px"><p class="t-caption muted2">A seguir</p><span class="src">{ic("cloud")}Mix Foco</span></div>
      <ul>{nxt}</ul>
      <p class="t-sub muted2" style="padding:14px 4px 4px">Quando a fila acabar, o Kazzu continua com músicas parecidas do YouTube Music. Dá para desligar em Ajustes.</p>
    </div>
  </div>
</section>'''
    t = TRACKS[NOW]
    behind = f'''<div class="player" aria-hidden="true"><div class="grabber"></div><div class="p-art"><img src="covers/cover-{NOW}.svg" alt="" width="640" height="640"></div>
  <div class="p-head"><div style="min-width:0"><h1>{esc(t['song'])}</h1><p class="artist">{esc(t['artist'])}</p></div></div></div>'''
    return head("Fila", NOW, screen="queue", immersive=True) + stage(NOW, True) + behind + body + tail()


# ------------------------------------------------------------------ AJUSTES
def settings_row(icon, t, d, end):
    return f'<div class="set-row"><span class="lead">{ic(icon)}</span><span><span class="t" style="display:block">{t}</span><span class="d">{d}</span></span>{end}</div>'


def switch(label, on=False, setting="", disabled=False):
    return f'<button class="switch" type="button" role="switch" aria-checked="{str(on).lower()}" aria-label="{label}"{f" data-setting={setting}" if setting else ""}{" aria-disabled=true" if disabled else ""}></button>'


def page_settings():
    body = f'''  {topbar_back("Ajustes")}
  <section class="section" aria-labelledby="g1"><h2 class="group-h" id="g1">Fontes de música</h2>
    <div class="group cell">
      {settings_row("cloud", "YouTube Music", "Ativo e padrão. Sem login.", '<span class="tag ok">Ativo</span>')}
      {settings_row("folder", "Arquivos neste aparelho", "1 pasta, 2 músicas", '<a class="btn btn-glass" href="states.html#s=permission">Adicionar pasta</a>')}
      {settings_row("plus", "Outras fontes", "Spotify, Tidal e Qobuz entram depois, só se você ligar.", '<span class="tag">Em breve</span>')}
    </div></section>
  <section class="section" aria-labelledby="g2"><h2 class="group-h" id="g2">Áudio</h2>
    <div class="group cell">
      {settings_row("wave", "Qualidade", "Original, sem equalizador. O que a fonte entrega é o que toca.", '<span class="tag">Original</span>')}
      {settings_row("eq", "Equalizador", "Desligado. Desligado significa sinal intacto.", '<span class="tag">Em breve</span>')}
      {settings_row("download", "Baixar só no Wi-Fi", "Evita gastar seu plano de dados.", switch("Baixar só no Wi-Fi", True))}
    </div></section>
  <section class="section" aria-labelledby="g3"><h2 class="group-h" id="g3">Aparência</h2>
    <div class="group cell">
      <div class="set-row" style="grid-template-columns:1fr"><div role="radiogroup" aria-label="Tema" class="seg lite"><button type="button" role="radio" aria-checked="false" data-theme="auto">Automático</button><button type="button" role="radio" aria-checked="true" data-theme="dark">Escuro</button><button type="button" role="radio" aria-checked="false" data-theme="light">Claro</button></div></div>
      <div class="set-row" style="grid-template-columns:1fr"><div role="radiogroup" aria-label="Cor de destaque" class="seg lite"><button type="button" role="radio" aria-checked="false" data-accent="dyn">Da capa</button><button type="button" role="radio" aria-checked="true" data-accent="brand">Âmbar Kazzu</button></div></div>
      {settings_row("moon", "Reduzir transparência", "Troca o vidro por painéis sólidos. Ajuda em telas lentas e em baixa visão.", switch("Reduzir transparência", False, "reduce-transparency"))}
      {settings_row("sliders", "Reduzir movimento", "Corta animações e a troca suave de capas.", switch("Reduzir movimento", False, "reduce-motion"))}
    </div></section>
  <section class="section" aria-labelledby="g4"><h2 class="group-h" id="g4">Privacidade</h2>
    <div class="group cell">
      {settings_row("shield", "Sem telemetria", "O Kazzu só recebe dados. Nada sobre você sai do aparelho. Não dá para desligar.", switch("Sem telemetria, sempre ligado", True, "", True))}
      {settings_row("lock", "Histórico de escuta", "Fica só neste aparelho.", '<button class="btn btn-glass" type="button" style="min-height:44px">Apagar</button>')}
    </div></section>
  <section class="section" aria-labelledby="g5"><h2 class="group-h" id="g5">Sobre</h2>
    <div class="group cell">
      {settings_row("info", "Kazzu 0.1", "Licença GPL-3.0, código aberto.", f'<a class="btn-ic" href="settings.html" aria-label="Ver licenças">{ic("chevron-right")}</a>')}
    </div></section>'''
    return shell("Ajustes", "settings", body, screen="settings")


# ------------------------------------------------------------------ FOLHAS
def page_sheets():
    menu_items = [("play", "Tocar em seguida", ""), ("queue", "Adicionar à fila", ""), ("heart", "Curtir", ""), ("playlist-add", "Adicionar à playlist", ""), ("download", "Baixar", "Wi-Fi"), ("disc", "Ir para o álbum", ""), ("user", "Ir para o artista", "")]
    items = "".join(f'<button class="menu-item" type="button">{ic(i)}<span>{t}</span><span class="hint">{h}</span></button>' for i, t, h in menu_items)
    t = TRACKS["08"]
    menu = f'''<div data-state="menu"><div class="scrim" aria-hidden="true"></div>
<section class="sheet glass g3" role="dialog" aria-modal="true" aria-label="Opções de {esc(t['song'])}"><div class="grabber" aria-hidden="true"></div>
  <div class="sheet-body menu"><div class="menu-head">{cov("08", size=112)}<span style="min-width:0;display:grid"><span class="t-headline ellipsis">{esc(t['song'])}</span><span class="t-sub muted2 ellipsis">{esc(t['artist'])}</span></span></div>{items}</div></section></div>'''
    pls = "".join(f'<button class="menu-item" type="button" style="grid-template-columns:48px minmax(0,1fr) auto">{cov(c, size=96)}<span><span style="display:block">{n}</span><span class="hint">{k}</span></span><span class="hint"></span></button>'
                  for c, n, k in [("07", "Manhã", "8 músicas"), ("06", "Para focar", "24 músicas"), ("03", "Estrada", "11 músicas")])
    plist = f'''<div data-state="playlist" hidden><div class="scrim" aria-hidden="true"></div>
<section class="sheet glass g3" role="dialog" aria-modal="true" aria-labelledby="pl-h"><div class="grabber" aria-hidden="true"></div>
  <div class="sheet-title"><h2 class="t-title2" id="pl-h">Adicionar à playlist</h2><a class="btn-ic" href="sheets.html" aria-label="Fechar">{ic("close")}</a></div>
  <div class="sheet-body menu"><button class="menu-item" type="button" data-goto="create">{ic("plus")}<span>Nova playlist</span><span></span></button>{pls}</div></section></div>'''
    create = f'''<div data-state="create" hidden><div class="scrim" aria-hidden="true"></div>
<section class="sheet glass g3" role="dialog" aria-modal="true" aria-labelledby="cr-h"><div class="grabber" aria-hidden="true"></div>
  <div class="sheet-title"><h2 class="t-title2" id="cr-h">Nova playlist</h2><a class="btn-ic" href="sheets.html" aria-label="Fechar">{ic("close")}</a></div>
  <div class="sheet-body" style="display:grid;gap:14px;padding-top:4px"><label class="field" style="display:flex"><span class="sr-only">Nome da playlist</span><input type="text" placeholder="Nome da playlist" value="Noite longa" data-autofocus></label>
  <p class="t-sub muted2">Fica só neste aparelho. Você pode adicionar músicas do YouTube Music e do seu acervo.</p>
  <button class="btn btn-primary" type="button" style="width:100%">Criar playlist</button></div></section></div>'''
    heroes = "".join(f'<a class="hero-card" href="player.html"><img src="covers/cover-{c}.svg" alt="" width="640" height="640"><span class="plate glass g1 scope-dark"><span class="txt"><span class="t-caption muted2">{cap}</span><span class="t-headline ellipsis">{n}</span></span><span class="go">{ic("play", "ic-lg")}</span></span></a>' for c, cap, n in [("08", "Descobrir", "Vidro Líquido"), ("04", "Mix", "Canal Sete")])
    behind = '<div class="page" aria-hidden="true">' + topbar("Início") + f'<div class="rail">{heroes}</div>' + '<ul class="group-list">' + "".join(row(c) for c in ["07", "08", "02", "05", "03", "04", "01"]) + '</ul></div>'
    return head("Folhas e menus", "08", screen="sheets") + stage("08") + f'<main id="main">{behind}</main>' + menu + plist + create + tail()


# ------------------------------------------------------------------ ESTADOS
def page_states():
    def state(icon, h, p, btn="", code=""):
        return f'<div class="state cell"><span class="art">{ic(icon)}</span><h2>{h}</h2><p>{p}</p>{btn}{code}</div>'
    perm = state("folder", "O Kazzu precisa de acesso à pasta", "Para tocar suas músicas locais, escolha uma pasta. O Kazzu lê só o que você escolher e nada sai do aparelho.", '<button class="btn btn-primary" type="button">Escolher pasta</button>')
    items = [
        ("empty", state("library", "Sua biblioteca está vazia", "Curta uma música ou adicione uma pasta e ela aparece aqui.", '<a class="btn btn-primary" href="states.html#s=permission">Adicionar pasta</a>')),
        ("loading", '<div class="cell card" aria-busy="true" aria-label="Carregando"><div style="display:grid;gap:2px">' + "".join('<div class="sk-row"><span class="sk"></span><span class="sk-lines"><span class="sk"></span><span class="sk"></span></span></div>' for _ in range(4)) + "</div></div>"),
        ("error", state("cloud-off", "Não deu para carregar o Início", "O YouTube Music não respondeu. Suas músicas locais continuam disponíveis.", f'<button class="btn btn-primary" type="button">{ic("refresh")}Tentar de novo</button>', '<span class="code">Código: provedor-indisponível</span>')),
        ("offline", '<div class="glass g2 offline-bar" role="status">' + ic("cloud-off") + '<span>Sem conexão. Tocando só o que está no aparelho.</span></div>' + state("download", "Seus baixados estão aqui", "Sem internet, o Kazzu mostra só o que já está no aparelho.", '<a class="btn btn-glass" href="library.html">Ver baixados</a>')),
        ("permission", perm),
        ("noresult", state("search", "Nada para “zzz”", "Confira a grafia ou busque só o nome do artista.", '<button class="btn btn-glass" type="button">Limpar filtros</button>')),
    ]
    names = {"empty": "Vazio", "loading": "Carregando", "error": "Erro de provedor", "offline": "Offline", "permission": "Sem permissão", "noresult": "Sem resultados"}
    chips = "".join(f'<a class="chip" href="#s={k}" aria-current="{"true" if k == "empty" else "false"}" data-pick="{k}">{names[k]}</a>' for k, _ in items)
    blocks = "".join(f'<section data-state="{k}"{"" if k == "empty" else " hidden"} aria-label="{names[k]}">{h}</section>' for k, h in items)
    body = f'''  {topbar_back("Estados")}
  <div class="chips" role="group" aria-label="Escolher estado">{chips}</div>
  {blocks}'''
    return shell("Estados", "library", body, screen="states")


# ------------------------------------------------------------------ PROVA DE VIDRO
def page_proof():
    def collage(order):
        return '<div class="collage" aria-hidden="true">' + "".join(f'<img src="covers/cover-{c}.svg" alt="" width="320" height="320">' for c in order) + "</div>"
    def samples(prefix=""):
        return f'''<div class="prow">
      <div><p class="label">Nível 1 · fino</p><div class="psample glass g1"><span class="t-headline">Legenda e botões</span><span class="t-sub muted2">Texto secundário sobre vidro</span></div></div>
      <div><p class="label">Nível 2 · regular</p><div class="psample glass g2"><span class="t-headline">Barra, mini player, cabeçalho</span><span class="t-sub muted2">Texto secundário sobre vidro</span></div></div>
      <div><p class="label">Nível 3 · espesso</p><div class="psample glass g3"><span class="t-headline">Folhas e menus</span><span class="t-sub muted2">Texto secundário sobre vidro</span></div></div>
    </div>
    <div class="pdock"><nav class="dock glass g2" aria-label="Exemplo de barra">{tab("#", "home", "Início", False)}{tab("#", "explore", "Explorar", True)}{tab("#", "library", "Biblioteca", False)}</nav><span class="search-btn glass g2">{ic("search", "ic-lg")}</span>
      <div class="mini glass g2" style="flex:1;min-width:260px;max-width:420px"><span class="cover">{cov("07", size=96)}</span><span class="meta"><span class="t-callout ellipsis">Quase Verão</span><span class="t-sub muted2 ellipsis">Nina Sabiá</span></span><span></span><span class="btn-ic">{ic("pause", "ic-lg")}</span><span class="prog" style="--p:31%"></span></div></div>'''
    dark = f'''<section class="pstage scope-dark" aria-label="Escuro">{collage(["07", "08", "02", "06", "01", "09", "04", "10", "03", "05", "07", "08"])}
    <h2 class="t-title2" style="color:#fff">Escuro</h2>{samples()}</section>'''
    light = f'''<section class="pstage scope-light" aria-label="Claro">{collage(["09", "03", "07", "05", "08", "01", "10", "04", "06", "02", "09", "03"])}
    <h2 class="t-title2" style="color:#fff;text-shadow:0 1px 8px #0006">Claro</h2>{samples()}</section>'''
    rt = f'''<section class="pstage scope-dark" aria-label="Reduzir transparência">{collage(["07", "08", "02", "06", "01", "09"])}
    <h2 class="t-title2" style="color:#fff">Reduzir transparência (fallback sólido, mesma hierarquia)</h2>
    <div class="prow"><div class="psample glass g1 solid-glass"><span class="t-headline">Nível 1</span><span class="t-sub muted2">Sólido neutro</span></div><div class="psample glass g2 solid-glass"><span class="t-headline">Nível 2</span><span class="t-sub muted2">Sólido neutro</span></div><div class="psample glass g3 solid-glass"><span class="t-headline">Nível 3</span><span class="t-sub muted2">Sólido neutro</span></div></div></section>'''
    old = f'''<section class="pstage scope-dark" aria-label="Antes e depois">{collage(["07", "08", "02", "06", "01", "09"])}
    <h2 class="t-title2" style="color:#fff">Antes (v2: fill tingido de marrom, saturado) × depois (v3: sombreado neutro + tint claro)</h2>
    <div class="prow"><div><p class="label">v2: "esfumaçado"</p><div class="psample old-glass"><span class="t-headline">Barra, mini player</span><span class="t-sub" style="opacity:.8">Texto secundário</span></div></div>
    <div><p class="label">v3: vidro claro</p><div class="psample glass g2"><span class="t-headline">Barra, mini player</span><span class="t-sub muted2">Texto secundário</span></div></div></div></section>'''
    html = head("Prova de vidro", "07", screen="proof") + f'''<main class="proof" id="main">
  <h1 class="t-large">Prova de vidro</h1>
  <p class="lead">O vidro só lê como vidro com conteúdo colorido passando por trás. Aqui, o mesmo material (blur 24/32/40 + saturação 1,8, sombreado neutro + tint branco, borda de luz em gradiente, fio especular, sombra neutra) sobre capas coloridas, nos temas escuro e claro. No Slint 1.19 só existe o blur (sem saturação): veja <code>../slint/proof-dark.png</code>.</p>
  {dark}
  {light}
  {old}
  {rt}
  <p class="pnote">Receita (DESIGN.md §6): blur → sombreado neutro (preto 32–48% no escuro) → tint claro (branco 10–20% no escuro, 55–75% no claro) → brilho especular → borda de luz 1 px em gradiente → fio especular → sombra neutra. Os textos de vidro usam <code>--ink</code> e <code>--ink-glass-2</code> (contraste medido em <code>ux/contrast-report.md</code>).</p>
</main>''' + tail()
    return html


# ------------------------------------------------------------------ HUB
def page_index():
    screens = [("home", "Início"), ("explore", "Explorar"), ("library", "Biblioteca"), ("album", "Álbum"), ("search", "Buscar"), ("player", "Player"), ("queue", "Fila"), ("sheets", "Folhas"), ("settings", "Ajustes"), ("states", "Estados")]
    opts = "".join(f'<option value="{k}.html">{n}</option>' for k, n in screens)
    html = '''<!doctype html>
<html lang="pt-BR" data-theme="dark">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Protótipos — Kazzu v3</title>
<link rel="icon" href="../assets/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="assets/tokens.css"><link rel="stylesheet" href="assets/covers.css"><link rel="stylesheet" href="assets/kazzu.css">
<style>
 body.hub{padding:24px 16px 64px;max-width:1560px;margin-inline:auto}
 .hub h1{font:700 28px/34px var(--font-sans)} .hub p{color:var(--ink-2);max-width:68ch;margin-top:6px}
 .ctl{display:flex;flex-wrap:wrap;gap:10px;align-items:center;margin:20px 0}
 .ctl select{min-height:48px;border-radius:24px;padding:0 16px;background:var(--chip);color:var(--ink);border:1px solid color-mix(in srgb,var(--ink) 20%,transparent);font:600 15px var(--font-sans)}
 .ctl .chip{cursor:pointer}
 .frames{display:flex;flex-wrap:wrap;gap:28px;align-items:flex-start}
 figure{margin:0} figcaption{margin-top:8px;color:var(--ink-2);font:500 12px/16px var(--font-mono)}
 .frame{position:relative;overflow:hidden;border-radius:34px;border:1px solid color-mix(in srgb,var(--ink) 22%,transparent);background:#000}
 .frame iframe{position:absolute;left:0;top:0;border:0;transform-origin:0 0}
 .f390{width:390px;height:844px} .f390 iframe{width:390px;height:844px}
 .f360{width:360px;height:740px} .f360 iframe{width:360px;height:740px}
 .f768{width:576px;height:768px;border-radius:20px} .f768 iframe{width:768px;height:1024px;transform:scale(.75)}
 .f1280{width:960px;height:600px;border-radius:14px} .f1280 iframe{width:1280px;height:800px;transform:scale(.75)}
 @media (max-width:1000px){.f768,.f1280{display:none}
</style></head>
<body class="hub cv-07">
@@SPRITE@@
<h1>Kazzu v3: protótipos mobile-first (Apple-like)</h1>
<p>Base em 390 e 360 px; depois tablet (768) e desktop (1280). O vidro usa <code>backdrop-filter</code> real. Cada tela abre sozinha e aceita <code>#theme=light|dark</code>, <code>#rt=1</code> (reduzir transparência), <code>#sat=0</code> (sem saturação, como no Slint) e <code>#c=01..10</code> (capa).</p>
<div class="ctl" role="group" aria-label="Controles">
  <label class="sr-only" for="scr">Tela</label><select id="scr">@@OPTS@@</select>
  <button class="chip" type="button" data-hub="theme" aria-pressed="false">Tema claro</button>
  <button class="chip" type="button" data-hub="rt" aria-pressed="false">Sem transparência</button>
  <button class="chip" type="button" data-hub="sat" aria-pressed="false">Sem saturação (Slint)</button>
  <a class="chip" href="glass-proof.html">Prova de vidro</a>
  <label class="sr-only" for="cvr">Capa</label><select id="cvr">@@COVOPTS@@</select>
</div>
<div class="frames" id="frames">
  <figure><div class="frame f390"><iframe title="390 px" id="i390" loading="lazy"></iframe></div><figcaption>390 × 844 (base)</figcaption></figure>
  <figure><div class="frame f360"><iframe title="360 px" id="i360" loading="lazy"></iframe></div><figcaption>360 × 740 (mínimo)</figcaption></figure>
  <figure><div class="frame f768"><iframe title="768 px" id="i768" loading="lazy"></iframe></div><figcaption>768 × 1024 (tablet)</figcaption></figure>
  <figure><div class="frame f1280"><iframe title="1280 px" id="i1280" loading="lazy"></iframe></div><figcaption>1280 × 800 (desktop)</figcaption></figure>
</div>
<script>
(function(){
  var st={theme:false,rt:false,sat:false}, scr=document.getElementById('scr'), cvr=document.getElementById('cvr');
  function go(){
    var h=[]; if(st.theme)h.push('theme=light'); if(st.rt)h.push('rt=1'); if(st.sat)h.push('sat=0');
    var s=scr.value; if(s==='player.html'||s==='queue.html'||s==='sheets.html') h.push('c='+cvr.value); else h.push('c='+cvr.value);
    ['390','360','768','1280'].forEach(function(w){document.getElementById('i'+w).src=s+'#'+h.join('&');});
    var q=new URLSearchParams(location.hash.slice(1)); history.replaceState(null,'','#s='+s.replace('.html',''));
  }
  var m=location.hash.match(/s=([a-z]+)/); if(m) scr.value=m[1]+'.html';
  scr.addEventListener('change',go); cvr.addEventListener('change',go);
  document.querySelectorAll('[data-hub]').forEach(function(b){b.addEventListener('click',function(){var k=b.dataset.hub; st[k]=!st[k]; b.setAttribute('aria-pressed',st[k]); if(k==='theme') document.documentElement.setAttribute('data-theme',st.theme?'light':'dark'); go();});});
  go();
})();
</script>
</body></html>
'''
    covopts = "".join(f'<option value="{k}">{COVERS[k]["title"]}</option>' for k in COVERS)
    return html.replace("@@SPRITE@@", SPRITE).replace("@@OPTS@@", opts).replace("@@COVOPTS@@", covopts)


def main():
    pages = dict(home=page_home, explore=page_explore, library=page_library, album=page_album, search=page_search, player=page_player,
                 queue=page_queue, settings=page_settings, sheets=page_sheets, states=page_states, **{'glass-proof': page_proof}, index=page_index)
    for name, fn in pages.items():
        (P / f"{name}.html").write_text(fn(), encoding="utf-8")
    print("ok:", ", ".join(pages))


if __name__ == "__main__":
    main()
