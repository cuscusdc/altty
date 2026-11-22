# Contraste medido nos protótipos renderizados

Gerado por `design/tools/check_render.py` (Chromium headless + Pillow). Para cada texto visível: contraste da cor do texto contra o **pior** pixel (percentil 5) do fundo atrás dele, já com o vidro, o palco e o desfoque aplicados. Limite WCAG 2.2 AA: 4,5:1 (3:1 para texto grande).

**2111 textos medidos em 117 capturas; 0 abaixo do limite; menor contraste 4.67:1.**

| Tela | Largura | Tema | Variante | Textos | Reprovados | Pior caso |
|---|---|---|---|---|---|---|
| home | 390 | dark |  | 15 | 0 | 6.14:1 “Treino” |
| home | 390 | light |  | 15 | 0 | 6.35:1 “Ver mais” |
| home | 360 | dark |  | 14 | 0 | 6.58:1 “Foco” |
| home | 768 | dark |  | 30 | 0 | 4.93:1 “Início” |
| home | 768 | light |  | 30 | 0 | 5.36:1 “Novo no YouTube Music” |
| home | 1280 | dark |  | 19 | 0 | 5.09:1 “Início” |
| home | 1280 | light |  | 19 | 0 | 5.11:1 “Nina Sabiá” |
| home | 390 | dark | sem transparência  | 15 | 0 | 6.14:1 “Treino” |
| home | 390 | light | sem transparência  | 15 | 0 | 6.37:1 “Ver mais” |
| explore | 390 | dark |  | 20 | 0 | 6.33:1 “Ilha Branca” |
| explore | 390 | light |  | 20 | 0 | 5.23:1 “Ilha Branca” |
| explore | 360 | dark |  | 16 | 0 | 6.78:1 “Ilha Branca” |
| explore | 768 | dark |  | 27 | 0 | 5.35:1 “Explorar” |
| explore | 768 | light |  | 27 | 0 | 4.97:1 “Rádio Cobre” |
| explore | 1280 | dark |  | 27 | 0 | 5.47:1 “Explorar” |
| explore | 1280 | light |  | 27 | 0 | 5.48:1 “Rádio Cobre” |
| explore | 390 | dark | sem transparência  | 20 | 0 | 6.33:1 “Ilha Branca” |
| explore | 390 | light | sem transparência  | 20 | 0 | 5.23:1 “Ilha Branca” |
| library | 390 | dark |  | 15 | 0 | 7.80:1 “Criar playlist” |
| library | 390 | light |  | 15 | 0 | 5.70:1 “Criar playlist” |
| library | 360 | dark |  | 14 | 0 | 7.80:1 “Criar playlist” |
| library | 768 | dark |  | 18 | 0 | 5.72:1 “Biblioteca” |
| library | 768 | light |  | 18 | 0 | 5.70:1 “Criar playlist” |
| library | 1280 | dark |  | 31 | 0 | 5.84:1 “Biblioteca” |
| library | 1280 | light |  | 31 | 0 | 5.70:1 “Criar playlist” |
| library | 390 | dark | sem transparência  | 15 | 0 | 7.80:1 “Criar playlist” |
| library | 390 | light | sem transparência  | 15 | 0 | 5.70:1 “Criar playlist” |
| album | 390 | dark |  | 21 | 0 | 5.44:1 “Nina Sabiá” |
| album | 390 | light |  | 21 | 0 | 4.90:1 “Nina Sabiá” |
| album | 360 | dark |  | 13 | 0 | 5.97:1 “Nina Sabiá” |
| album | 768 | dark |  | 32 | 0 | 5.45:1 “Nina Sabiá” |
| album | 768 | light |  | 32 | 0 | 4.70:1 “Nina Sabiá” |
| album | 1280 | dark |  | 25 | 0 | 5.84:1 “Biblioteca” |
| album | 1280 | light |  | 25 | 0 | 4.99:1 “Nina Sabiá” |
| album | 390 | dark | sem transparência  | 21 | 0 | 5.44:1 “Nina Sabiá” |
| album | 390 | light | sem transparência  | 21 | 0 | 4.90:1 “Nina Sabiá” |
| search | 390 | dark |  | 20 | 0 | 6.26:1 “Limpar” |
| search | 390 | light |  | 20 | 0 | 5.11:1 “Limpar” |
| search | 360 | dark |  | 16 | 0 | 6.22:1 “Playlists” |
| search | 768 | dark |  | 23 | 0 | 6.13:1 “Buscar” |
| search | 768 | light |  | 23 | 0 | 5.01:1 “Limpar” |
| search | 1280 | dark |  | 25 | 0 | 6.26:1 “Limpar” |
| search | 1280 | light |  | 25 | 0 | 5.06:1 “Limpar” |
| search | 390 | dark | sem transparência  | 20 | 0 | 6.26:1 “Limpar” |
| search | 390 | light | sem transparência  | 20 | 0 | 5.11:1 “Limpar” |
| player | 390 | dark |  | 12 | 0 | 7.52:1 “Nina Sabiá” |
| player | 390 | light |  | 12 | 0 | 7.52:1 “Nina Sabiá” |
| player | 360 | dark |  | 10 | 0 | 7.55:1 “Este celular” |
| player | 768 | dark |  | 12 | 0 | 7.45:1 “Nina Sabiá” |
| player | 768 | light |  | 12 | 0 | 7.45:1 “Nina Sabiá” |
| player | 1280 | dark |  | 12 | 0 | 7.28:1 “Nina Sabiá” |
| player | 1280 | light |  | 12 | 0 | 7.28:1 “Nina Sabiá” |
| player | 390 | dark | sem transparência  | 12 | 0 | 7.52:1 “Nina Sabiá” |
| player | 390 | light | sem transparência  | 12 | 0 | 7.52:1 “Nina Sabiá” |
| queue | 390 | dark |  | 18 | 0 | 6.31:1 “Nina Sabiá” |
| queue | 390 | light |  | 18 | 0 | 6.65:1 “Nina Sabiá” |
| queue | 360 | dark |  | 18 | 0 | 6.51:1 “Nina Sabiá” |
| queue | 768 | dark |  | 18 | 0 | 6.09:1 “Nina Sabiá” |
| queue | 768 | light |  | 18 | 0 | 6.69:1 “Nina Sabiá” |
| queue | 1280 | dark |  | 18 | 0 | 6.41:1 “Nina Sabiá” |
| queue | 1280 | light |  | 18 | 0 | 6.64:1 “Nina Sabiá” |
| queue | 390 | dark | sem transparência  | 18 | 0 | 9.89:1 “Nina Sabiá” |
| queue | 390 | light | sem transparência  | 18 | 0 | 7.62:1 “Nina Sabiá” |
| sheets | 390 | dark |  | 10 | 0 | 8.55:1 “Kaito e Vê” |
| sheets | 390 | light |  | 10 | 0 | 6.27:1 “Wi-Fi” |
| sheets | 360 | dark |  | 10 | 0 | 8.56:1 “Kaito e Vê” |
| sheets | 768 | dark |  | 10 | 0 | 8.93:1 “Kaito e Vê” |
| sheets | 768 | light |  | 10 | 0 | 6.31:1 “Wi-Fi” |
| sheets | 1280 | dark |  | 10 | 0 | 8.96:1 “Kaito e Vê” |
| sheets | 1280 | light |  | 10 | 0 | 6.32:1 “Wi-Fi” |
| sheets | 390 | dark | sem transparência  | 10 | 0 | 10.47:1 “Wi-Fi” |
| sheets | 390 | light | sem transparência  | 10 | 0 | 7.96:1 “Wi-Fi” |
| settings | 390 | dark |  | 25 | 0 | 5.64:1 “Fontes de música” |
| settings | 390 | light |  | 25 | 0 | 5.20:1 “Fontes de música” |
| settings | 360 | dark |  | 19 | 0 | 5.75:1 “Fontes de música” |
| settings | 768 | dark |  | 37 | 0 | 4.94:1 “Automático” |
| settings | 768 | light |  | 37 | 0 | 5.14:1 “Fontes de música” |
| settings | 1280 | dark |  | 30 | 0 | 5.69:1 “Nina Sabiá” |
| settings | 1280 | light |  | 30 | 0 | 5.07:1 “Fontes de música” |
| settings | 390 | dark | sem transparência  | 25 | 0 | 5.64:1 “Fontes de música” |
| settings | 390 | light | sem transparência  | 25 | 0 | 5.20:1 “Fontes de música” |
| states | 390 | dark |   | erro: sem probe | | |
| states | 390 | light |  | 13 | 0 | 5.70:1 “Adicionar pasta” |
| states | 360 | dark |  | 13 | 0 | 6.07:1 “Carregando” |
| states | 768 | dark |  | 17 | 0 | 5.72:1 “Biblioteca” |
| states | 768 | light |  | 17 | 0 | 5.70:1 “Adicionar pasta” |
| states | 1280 | dark |  | 19 | 0 | 5.84:1 “Biblioteca” |
| states | 1280 | light |  | 19 | 0 | 5.70:1 “Adicionar pasta” |
| states | 390 | dark | sem transparência  | 13 | 0 | 5.96:1 “Carregando” |
| states | 390 | light | sem transparência  | 13 | 0 | 5.70:1 “Adicionar pasta” |
| home | 390 | dark | scroll=420 | 28 | 0 | 5.37:1 “Novo no YouTube Music” |
| home | 390 | light | scroll=420 | 28 | 0 | 4.67:1 “Nina Sabiá” |
| library | 390 | dark | scroll=420 | 25 | 0 | 7.42:1 “Adicionar pasta” |
| library | 390 | light | scroll=420 | 25 | 0 | 6.04:1 “Nina Sabiá” |
| explore | 390 | dark | scroll=420 | 20 | 0 | 6.79:1 “Do YouTube Music” |
| explore | 390 | light | scroll=420 | 20 | 0 | 5.91:1 “Lia Cantos” |
| player | 390 | dark | c=01 | 12 | 0 | 8.10:1 “Aurora Vento” |
| home | 390 | dark | c=01 | 15 | 0 | 4.85:1 “Treino” |
| home | 390 | light | c=01 | 15 | 0 | 6.16:1 “Ver mais” |
| player | 390 | dark | c=03 | 12 | 0 | 5.93:1 “Lia Cantos” |
| home | 390 | dark | c=03 | 15 | 0 | 5.15:1 “Treino” |
| home | 390 | light | c=03 | 15 | 0 | 6.35:1 “Ver mais” |
| player | 390 | dark | c=05 | 12 | 0 | 7.29:1 “Tomás Reis” |
| home | 390 | dark | c=05 | 15 | 0 | 6.28:1 “Treino” |
| home | 390 | light | c=05 | 15 | 0 | 6.38:1 “Nina Sabiá” |
| player | 390 | dark | c=06 | 12 | 0 | 10.88:1 “Este celular” |
| home | 390 | dark | c=06 | 15 | 0 | 8.14:1 “Treino” |
| home | 390 | light | c=06 | 15 | 0 | 5.20:1 “Treino” |
| player | 390 | dark | c=08 | 12 | 0 | 9.56:1 “Kaito e Vê” |
| home | 390 | dark | c=08 | 15 | 0 | 6.24:1 “Treino” |
| home | 390 | light | c=08 | 15 | 0 | 6.17:1 “Ver mais” |
| player | 390 | dark | c=09 | 12 | 0 | 6.51:1 “Ilha Branca” |
| home | 390 | dark |  c=09 | erro: sem probe | | |
| home | 390 | light | c=09 | 15 | 0 | 6.27:1 “Ver mais” |
| player | 390 | dark | c=10 | 12 | 0 | 9.45:1 “Este celular” |
| home | 390 | dark | c=10 | 15 | 0 | 6.06:1 “Treino” |
| home | 390 | light | c=10 | 15 | 0 | 6.23:1 “Ver mais” |
