#!/usr/bin/env python3
"""Baixa e prepara as capas de demonstração da v4 (fotografias CC0 ou de domínio público do Wikimedia Commons).

Fonte da lista: biblioteca de fotos da skill hallmark (.claude/skills/hallmark/scripts/photos.json), filtrada por
licença CC0 / Public domain, para que as imagens possam viver no repositório sem obrigação de atribuição.
Mesmo assim os créditos ficam em design/v4/assets/covers/CREDITS.md.

Para cada capa grava:
  covers/NN.jpg        640x640  (capa)
  covers/NN-m.jpg      344x344  (ladrilho)
  covers/NN-s.jpg      160x160  (miniatura de linha)
  covers/NN-blur.jpg   96x96    (pré-desfocada e levemente saturada: o que o Rust entrega em `backdrop`)
e um covers.json com título, artista (fictícios), cor média, luminância e o escurecimento do vidro do player.

Uso: python3 design/v4/tools/make_covers.py   (precisa de rede na primeira vez; reaproveita design/v4/assets/covers/.src)
"""
import json
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
OUT = ROOT / "assets" / "covers"
SRC = OUT / ".src"
LIB = REPO / ".claude" / "skills" / "hallmark" / "scripts" / "photos.json"

# (trecho do campo `subject` na biblioteca, título fictício, artista fictício, tipo, foco x, foco y)
PICKS = [
    ("balls of bright cotton thread", "Novelo", "Clara Vento", "Álbum", 0.5, 0.5),
    ("red ceiling lights in a station hall", "Plataforma 9", "Estação Lume", "Álbum", 0.5, 0.5),
    ("river delta pouring silt", "Delta", "Maré Alta Trio", "Álbum", 0.5, 0.5),
    ("raindrops on a window glowing", "Chuva na Janela", "Dona Aurora", "Single", 0.5, 0.5),
    ("fresh red bird's eye chillies", "Pimenta", "Os Tangarás", "Álbum", 0.5, 0.5),
    ("yellow fishing net with teal rope", "Rede", "Jangada Elétrica", "EP", 0.45, 0.5),
    ("blue glass of sparkling water", "Água com Gás", "Lia Marfim", "Single", 0.5, 0.5),
    ("pale sand dunes below pink lit desert", "Duna Rosa", "Sertão Sideral", "Álbum", 0.5, 0.6),
    ("ballet dancer leaping on stage", "Salto", "Orquestra do Cais", "Ao vivo", 0.42, 0.5),
    ("hands working a glowing digital mixing desk", "Mesa de Som", "Caio Fader", "Mix", 0.5, 0.55),
    ("wild blueberries and red cloudberries", "Fruta Miúda", "Trio Cambuci", "EP", 0.5, 0.5),
    ("plastic forks and spoons glowing with rainbow", "Prisma", "Vidro Fumê", "Álbum", 0.5, 0.5),
    ("black vinyl LP with a plain yellow centre label", "Lado B", "Selo Amarelo", "Coletânea", 0.5, 0.5),
    ("two old acoustic guitars resting in a teal velvet", "Poltrona Verde", "Irmãos Veludo", "Álbum", 0.5, 0.55),
    ("heap of dried lemon slices", "Limão Seco", "Banda Azeda", "Single", 0.5, 0.5),
    ("pink and white apple blossom", "Flor de Maçã", "Nina Pomar", "Álbum", 0.5, 0.5),
    ("golden haze over rolling farm hills", "Neblina Dourada", "Serra Mansa", "Álbum", 0.5, 0.6),
    ("dark train sliding past at night", "Último Trem", "Linha Noturna", "Álbum", 0.5, 0.5),
    ("hand fretting chords on a small string instrument", "Cavaquinho às Três", "Zé do Traste", "Álbum", 0.5, 0.5),
    ("tall glass cylinders of amber and gold liquids", "Âmbar", "Laboratório Solar", "EP", 0.5, 0.5),
    ("three brass piano pedals", "Pedal", "Helena Marfim", "Álbum", 0.5, 0.5),
    ("river in morning fog with reeds", "Margem", "Garoa Fina", "Álbum", 0.5, 0.5),
]


def lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lum(rgb):
    return 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])


def thumb_url(file_url: str, px: int) -> str:
    # https://upload.wikimedia.org/wikipedia/commons/a/ab/Name.jpg -> .../commons/thumb/a/ab/Name.jpg/960px-Name.jpg
    head, tail = file_url.split("/commons/", 1)
    name = tail.rsplit("/", 1)[1]
    return f"{head}/commons/thumb/{tail}/{px}px-{name}"


def fetch(url: str, dest: Path):
    """Baixa com o curl (o Commons recusa o cliente HTTP padrão do Python), com espaçamento e recuo em 429."""
    import subprocess
    import time
    if dest.exists() and dest.stat().st_size > 2000:
        return
    ua = "CoverStandInFetch/1.0 (one-off static asset download for a design mockup)"
    for attempt in range(5):
        r = subprocess.run(["curl", "-s", "-L", "-o", str(dest), "-w", "%{http_code}", "-A", ua, url], capture_output=True, text=True, timeout=120)
        if r.stdout.strip() == "200" and dest.stat().st_size > 2000:
            time.sleep(1.2)
            return
        dest.unlink(missing_ok=True)
        time.sleep(8 * (attempt + 1))
    raise RuntimeError(f"não baixou: {url} ({r.stdout})")


