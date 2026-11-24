#!/usr/bin/env python3
"""Gera os protótipos HTML da v4 (design/v4/prototypes/*.html).

Uma árvore por tela, a mesma para 360, 390, 768 e 1280: o CSS (assets/kazzu.css) é quem adapta.
Ícones entram inline (design/v4/assets/icons). Capas e dados fictícios vêm de assets/covers/covers.json.
Opções pela URL: #theme=dark|light  #rt=1 (reduzir transparência)  #rm=1 (reduzir movimento)  #scroll=N  #c=NN (capa do player/álbum)
Uso: python3 design/v4/tools/build_prototypes.py
"""
import html
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "prototypes"
ICONS = ROOT / "assets" / "icons"
COVERS = {c["id"]: c for c in json.loads((ROOT / "assets" / "covers" / "covers.json").read_text(encoding="utf-8"))}
e = html.escape


def ic(name, cls=""):
    svg = (ICONS / f"{name}.svg").read_text(encoding="utf-8").strip()
    svg = re.sub(r'\s(width|height)="[^"]+"', "", svg, count=2)
    svg = svg.replace('xmlns="http://www.w3.org/2000/svg" ', "")
    return svg.replace("<svg ", f'<svg class="ic{(" " + cls) if cls else ""}" aria-hidden="true" focusable="false" ', 1)


def cov(i, size=""):
    return f"../assets/covers/{i}{size}.jpg"


def img(i, size="", alt="", cls="", lazy=True, w=640):
    a = f' class="{cls}"' if cls else ""
    load = ' decoding="async"' if lazy else ' fetchpriority="high"'
    return f'<img{a} src="{cov(i, size)}" width="{w}" height="{w}" alt="{e(alt)}"{load}>'


WORDMARK = ('<svg viewBox="-1 0 153 40" fill="none" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="Kazzu">'
            '<path d="M3 2.5V37.5"/><path class="k2" d="M23 16.5 11 27 23 37.5"/>'
            '<g transform="translate(33 0)"><circle cx="13" cy="27" r="10.5"/><path d="M23.5 16.5V37.5"/></g>'
            '<path d="M70 16.5H88L70 37.5H88"/><path d="M100 16.5H118L100 37.5H118"/>'
            '<g transform="translate(126 0)"><path d="M3 16.5V28A9.5 9.5 0 0 0 22 28V16.5"/><path d="M22 16.5V37.5"/></g></svg>')
MARK = ('<svg viewBox="-1 0 27 40" fill="none" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">'
        '<path d="M3 2.5V37.5"/><path class="k2" d="M23 16.5 11 27 23 37.5"/></svg>')

NOW = {"cover": "05", "title": "Brasa Viva", "artist": "Os Tangarás", "album": "Pimenta"}

TABS = [("home", "Início", "home"), ("explore", "Explorar", "explore"), ("library", "Biblioteca", "library")]


def toggle(name_line, name_fill, label, pressed=False, cls="", extra=""):
    return (f'<button type="button" class="iconbtn tg {cls}" aria-label="{e(label)}" aria-pressed="{"true" if pressed else "false"}"{extra}>'
            f'{ic(name_line, "line")}{ic(name_fill, "fill")}</button>')


def dock(active, now=True):
    mini = ""
    if now:
        c = NOW
        mini = (f'<div class="mini glass" role="group" aria-label="Tocando agora" style="--dim:{COVERS[c["cover"]]["mini"]}">{img(c["cover"], "-blur", "", "tint", w=96)}{img(c["cover"], "-s", "", "art", w=160)}'
                f'<a class="open" href="player.html" aria-label="Abrir o player: {e(c["title"])}, {e(c["artist"])}"><b class="one">{e(c["title"])}</b><span class="one">{e(c["artist"])}</span></a>'
                f'<button type="button" class="iconbtn pp" aria-label="Pausar">{ic("pause", "i-pause")}{ic("play", "i-play")}</button>'
                f'<button type="button" class="iconbtn" aria-label="Próxima">{ic("next")}</button><i class="prog" aria-hidden="true"></i></div>')
    tabs = "".join(
        f'<a class="tab" href="{k}.html"{" aria-current=\"page\"" if k == active else ""}>{ic(i, "line")}{ic(i + "-fill", "fill")}<span>{t}</span></a>' for k, t, i in TABS)
    orb = f'<a class="orb glass" href="search.html" aria-label="Buscar"{" aria-current=\"page\"" if active == "search" else ""}>{ic("search")}</a>'
    return f'<div class="dock">{mini}<div class="dockrow"><nav class="tabbar glass" aria-label="Principal">{tabs}</nav>{orb}</div></div>'


PLAYLISTS = [("Estrada", "42 músicas", ["08", "17", "18", "02"]), ("Domingo de manhã", "18 músicas", ["16", "22", "14", "20"]),
             ("Para cozinhar", "27 músicas", ["05", "15", "11", "06"]), ("Foco", "61 músicas", ["21", "07", "03", "12"])]


def side(active):
    nav = "".join(f'<a class="nv" href="{k}.html"{" aria-current=\"page\"" if k == active else ""}>{ic(i, "line")}{ic(i + "-fill", "fill")}<span>{t}</span></a>' for k, t, i in TABS)
    nav += f'<a class="nv" href="search.html"{" aria-current=\"page\"" if active == "search" else ""}>{ic("search", "line")}{ic("search", "fill")}<span>Buscar</span></a>'
    pls = "".join(f'<a class="nv" href="playlist.html">{img(c[0], "-s", "", w=160)}<span class="one">{e(n)}</span></a>' for n, _, c in PLAYLISTS)
    return (f'<aside class="side" aria-label="Navegação"><a class="logo" href="home.html" aria-label="Kazzu, ir para o Início">{WORDMARK}</a>'
            f'<nav aria-label="Principal, barra lateral">{nav}</nav><div class="pl"><h2>Suas playlists</h2>{pls}</div></aside>')


