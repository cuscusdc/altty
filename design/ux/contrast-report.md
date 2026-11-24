# Contraste medido nos protótipos renderizados

Gerado por `design/tools/check_render.py` (Chromium headless + Pillow). Para cada texto visível: contraste da cor do texto contra o **pior** pixel (percentil 5) do fundo atrás dele, já com o vidro, o palco e o desfoque aplicados. Limite WCAG 2.2 AA: 4,5:1 (3:1 para texto grande).

**2122 textos medidos em 117 capturas; 0 abaixo do limite; menor contraste 4.83:1.**

| Tela | Largura | Tema | Variante | Textos | Reprovados | Pior caso |
|---|---|---|---|---|---|---|
| home | 390 | dark |  | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light |  | 14 | 0 | 5.98:1 “Do YouTube Music” |
| home | 360 | dark |  | 16 | 0 | 4.85:1 “Quase Verão” |
| home | 768 | dark |  | 27 | 0 | 6.19:1 “Início” |
| home | 768 | light |  | 27 | 0 | 4.99:1 “Ver mais” |
| home | 1280 | dark |  | 19 | 0 | 6.29:1 “Início” |
| home | 1280 | light |  | 19 | 0 | 5.98:1 “Do YouTube Music” |
| home | 390 | dark | sem transparência  | 14 | 0 | 7.56:1 “Início” |
| home | 390 | light | sem transparência  | 14 | 0 | 5.98:1 “Do YouTube Music” |
| explore | 390 | dark |  | 19 | 0 | 6.26:1 “Nina Sabiá” |
| explore | 390 | light |  | 19 | 0 | 5.98:1 “Do YouTube Music” |
| explore | 360 | dark |  | 17 | 0 | 6.56:1 “Explorar” |
| explore | 768 | dark |  | 25 | 0 | 4.84:1 “Quase Verão” |
| explore | 768 | light |  | 25 | 0 | 5.98:1 “Do YouTube Music” |
| explore | 1280 | dark |  | 27 | 0 | 6.69:1 “Explorar” |
| explore | 1280 | light |  | 27 | 0 | 5.98:1 “Do YouTube Music” |
| explore | 390 | dark | sem transparência  | 19 | 0 | 7.56:1 “Explorar” |
| explore | 390 | light | sem transparência  | 19 | 0 | 5.98:1 “Do YouTube Music” |
| library | 390 | dark |  | 18 | 0 | 7.33:1 “Biblioteca” |
| library | 390 | light |  | 18 | 0 | 7.74:1 “2 músicas locais” |
| library | 360 | dark |  | 15 | 0 | 7.80:1 “Biblioteca” |
| library | 768 | dark |  | 19 | 0 | 7.11:1 “Biblioteca” |
| library | 768 | light |  | 19 | 0 | 7.94:1 “10 álbuns” |
| library | 1280 | dark |  | 32 | 0 | 7.11:1 “Biblioteca” |
| library | 1280 | light |  | 32 | 0 | 7.54:1 “Tomás Reis” |
| library | 390 | dark | sem transparência  | 18 | 0 | 7.56:1 “Biblioteca” |
| library | 390 | light | sem transparência  | 18 | 0 | 7.74:1 “2 músicas locais” |
| album | 390 | dark |  | 22 | 0 | 7.23:1 “Biblioteca” |
| album | 390 | light |  | 22 | 0 | 5.98:1 “Do YouTube Music” |
| album | 360 | dark |  | 13 | 0 | 7.33:1 “Biblioteca” |
| album | 768 | dark |  | 32 | 0 | 7.11:1 “Biblioteca” |
| album | 768 | light |  | 32 | 0 | 5.98:1 “Do YouTube Music” |
| album | 1280 | dark |  | 23 | 0 | 7.11:1 “Biblioteca” |
| album | 1280 | light |  | 23 | 0 | 5.98:1 “Do YouTube Music” |
| album | 390 | dark | sem transparência  | 22 | 0 | 7.56:1 “Biblioteca” |
| album | 390 | light | sem transparência  | 22 | 0 | 5.98:1 “Do YouTube Music” |
| search | 390 | dark |  | 20 | 0 | 10.48:1 “Limpar” |
| search | 390 | light |  | 20 | 0 | 5.03:1 “Limpar” |
| search | 360 | dark |  | 15 | 0 | 10.37:1 “Quase Verão” |
| search | 768 | dark |  | 23 | 0 | 7.57:1 “Buscar” |
| search | 768 | light |  | 23 | 0 | 5.03:1 “Limpar” |
| search | 1280 | dark |  | 25 | 0 | 7.69:1 “Buscar” |
| search | 1280 | light |  | 25 | 0 | 5.03:1 “Limpar” |
| search | 390 | dark | sem transparência  | 20 | 0 | 10.48:1 “Limpar” |
| search | 390 | light | sem transparência  | 20 | 0 | 5.03:1 “Limpar” |
| player | 390 | dark |  | 12 | 0 | 10.64:1 “Tocando de” |
| player | 390 | light |  | 12 | 0 | 10.64:1 “Tocando de” |
| player | 360 | dark |  | 10 | 0 | 10.68:1 “Tocando de” |
| player | 768 | dark |  | 12 | 0 | 10.70:1 “Tocando de” |
| player | 768 | light |  | 12 | 0 | 10.70:1 “Tocando de” |
| player | 1280 | dark |  | 12 | 0 | 10.79:1 “Nina Sabiá” |
| player | 1280 | light |  | 12 | 0 | 10.79:1 “Nina Sabiá” |
| player | 390 | dark | sem transparência  | 12 | 0 | 10.64:1 “Tocando de” |
| player | 390 | light | sem transparência  | 12 | 0 | 10.64:1 “Tocando de” |
| queue | 390 | dark |  | 18 | 0 | 4.83:1 “Limpar fila” |
| queue | 390 | light |  | 18 | 0 | 4.83:1 “Limpar fila” |
| queue | 360 | dark |  | 18 | 0 | 4.95:1 “Limpar fila” |
| queue | 768 | dark |  | 18 | 0 | 4.97:1 “Limpar fila” |
| queue | 768 | light |  | 18 | 0 | 4.97:1 “Limpar fila” |
| queue | 1280 | dark |  | 18 | 0 | 5.52:1 “Limpar fila” |
| queue | 1280 | light |  | 18 | 0 | 5.52:1 “Limpar fila” |
| queue | 390 | dark | sem transparência  | 18 | 0 | 8.44:1 “Nina Sabiá” |
| queue | 390 | light | sem transparência  | 18 | 0 | 8.44:1 “Nina Sabiá” |
| sheets | 390 | dark |  | 10 | 0 | 6.16:1 “Kaito e Vê” |
| sheets | 390 | light |  | 10 | 0 | 7.63:1 “Kaito e Vê” |
| sheets | 360 | dark |  | 10 | 0 | 5.54:1 “Kaito e Vê” |
| sheets | 768 | dark |  | 10 | 0 | 7.01:1 “Kaito e Vê” |
| sheets | 768 | light |  | 10 | 0 | 8.20:1 “Wi-Fi” |
| sheets | 1280 | dark |  | 10 | 0 | 5.53:1 “Kaito e Vê” |
| sheets | 1280 | light |  | 10 | 0 | 7.81:1 “Kaito e Vê” |
| sheets | 390 | dark | sem transparência  | 10 | 0 | 12.25:1 “Kaito e Vê” |
| sheets | 390 | light | sem transparência  | 10 | 0 | 9.67:1 “Kaito e Vê” |
| settings | 390 | dark |  | 24 | 0 | 8.34:1 “Ativo e padrão. Sem login.” |
| settings | 390 | light |  | 24 | 0 | 7.17:1 “Fontes de música” |
| settings | 360 | dark |  | 18 | 0 | 8.34:1 “Ativo e padrão. Sem login.” |
| settings | 768 | dark |  | 36 | 0 | 5.82:1 “Automático” |
| settings | 768 | light |  | 36 | 0 | 5.85:1 “Privacidade” |
| settings | 1280 | dark |  | 30 | 0 | 7.80:1 “1:02” |
| settings | 1280 | light |  | 30 | 0 | 7.11:1 “Aparência” |
| settings | 390 | dark | sem transparência  | 24 | 0 | 8.34:1 “Ativo e padrão. Sem login.” |
| settings | 390 | light | sem transparência  | 24 | 0 | 7.17:1 “Fontes de música” |
| states | 390 | dark |  | 13 | 0 | 7.94:1 “Biblioteca” |
| states | 390 | light |  | 13 | 0 | 8.01:1 “Curta uma música ou adicione uma pasta e ela apa” |
| states | 360 | dark |  | 12 | 0 | 7.94:1 “Biblioteca” |
| states | 768 | dark |  | 17 | 0 | 7.11:1 “Biblioteca” |
| states | 768 | light |  | 17 | 0 | 8.01:1 “Curta uma música ou adicione uma pasta e ela apa” |
| states | 1280 | dark |  | 19 | 0 | 7.11:1 “Biblioteca” |
| states | 1280 | light |  | 19 | 0 | 8.01:1 “Curta uma música ou adicione uma pasta e ela apa” |
| states | 390 | dark | sem transparência  | 13 | 0 | 7.56:1 “Biblioteca” |
| states | 390 | light | sem transparência  | 13 | 0 | 8.01:1 “Curta uma música ou adicione uma pasta e ela apa” |
| home | 390 | dark | scroll=420 | 25 | 0 | 5.31:1 “Início” |
| home | 390 | light | scroll=420 | 25 | 0 | 5.03:1 “Ver mais” |
| library | 390 | dark | scroll=420 | 30 | 0 | 7.77:1 “Biblioteca” |
| library | 390 | light | scroll=420 | 30 | 0 | 8.01:1 “10 álbuns” |
| explore | 390 | dark | scroll=420 | 20 | 0 | 7.33:1 “Explorar” |
| explore | 390 | light | scroll=420 | 20 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=01 | 12 | 0 | 10.71:1 “Tocando de” |
| home | 390 | dark | c=01 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=01 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=03 | 12 | 0 | 9.92:1 “Tocando de” |
| home | 390 | dark | c=03 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=03 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=05 | 12 | 0 | 10.04:1 “Tocando de” |
| home | 390 | dark | c=05 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=05 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=06 | 12 | 0 | 12.65:1 “Tocando de” |
| home | 390 | dark | c=06 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=06 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=08 | 12 | 0 | 11.38:1 “Tocando de” |
| home | 390 | dark | c=08 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=08 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=09 | 12 | 0 | 9.49:1 “Tocando de” |
| home | 390 | dark | c=09 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=09 | 14 | 0 | 5.98:1 “Do YouTube Music” |
| player | 390 | dark | c=10 | 12 | 0 | 11.70:1 “Tocando de” |
| home | 390 | dark | c=10 | 14 | 0 | 6.52:1 “Quase Verão” |
| home | 390 | light | c=10 | 14 | 0 | 5.98:1 “Do YouTube Music” |
