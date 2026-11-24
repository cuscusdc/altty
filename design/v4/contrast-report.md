# Contraste medido nos protótipos v4 renderizados

Gerado por `design/v4/tools/check_render.py` (Chromium headless + Pillow). Para cada texto visível: contraste da cor do texto contra o **pior** pixel (percentil 5) do fundo atrás dele, já com vidro, desfoque e capas. Limite WCAG 2.2 AA: 4,5:1 (3:1 para texto grande).

**3496 textos medidos em 188 capturas; 0 abaixo do limite; menor contraste 4.55:1.** Fora da conta, como a WCAG permite: 44 textos de controles desativados (itens "Em breve").

| Tela | Largura | Tema | Variante | Textos | Reprovados | Pior caso |
|---|---|---|---|---|---|---|
| home | 390 | dark |  | 16 | 0 | 6.13:1 “Início” |
| home | 390 | light |  | 16 | 0 | 4.86:1 “Ver tudo” |
| home | 390 | dark | rt=1 | 16 | 0 | 6.13:1 “Início” |
| home | 390 | light | rt=1 | 16 | 0 | 4.86:1 “Ver tudo” |
| explore | 390 | dark |  | 14 | 0 | 6.13:1 “Explorar” |
| explore | 390 | light |  | 14 | 0 | 5.86:1 “Brasa Viva” |
| explore | 390 | dark | rt=1 | 14 | 0 | 6.13:1 “Explorar” |
| explore | 390 | light | rt=1 | 14 | 0 | 18.42:1 “Foco” |
| search | 390 | dark |  | 15 | 0 | 7.03:1 “Em breve” |
| search | 390 | light |  | 15 | 0 | 4.86:1 “Limpar” |
| search | 390 | dark | rt=1 | 15 | 0 | 7.03:1 “Em breve” |
| search | 390 | light | rt=1 | 15 | 0 | 4.86:1 “Limpar” |
| search-results | 390 | dark |  | 24 | 0 | 6.13:1 “Tudo” |
| search-results | 390 | light |  | 24 | 0 | 6.03:1 “Brasa Viva” |
| search-results | 390 | dark | rt=1 | 24 | 0 | 6.13:1 “Tudo” |
| search-results | 390 | light | rt=1 | 24 | 0 | 5.72:1 “4:01” |
| library | 390 | dark |  | 14 | 0 | 6.13:1 “Biblioteca” |
| library | 390 | light |  | 14 | 0 | 6.03:1 “Brasa Viva” |
| library | 390 | dark | rt=1 | 14 | 0 | 6.13:1 “Biblioteca” |
| library | 390 | light | rt=1 | 14 | 0 | 7.34:1 “128 músicas” |
| album | 390 | dark |  | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light |  | 22 | 0 | 4.86:1 “Maré de Março” |
| album | 390 | dark | rt=1 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | rt=1 | 22 | 0 | 4.86:1 “Maré de Março” |
| playlist | 390 | dark |  | 20 | 0 | 6.13:1 “Biblioteca” |
| playlist | 390 | light |  | 20 | 0 | 5.94:1 “Garoa Fina” |
| playlist | 390 | dark | rt=1 | 20 | 0 | 6.13:1 “Biblioteca” |
| playlist | 390 | light | rt=1 | 20 | 0 | 6.06:1 “4:26” |
| artist | 390 | dark |  | 21 | 0 | 6.13:1 “Início” |
| artist | 390 | light |  | 21 | 0 | 6.03:1 “Brasa Viva” |
| artist | 390 | dark | rt=1 | 21 | 0 | 6.13:1 “Início” |
| artist | 390 | light | rt=1 | 21 | 0 | 5.72:1 “3:12” |
| player | 390 | dark |  | 6 | 0 | 7.11:1 “Os Tangarás” |
| player | 390 | light |  | 6 | 0 | 7.11:1 “Os Tangarás” |
| player | 390 | dark | rt=1 | 6 | 0 | 19.07:1 “Tocando de” |
| player | 390 | light | rt=1 | 6 | 0 | 19.07:1 “Tocando de” |
| queue | 390 | dark |  | 21 | 0 | 7.76:1 “Limpar” |
| queue | 390 | light |  | 21 | 0 | 8.10:1 “Quando a fila acabar, o Kazzu continua com músic” |
| queue | 390 | dark | rt=1 | 21 | 0 | 16.15:1 “Limpar” |
| queue | 390 | light | rt=1 | 21 | 0 | 18.42:1 “Limpar” |
| sheet-menu | 390 | dark |  | 11 | 0 | 7.03:1 “Em breve” |
| sheet-menu | 390 | light |  | 11 | 0 | 6.47:1 “Em breve” |
| sheet-menu | 390 | dark | rt=1 | 11 | 0 | 7.03:1 “Em breve” |
| sheet-menu | 390 | light | rt=1 | 11 | 0 | 6.47:1 “Em breve” |
| sheet-playlists | 390 | dark |  | 10 | 0 | 9.10:1 “Adicionar à playlist” |
| sheet-playlists | 390 | light |  | 10 | 0 | 13.52:1 “Nova playlist” |
| sheet-playlists | 390 | dark | rt=1 | 10 | 0 | 16.15:1 “Nova playlist” |
| sheet-playlists | 390 | light | rt=1 | 10 | 0 | 18.42:1 “Nova playlist” |
| sheet-new-playlist | 390 | dark |  | 5 | 0 | 11.07:1 “Nova playlist” |
| sheet-new-playlist | 390 | light |  | 5 | 0 | 13.67:1 “Nome da playlist” |
| sheet-new-playlist | 390 | dark | rt=1 | 5 | 0 | 13.62:1 “Cancelar” |
| sheet-new-playlist | 390 | light | rt=1 | 5 | 0 | 15.03:1 “Cancelar” |
| settings | 390 | dark |  | 22 | 0 | 6.13:1 “Original” |
| settings | 390 | light |  | 22 | 0 | 5.94:1 “Brasa Viva” |
| settings | 390 | dark | rt=1 | 22 | 0 | 6.13:1 “Original” |
| settings | 390 | light | rt=1 | 22 | 0 | 7.34:1 “Reprodução” |
| home-first-run | 390 | dark |  | 16 | 0 | 6.13:1 “Início” |
| home-first-run | 390 | light |  | 16 | 0 | 4.86:1 “Ver tudo” |
| home-first-run | 390 | dark | rt=1 | 16 | 0 | 6.13:1 “Início” |
| home-first-run | 390 | light | rt=1 | 16 | 0 | 4.86:1 “Ver tudo” |
| home-offline | 390 | dark |  | 17 | 0 | 6.13:1 “Início” |
| home-offline | 390 | light |  | 17 | 0 | 4.86:1 “Ver tudo” |
| home-offline | 390 | dark | rt=1 | 17 | 0 | 6.13:1 “Início” |
| home-offline | 390 | light | rt=1 | 17 | 0 | 4.86:1 “Ver tudo” |
| home-error | 390 | dark |  | 7 | 0 | 6.13:1 “Início” |
| home-error | 390 | light |  | 7 | 0 | 7.93:1 “O YouTube Music não respondeu. Sua música do apa” |
| home-error | 390 | dark | rt=1 | 7 | 0 | 6.13:1 “Início” |
| home-error | 390 | light | rt=1 | 7 | 0 | 7.93:1 “O YouTube Music não respondeu. Sua música do apa” |
| search-empty | 390 | dark |  | 8 | 0 | 8.33:1 “Confira a grafia ou tente o nome do artista.” |
| search-empty | 390 | light |  | 8 | 0 | 6.03:1 “Brasa Viva” |
| search-empty | 390 | dark | rt=1 | 8 | 0 | 8.33:1 “Confira a grafia ou tente o nome do artista.” |
| search-empty | 390 | light | rt=1 | 8 | 0 | 7.93:1 “Confira a grafia ou tente o nome do artista.” |
| library-empty | 390 | dark |  | 12 | 0 | 6.13:1 “Biblioteca” |
| library-empty | 390 | light |  | 12 | 0 | 7.34:1 “0 músicas” |
| library-empty | 390 | dark | rt=1 | 12 | 0 | 6.13:1 “Biblioteca” |
| library-empty | 390 | light | rt=1 | 12 | 0 | 7.34:1 “0 músicas” |
| playlist-empty | 390 | dark |  | 9 | 0 | 6.13:1 “Biblioteca” |
| playlist-empty | 390 | light |  | 9 | 0 | 6.03:1 “Brasa Viva” |
| playlist-empty | 390 | dark | rt=1 | 9 | 0 | 6.13:1 “Biblioteca” |
| playlist-empty | 390 | light | rt=1 | 9 | 0 | 7.93:1 “Toque em Mais numa música e escolha Adicionar à ” |
| home | 360 | dark |  | 13 | 0 | 6.13:1 “Início” |
| home | 768 | dark |  | 23 | 0 | 6.13:1 “Início” |
| home | 1280 | dark |  | 23 | 0 | 6.13:1 “Início” |
| home | 1280 | light |  | 23 | 0 | 4.86:1 “Ver tudo” |
| explore | 360 | dark |  | 13 | 0 | 6.13:1 “Explorar” |
| explore | 768 | dark |  | 25 | 0 | 6.13:1 “Explorar” |
| explore | 1280 | dark |  | 20 | 0 | 6.13:1 “Explorar” |
| explore | 1280 | light |  | 20 | 0 | 7.93:1 “Suas playlists” |
| library | 360 | dark |  | 13 | 0 | 6.13:1 “Biblioteca” |
| library | 768 | dark |  | 25 | 0 | 6.13:1 “Biblioteca” |
| library | 1280 | dark |  | 26 | 0 | 6.13:1 “Biblioteca” |
| library | 1280 | light |  | 26 | 0 | 7.34:1 “128 músicas” |
| album | 360 | dark |  | 18 | 0 | 6.13:1 “Início” |
| album | 768 | dark |  | 29 | 0 | 6.13:1 “Início” |
| album | 1280 | dark |  | 53 | 0 | 4.96:1 “10” |
| album | 1280 | light |  | 53 | 0 | 4.86:1 “Maré de Março” |
| player | 360 | dark |  | 6 | 0 | 7.24:1 “Os Tangarás” |
| player | 768 | dark |  | 6 | 0 | 10.55:1 “Pimenta” |
| player | 1280 | dark |  | 6 | 0 | 10.71:1 “Pimenta” |
| player | 1280 | light |  | 6 | 0 | 10.71:1 “Pimenta” |
| settings | 360 | dark |  | 20 | 0 | 6.13:1 “Original” |
| settings | 768 | dark |  | 28 | 0 | 6.13:1 “Original” |
| settings | 1280 | dark |  | 30 | 0 | 6.13:1 “Original” |
| settings | 1280 | light |  | 30 | 0 | 7.08:1 “Biblioteca” |
| search-results | 360 | dark |  | 18 | 0 | 6.13:1 “Tudo” |
| search-results | 768 | dark |  | 28 | 0 | 6.13:1 “Tudo” |
| search-results | 1280 | dark |  | 30 | 0 | 6.13:1 “Tudo” |
| search-results | 1280 | light |  | 30 | 0 | 6.83:1 “Nina Pomar” |
| home | 390 | dark | scroll=120 | 19 | 0 | 6.13:1 “Início” |
| home | 390 | light | scroll=120 | 19 | 0 | 4.86:1 “Ver tudo” |
| home | 390 | dark | scroll=260 | 20 | 0 | 6.13:1 “Início” |
| home | 390 | light | scroll=260 | 20 | 0 | 4.86:1 “Ver tudo” |
| home | 390 | dark | scroll=420 | 22 | 0 | 6.13:1 “Início” |
| home | 390 | light | scroll=420 | 22 | 0 | 4.86:1 “Ver tudo” |
| explore | 390 | dark | scroll=120 | 17 | 0 | 6.13:1 “Explorar” |
| explore | 390 | light | scroll=120 | 17 | 0 | 5.86:1 “Brasa Viva” |
| explore | 390 | dark | scroll=260 | 20 | 0 | 6.13:1 “Explorar” |
| explore | 390 | light | scroll=260 | 20 | 0 | 5.82:1 “Caio Fader” |
| explore | 390 | dark | scroll=420 | 26 | 0 | 6.13:1 “Explorar” |
| explore | 390 | light | scroll=420 | 26 | 0 | 6.12:1 “Brasa Viva” |
| library | 390 | dark | scroll=120 | 16 | 0 | 6.13:1 “Biblioteca” |
| library | 390 | light | scroll=120 | 16 | 0 | 6.03:1 “Brasa Viva” |
| library | 390 | dark | scroll=260 | 22 | 0 | 6.13:1 “Biblioteca” |
| library | 390 | light | scroll=260 | 22 | 0 | 5.94:1 “312 músicas, lida hoje” |
| library | 390 | dark | scroll=420 | 22 | 0 | 6.13:1 “Biblioteca” |
| library | 390 | light | scroll=420 | 22 | 0 | 5.85:1 “Brasa Viva” |
| album | 390 | dark | scroll=120 | 30 | 0 | 6.13:1 “Início” |
| album | 390 | light | scroll=120 | 30 | 0 | 4.86:1 “Maré de Março” |
| album | 390 | dark | scroll=260 | 37 | 0 | 6.13:1 “Início” |
| album | 390 | light | scroll=260 | 37 | 0 | 4.86:1 “Maré de Março” |
| album | 390 | dark | scroll=420 | 45 | 0 | 6.13:1 “Início” |
| album | 390 | light | scroll=420 | 45 | 0 | 4.69:1 “Maré de Março” |
| playlist | 390 | dark | scroll=120 | 25 | 0 | 6.13:1 “Biblioteca” |
| playlist | 390 | light | scroll=120 | 25 | 0 | 6.03:1 “Brasa Viva” |
| playlist | 390 | dark | scroll=260 | 30 | 0 | 6.13:1 “Biblioteca” |
| playlist | 390 | light | scroll=260 | 30 | 0 | 6.03:1 “Brasa Viva” |
| playlist | 390 | dark | scroll=420 | 33 | 0 | 6.13:1 “Biblioteca” |
| playlist | 390 | light | scroll=420 | 33 | 0 | 6.03:1 “Brasa Viva” |
| settings | 390 | dark | scroll=120 | 24 | 0 | 6.13:1 “Original” |
| settings | 390 | light | scroll=120 | 24 | 0 | 5.85:1 “Brasa Viva” |
| settings | 390 | dark | scroll=260 | 22 | 0 | 6.13:1 “Escuro” |
| settings | 390 | light | scroll=260 | 22 | 0 | 5.88:1 “Brasa Viva” |
| settings | 390 | dark | scroll=420 | 27 | 0 | 6.63:1 “Escuro” |
| settings | 390 | light | scroll=420 | 27 | 0 | 5.85:1 “Brasa Viva” |
| player | 390 | dark | c=01 | 6 | 0 | 7.61:1 “Clara Vento” |
| queue | 390 | dark | c=01 | 21 | 0 | 7.14:1 “Limpar” |
| album | 390 | dark | c=01 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=01 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=03 | 6 | 0 | 6.99:1 “Maré Alta Trio” |
| queue | 390 | dark | c=03 | 21 | 0 | 7.97:1 “Limpar” |
| album | 390 | dark | c=03 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=03 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=06 | 6 | 0 | 6.52:1 “Jangada Elétrica” |
| queue | 390 | dark | c=06 | 21 | 0 | 6.65:1 “Limpar” |
| album | 390 | dark | c=06 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=06 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=07 | 6 | 0 | 4.97:1 “Lia Marfim” |
| queue | 390 | dark | c=07 | 21 | 0 | 6.78:1 “Limpar” |
| album | 390 | dark | c=07 | 22 | 0 | 5.54:1 “Single de Lia Marfim” |
| album | 390 | light | c=07 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=08 | 6 | 0 | 5.61:1 “Duna Rosa” |
| queue | 390 | dark | c=08 | 21 | 0 | 7.26:1 “Limpar” |
| album | 390 | dark | c=08 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=08 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=09 | 6 | 0 | 14.90:1 “Tocando de” |
| queue | 390 | dark | c=09 | 21 | 0 | 8.93:1 “Limpar” |
| album | 390 | dark | c=09 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=09 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=13 | 6 | 0 | 4.89:1 “Selo Amarelo” |
| queue | 390 | dark | c=13 | 21 | 0 | 6.65:1 “Limpar” |
| album | 390 | dark | c=13 | 22 | 0 | 5.27:1 “Coletânea de Selo Amarelo” |
| album | 390 | light | c=13 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=16 | 6 | 0 | 7.88:1 “Nina Pomar” |
| queue | 390 | dark | c=16 | 21 | 0 | 6.50:1 “Limpar” |
| album | 390 | dark | c=16 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=16 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=17 | 6 | 0 | 4.69:1 “Tocando de” |
| queue | 390 | dark | c=17 | 21 | 0 | 6.05:1 “Limpar” |
| album | 390 | dark | c=17 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=17 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=20 | 6 | 0 | 4.90:1 “Laboratório Solar” |
| queue | 390 | dark | c=20 | 21 | 0 | 8.38:1 “Limpar” |
| album | 390 | dark | c=20 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=20 | 22 | 0 | 4.86:1 “Maré de Março” |
| player | 390 | dark | c=22 | 6 | 0 | 5.95:1 “Margem” |
| queue | 390 | dark | c=22 | 21 | 0 | 6.77:1 “Limpar” |
| album | 390 | dark | c=22 | 22 | 0 | 6.13:1 “Início” |
| album | 390 | light | c=22 | 22 | 0 | 4.86:1 “Maré de Março” |