def nowbar():
    c = NOW
    return (f'<div class="nowbar glass" role="group" aria-label="Tocando agora, barra">'
            f'<a class="np" href="player.html" aria-label="Abrir o player">{img(c["cover"], "-s", "", w=160)}<div><b class="one">{e(c["title"])}</b><span class="one">{e(c["artist"])}</span></div></a>'
            f'<div class="ctl"><button type="button" class="iconbtn" aria-label="Anterior">{ic("prev")}</button><button type="button" class="iconbtn play pp" aria-label="Pausar">{ic("pause", "i-pause")}{ic("play", "i-play")}</button><button type="button" class="iconbtn" aria-label="Próxima">{ic("next")}</button></div>'
            f'<div class="end">{toggle("heart", "heart-fill", "Curtir")}<a class="iconbtn" href="queue.html" aria-label="Fila">{ic("queue")}</a></div></div>')


def page(name, title, desc, body, cls="", active=None, now=True, chrome=True, mode_note=""):
    extra = ""
    if chrome:
        extra = side(active) + dock(active, now) + (nowbar() if now else "")
    classes = " ".join(x for x in [cls, "has-now" if (now and chrome) else "", "playing"] if x)
    doc = f"""<!doctype html>
<html lang="pt-BR" data-type="software" data-mode="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>{e(title)}</title>
<meta name="description" content="{e(desc)}">
<meta name="page-type" content="software">
<meta name="site-kind" content="software">
<meta name="theme-color" content="#07080a">
<link rel="icon" href="../assets/kazzu-icon.svg">
<link rel="stylesheet" href="assets/kazzu.css">
<script>document.documentElement.classList.add('js');(function(){{var h=new URLSearchParams(location.hash.slice(1)),d=document.documentElement;if(h.get('theme'))d.dataset.mode=h.get('theme');if(h.get('rt')==='1')d.dataset.rt='';if(h.get('rm')==='1')d.dataset.rm='';}})();</script>
</head>
<body class="{classes}">
<a class="skip" href="#conteudo">Pular para o conteúdo</a>
{body}
{extra}
<script src="assets/covers.js"></script>
<script src="assets/app.js"></script>
</body>
</html>
"""
    (OUT / f"{name}.html").write_text(doc, encoding="utf-8")


def top_logo():
    return (f'<header class="top"><a class="logo" href="home.html" aria-label="Kazzu, ir para o Início">{WORDMARK}</a>'
            f'<a class="iconbtn iconbtn--tonal" href="settings.html" aria-label="Ajustes">{ic("gear")}</a></header>'
            f'<div class="topbar topbar--mark glass" aria-hidden="true">{MARK}</div>')


def top_title(t, right=""):
    r = right or f'<a class="iconbtn iconbtn--tonal" href="settings.html" aria-label="Ajustes">{ic("gear")}</a>'
    return (f'<header class="top"><a class="logo" href="home.html" aria-label="Kazzu, ir para o Início">{WORDMARK}</a>{r}</header>'
            f'<div class="topbar glass" aria-hidden="true">{e(t)}</div>')


def shelf(title, inner, more="", hid=""):
    m = f'<a class="link" href="{more}" aria-label="Ver tudo: {e(title)}">Ver tudo</a>' if more else ""
    return f'<section class="shelf" aria-labelledby="{hid}"><header><h2 id="{hid}">{e(title)}</h2>{m}</header>{inner}</section>'


def pick(i, kind=None):
    c = COVERS[i]
    return (f'<a class="pick" href="album.html#c={i}" aria-label="{e(c["title"])}, {e(c["kind"])} de {e(c["artist"])}">{img(i, "", "", lazy=False)}'
            f'<div class="plate glass--media"><div><b class="one">{e(c["title"])}</b><span class="one">{e(c["kind"])}, {e(c["artist"])}</span></div>'
            f'<span class="playdot">{ic("play")}</span></div></a>')


def tile(i, title=None, sub=None, href="album.html", rnd=False):
    c = COVERS[i]
    t, s = title or c["title"], sub or c["artist"]
    return (f'<a class="tile{" tile--round" if rnd else ""}" href="{href}#c={i}"><div class="art">{img(i, "-m", "", w=344)}</div>'
            f'<div><b class="one">{e(t)}</b><span class="meta one">{e(s)}</span></div></a>')


def rail(items, label):
    return f'<div class="rail" tabindex="0" role="group" aria-label="{e(label)}">{"".join(items)}</div>'


def row(i, title, artist, dur="", current=False, num=None, more=True, href="player.html", trailing=""):
    lead = f'<span class="num" aria-hidden="true">{num}</span>' if num is not None and i is None else ""
    if num is not None and i is not None:
        lead = f'<span class="num" aria-hidden="true">{num}</span>' + img(i, "-s", "", "thumb", w=160)
    elif i is not None:
        lead = img(i, "-s", "", "thumb", w=160)
    if current and num is not None and i is None:
        lead = '<span class="num"><span class="eq" aria-hidden="true"><i></i><i></i><i></i></span></span>'
    d = f'<span class="dur">{dur}</span>' if dur else ""
    eq = '<span class="eq" aria-hidden="true"><i></i><i></i><i></i></span>' if current and not (num is not None and i is None) else ""
    m = f'<a class="iconbtn" href="sheet-menu.html" aria-label="Mais ações para {e(title)}">{ic("more")}</a>' if more else ""
    cur = ' aria-current="true"' if current else ""
    return (f'<li class="row"{cur}>{lead}<div class="rbody"><a class="rt" href="{href}"><b class="one">{e(title)}</b><span class="sub one">{e(artist)}</span></a>'
            f'{eq}{d}{trailing}{m}</div></li>')


TRACKS = {
    "05": [("Brasa Viva", "3:21"), ("Dedo-de-Moça", "2:58"), ("Malagueta", "4:02"), ("Tigela Azul", "3:40"), ("Ardência", "3:15"), ("Feira de Sábado", "2:47"), ("Pimenta do Reino", "3:56"), ("Tempero", "3:09"), ("Rama", "4:11"), ("Fogo Baixo", "5:02")],
    "03": [("Maré de Março", "3:48"), ("Foz", "4:12"), ("Sedimento", "3:05"), ("Água Doce", "2:59"), ("Mangue", "4:30"), ("Correnteza", "3:33"), ("Banco de Areia", "3:51"), ("Estuário", "5:14"), ("Vista de Cima", "2:44"), ("Mar Aberto", "4:47")],
}