def square(im: Image.Image, fx: float, fy: float) -> Image.Image:
    w, h = im.size
    s = min(w, h)
    x = int(max(0, min(w - s, w * fx - s / 2)))
    y = int(max(0, min(h - s, h * fy - s / 2)))
    return im.crop((x, y, x + s, y + s))


def main():
    lib = json.loads(LIB.read_text(encoding="utf-8"))
    lib = lib if isinstance(lib, list) else lib.get("photos", list(lib.values()))
    free = [p for p in lib if p.get("license") in ("CC0", "Public domain")]
    OUT.mkdir(parents=True, exist_ok=True)
    SRC.mkdir(exist_ok=True)
    meta, credits = [], ["# Créditos das capas de demonstração (v4)", "",
                         "Fotografias do Wikimedia Commons, todas **CC0 ou domínio público**, escolhidas pela biblioteca da skill hallmark.",
                         "São substitutas: no app de verdade as capas vêm das músicas. Títulos e artistas são fictícios.", "",
                         "| Capa | Título fictício | Autor da foto | Licença | Página |", "|---|---|---|---|---|"]
    for i, (key, title, artist, kind, fx, fy) in enumerate(PICKS, 1):
        hit = [p for p in free if key in p["subject"]]
        assert len(hit) == 1, (key, len(hit))
        p = hit[0]
        nn = f"{i:02d}"
        raw = SRC / f"{nn}.jpg"
        fetch(thumb_url(p["file"], 960), raw)
        im = square(Image.open(raw).convert("RGB"), fx, fy)
        big = im.resize((640, 640), Image.LANCZOS)
        big.save(OUT / f"{nn}.jpg", quality=84, optimize=True, progressive=True)
        im.resize((160, 160), Image.LANCZOS).save(OUT / f"{nn}-s.jpg", quality=82, optimize=True)
        im.resize((344, 344), Image.LANCZOS).save(OUT / f"{nn}-m.jpg", quality=84, optimize=True)   # ladrilhos de 148 a 172 px em 2x
        # o que o Rust faria: reduzir, desfocar forte, subir um pouco a saturação (o Slint não tem saturate no backdrop)
        small = im.resize((96, 96), Image.LANCZOS).filter(ImageFilter.GaussianBlur(9))
        small = ImageEnhance.Color(small).enhance(1.35)
        small.save(OUT / f"{nn}-blur.jpg", quality=80, optimize=True)
        # fundo do player no celular: a capa e o REFLEXO dela (espelhada) logo abaixo. A UI desenha esta imagem com a
        # largura da tela, a partir do topo: os 192 px de cima ficam atrás da capa nítida e o console de vidro cai sobre
        # o reflexo. O desfoque aqui é leve de propósito: quem desfoca é o vidro, ao vivo, e as formas grandes da capa
        # continuam reconhecíveis através dele. Sem degradê desenhado: toda a cor é a capa.
        s192 = im.resize((192, 192), Image.LANCZOS)
        tall = Image.new("RGB", (192, 464))
        tall.paste(s192, (0, 0))
        tall.paste(s192.transpose(Image.FLIP_TOP_BOTTOM), (0, 192))
        tall.paste(s192.crop((0, 0, 192, 80)), (0, 384))
        ImageEnhance.Color(tall.filter(ImageFilter.GaussianBlur(3))).enhance(1.2).save(OUT / f"{nn}-back.jpg", quality=84, optimize=True)
        avg = small.resize((1, 1), Image.BOX).getpixel((0, 0))
        # Pior região da capa (grade 12x12, perto do que sobra depois do desfoque do vidro): vira o cinza sRGB equivalente V.
        # O console do player usa tinta quase preta (valor ~12) com alfa `dim`, e todo texto nele é branco puro.
        # Para o branco passar de 4,5:1 com folga (5:1) o fundo composto precisa ficar <= 112: V*(1-dim) + 12*dim <= 112.
        cells = list(im.resize((12, 12), Image.BOX).get_flattened_data())
        worst = max(lum(px) for px in cells)
        v = 255 * (12.92 * worst if worst <= 0.0031308 else 1.055 * worst ** (1 / 2.4) - 0.055)
        need = 0.0 if v <= 112 else (v - 112) / (v - 12)
        dim = round(min(0.64, max(0.24, need + 0.02)), 2)
        # Mini-player: atrás do texto há 55% da capa desfocada e 45% do que rola por trás (pior caso: branco puro).
        x = 0.45 * 255 + 0.55 * v
        mini = round(min(0.70, max(0.30, (x - 112) / (x - 12) + 0.02)), 2)
        meta.append({"id": nn, "title": title, "artist": artist, "kind": kind,
                     "avg": "#%02x%02x%02x" % avg, "lum": round(lum(avg), 3), "dim": dim, "mini": mini,
                     "alt": p["subject"], "license": p["license"], "author": p.get("author", ""), "page": p["page"]})
        credits.append(f"| {nn} | {title} | {p.get('author', '')} | {p['license']} | {p['page']} |")
        print(nn, title, "avg", meta[-1]["avg"], "lum", meta[-1]["lum"], "dim", dim)
    (OUT / "covers.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1), encoding="utf-8")
    (OUT / "CREDITS.md").write_text("\n".join(credits) + "\n", encoding="utf-8")
    (OUT / ".gitignore").write_text(".src/\n", encoding="utf-8")


if __name__ == "__main__":
    main()
