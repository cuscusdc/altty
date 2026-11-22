#!/usr/bin/env python3
"""Gera os protótipos HTML estáticos em design/prototypes/ (index, home, explore, player, library, queue, search, settings, states).
Uso: python3 design/tools/build_prototypes.py   (antes rode gen_tokens.py e make_assets.py)
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
P = ROOT / "prototypes"
SPRITE = (P / "assets" / "sprite.svg.html").read_text(encoding="utf-8")

# ------------------------------------------------------------------ dados fictícios
TRACKS = {
    "01": dict(song="Maré Alta", artist="Aurora Vento", album="Cidade Submersa", fmt="FLAC", bits="24-bit/96 kHz", short="24/96", dur="4:12", src="local"),
    "02": dict(song="Hora do Café", artist="Doma", album="Nove Horas", fmt="FLAC", bits="16-bit/44,1 kHz", short="16/44,1", dur="3:38", src="local"),
    "03": dict(song="Mapa de Neblina", artist="Lia Cantos", album="Bruma", fmt="MP3", bits="320 kbps", short="320", dur="3:05", src="local"),
    "04": dict(song="Canal Sete", artist="Os Faróis", album="Quadra 7", fmt="WAV", bits="16-bit/44,1 kHz", short="16/44,1", dur="2:57", src="local"),
    "05": dict(song="Escada de Sal", artist="Tomás Reis", album="Sal e Cinza", fmt="FLAC", bits="24-bit/192 kHz", short="24/192", dur="5:44", src="local"),
    "06": dict(song="Linha de Fuga", artist="Meridiano", album="Linha de Fuga", fmt="FLAC", bits="24-bit/48 kHz", short="24/48", dur="6:20", src="local"),
    "07": dict(song="Quase Verão", artist="Nina Sabiá", album="Calor de Março", fmt="FLAC", bits="24-bit/96 kHz", short="24/96", dur="3:21", src="local"),
    "08": dict(song="Respiração de Vidro", artist="Kaito e Vê", album="Vidro Líquido", fmt="OPUS", bits="160 kbps", short="160", dur="3:49", src="YT Music"),
    "09": dict(song="Seda e Sal", artist="Ilha Branca", album="Papel de Seda", fmt="FLAC", bits="16-bit/44,1 kHz", short="16/44,1", dur="4:03", src="local"),
    "10": dict(song="Fita sem Fim", artist="Rádio Cobre", album="Fita Magnética", fmt="WAV", bits="24-bit/96 kHz", short="24/96", dur="7:15", src="local"),
}
NOW = "07"
ALBUM = {k: v["album"] for k, v in TRACKS.items()}


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;")


def ic(name, cls=""):
    return f'<svg class="ic {cls}" aria-hidden="true"><use href="#i-{name}"/></svg>'


def cov(cid, cls="", size=None):
    return f'<span class="cover {cls}"><img src="covers/cover-{cid}.svg" alt="" width="{size or 96}" height="{size or 96}" loading="lazy"></span>'


def fmt_badge(t):
    hires = " hires" if t["bits"].startswith("24") else ""
    return f'<span class="fmt{hires}"><b>{t["fmt"]}</b><i></i>{t["short"]}</span>'


def wordmark(cls="logo"):
    return (f'<img class="{cls} th-d" src="../assets/kazzu-wordmark-dark.svg" alt="Kazzu" width="108" height="28">'
            f'<img class="{cls} th-l" src="../assets/kazzu-wordmark-light.svg" alt="Kazzu" width="108" height="28">')


def head(title, cid=NOW, body_cls="app dyn", screen=""):
    return f'''<!doctype html>
<html lang="pt-BR" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>{title} — Kazzu</title>
<meta name="color-scheme" content="dark light">
<link rel="icon" href="../assets/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="assets/tokens.css">
<link rel="stylesheet" href="assets/covers.css">
<link rel="stylesheet" href="assets/kazzu.css">
</head>
<body class="{body_cls} cv-{cid}" data-screen="{screen}" data-cover="{cid}">
{SPRITE}
'''


def tail():
    return '\n<script src="assets/app.js"></script>\n</body>\n</html>\n'


def ambient(cid=NOW):
    return f'<div class="ambient" aria-hidden="true"><img src="covers/cover-{cid}.svg" alt="" width="640" height="640" data-ambient></div>'


NAV = [("home", "home.html", "Início", "home"), ("explore", "explore.html", "Explorar", "explore"), ("library", "library.html", "Biblioteca", "library")]


def sidebar(active):
    links = "".join(
        f'<a class="side-link" href="{href}"{" aria-current=\"page\"" if key == active else ""}>{ic(icon)}<span>{label}</span></a>'
        for key, href, label, icon in NAV)
    s_cur = ' aria-current="page"' if active == "search" else ""
    st_cur = ' aria-current="page"' if active == "settings" else ""
    return f'''<aside class="sidebar glass" aria-label="Navegação principal">
  <a class="logo-link" href="home.html" aria-label="Kazzu, ir para Início">
    <span class="logo-rail"><img class="mark th-d" src="../assets/kazzu-mark-dark.svg" alt="" width="36" height="36"><img class="mark th-l" src="../assets/kazzu-mark-light.svg" alt="" width="36" height="36"></span>
    <span class="logo-wide">{wordmark()}</span>
  </a>
  <nav style="display:grid;gap:4px">{links}
    <div class="side-sep" role="presentation"></div>
    <a class="side-link side-search" href="search.html" aria-label="Buscar"{s_cur}>{ic("search")}<span>Buscar</span></a>
  </nav>
  <div class="side-spacer"></div>
  <a class="side-link" href="settings.html"{st_cur}>{ic("sliders")}<span>Configurações</span></a>
</aside>'''


def mini_html(cid=NOW):
    t = TRACKS[cid]
    return f'''<a class="mini glass" href="player.html" aria-label="Abrir player: {esc(t['song'])}, {esc(t['artist'])}">
    {cov(cid, size=48)}
    <span class="meta"><span class="t-callout ellipsis" style="font-weight:600">{esc(t['song'])}</span><span class="t-sub muted ellipsis">{esc(t['artist'])}</span></span>
    <button class="btn-ic" type="button" aria-label="Pausar" data-stop>{ic("pause", "ic-lg")}</button>
    <button class="btn-ic" type="button" aria-label="Próxima faixa" data-stop>{ic("next")}</button>
    <span class="prog" aria-hidden="true" style="--p:31%"><i></i></span>
  </a>'''


def dock(active):
    tabs = "".join(
        f'<a class="tab" href="{href}"{" aria-current=\"page\"" if key == active else ""}>{ic(icon)}<span>{label}</span></a>'
        for key, href, label, icon in NAV)
    cur = ' aria-current="page"' if active == "search" else ""
    return f'''<div class="dockwrap">
  {mini_html()}
  <nav class="dock glass strong" aria-label="Navegação principal">
    <div class="tabs">{tabs}</div>
    <span class="sep" aria-hidden="true"></span>
    <a class="search-btn" href="search.html" aria-label="Buscar"{cur}>{ic("search", "ic-lg")}</a>
  </nav>
</div>
<div class="nowbar glass" role="region" aria-label="Tocando agora">
  {cov(NOW, size=56)}
  <span class="meta"><span class="t-callout ellipsis" style="font-weight:600">{esc(TRACKS[NOW]['song'])}</span><span class="t-sub muted ellipsis">{esc(TRACKS[NOW]['artist'])}</span></span>
  <span class="mid"><div class="slider" role="slider" tabindex="0" aria-label="Progresso" aria-valuemin="0" aria-valuemax="201" aria-valuenow="62" aria-valuetext="1:02 de 3:21" style="--p:31%" data-kind="seek"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div><span class="t-mono"><span>1:02</span><span>3:21</span></span></span>
  <span class="btns"><button class="btn-ic" type="button" aria-label="Anterior">{ic("prev")}</button><button class="btn-ic" type="button" aria-label="Pausar">{ic("pause", "ic-lg")}</button><button class="btn-ic" type="button" aria-label="Próxima">{ic("next")}</button></span>
  <a class="btn-ic" href="queue.html" aria-label="Abrir fila">{ic("queue")}</a>
  <a class="btn-ic" href="player.html" aria-label="Abrir player em tela cheia">{ic("chevron-down", "")}</a>
</div>'''


def shell(title, active, body, cid=NOW, screen="", extra_head=""):
    return head(title, cid, screen=screen) + ambient(cid) + sidebar(active) + body + dock(active) + tail()


def row(cid, playing=False, show_album=True):
    t = TRACKS[cid]
    lead = f'<span class="lead">{ic("grip")}</span>' if False else ""
    sub = esc(t["artist"]) if t["src"] == "local" else f'{esc(t["artist"])}, {t["src"]}'
    now = f'<span class="nowbars" role="img" aria-label="Tocando agora"><i></i><i></i><i></i></span>' if playing else ""
    return f'''<li><a class="row{" playing" if playing else ""}" href="player.html">
      {cov(cid, size=96)}
      <span class="meta"><span class="title ellipsis">{esc(t['song'])}</span><span class="sub ellipsis">{sub}</span></span>
      <span class="album ellipsis">{esc(t['album'])}</span>
      <span class="end">{now}{fmt_badge(t)}<button class="btn-ic" type="button" aria-label="Mais opções para {esc(t['song'])}" data-stop>{ic("more")}</button></span>
    </a></li>'''


# ------------------------------------------------------------------ INÍCIO
def page_home():
    hero = [
        ("07", "Novo de Nina Sabiá", "Calor de Março", "Álbum, 12 faixas", True),
        ("06", "Mix para foco", "Linha de Fuga e mais", "Feito com o que você baixou", True),
        ("03", "Do seu acervo", "Redescobertos", "214 faixas paradas há um ano", True),
        ("08", "Direto do YT Music", "Vidro Líquido", "Kaito e Vê, 9 faixas", True),
    ]
    cards = "".join(f'''<a class="hero-card" href="player.html" aria-label="{esc(t)}, {esc(c)}">
      <img src="covers/cover-{cid}.svg" alt="" width="640" height="640">
      <span class="plate with-go"><span class="t-caption">{esc(cap)}</span><span class="t-title2">{esc(t)}</span><span class="t-sub">{esc(c)}</span></span>
      <span class="go" aria-hidden="true">{ic("play", "ic-lg")}</span>
    </a>''' for cid, cap, t, c, _ in hero)
    recents = "".join(row(c, playing=(c == "07")) for c in ["07", "08", "02", "05", "03", "04"])
    tiles = "".join(f'''<a class="tile" href="player.html">{cov(c, size=352)}<span><span class="title ellipsis" style="display:block">{esc(ALBUM[c])}</span><span class="sub ellipsis" style="display:block">{esc(TRACKS[c]['artist'])}</span></span></a>''' for c in ["01", "02", "06", "09", "10", "04"])
    body = f'''<main class="content" id="main">
  <header class="topbar mobile-only">
    <a class="logo-hit" href="home.html" aria-label="Kazzu, ir para Início">{wordmark()}</a>
    <a class="avatar" href="settings.html" aria-label="Conta e configurações">C</a>
  </header>
  <header class="page-intro"><h1 class="t-large">Início</h1><a class="avatar" href="settings.html" aria-label="Conta e configurações">C</a></header>

  <section class="section" style="margin-top:var(--sp-4)" aria-labelledby="h-dest">
    <div class="section-head"><h2 class="t-title2" id="h-dest">Destaques</h2>
      <div class="arrows"><button class="btn-ic" type="button" aria-label="Destaques anteriores" data-rail="-1" data-target="rail1">{ic("chevron-left")}</button><button class="btn-ic" type="button" aria-label="Próximos destaques" data-rail="1" data-target="rail1">{ic("chevron-right")}</button></div></div>
    <div class="rail" id="rail1" tabindex="0" role="region" aria-label="Destaques, role para os lados">{cards}</div>
  </section>

  <section class="section" aria-labelledby="h-rec">
    <div class="section-head"><h2 class="t-title2" id="h-rec">Recentes</h2><a class="btn-text" href="library.html">Ver tudo</a></div>
    <ul class="rows">{recents}</ul>
  </section>

  <section class="section" aria-labelledby="h-alb">
    <div class="section-head"><h2 class="t-title2" id="h-alb">Álbuns que você ouve</h2><a class="btn-text" href="library.html">Ver tudo</a></div>
    <div class="tiles">{tiles}</div>
  </section>
</main>'''
    return shell("Início", "home", body, screen="home")


# ------------------------------------------------------------------ EXPLORAR
def page_explore():
    moods = [("Foco", "06"), ("Madrugada", "01"), ("Energia", "02"), ("Verão", "07"), ("Calmaria", "09"), ("Estrada", "04")]
    tiles = "".join(f'''<a class="col-card mood" href="search.html" style="padding:0;overflow:hidden;aspect-ratio:1.6;border-radius:20px;isolation:isolate;background:var(--surface-2)">
        <img src="covers/cover-{c}.svg" alt="" style="position:absolute;inset:0;width:100%;height:100%;object-fit:cover;z-index:-2;border-radius:inherit">
        <span class="t-title2" style="position:absolute;left:14px;bottom:12px;color:#fff;text-shadow:0 1px 8px rgba(0,0,0,.55)">{n}</span></a>''' for n, c in moods)
    new = "".join(row(c) for c in ["08", "01", "09"])
    body = f'''<main class="content" id="main">
  <header class="topbar mobile-only"><a class="logo-hit" href="home.html" aria-label="Kazzu, ir para Início">{wordmark()}</a><a class="avatar" href="settings.html" aria-label="Conta e configurações">C</a></header>
  <header class="page-intro"><h1 class="t-large">Explorar</h1></header>
  <div class="screen-title mobile-only" style="padding-top:0"><h1 class="t-large">Explorar</h1></div>
  <section class="section" style="margin-top:var(--sp-2)" aria-labelledby="h-h"><div class="section-head"><h2 class="t-title2" id="h-h">Humores</h2></div>
    <div class="collection-grid" style="grid-template-columns:repeat(2,minmax(0,1fr))">{tiles}</div></section>
  <section class="section" aria-labelledby="h-n"><div class="section-head"><h2 class="t-title2" id="h-n">Novidades dos seus provedores</h2></div>
    <div class="chips" style="margin-bottom:var(--sp-3)"><button class="chip" aria-pressed="true" type="button">Todos</button><button class="chip" aria-pressed="false" type="button">Local</button><button class="chip" aria-pressed="false" type="button">YT Music</button><button class="chip" aria-pressed="false" type="button">Spotify</button></div>
    <ul class="rows">{new}</ul></section>
</main>'''
    return shell("Explorar", "explore", body, screen="explore")


# ------------------------------------------------------------------ BIBLIOTECA
def mosaic(ids):
    return '<span class="mosaic">' + "".join(f'<img src="covers/cover-{c}.svg" alt="" width="176" height="176" loading="lazy">' for c in ids) + "</span>"


def page_library():
    cards = f'''
    <a class="col-card glass" href="library.html#baixados"><span class="col-art dl">{ic("download", "ic-xl")}</span><span><span class="title" style="display:block">Baixados</span><span class="sub" style="display:block">214 músicas</span></span></a>
    <a class="col-card glass" href="library.html#curtidos"><span class="col-art liked">{ic("heart-fill", "ic-xl")}</span><span><span class="title" style="display:block">Curtidos</span><span class="sub" style="display:block">87 músicas</span></span></a>
    <a class="col-card glass" href="library.html#albuns"><span class="col-art fan"><img src="covers/cover-01.svg" alt="" width="176" height="176"><img src="covers/cover-06.svg" alt="" width="176" height="176"><img src="covers/cover-07.svg" alt="" width="176" height="176"></span><span><span class="title" style="display:block">Álbuns</span><span class="sub" style="display:block">46 álbuns</span></span></a>
    <a class="col-card glass" href="library.html#playlists"><span class="col-art" style="padding:0">{mosaic(["02", "05", "08", "10"])}</span><span><span class="title" style="display:block">Playlists</span><span class="sub" style="display:block">9 playlists</span></span></a>'''
    pls = [("Estrada à noite", ["01", "06", "08", "04"], "42 músicas"), ("Foco profundo", ["06", "03", "09", "01"], "68 músicas"),
           ("Domingo de faxina", ["07", "02", "10", "05"], "35 músicas"), ("Treino", ["04", "07", "02", "08"], "27 músicas"),
           ("Madrugada", ["01", "09", "03", "06"], "51 músicas"), ("Café da manhã", ["02", "10", "07", "05"], "19 músicas")]
    pl = "".join(f'<a class="pl-card" href="library.html#pl">{mosaic(m)}<span><span class="title ellipsis" style="display:block">{n}</span><span class="sub" style="display:block">{c}</span></span></a>' for n, m, c in pls)
    body = f'''<main class="content" id="main">
  <header class="topbar mobile-only"><a class="logo-hit" href="home.html" aria-label="Kazzu, ir para Início">{wordmark()}</a><a class="avatar" href="settings.html" aria-label="Conta e configurações">C</a></header>
  <div class="screen-title"><h1 class="t-large">Biblioteca</h1><button class="btn-pill btn-primary" type="button">{ic("playlist-add", "ic-sm")}Criar playlist</button></div>
  <section aria-label="Coleções"><div class="collection-grid">{cards}</div></section>
  <section class="section" aria-labelledby="h-pl"><div class="section-head"><h2 class="t-title2" id="h-pl">Suas playlists</h2><a class="btn-text" href="library.html#playlists">Ver tudo</a></div>
    <div class="pl-grid">{pl}</div></section>
</main>'''
    return shell("Biblioteca", "library", body, screen="library")


# ------------------------------------------------------------------ PLAYER
def page_player():
    t = TRACKS[NOW]
    covers_js = json.dumps({k: dict(song=v["song"], artist=v["artist"], fmt=v["fmt"], bits=v["bits"], hires=v["bits"].startswith("24")) for k, v in TRACKS.items()}, ensure_ascii=False)
    sw = "".join(f'<button type="button" data-cover-pick="{k}" aria-label="Capa {k}: {esc(v["album"])}"><img src="covers/cover-{k}.svg" alt="" width="22" height="22"></button>' for k, v in TRACKS.items())
    h = head("Player", NOW, body_cls="player dyn", screen="player").replace('<body class="player dyn', '<body class="player dyn')
    h += f'''<div class="pl-bg" aria-hidden="true"><img src="covers/cover-{NOW}.svg" alt="" width="640" height="640" data-blur></div>
<div class="pl">
  <header class="pl-top">
    <a class="btn-ic" href="home.html" aria-label="Minimizar player">{ic("chevron-down", "ic-lg")}</a>
    <div class="src">Tocando de Recentes<span>Fones do estúdio</span></div>
    <button class="btn-ic" type="button" aria-label="Mais opções">{ic("more")}</button>
  </header>
  <main class="pl-main" id="main">
    <div class="pl-art"><img src="covers/cover-{NOW}.svg" alt="Capa de {esc(t['album'])}, de {esc(t['artist'])}" width="640" height="640" data-art></div>
    <section class="pl-ctl" aria-label="Controles de reprodução">
      <div class="pl-meta">
        <div class="txt"><h1 class="ellipsis" data-song>{esc(t['song'])}</h1><p class="artist ellipsis" data-artist>{esc(t['artist'])}</p></div>
        <button class="btn-ic" type="button" aria-label="Curtir" aria-pressed="true" data-toggle>{ic("heart-fill")}</button>
        <button class="btn-ic" type="button" aria-label="Mais opções da faixa">{ic("more")}</button>
      </div>
      <div>
        <div class="slider accent" role="slider" tabindex="0" aria-label="Progresso da faixa" aria-valuemin="0" aria-valuemax="201" aria-valuenow="62" aria-valuetext="1:02 de 3:21" style="--p:31%" data-kind="seek"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div>
        <div class="pl-times t-mono"><span data-cur>1:02</span><span class="fmt hires" data-fmt-badge title="Qualidade original"><b data-fmt>{t['fmt']}</b><i></i><span data-bits>{t['bits']}</span></span><span class="r" data-left>−2:19</span></div>
      </div>
      <div class="transport">
        <button class="btn-ic sm" type="button" aria-label="Aleatório" aria-pressed="false" data-toggle>{ic("shuffle")}</button>
        <button class="btn-ic" type="button" aria-label="Anterior">{ic("prev")}</button>
        <button class="play-main" type="button" aria-label="Pausar" data-playtoggle>{ic("pause")}</button>
        <button class="btn-ic" type="button" aria-label="Próxima">{ic("next")}</button>
        <button class="btn-ic sm" type="button" aria-label="Repetir" aria-pressed="false" data-toggle>{ic("repeat")}</button>
      </div>
      <div class="vol">{ic("vol-low")}<div class="slider" role="slider" tabindex="0" aria-label="Volume" aria-valuemin="0" aria-valuemax="100" aria-valuenow="68" aria-valuetext="68%" style="--p:68%" data-kind="vol"><div class="track"><div class="fill"></div><div class="thumb"></div></div></div>{ic("vol-high")}</div>
      <div class="pl-actions">
        <button class="pl-action" type="button">{ic("lyrics", "ic-sm")}Letra</button>
        <button class="pl-action" type="button" aria-label="Saída de áudio: Fones do estúdio">{ic("output", "ic-sm")}<span class="ellipsis">Fones do estúdio</span></button>
        <a class="pl-action" href="queue.html">{ic("queue", "ic-sm")}Fila</a>
      </div>
      <p class="pl-note">{ic("check", "ic-sm")}Original, sem equalizador</p>
    </section>
  </main>
  <div class="demo-swatches" role="group" aria-label="Protótipo: trocar a capa para ver o acento dinâmico">{sw}</div>
</div>
<script>window.KAZZU_TRACKS = {covers_js};</script>
'''
    return h + tail()


# ------------------------------------------------------------------ FILA
def page_queue():
    nxt = ["08", "02", "05", "03", "04", "09"]
    items = "".join(f'''<li><div class="row">{f'<span class="drag" aria-hidden="true">{ic("grip")}</span>'}{cov(c, size=96)}
      <span class="meta"><span class="title ellipsis">{esc(TRACKS[c]['song'])}</span><span class="sub ellipsis">{esc(TRACKS[c]['artist'])}</span></span>
      <span class="end">{fmt_badge(TRACKS[c])}<span class="t-mono muted" style="width:40px;text-align:right">{TRACKS[c]['dur']}</span><button class="btn-ic" type="button" aria-label="Remover {esc(TRACKS[c]['song'])} da fila">{ic("close", "ic-sm")}</button></span></div></li>''' for c in nxt)
    t = TRACKS[NOW]
    body = f'''<main class="content" id="main" style="max-width:860px">
  <div class="screen-title"><span style="display:flex;align-items:center;gap:var(--sp-2)"><a class="btn-ic" href="player.html" aria-label="Voltar ao player" style="margin-left:-10px">{ic("back")}</a><h1 class="t-large">Fila</h1></span>
    <button class="btn-pill btn-quiet" type="button">{ic("trash", "ic-sm")}Limpar</button></div>
  <section aria-labelledby="h-now"><h2 class="t-headline muted2" id="h-now" style="margin-bottom:var(--sp-3);font-weight:600">Tocando agora</h2>
    <div class="q-now">{cov(NOW, size=128)}<span style="flex:1;min-width:0;display:grid"><span class="title ellipsis">{esc(t['song'])}</span><span class="t-callout muted2 ellipsis">{esc(t['artist'])}</span></span>{fmt_badge(t)}<span class="nowbars" role="img" aria-label="Tocando agora"><i></i><i></i><i></i></span></div></section>
  <section class="section" aria-labelledby="h-nx"><div class="section-head"><h2 class="t-headline muted2" id="h-nx" style="font-weight:600">A seguir, 6 músicas, 24 min</h2></div>
    <ul class="rows">{items}</ul></section>
  <section class="section"><div class="group"><div class="set-row"><span class="txt"><span class="t-headline">Reprodução automática</span><span class="t-sub">Quando a fila acabar, o Kazzu toca músicas parecidas.</span></span><button class="switch" role="switch" aria-checked="true" aria-label="Reprodução automática" type="button"></button></div></div></section>
</main>'''
    return shell("Fila", "", body, screen="queue")


# ------------------------------------------------------------------ BUSCA
def page_search():
    res = "".join(row(c) for c in ["01", "08", "05"])
    tiles = "".join(f'<a class="tile" href="player.html">{cov(c, size=352)}<span><span class="title ellipsis" style="display:block">{esc(ALBUM[c])}</span><span class="sub ellipsis" style="display:block">{esc(TRACKS[c]["artist"])}</span></span></a>' for c in ["01", "06", "10"])
    recent = "".join(f'<li><a class="row" href="search.html" style="min-height:52px">{ic("clock", "")}<span class="meta"><span class="title ellipsis" style="font-weight:400">{q}</span></span><button class="btn-ic" type="button" aria-label="Remover {q} do histórico" data-stop>{ic("close", "ic-sm")}</button></a></li>' for q in ["aurora vento", "jazz de madrugada", "nina sabiá ao vivo", "flac 24 bits"])
    body = f'''<main class="content" id="main">
  <div class="screen-title mobile-only" style="padding-bottom:var(--sp-3)"><h1 class="t-large">Buscar</h1></div>
  <header class="page-intro" style="margin-bottom:var(--sp-4)"><h1 class="t-large">Buscar</h1></header>
  <form role="search" onsubmit="return false"><label class="field">{ic("search")}<span class="sr-only">Buscar músicas, álbuns e artistas</span><input type="search" placeholder="Músicas, álbuns, artistas" value="aurora" autocomplete="off" data-search-input>
    <button class="btn-ic" type="button" aria-label="Limpar busca" data-clear>{ic("close", "ic-sm")}</button>
    <button class="btn-ic" type="button" aria-label="Reconhecer música pelo som ambiente" style="background:var(--accent-soft);color:var(--accent)">{ic("wave")}</button></label></form>
  <div class="chips" style="margin-top:var(--sp-4)" role="group" aria-label="Filtrar por tipo"><button class="chip" aria-pressed="true" type="button">Tudo</button><button class="chip" aria-pressed="false" type="button">Músicas</button><button class="chip" aria-pressed="false" type="button">Álbuns</button><button class="chip" aria-pressed="false" type="button">Artistas</button><button class="chip" aria-pressed="false" type="button">Playlists</button></div>
  <div class="chips" style="margin-top:var(--sp-2)" role="group" aria-label="Fontes"><button class="chip" aria-pressed="true" type="button">Local</button><button class="chip" aria-pressed="true" type="button">YT Music</button><button class="chip" aria-pressed="false" type="button">Spotify</button></div>

  <div data-state="results">
    <section class="section" aria-labelledby="h-m"><div class="section-head"><h2 class="t-title2" id="h-m">Músicas</h2></div><ul class="rows">{res}</ul></section>
    <section class="section" aria-labelledby="h-a"><div class="section-head"><h2 class="t-title2" id="h-a">Álbuns</h2></div><div class="tiles">{tiles}</div></section>
  </div>
  <div data-state="idle" hidden>
    <section class="section" aria-labelledby="h-r"><div class="section-head"><h2 class="t-title2" id="h-r">Buscas recentes</h2><button class="btn-text" type="button">Limpar</button></div><ul class="rows">{recent}</ul></section>
  </div>
  <div data-state="empty" hidden>
    <div class="empty"><span class="art">{ic("search", "ic-lg")}</span><h2 class="t-title2">Nada para “aurorx”</h2><p>Confira a grafia ou tente buscar só o nome do artista. A busca também olha seus arquivos locais.</p><button class="btn-pill btn-quiet" type="button">Limpar filtros</button></div>
  </div>
</main>'''
    return shell("Buscar", "search", body, screen="search")


# ------------------------------------------------------------------ CONFIGURAÇÕES
def set_row(icon, title, sub="", ctl=""):
    return f'<div class="set-row"><span class="lead">{ic(icon, "ic-sm")}</span><span class="txt"><span class="t-body" style="font-weight:600">{title}</span>{f"<span class=\"t-sub\">{sub}</span>" if sub else ""}</span>{ctl}</div>'


def switch(label, on=False, extra=""):
    return f'<button class="switch" role="switch" aria-checked="{"true" if on else "false"}" aria-label="{label}" type="button" {extra}></button>'


def seg(label, opts, sel, kind):
    b = "".join(f'<button type="button" role="radio" aria-checked="{"true" if i == sel else "false"}" data-{kind}="{o[1]}">{o[0]}</button>' for i, o in enumerate(opts))
    return f'<div class="seg" role="radiogroup" aria-label="{label}">{b}</div>'


def page_settings():
    bands = "".join(f'<div style="display:grid;justify-items:center;gap:6px"><div class="slider light" role="slider" aria-disabled="true" aria-label="Banda {f}" aria-valuemin="-12" aria-valuemax="12" aria-valuenow="0" style="--p:50%;height:84px;width:28px;flex-direction:column"><div class="track" style="width:6px;height:100%"><div class="fill" style="width:100%;height:50%;top:auto;bottom:0;opacity:0"></div></div><div class="thumb" style="opacity:1;left:50%;top:50%;background:var(--ink-3)"></div></div><span class="t-mono muted" style="font-size:12px">{f}</span></div>' for f in ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"])
    body = f'''<main class="content" id="main" style="max-width:760px">
  <header class="topbar mobile-only"><a class="logo-hit" href="home.html" aria-label="Kazzu, ir para Início">{wordmark()}</a><a class="avatar" href="settings.html" aria-label="Conta">C</a></header>
  <div class="screen-title"><h1 class="t-large">Configurações</h1></div>

  <h2 class="t-title2 group-title" style="margin-top:var(--sp-3)">Provedores</h2>
  <div class="group">
    {set_row("folder", "Biblioteca local", "3 pastas, 1.284 músicas", f'<span class="tag">Ativa</span>')}
    {set_row("disc", "YT Music", "Conectado, conta cuscusdoce", '<button class="btn-text" type="button">Sair</button>')}
    {set_row("disc", "Spotify", "Toque para conectar sua conta", '<button class="btn-pill btn-quiet" type="button">Conectar</button>')}
    {set_row("disc", "Tidal e Qobuz", "Chegam em uma próxima versão", '<span class="tag">Em breve</span>')}
  </div>

  <h2 class="t-title2 group-title">Áudio</h2>
  <div class="group">
    {set_row("output", "Saída", "Padrão do sistema: Fones do estúdio", f'<button class="btn-text" type="button">Trocar</button>')}
    {set_row("check", "Qualidade original", "Sem normalizar e sem reamostrar. Só convertemos a taxa se o dispositivo de saída exigir.", '<span class="tag">Sempre</span>')}
    {set_row("eq", "Equalizador", "Desligado: reprodução bit-perfect.", switch("Equalizador", False))}
    <div class="set-row" style="display:block;padding:var(--sp-4) 0 var(--sp-5)" aria-label="Prévia do equalizador, indisponível">
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:var(--sp-3)"><span class="t-sub muted">Equalizador de 10 bandas, com curvas por fone</span><span class="tag">Em breve</span></div>
      <div style="display:flex;justify-content:space-between;opacity:.55;pointer-events:none">{bands}</div>
    </div>
  </div>

  <h2 class="t-title2 group-title">Aparência</h2>
  <div class="group">
    {set_row("moon", "Tema", "", seg("Tema", [("Auto", "auto"), ("Claro", "light"), ("Escuro", "dark")], 2, "theme"))}
    {set_row("sliders", "Cor de destaque", "A cor vem da capa da faixa que toca.", seg("Cor de destaque", [("Da capa", "dyn"), ("Âmbar", "brand")], 0, "accent"))}
    {set_row("info", "Reduzir transparência", "Troca o vidro desfocado por painéis sólidos. Ajuda em telas lentas.", switch("Reduzir transparência", False, 'data-setting="reduce-transparency"'))}
    {set_row("info", "Reduzir movimento", "Remove transições e animações.", switch("Reduzir movimento", False, 'data-setting="reduce-motion"'))}
  </div>

  <h2 class="t-title2 group-title">Privacidade</h2>
  <div class="group">
    {set_row("shield", "Sem telemetria", "O Kazzu só se conecta quando você pede algo. Nenhum dado de uso sai do seu aparelho, e esta opção não pode ser desligada.", '<button class="switch locked" role="switch" aria-checked="true" aria-disabled="true" aria-label="Sem telemetria, sempre ativado" type="button"></button>')}
  </div>

  <h2 class="t-title2 group-title">Sobre</h2>
  <div class="group">
    {set_row("info", "Kazzu 0.1.0", "Código aberto, licença GPL-3.0.", '<button class="btn-text" type="button">Licenças</button>')}
  </div>
</main>'''
    return shell("Configurações", "settings", body, screen="settings")


# ------------------------------------------------------------------ ESTADOS VAZIOS
def page_states():
    def card(icon, title, text, btn):
        return f'<div class="group" style="padding:var(--sp-5) var(--sp-4)"><div class="empty" style="padding:var(--sp-6) 0"><span class="art">{ic(icon, "ic-lg")}</span><h2 class="t-title2">{title}</h2><p>{text}</p><button class="btn-pill btn-primary" type="button">{btn}</button></div></div>'
    skel = "".join('<div class="row" style="pointer-events:none"><span class="cover" style="width:48px;background:var(--surface-3)"></span><span class="meta" style="gap:6px"><span style="height:14px;width:55%;border-radius:7px;background:var(--surface-3)"></span><span style="height:12px;width:34%;border-radius:6px;background:var(--surface-2)"></span></span></div>' for _ in range(3))
    body = f'''<main class="content" id="main" style="max-width:1100px">
  <div class="screen-title"><h1 class="t-large">Estados vazios</h1></div>
  <p class="t-callout muted" style="max-width:56ch;margin-bottom:var(--sp-6)">Cada estado diz o que está vazio, por que importa e oferece uma ação. Sem ilustração decorativa: só um ícone do sistema.</p>
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:var(--sp-4)">
    {card("library", "Sua biblioteca está vazia", "Adicione uma pasta de músicas ou conecte um provedor. O Kazzu encontra capas e tags sozinho.", "Adicionar pasta")}
    {card("search", "Nada para “aurorx”", "Confira a grafia ou busque só o nome do artista.", "Limpar filtros")}
    {card("queue", "A fila está vazia", "Toque em uma música e ela aparece aqui, junto com as próximas.", "Ir para o Início")}
    {card("cloud-off", "Sem conexão", "Suas músicas baixadas continuam tocando. O resto volta quando a rede voltar.", "Ver baixados")}
  </div>
  <h2 class="t-title2 group-title">Carregando</h2>
  <div class="group" aria-busy="true" aria-label="Carregando lista">{skel}</div>
  <p class="t-sub muted" style="margin-top:var(--sp-3)">Placeholders estáticos: o spinner só aparece depois de 300 ms e fica visível por pelo menos 300 ms.</p>
</main>'''
    return shell("Estados vazios", "", body, screen="states")


# ------------------------------------------------------------------ HUB
def page_hub():
    screens = [("home", "Início"), ("player", "Player"), ("library", "Biblioteca"), ("queue", "Fila"), ("search", "Busca"), ("settings", "Configurações"), ("explore", "Explorar"), ("states", "Estados vazios")]
    cards = ""
    for key, label in screens:
        cards += f'''<section class="hub-screen" aria-labelledby="h-{key}">
  <div class="hub-bar"><h2 id="h-{key}">{label}</h2><span><a href="{key}.html">Abrir</a></span></div>
  <div class="hub-frames">
    <figure class="f-mobile"><div class="frame m"><iframe src="{key}.html" title="{label}, 390 px" loading="lazy" data-base="{key}.html"></iframe></div><figcaption>390 × 844</figcaption></figure>
    <figure class="f-desk"><div class="frame d"><iframe src="{key}.html" title="{label}, 1280 px" loading="lazy" data-base="{key}.html"></iframe></div><figcaption>1280 × 800</figcaption></figure>
  </div>
</section>'''
    return f'''<!doctype html>
<html lang="pt-BR" data-theme="dark">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Protótipos — Kazzu</title>
<link rel="icon" href="../assets/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="assets/tokens.css"><link rel="stylesheet" href="assets/covers.css"><link rel="stylesheet" href="assets/kazzu.css">
<style>
 body.hub{{padding:var(--sp-6) var(--sp-5) var(--sp-10);max-width:1500px;margin:0 auto}}
 .hub-top{{display:flex;flex-wrap:wrap;gap:var(--sp-4);align-items:center;justify-content:space-between;margin-bottom:var(--sp-7)}}
 .hub-top p{{max-width:62ch;color:var(--ink-3);margin-top:var(--sp-2)}}
 .hub-ctl{{display:flex;flex-wrap:wrap;gap:var(--sp-3);align-items:center}}
 .hub-screen{{margin-bottom:var(--sp-9)}}
 .hub-bar{{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:var(--sp-3)}} .hub-bar h2{{font:700 24px/30px var(--font-sans)}} .hub-bar a{{color:var(--accent);font-weight:600}}
 .hub-frames{{display:flex;gap:var(--sp-6);flex-wrap:wrap;align-items:flex-start}}
 figure{{margin:0}} figcaption{{margin-top:var(--sp-2);color:var(--ink-3);font:400 12px/16px var(--font-mono)}}
 .frame{{position:relative;overflow:hidden;border-radius:28px;border:1px solid var(--line-strong);background:var(--bg)}}
 .frame iframe{{position:absolute;left:0;top:0;border:0;transform-origin:0 0}}
 .frame.m{{width:312px;height:675px}} .frame.m iframe{{width:390px;height:844px;transform:scale(.8)}}
 .frame.d{{width:960px;height:600px;border-radius:16px}} .frame.d iframe{{width:1280px;height:800px;transform:scale(.75)}}
 @media (max-width:1450px){{ .frame.d{{width:720px;height:450px}} .frame.d iframe{{transform:scale(.5625)}} }}
 @media (max-width:760px){{ .f-desk{{display:none}} body{{padding-inline:16px}} }}
</style>
</head>
<body class="hub">
{SPRITE}
<header class="hub-top">
  <div>{wordmark()}<p style="margin-top:var(--sp-3)">Protótipos estáticos da etapa de design. Todo vidro usa fundo translúcido, blur de fundo, borda de 1 px, brilho no topo e sombra, o mesmo que o Slint 1.19 renderiza com <code>backdrop-blur</code>.</p></div>
  <div class="hub-ctl">
    <div class="seg" role="radiogroup" aria-label="Tema"><button type="button" role="radio" aria-checked="true" data-hub-theme="dark">Escuro</button><button type="button" role="radio" aria-checked="false" data-hub-theme="light">Claro</button></div>
    <div class="seg" role="radiogroup" aria-label="Transparência"><button type="button" role="radio" aria-checked="true" data-hub-rt="0">Vidro</button><button type="button" role="radio" aria-checked="false" data-hub-rt="1">Sem transparência</button></div>
  </div>
</header>
{cards}
<script src="assets/app.js"></script>
<script>
(function(){{
  var theme='dark', rt='0';
  function push(){{ document.querySelectorAll('iframe[data-base]').forEach(function(f){{ var h='#theme='+theme+(rt==='1'?'&rt=1':''); var w=f.contentWindow; try{{ w.location.hash=h; }}catch(e){{ f.src=f.dataset.base+h; }} }}); }}
  function init(){{ document.querySelectorAll('iframe[data-base]').forEach(function(f){{ f.src=f.dataset.base+'#theme='+theme+(rt==='1'?'&rt=1':''); }}); }}
  document.querySelectorAll('[data-hub-theme]').forEach(function(b){{ b.addEventListener('click',function(){{ theme=b.dataset.hubTheme; document.documentElement.dataset.theme=theme; document.querySelectorAll('[data-hub-theme]').forEach(function(x){{x.setAttribute('aria-checked',x===b)}}); push(); }}); }});
  document.querySelectorAll('[data-hub-rt]').forEach(function(b){{ b.addEventListener('click',function(){{ rt=b.dataset.hubRt; document.documentElement.classList.toggle('reduce-transparency',rt==='1'); document.querySelectorAll('[data-hub-rt]').forEach(function(x){{x.setAttribute('aria-checked',x===b)}}); push(); }}); }});
  init();
}})();
</script>
</body>
</html>
'''


PAGES = {"index": page_hub, "home": page_home, "explore": page_explore, "player": page_player, "library": page_library,
         "queue": page_queue, "search": page_search, "settings": page_settings, "states": page_states}
for name, fn in PAGES.items():
    (P / f"{name}.html").write_text(fn(), encoding="utf-8")
    print("escrito", f"design/prototypes/{name}.html")