def tracks_for(i):
    return TRACKS.get(i, TRACKS["03"])


# ------------------------------------------------------------------ telas
def home_shelves():
    quick = [("01", "Fio Solto", "Clara Vento"), ("18", "Trilho", "Linha Noturna"), ("22", "Garoa", "Garoa Fina"), ("14", "Veludo", "Irmãos Veludo")]
    return (
        shelf("Para ouvir agora", rail([pick(i) for i in ("03", "01", "09", "12", "08")], "Para ouvir agora"), hid="s-agora")
        + shelf("Faixas para você", '<ul class="rows">' + "".join(row(i, t, a) for i, t, a in quick) + "</ul>", "explore.html", hid="s-rapidas")
        + shelf("Mixes", rail([tile(i, t, s, "playlist.html") for i, t, s in (("17", "Mix da Manhã", "Serra Mansa e mais"), ("21", "Mix Foco", "Helena Marfim e mais"), ("08", "Mix Estrada", "Sertão Sideral e mais"), ("10", "Mix Festa", "Caio Fader e mais"), ("16", "Mix Calma", "Nina Pomar e mais"))], "Mixes"), hid="s-mixes")
        + shelf("Álbuns em destaque", rail([tile(i) for i in ("13", "14", "20", "11", "06", "19")], "Álbuns em destaque"), "explore.html", hid="s-albuns")
    )


def build_home():
    body = top_logo() + f'<main id="conteudo"><h1 class="visually-hidden">Início</h1>{home_shelves()}</main>'
    page("home", "Kazzu: Início", "Início do Kazzu: destaques, escolhas rápidas, mixes e álbuns para tocar com um toque.", body, active="home")
    first = (f'<div class="banner" role="status">{ic("badge-check")}<p><b>Pronto para tocar</b>O YouTube Music já vem ligado. Não precisa de conta.</p>'
             f'<button type="button" class="link" data-dismiss>Entendi</button></div>')
    body = top_logo() + f'<main id="conteudo"><h1 class="visually-hidden">Início</h1>{first}{home_shelves()}</main>'
    page("home-first-run", "Kazzu: primeiro uso", "Início do Kazzu na primeira abertura, com o aviso de que o YouTube Music já está ligado.", body, active="home", now=False)
    sk = ('<section class="shelf" aria-busy="true" aria-label="Carregando o Início"><header><span class="sk sk--line" style="width:160px;height:22px"></span></header>'
          '<div class="rail"><span class="sk sk--card"></span><span class="sk sk--card"></span></div></section>'
          '<section class="shelf" aria-hidden="true"><header><span class="sk sk--line" style="width:180px;height:22px"></span></header><ul class="rows">'
          + "".join('<li class="row"><span class="sk thumb"></span><div class="rbody"><div class="rt" style="gap:8px"><span class="sk sk--line" style="width:58%"></span><span class="sk sk--line" style="width:34%"></span></div></div></li>' for _ in range(4))
          + "</ul></section>")
    body = top_logo() + f'<main id="conteudo"><h1 class="visually-hidden">Início</h1>{sk}</main>'
    page("home-loading", "Kazzu: carregando o Início", "Início do Kazzu carregando: esqueleto no formato dos cartões e das linhas.", body, active="home", now=False)
    local = [("19", "Cavaquinho às Três", "Zé do Traste"), ("21", "Pedal", "Helena Marfim"), ("13", "Lado B", "Selo Amarelo")]
    off = (f'<div class="banner" role="status">{ic("cloud-off")}<p><b>Sem conexão</b>A música do aparelho continua aqui.</p><button type="button" class="link">Tentar de novo</button></div>'
           + shelf("Neste aparelho", '<ul class="rows">' + "".join(row(i, t, a) for i, t, a in local) + "</ul>", "library.html", hid="s-local")
           + shelf("Álbuns do aparelho", rail([tile(i) for i in ("19", "21", "13", "20")], "Álbuns do aparelho"), hid="s-local-albuns"))
    body = top_logo() + f'<main id="conteudo"><h1 class="visually-hidden">Início</h1>{off}</main>'
    page("home-offline", "Kazzu: sem conexão", "Início do Kazzu sem conexão: mostra a música guardada no aparelho.", body, active="home")
    err = (f'<section class="state" role="alert">{ic("cloud-off")}<h2>Não deu para carregar o Início</h2><p>O YouTube Music não respondeu. Sua música do aparelho continua na Biblioteca.</p>'
           f'<div class="acts"><button type="button" class="btn btn--primary">Tentar de novo</button><a class="btn" href="library.html">Abrir a Biblioteca</a></div></section>')
    body = top_logo() + f'<main id="conteudo"><h1 class="visually-hidden">Início</h1>{err}</main>'
    page("home-error", "Kazzu: erro ao carregar", "Início do Kazzu quando o YouTube Music não responde e não há nada salvo.", body, active="home", now=False)


def build_explore():
    moods = [("Foco", 1), ("Treino", 2), ("Festa", 3), ("Calma", 4), ("Estrada", 5), ("Samba", 6), ("Rock", 7), ("Sertanejo", 8)]
    mh = "".join(f'<a class="mood" href="playlist.html"><span class="one">{e(t)}</span>{ic("chevron-right")}</a>' for t, n in moods)
    hot = [("02", "Plataforma 9", "Estação Lume"), ("10", "Fader no Talo", "Caio Fader"), ("12", "Prisma", "Vidro Fumê"), ("04", "Chuva na Janela", "Dona Aurora"), ("15", "Limão Seco", "Banda Azeda")]
    body = (top_title("Explorar") + '<main id="conteudo"><div class="title"><h1>Explorar</h1></div>'
            + shelf("Lançamentos", rail([pick(i) for i in ("02", "10", "12", "19", "15", "04")], "Lançamentos"), hid="s-lanc")
            + f'<section class="shelf" aria-labelledby="s-moods"><header><h2 id="s-moods">Momentos e gêneros</h2></header><div class="moods">{mh}</div></section>'
            + shelf("Em alta", '<ol class="rows rows--cols">' + "".join(row(i, t, a, num=n + 1) for n, (i, t, a) in enumerate(hot)) + "</ol>", hid="s-alta")
            + "</main>")
    page("explore", "Kazzu: Explorar", "Explorar no Kazzu: lançamentos, momentos, gêneros e as músicas em alta.", body, active="explore")


def search_field(value=""):
    v = f' value="{e(value)}"' if value else ""
    clear = f'<button type="button" class="iconbtn" aria-label="Limpar a busca" data-clear>{ic("close")}</button>' if value else ""
    return (f'<form class="stack" role="search" data-search>'
            f'<div class="field">{ic("search")}<input id="q" name="q" type="search" aria-labelledby="t-buscar" inputmode="search" autocomplete="off" enterkeyhint="search" placeholder="Músicas, artistas, álbuns"{v}>{clear}</div></form>')


def build_search():
    rec = ["aurora", "lo-fi para estudar", "Clara Vento", "samba de roda"]
    recents = "".join(
        f'<li class="row row--icon"><span class="lead">{ic("clock")}</span><div class="rbody"><a class="rt" href="search-results.html"><b class="one">{e(t)}</b></a>'
        f'<button type="button" class="iconbtn" aria-label="Tirar {e(t)} das buscas recentes">{ic("close")}</button></div></li>' for t in rec)
    listen = (f'<div class="group"><div class="item" aria-disabled="true">{ic("wave")}<div class="it"><span>Reconhecer pelo som</span>'
              f'<span class="meta">Descubra a música que está tocando perto de você.</span></div><span class="tag">Em breve</span></div></div>')
    body = (top_title("Buscar") + '<main id="conteudo"><div class="title"><h1 id="t-buscar">Buscar</h1></div>' + search_field() + listen
            + f'<section class="shelf" aria-labelledby="s-rec"><header><h2 id="s-rec">Buscas recentes</h2><button type="button" class="link">Limpar</button></header><ul class="rows">{recents}</ul></section>'
            + "</main>")
    page("search", "Kazzu: Buscar", "Busca do Kazzu: campo de texto, reconhecer pelo som e as buscas recentes.", body, active="search", cls="narrow")
    seg = "".join(f'<button type="button" role="tab" aria-selected="{"true" if n == 0 else "false"}">{t}</button>' for n, t in enumerate(["Tudo", "Músicas", "Álbuns", "Artistas", "Playlists"]))
    c = COVERS["01"]
    best = (f'<section class="shelf" aria-labelledby="s-best"><header><h2 id="s-best">Melhor resultado</h2></header><div class="group"><div class="item">'
            f'{img("01", "-s", "", "thumb", w=160)}<a class="it" href="album.html#c=01"><b class="one">{e(c["title"])}</b><span class="meta one">Álbum, {e(c["artist"])}</span></a>'
            f'<a class="playdot" href="player.html" aria-label="Tocar {e(c["title"])}">{ic("play")}</a></div></div></section>')
    songs = [("01", "Novelo", "Clara Vento", "3:12"), ("01", "Fio Solto", "Clara Vento", "2:54"), ("16", "Novelo de Flor", "Nina Pomar", "4:01"), ("14", "Nó e Novelo", "Irmãos Veludo", "3:37")]
    body = (top_title("Buscar") + '<main id="conteudo"><div class="title"><h1 id="t-buscar">Buscar</h1></div>' + search_field("novelo")
            + f'<div class="pad"><div class="seg" role="tablist" aria-label="Tipo de resultado">{seg}</div></div>' + best
            + shelf("Músicas", '<ul class="rows">' + "".join(row(i, t, a, d) for i, t, a, d in songs) + "</ul>", hid="s-mus")
            + shelf("Álbuns", rail([tile(i) for i in ("01", "16", "14")], "Álbuns encontrados"), hid="s-alb")
            + shelf("Artistas", rail([tile(i, COVERS[i]["artist"], "Artista", "artist.html", True) for i in ("01", "16", "14")], "Artistas encontrados"), hid="s-art")
            + "</main>")
    page("search-results", "Kazzu: resultados da busca", "Resultados da busca no Kazzu: melhor resultado, músicas, álbuns e artistas.", body, active="search", cls="narrow")
    empty = (f'<section class="state" role="status">{ic("search")}<h2>Nada para “xyzzq”</h2><p>Confira a grafia ou tente o nome do artista.</p></section>')
    body = top_title("Buscar") + '<main id="conteudo"><div class="title"><h1 id="t-buscar">Buscar</h1></div>' + search_field("xyzzq") + empty + "</main>"
    page("search-empty", "Kazzu: busca sem resultados", "Busca do Kazzu quando nada é encontrado, com a dica do que tentar.", body, active="search", cls="narrow")


def lib_tile(kind):
    if kind == "down":
        return (f'<a class="tile soon" href="library.html" aria-disabled="true" aria-label="Baixados, em breve"><div class="art art--empty">{ic("download")}</div>'
                f'<div><b class="one">Baixados</b><span class="meta one">Em breve</span></div></a>')
    if kind == "liked":
        return (f'<a class="tile" href="playlist.html#c=11"><div class="art">{img("11", "", "")}<span class="badge badge--liked glass--media">{ic("heart-fill")}</span></div>'
                f'<div><b class="one">Curtidos</b><span class="meta one">128 músicas</span></div></a>')
    cs, name, count, icn = (("13", "14", "20", "21"), "Álbuns", "24 álbuns", "disc") if kind == "albums" else (("08", "16", "02", "06"), "Playlists", "6 playlists", "queue")
    mos = "".join(img(i, "-s", "", w=160) for i in cs)
    return (f'<a class="tile" href="{"explore" if kind == "albums" else "playlist"}.html"><div class="art mosaic">{mos}<span class="badge glass--media">{ic(icn)}</span></div>'
            f'<div><b class="one">{name}</b><span class="meta one">{count}</span></div></a>')


def build_library():
    create = f'<a class="btn" href="sheet-new-playlist.html">{ic("plus")}Criar playlist</a>'
    pls = "".join(
        f'<li class="row"><span class="thumb mosaic" style="overflow:hidden">{"".join(img(i, "-s", "", w=160) for i in c)}</span><div class="rbody"><a class="rt" href="playlist.html"><b class="one">{e(n)}</b><span class="sub one">{e(cnt)}</span></a>'
        f'<a class="iconbtn" href="sheet-menu.html" aria-label="Mais ações para a playlist {e(n)}">{ic("more")}</a></div></li>' for n, cnt, c in PLAYLISTS)
    local = (f'<div class="group"><div class="item">{ic("folder")}<div class="it"><span class="one">Music/FLAC</span><span class="meta">312 músicas, lida hoje</span></div></div>'
             f'<a class="item" href="settings.html">{ic("plus")}<div class="it"><span>Adicionar pasta</span></div>{ic("chevron-right", "chev")}</a></div>')
    grid = '<div class="grid2">' + "".join(lib_tile(k) for k in ("down", "liked", "albums", "playlists")) + "</div>"
    body = (top_title("Biblioteca") + f'<main id="conteudo"><div class="title"><h1>Biblioteca</h1>{create}</div>' + grid
            + shelf("Suas playlists", f'<ul class="rows">{pls}</ul>', hid="s-pl")
            + f'<section class="shelf" aria-labelledby="s-dev"><header><h2 id="s-dev">Neste aparelho</h2></header>{local}</section></main>')
    page("library", "Kazzu: Biblioteca", "Biblioteca do Kazzu: baixados, curtidos, álbuns, playlists e a música do aparelho.", body, active="library")
    empty_tiles = ('<div class="grid2">' + lib_tile("down")
                   + "".join(f'<a class="tile" href="library-empty.html"><div class="art art--empty">{ic(i)}</div><div><b class="one">{n}</b><span class="meta one">{c}</span></div></a>'
                             for i, n, c in (("heart", "Curtidos", "0 músicas"), ("disc", "Álbuns", "0 álbuns"), ("queue", "Playlists", "0 playlists"))) + "</div>")
    st = (f'<section class="state">{ic("folder")}<h2>Traga a música do aparelho</h2><p>Escolha uma pasta e o Kazzu toca FLAC, WAV, Opus e MP3 como são, sem mexer no som.</p>'
          f'<div class="acts"><button type="button" class="btn btn--primary">Adicionar pasta</button></div></section>')
    body = top_title("Biblioteca") + f'<main id="conteudo"><div class="title"><h1>Biblioteca</h1>{create}</div>{empty_tiles}{st}</main>'
    page("library-empty", "Kazzu: Biblioteca vazia", "Biblioteca do Kazzu ainda vazia, com o convite para adicionar uma pasta do aparelho.", body, active="library", now=False)


def detail_hero(i, kind_line, artist=False):
    c = COVERS[i]
    name = c["artist"] if artist else c["title"]
    over = (f'<div class="over"><a class="iconbtn iconbtn--glass glass--media back" href="home.html" aria-label="Voltar">{ic("chevron-left")}</a>'
            f'<div>{toggle("heart", "heart-fill", "Salvar na Biblioteca", cls="iconbtn--glass glass--media")}'
            f'<a class="iconbtn iconbtn--glass glass--media" href="sheet-menu.html" aria-label="Mais ações">{ic("more")}</a></div></div>')
    return (f'<div class="hero{" hero--artist" if artist else ""}">{img(i, "", "", lazy=False)}{over}'
            f'<div class="plate glass--media"><h1 data-c-title>{e(name)}</h1><p data-c-sub>{e(kind_line)}</p></div></div>')


def acts():
    return (f'<div class="actions"><a class="btn btn--primary btn--wide" href="player.html">{ic("play")}Tocar</a>'
            f'<a class="btn btn--wide" href="player.html">{ic("shuffle")}Aleatório</a></div>')


def build_detail():
    i = "03"
    c = COVERS[i]
    rows = "".join(row(None, t, c["artist"], d, num=n + 1, current=(n == 0)) for n, (t, d) in enumerate(tracks_for(i)))
    body = (f'<main id="conteudo">{detail_hero(i, c["kind"] + " de " + c["artist"] + ", 2026")}{acts()}'
            f'<ol class="rows" aria-label="Músicas do álbum">{rows}</ol>'
            f'<p class="meta foot">10 músicas, 40 min. Do YouTube Music.</p></main>'
            f'<div class="topbar glass" aria-hidden="true">{e(c["title"])}</div>')
    page("album", "Kazzu: álbum Delta", "Página de álbum no Kazzu: capa, tocar, aleatório e a lista de músicas.", body, cls="detail", active="home")
    pl = [("08", "Poeira Rosa", "Sertão Sideral", "4:10"), ("17", "Neblina Dourada", "Serra Mansa", "3:44"), ("18", "Último Trem", "Linha Noturna", "5:02"), ("02", "Plataforma 9", "Estação Lume", "3:18"),
          ("19", "Cavaquinho às Três", "Zé do Traste", "2:51"), ("22", "Margem", "Garoa Fina", "4:26"), ("14", "Veludo", "Irmãos Veludo", "3:29"), ("05", "Brasa Viva", "Os Tangarás", "3:21")]
    rows = "".join(row(ci, t, a, d) for ci, t, a, d in pl)
    body = (f'<main id="conteudo">{detail_hero("08", "Playlist, 42 músicas")}'.replace("Duna Rosa", "Estrada") + acts()
            + f'<ul class="rows" aria-label="Músicas da playlist">{rows}</ul><p class="meta foot">42 músicas, 2 h 38 min. Guardada neste aparelho.</p></main>'
            + '<div class="topbar glass" aria-hidden="true">Estrada</div>')
    page("playlist", "Kazzu: playlist Estrada", "Página de playlist no Kazzu, com músicas de várias fontes numa lista só.", body, cls="detail", active="library")
    ar = COVERS["01"]
    top = "".join(row("01", t, "Novelo", d) for t, d in (("Fio Solto", "2:54"), ("Ponto Cruz", "3:30"), ("Tear", "4:05"), ("Avesso", "3:12")))
    body = (f'<main id="conteudo">{detail_hero("01", "Artista", artist=True)}{acts()}'
            + shelf("Mais tocadas", f'<ul class="rows">{top}</ul>', hid="s-top")
            + shelf("Álbuns", rail([tile(i, t, s) for i, t, s in (("01", "Novelo", "2026"), ("16", "Flor de Maçã", "2025"), ("06", "Rede", "2024"))], "Álbuns de " + ar["artist"]), hid="s-disc")
            + f'</main><div class="topbar glass" aria-hidden="true">{e(ar["artist"])}</div>')
    page("artist", "Kazzu: artista Clara Vento", "Página de artista no Kazzu: mais tocadas e álbuns.", body, cls="detail", active="home")
    empty = (f'<section class="state">{ic("queue")}<h2>Esta playlist está vazia</h2><p>Toque em Mais numa música e escolha Adicionar à playlist.</p>'
             f'<div class="acts"><a class="btn btn--primary" href="search.html">Buscar músicas</a></div></section>')
    body = (top_title("Domingo", f'<a class="iconbtn" href="sheet-menu.html" aria-label="Mais ações">{ic("more")}</a>').replace(f'<a class="logo" href="home.html" aria-label="Kazzu, ir para o Início">{WORDMARK}</a>', f'<a class="iconbtn back" href="library.html" aria-label="Voltar">{ic("chevron-left")}</a>')
            + f'<main id="conteudo"><div class="title"><h1>Domingo</h1></div>{empty}</main>')
    page("playlist-empty", "Kazzu: playlist vazia", "Playlist recém-criada no Kazzu, ainda sem músicas, com o próximo passo.", body, active="library", cls="narrow")


def player_markup(inert=False):
    c = NOW
    cv = COVERS[c["cover"]]
    att = ' inert aria-hidden="true"' if inert else ""
    return (f'<div class="stage"{att} data-player><img class="backdrop" src="{cov(c["cover"], "-back")}" width="192" height="464" alt="" fetchpriority="high"><img class="backdrop backdrop--wide" src="{cov(c["cover"], "-blur")}" width="96" height="96" alt="" loading="lazy">'
            f'<div style="position:relative">{img(c["cover"], "", "Capa de " + c["album"], "art", lazy=False)}'
            f'<div class="over"><a class="iconbtn iconbtn--glass glass--media" href="home.html" aria-label="Fechar o player">{ic("chevron-down")}</a>'
            f'<p class="ctx glass--media one">Tocando de <b data-c-album>{e(c["album"])}</b></p>'
            f'<a class="iconbtn iconbtn--glass glass--media" href="sheet-menu.html" aria-label="Mais ações">{ic("more")}</a></div></div>'
            f'<section class="console" style="--dim:{cv["dim"]}" aria-label="Controles">'
            f'<div class="now"><div><h1 class="one" data-c-track>{e(c["title"])}</h1><p class="by one" data-c-artist>{e(c["artist"])}</p></div>{toggle("heart", "heart-fill", "Curtir")}</div>'
            f'<div class="scrub"><div class="bar" role="slider" tabindex="0" aria-label="Posição" aria-valuemin="0" aria-valuemax="201" aria-valuenow="76" aria-valuetext="1 minuto e 16 segundos de 3 minutos e 21 segundos"></div>'
            f'<div class="times"><span>1:16</span><span class="src">YouTube Music, Opus 160 kbps</span><span aria-label="Faltam 2 minutos e 5 segundos">-2:05</span></div></div>'
            f'<div class="transport"><button type="button" class="iconbtn" aria-label="Aleatório" aria-pressed="false">{ic("shuffle")}</button>'
            f'<button type="button" class="iconbtn big" aria-label="Anterior">{ic("prev")}</button>'
            f'<button type="button" class="iconbtn play pp" aria-label="Pausar">{ic("pause", "i-pause")}{ic("play", "i-play")}</button>'
            f'<button type="button" class="iconbtn big" aria-label="Próxima">{ic("next")}</button>'
            f'<button type="button" class="iconbtn" aria-label="Repetir" aria-pressed="false">{ic("repeat")}</button></div>'
            f'<div class="volume">{ic("vol-low")}<div class="bar" role="slider" tabindex="0" aria-label="Volume" aria-valuemin="0" aria-valuemax="100" aria-valuenow="70"></div>{ic("vol-high")}</div>'
            f'<div class="extras"><button type="button" aria-label="Saída: este aparelho">{ic("device")}<span>Este aparelho</span></button>'
            f'<a href="queue.html">{ic("queue")}<span>Fila</span></a></div>'
            f'</section></div>')


def build_player():
    page("player", "Kazzu: tocando Brasa Viva", "Player do Kazzu: a capa de ponta a ponta e um console de vidro com os controles.", f'<main id="conteudo" style="display:block;padding:0;max-width:none">{player_markup()}</main>', cls="player", chrome=False)
    nxt = [("05", "Dedo-de-Moça", "Os Tangarás"), ("05", "Malagueta", "Os Tangarás"), ("05", "Tigela Azul", "Os Tangarás"), ("05", "Ardência", "Os Tangarás"), ("05", "Feira de Sábado", "Os Tangarás"), ("03", "Maré de Março", "Maré Alta Trio"), ("01", "Fio Solto", "Clara Vento")]
    tr = lambda t: (f'<button type="button" class="iconbtn" aria-label="Mover {e(t)} para cima">{ic("arrow-up")}</button>'
                    f'<button type="button" class="iconbtn" aria-label="Tirar {e(t)} da fila">{ic("close")}</button>')
    rows = "".join(row(i, t, a, more=False, trailing=tr(t)) for i, t, a in nxt)
    sheet = (f'<div class="scrim"></div><section class="sheet sheet--tall glass glass--sheet" role="dialog" aria-modal="true" aria-labelledby="q-t"><div><div class="grab" aria-hidden="true"></div>'
             f'<div class="head"><h1 id="q-t">Fila</h1><div><button type="button" class="link" style="padding-inline:var(--space-sm)">Limpar</button><a class="iconbtn" href="player.html" aria-label="Fechar a fila">{ic("close")}</a></div></div></div>'
             f'<div class="body"><div class="stack"><h2 class="grouphead">Tocando agora</h2><ul class="rows">{row("05", "Brasa Viva", "Os Tangarás", current=True, more=False)}</ul></div>'
             f'<div class="stack"><h2 class="grouphead">A seguir, de Pimenta</h2><ul class="rows">{rows}</ul></div>'
             f'<p class="note">Quando a fila acabar, o Kazzu continua com músicas parecidas. Você pode desligar isso em Ajustes.</p></div></section>')
    page("queue", "Kazzu: fila", "Fila do Kazzu: a música atual e as próximas, com mover e remover.", f'<main id="conteudo" style="display:block;padding:0;max-width:none">{player_markup(True)}{sheet}</main>', cls="player", chrome=False)


def build_sheets():
    c = COVERS["03"]
    rows = "".join(row(None, t, c["artist"], d, num=n + 1, current=(n == 0)) for n, (t, d) in enumerate(tracks_for("03")))
    under = (f'<div inert aria-hidden="true">{detail_hero("03", c["kind"] + " de " + c["artist"] + ", 2026")}{acts()}<ol class="rows">{rows}</ol></div>')

    def act(icon, label, soon=False, pressed=None):
        dis = ' aria-disabled="true"' if soon else ""
        tag = '<span class="tag">Em breve</span>' if soon else ""
        return f'<li><button type="button" class="act"{dis}>{ic(icon)}<span>{e(label)}</span>{tag}</button></li>'
    head = (f'<div><div class="grab" aria-hidden="true"></div><div class="head" style="padding-block:var(--space-xs)"><div class="row" style="flex:1;min-width:0">{img("03", "-s", "", "thumb", w=160)}'
            f'<div class="rt"><h1 id="m-t" class="one" style="font-size:var(--text-lg);line-height:var(--lh-lg);font-family:var(--font-body);font-weight:600;letter-spacing:0">Maré de Março</h1><span class="sub one">{e(c["artist"])}</span></div></div>'
            f'<a class="iconbtn" href="album.html" aria-label="Fechar">{ic("close")}</a></div></div>')
    menu = (f'<div class="scrim"></div><section class="sheet glass glass--sheet" role="dialog" aria-modal="true" aria-labelledby="m-t">{head}<div class="body"><ul class="acts">'
            + act("next", "Tocar em seguida") + act("queue", "Adicionar à fila") + act("playlist-add", "Adicionar à playlist") + act("heart", "Curtir")
            + '</ul><div class="sep" role="separator"></div><ul class="acts">' + act("disc", "Ir para o álbum") + act("user", "Ir para o artista")
            + '</ul><div class="sep" role="separator"></div><ul class="acts">' + act("lyrics", "Ver a letra", True) + act("timer", "Timer para dormir", True) + act("download", "Baixar", True) + "</ul></div></section>")
    page("sheet-menu", "Kazzu: ações da música", "Folha de ações de uma música no Kazzu: fila, playlist, curtir, álbum e artista.", f'<main id="conteudo" style="display:block;padding:0">{under}{menu}</main>', cls="detail", chrome=False)
    pls = "".join(
        f'<li class="row"><span class="thumb mosaic" style="overflow:hidden">{"".join(img(i, "-s", "", w=160) for i in cs)}</span><div class="rbody"><button type="button" class="rt" style="text-align:left"><b class="one">{e(n)}</b><span class="sub one">{e(cnt)}</span></button></div></li>'
        for n, cnt, cs in PLAYLISTS)
    addpl = (f'<div class="scrim"></div><section class="sheet glass glass--sheet" role="dialog" aria-modal="true" aria-labelledby="p-t"><div><div class="grab" aria-hidden="true"></div>'
             f'<div class="head"><h1 id="p-t">Adicionar à playlist</h1><a class="iconbtn" href="album.html" aria-label="Fechar">{ic("close")}</a></div></div>'
             f'<div class="body"><ul class="acts"><li><a class="act" href="sheet-new-playlist.html">{ic("plus")}<span>Nova playlist</span></a></li></ul><ul class="rows">{pls}</ul></div></section>')
    page("sheet-playlists", "Kazzu: adicionar à playlist", "Folha para escolher a playlist que recebe a música, ou criar uma nova.", f'<main id="conteudo" style="display:block;padding:0">{under}{addpl}</main>', cls="detail", chrome=False)
    newpl = (f'<div class="scrim"></div><section class="sheet glass glass--sheet" role="dialog" aria-modal="true" aria-labelledby="n-t"><div><div class="grab" aria-hidden="true"></div>'
             f'<div class="head"><h1 id="n-t">Nova playlist</h1><a class="iconbtn" href="library.html" aria-label="Fechar">{ic("close")}</a></div></div>'
             f'<form class="body" data-newpl novalidate><div><label class="lbl" for="pn">Nome da playlist</label><div class="field field--plain"><input id="pn" name="nome" type="text" autocomplete="off" maxlength="60" value="Estrada" required></div>'
             f'<p class="help meta" id="pn-h" aria-live="polite">Fica guardada neste aparelho.</p></div>'
             f'<div class="actions"><a class="btn btn--wide" href="library.html">Cancelar</a><button type="submit" class="btn btn--primary btn--wide">Criar playlist</button></div></form></section>')
    lib_under = '<div inert aria-hidden="true"><div class="title" style="padding-top:calc(var(--safe-top) + var(--space-3xl))"><h1>Biblioteca</h1></div><div class="grid2">' + "".join(lib_tile(k) for k in ("down", "liked", "albums", "playlists")) + "</div></div>"
    page("sheet-new-playlist", "Kazzu: nova playlist", "Folha para dar nome a uma playlist nova no Kazzu.", f'<main id="conteudo" style="display:block;padding:0">{lib_under}{newpl}</main>', chrome=False)


def build_settings():
    def sw(label, sub, on, name):
        return (f'<div class="item"><div class="it"><span id="{name}">{e(label)}</span>' + (f'<span class="meta">{e(sub)}</span>' if sub else "")
                + f'</div><span class="switchhit"><button type="button" class="switch" role="switch" aria-checked="{"true" if on else "false"}" aria-labelledby="{name}" data-switch="{name}"></button></span></div>')

    def seg(label, opts, cur, name):
        o = "".join(f'<button type="button" aria-pressed="{"true" if n == cur else "false"}">{e(t)}</button>' for n, t in enumerate(opts))
        return f'<div class="item item--col"><span id="{name}">{e(label)}</span><div class="seg" role="group" aria-labelledby="{name}">{o}</div></div>'

    def val(icon, label, value, sub=""):
        return (f'<div class="item">{ic(icon) if icon else ""}<div class="it"><span>{e(label)}</span>' + (f'<span class="meta">{e(sub)}</span>' if sub else "")
                + f'</div><span class="val">{e(value)}</span></div>')

    def nav(icon, label, sub=""):
        return (f'<a class="item" href="settings.html">{ic(icon) if icon else ""}<div class="it"><span>{e(label)}</span>' + (f'<span class="meta">{e(sub)}</span>' if sub else "")
                + f'</div>{ic("chevron-right", "chev")}</a>')

    def grp(title, inner, note=""):
        n = f'<p class="meta groupnote">{e(note)}</p>' if note else ""
        return f'<section class="stack"><h2 class="grouphead">{e(title)}</h2><div class="group">{inner}</div>{n}</section>'
    body = (f'<header class="top"><a class="iconbtn back" href="home.html" aria-label="Voltar">{ic("chevron-left")}</a><span></span></header><div class="topbar glass" aria-hidden="true">Ajustes</div>'
            '<main id="conteudo"><div class="title"><h1>Ajustes</h1></div>'
            + grp("Reprodução", seg("Qualidade do streaming", ["Original", "Alta", "Econômica"], 0, "st-q") + sw("Continuar com músicas parecidas", "Quando a fila acabar.", True, "st-auto"))
            + grp("Aparência", seg("Tema", ["Automático", "Claro", "Escuro"], 2, "st-tema") + sw("Reduzir transparência", "Troca o vidro por superfícies sólidas.", False, "rt") + sw("Reduzir movimento", "Tira os deslocamentos das transições.", False, "rm"))
            + grp("Biblioteca", val("folder", "Music/FLAC", "312 músicas") + nav("plus", "Adicionar pasta") + nav("refresh", "Ler pastas de novo"))
            + grp("Fontes de música", val("badge-check", "YouTube Music", "Ligado", "Sem conta.") + val("", "Spotify", "Em breve") + val("", "Tidal", "Em breve") + val("", "Qobuz", "Em breve"))
            + grp("Privacidade", val("shield", "Sem telemetria", "", "O Kazzu não coleta nada. Não há conta.") + nav("trash", "Apagar histórico", "Buscas e reprodução deste aparelho.") + nav("info", "Relatar um problema", "Gera um arquivo aqui. Você decide se envia."))
            + grp("Sobre", val("", "Versão", "Kazzu 0.1.0") + nav("", "Licenças de código aberto"))
            + "</main>")
    page("settings", "Kazzu: Ajustes", "Ajustes do Kazzu: reprodução, aparência, biblioteca, fontes de música e privacidade.", body, cls="settings narrow", active=None)


def build_index():
    groups = [
        ("Abas", [("home", "Início"), ("explore", "Explorar"), ("library", "Biblioteca"), ("search", "Buscar")]),
        ("Tocando", [("player", "Player"), ("queue", "Fila"), ("sheet-menu", "Ações da música"), ("sheet-playlists", "Adicionar à playlist"), ("sheet-new-playlist", "Nova playlist")]),
        ("Detalhe", [("album", "Álbum"), ("playlist", "Playlist"), ("artist", "Artista"), ("search-results", "Resultados da busca"), ("settings", "Ajustes")]),
        ("Estados", [("home-first-run", "Primeiro uso"), ("home-loading", "Carregando"), ("home-offline", "Sem conexão"), ("home-error", "Erro"), ("search-empty", "Busca sem resultados"), ("library-empty", "Biblioteca vazia"), ("playlist-empty", "Playlist vazia")]),
    ]
    secs = "".join(
        f'<section class="stack" aria-labelledby="g{n}"><h2 id="g{n}">{e(t)}</h2><div class="links">' + "".join(f'<a href="{k}.html"><span>{e(l)}</span><span class="meta">{k}.html</span></a>' for k, l in items) + "</div></section>"
        for n, (t, items) in enumerate(groups))
    body = (f'<header class="top"><a class="logo" href="home.html" aria-label="Kazzu, abrir o protótipo do Início">{WORDMARK}</a><span></span></header>'
            f'<main id="conteudo"><div class="stack"><h1>Protótipos da v4</h1>'
            f'<p class="sub">Telas do app para celular. Abra numa janela de 390 de largura, ou em 768 e 1280 para ver o mesmo desenho crescer. Tema e acessibilidade pela URL: #theme=light, #rt=1, #rm=1.</p></div>{secs}</main>')
    page("index", "Kazzu v4: protótipos das telas", "Índice dos protótipos do Kazzu v4: abas, player, folhas, detalhe e estados.", body, cls="hub", chrome=False)


def main():
    OUT.mkdir(exist_ok=True)
    build_home()
    build_explore()
    build_search()
    build_library()
    build_detail()
    build_player()
    build_sheets()
    build_settings()
    build_index()
    covers_js = {k: {"t": v["title"], "a": v["artist"], "k": v["kind"], "d": v["dim"], "m": v["mini"]} for k, v in COVERS.items()}
    (OUT / "assets" / "covers.js").write_text("window.KZ_COVERS=" + json.dumps(covers_js, ensure_ascii=False) + ";\n", encoding="utf-8")
    print("páginas:", len(list(OUT.glob("*.html"))))


if __name__ == "__main__":
    main()
