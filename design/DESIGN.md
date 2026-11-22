# Kazzu — design system (v2: mobile-first, vidro real)

Versão 0.2 · Slint 1.19 · pt-BR. Fonte dos números: `design/tokens.json` (gerador: `tools/gen_tokens.py`). Telas navegáveis: `design/prototypes/index.html` (390 e 360 px primeiro; depois 768 e 1280). Planejamento de UX (personas, fluxos, estados, gestos, acessibilidade, validação sem telemetria): `design/ux/UX_PLAN.md`. Medidas e árvores de componentes: `design/SLINT_SPEC.md`.

**Por que existe a v2.** O autor reprovou a v1 por três motivos: não era *mobile-first*, o UX não tinha planejamento de verdade, e **não parecia glassmorfismo**. Este documento corrige os três e passa a tratar o YouTube Music como fonte padrão, ativa, sem login.

## 1. A ideia

> **A capa é a luz; o vidro é o que a deixa passar.**

O Kazzu é um palco colorido atrás e painéis de vidro na frente. A cor do palco nasce da capa da faixa atual (ou do âmbar da marca quando nada toca). Os painéis são translúcidos de verdade: deixam a lista e o palco aparecerem desfocados por trás, com borda de luz, brilho no topo e sombra tingida. Tudo o que não é vidro é plano e quieto.

O que o app tem de diferente (e o design mostra, não descreve):

- **Fidelidade visível.** Cada faixa tem o selo do que é de verdade (`OPUS 160`, `FLAC 24/96`). O player diz "Original, sem equalizador" quando o caminho é bit-perfect.
- **Vidro só onde flutua** (barra, mini player, cartões, folhas), em **3 níveis de profundidade**. Listas ficam planas.
- **Polegar primeiro.** Ações frequentes na metade de baixo; o topo só tem identidade e Ajustes.
- **Quietude.** Sem loops, sem confete, sem pesquisa de opinião, sem selo "novo".

### 1.1 Por que a v1 não parecia vidro (diagnóstico)

| Causa na v1 | Efeito | Correção na v2 |
|---|---|---|
| Fundo `#0c0806` quase preto e opaco | o blur não tinha o que borrar; painéis viravam "cartão escuro" | **Palco vivo**: malha de 4 manchões + a capa em 150%, tudo com croma alto (L 41–50%, C 0,14–0,19) |
| Fill de grafite escuro (`#251e19` a 55%) | tom marrom-cinza sobre preto: nenhum contraste de material | Fill = **tint do matiz da capa**, 40–72% conforme o nível; o palco colorido transparece |
| Sem saturação no backdrop | cores desbotadas atrás do painel | `saturate(1,45)` no protótipo; **no Slint, palco já saturado** (compensação) |
| Borda única de 6% de branco | sem arestas | **Borda de luz em gradiente** (forte no canto de onde vem a luz) + **fio especular** |
| Uma só receita para tudo | tudo no mesmo plano, sem profundidade | **3 níveis** (conteúdo, cromo flutuante, folhas) com blur, opacidade e sombra crescentes |
| Véu de 78% para `bg` no ambiente | a cor da capa sumia depois de 40% da altura | Palco **ocupa a tela inteira**, fixo; só um véu de 55% no rodapé |

## 2. Princípios (e o que traduzimos das skills `hallmark` e `taste-skill`)

As skills são de web; o que vale para um app nativo foi aplicado:

| Princípio | Como aparece no Kazzu |
|---|---|
| **Direção própria, nada de template** | Marca desenhada à mão (K de barras), âmbar + capas fictícias abstratas, par Hanken Grotesk + Geist Mono. Nenhuma arte ou desenho copiado de Bitchord, Umihi, Intertune ou Metrolist: só **convenções** (barra flutuante, mini player, folhas, chips) |
| **Hierarquia por escala e peso** | 3 pesos (400/600/700), escala 12–34 (26 no player), **um** botão primário cheio por tela |
| **Cor com função** | Acento: ativo, foco, primário, progresso, coração. **Nunca em texto corrido** (sobre o palco só ícones e preenchimentos) |
| **Contraste e foco são requisitos** | AA **medido em captura real** (`ux/contrast-report.md`), foco de 2 px que aparece na hora, alvos de 48 |
| **Movimento reduzido de verdade** | Tudo que anima lê `Theme.reduce-motion`; nenhum loop infinito |
| **Anti "cara de IA"** | Sem gradiente roxo-azul (o palco usa o matiz da capa; sem capa, âmbar), sem *glow* (a sombra é tingida, não luminosa), sem emoji como ícone, sem cartão dentro de cartão (filhos do vidro são simples), sem rótulos em caixa-alta espaçada, sem *eyebrow*, sem `Artista · Álbum` (vírgula), sem pontos decorativos de status, sem números inventados |
| **Textos específicos** | Títulos nomeiam coisas; botões = verbo + objeto; vazio diz o que falta, por quê e dá uma ação (UX_PLAN §7.3) |
| **Uma ideia por tela** | Início: o destaque. Player: a capa. Biblioteca: as quatro coleções. Busca: o campo |

## 3. Cor

Cores em `tokens.json` (OKLCH ou hex) → hex em `tokens.slint` (`Theme.*`) e `tokens.css`. Claro e escuro são dois renderings do mesmo sistema (mesmo matiz, luminosidades re-derivadas).

### 3.1 Neutros

| Token | Escuro | Claro | Uso |
|---|---|---|---|
| `stage-base` | `oklch(20% .05 h)` | `oklch(94% .03 h)` | base do palco (matiz da capa) |
| `bg` | `oklch(15% .022 55)` | `oklch(95% .02 85)` | fallback sólido, janela sem palco |
| `surface` … `surface-3` | L 20 / 25 / 31% | L 93,5 / 90,5 / 87% | poços e estados |
| `ink` | `oklch(98%)` | `oklch(20%)` | texto principal |
| `ink-2` | `oklch(90%)` | `oklch(38%)` | secundário (**único** secundário permitido direto sobre o palco) |
| `ink-3` | `oklch(82%)` | `oklch(43%)` | meta; **só sobre vidro** |
| `line` / `line-strong` | branco 14% / 26% | tinta 10% / 20% | divisores / borda de campo |

### 3.2 Palco (fundo ambiente vivo)

Quatro manchões `stage-a..d` + a capa borrada. Fórmula (mesma no CSS dos protótipos e no Rust): dado o **matiz `h`** do acento dinâmico (58° = âmbar quando a capa é cinza), `stage-X = oklch(L, C, h + dh)` com:

| Manchão | Escuro `[L, C, dh]` | Claro `[L, C, dh]` | Posição (x, y) |
|---|---|---|---|
| a | 0,50 · 0,19 · 0° | 0,86 · 0,09 · 0° | 12%, 8% |
| b | 0,45 · 0,18 · −36° | 0,86 · 0,085 · −36° | 98%, 34% |
| c | 0,41 · 0,14 · +147° | 0,89 · 0,06 · +147° | 10%, 72% |
| d | 0,44 · 0,15 · +37° | 0,90 · 0,08 · +37° | 88%, 96% |

Em cima, a **capa em 150%**, desfocada (CSS: `blur(64px) saturate(1,5) brightness(.62)`; Slint: imagem 96×96 já desfocada pelo Rust), `opacity` 0,50 escuro / 0,35 claro, e um véu de 55% no rodapé. O L máximo do escuro (0,50) é **limite de contraste**, não gosto: acima disso o texto branco no vidro perde o AA. Vivacidade vem do croma e do contraste de matiz, não de luminosidade.

### 3.3 Marca: âmbar Kazzu

`ember` `#ffa154` (escuro) / `#ab4b01` (claro); `accent-ink` `#190c04` / `#fefbf8`. Aparece no ícone do app, no K da marca, como acento padrão e quando a capa não tem cor.

### 3.4 Acento dinâmico (derivado da capa)

Algoritmo da v1 mantido (reduzir a 32×32, OKLCH, 12 baldes de matiz, faixa de luminosidade presa ao tema); apenas a faixa escura subiu para L ∈ [0,76; 0,88] (ícones e preenchimentos sobre o palco). O **matiz do acento alimenta o palco** (§3.2). Acento é para ícone, preenchimento e foco sobre o palco; texto de acento só sobre vidro.

### 3.5 Imersivo (player)

O player é **sempre escuro** (o fundo é a capa): `scope-dark` no CSS, `Theme.dark` forçado na tela em Slint. Dois acréscimos da v2 só no player: véu superior de 62% nos primeiros 24% da altura (garante o "Tocando de" sobre capas claras) e o brilho da capa em ×0,62.

## 4. Tipografia

Hanken Grotesk (interface) e Geist Mono (selos e tempos), ambas OFL, em `design/fonts/`. Escala (px lógicos; peso; linha; tracking): `large-title` 34/40 700; `title-1` 28/34 700; **`player-title` 26/32 700** (celular); `title-2` 22/28 600; `headline` 17/22 600; `body` 16/22 400; `callout` 15/20; `subhead` 14/18; `caption` 12/16 600; `timecode` 13/16 mono 500; `badge` 12/14 mono 500. Três pesos; rótulos em minúsculas; mínimo de 12 (só rótulos de barra e selos); corpo nunca abaixo de 14. Respeita a escala de texto do sistema até 200% (UX_PLAN §9.3).

## 5. Espaçamento, raios, tamanhos

Escala de 4 px. **Mobile-first:**

| Token | Valor | Nota |
|---|---|---|
| `hit` | **48** | alvo mínimo (era 44); `hit-min` 44 só para chips (alvo estendido a 48) |
| `gutter-mobile` | **16** (24 ≥ 600; 32 ≥ 1200) | |
| `dock-h` / `dock-radius` | 64 / 32 | barra flutuante |
| `dock-search` | 56 | botão circular de busca |
| `mini-h` / `mini-radius` | 64 / 24 | |
| `sheet-radius` | 32 | folhas (nível 3) |
| `chip-h` / `field-h` / `badge-h` | 40 / 56 / 24 | |
| `play-main` | 76 (72 no desktop) | |
| `progress-h` / ativo / `slider-thumb` | 8 / 12 / 22 | |
| `rail-w` / `rail-w-wide` | 88 / 248 | trilho e barra lateral |
| `hero-card-w-mobile` | 300 (máx. 78% da janela) | |

Raios: `xs 6`, `sm 10`, `md 14`, `lg 20`, `xl 28`, `full`. Raios concêntricos (interno = externo − padding).

## 6. Materiais: vidro em 3 níveis

### 6.1 Níveis de profundidade

| Nível | Quem | Blur | Fill (`glass-tint` × alfa) escuro / claro | Sombra |
|---|---|---|---|---|
| **1 · conteúdo** | cartões de coleção, banner, placas, grupos | 20 | 0,40 / 0,52 | 24, y8 |
| **2 · cromo flutuante** | barra inferior, Buscar, mini player, campo de busca, trilho | 32 | 0,56 / 0,66 | 32, y12 |
| **3 · folhas** | player (pílula de transporte), fila, menus, diálogos | 40 | 0,72 / 0,80 | 56, y24 |

Quanto mais alto o nível: mais opaco (contraste garantido sobre qualquer coisa atrás), mais blur e mais sombra. `glass-tint` é o matiz da capa em L 24% (escuro) ou L 99% (claro).

### 6.2 Receita (de baixo para cima)

1. **Backdrop blur** (+ `saturate` no CSS).
2. **Fill translúcido** = `glass-tint` com a alfa do nível.
3. **Brilho superior**: gradiente de `glass-sheen` (branco 14% escuro / 60% claro) até transparente em 52% da altura.
4. **Borda de luz de 1 px em gradiente**: `155°`, `glass-edge` (branco 46% / 95%) no canto superior esquerdo → `glass-edge-low` (10% / 40%) no meio → metade do `edge` no canto inferior direito. CSS: máscara em `::before`; Slint: `border-color: @linear-gradient(...)` (FemtoVG e Skia aceitam brush; **conferir no primeiro build**).
5. **Fio especular** de 1 px no topo (inset = raio, esvaece nas pontas).
6. **Sombra tingida** do nível (cor `shadow`, derivada do tom do palco; nunca luminosa).

Regras: **nunca aninhar vidro** (o filho borraria o pai; filhos usam fill simples `ink` 12%); **no máximo 6** painéis com blur visíveis; sem vidro dentro de `ListView` nem de linhas; a barra e o mini player têm por baixo um **fade** de 190 px (do transparente a `stage-base` 82%) para o conteúdo não brigar com o texto. Cartões sobre imagem (destaque, humores) usam **vidro escuro fixo** (`scope-dark`) sobre um véu preto de 50% na imagem: a legibilidade não depende da capa.

### 6.3 Contraste garantido (calculado + medido)

- **Calculado** (`gen_tokens.py`, pior manchão do palco): ink / vidro 1·2·3 = 8,9 · 10,5 · 12,9 :1 no escuro e 14,4 · 15,3 · 16,2 :1 no claro; `ink-2` ≥ 7,0 e `ink-3` ≥ 5,4 em todos; sólidos (reduzir transparência) ≥ 7 : 1.
- **Quanto de claridade pode haver atrás** (conteúdo rolando, capas) mantendo `ink-2` ≥ 4,5 : 1 (cinza, luminância relativa): nível 1 até 0,28, nível 2 até 0,50, nível 3 qualquer. É por isso que a barra e o mini player (nível 2) toleram capas claras passando atrás e os cartões de conteúdo (nível 1) não ficam sobre imagem sem véu.
- **Medido em captura real** (`tools/check_render.py` → `ux/contrast-report.md`): **2.111 textos em 117 capturas, 0 abaixo de AA, menor contraste 4,67 : 1** (matriz: 10 telas × 390/360/768/1280 × claro/escuro, com e sem transparência, rolagem sob a barra, 7 capas).

### 6.4 Fallback ("Reduzir transparência")

`Theme.glass-live = blur-supported && !reduce-transparency`. Falso ⇒ blur 0 e fill sólido (`glass-solid`, `-strong`, `-3`; tingidos, contraste ≥ 7 : 1). Gatilhos: renderer software; chave em Ajustes › Aparência; preferência do sistema quando houver; oferta automática (não imposta) após 2 s acima de 16 ms/quadro com vidro. O palco continua (estático se o movimento estiver reduzido) e a hierarquia dos 3 níveis permanece por opacidade, borda e sombra. Protótipo: `#rt=1`.

### 6.5 Limites conhecidos

- O backdrop do Slint 1.19 **só tem blur**: sem `saturate`, sem `brightness`. Compensação: palco já saturado; protótipo com `#sat=0` mostra o resultado sem saturação.
- Blur de 40 px em tela cheia pode custar quadros em GPU fraca: a folha do player usa **palco + capa pré-desfocada pelo Rust**, e só a pílula de transporte tem blur.
- Placas de texto sobre capa clara: véu de 50% na imagem + vidro escuro fixo (ver 6.2).
- Blur só do conteúdo da própria janela (nada de acrílico do sistema nesta versão).

### 6.6 Elevação

`e1` blur 8/y2, `e2` 24/8, `e3` 48/20 para elementos soltos; vidro usa as sombras do nível (§6.1). Cor `shadow`: marrom-quente escuro 60% (escuro) / 30% (claro); capa do player e destaques usam sombra tingida pelo `stage-a`.

## 7. Marca e ícones

Marca (K de barras), wordmark, ícone do app e favicons: **inalterados** (`design/assets/`). Wordmark no canto superior esquerdo (28 de altura) em todas as telas compactas; trilho: marca 36; lateral ≥ 1200: wordmark. **54 ícones** de grade 24, traço 1,75, pontas redondas, desenho próprio; a v2 acrescentou `gear`, `refresh`, `cloud`, `stats`, `timer`, `arrow-up`, `arrow-down`, `minus`, `badge-check`. Cabeçalhos "Do YouTube Music" usam o ícone genérico `cloud`: **nada do logotipo ou da marca do YouTube** é reproduzido.

## 8. Movimento

`dur-micro` 120, `dur-short` 200, `dur-medium` 320, `dur-long` 480 (troca de capa). Curvas: `ease-out cubic-bezier(.22,.9,.28,1)`, `ease-in-out (.45,0,.2,1)`. Sem mola, sem loop. Folhas sobem em `dur-medium` e **seguem o dedo** ao arrastar; troca de capa = cross-fade 480 ms. `reduce-motion` zera tudo (corte seco, sem camada de capa se preferido).

## 9. Acessibilidade

Detalhe e critérios em `UX_PLAN.md` §9. Resumo de projeto: AA medido em captura; foco visível e **não obscurecido** (reserva de 184 dp); alvos de 48; TalkBack com rótulos, estados e ações personalizadas no lugar de gestos; WCAG 2.5.7 (alternativa a todo arrastar); **Reduzir transparência** e **Reduzir movimento** independentes; texto até 200%; cor nunca é o único sinal (ativo = pílula + rótulo em negrito + ícone).

## 10. Estados vazios, carregando e erro

Estrutura única: vidro nível 1, ícone de sistema (88, raio 28), título, até 2 frases, **uma** ação. Seis estados em `prototypes/states.html` (vazio, carregando com esqueleto estático, erro de provedor com código, offline, sem permissão, sem resultados). Textos: `UX_PLAN.md` §7.3.

## 11. Responsividade (mobile-first)

| Classe | Largura | Navegação | Mini player | Conteúdo |
|---|---|---|---|---|
| **compact** (base) | 360–599 | barra flutuante + Buscar circular | pílula acima da barra | 1 coluna; coleções 2×2 |
| **medium** | 600–839 | trilho 88 | flutuante, até 640 | 2–3 colunas |
| **expanded** | 840–1199 | trilho 88 | idem | coleções 4×1; player em 2 colunas; coluna de álbum |
| **large** | ≥ 1200 | barra lateral 248 | NowBar com progresso e volume | até 1240 centrado |

O celular é o desenho; o resto **acrescenta**. Busca é botão próprio em todos os tamanhos (nunca barra no topo fora da própria tela).

## 12. Telas (protótipos)

`design/prototypes/index.html` (hub com seletor de tela, tema, sem transparência, sem saturação e capa) mostra 390, 360, 768 e 1280. Cada tela abre sozinha e aceita `#theme=light|dark`, `#rt=1`, `#sat=0`, `#c=01..10`, `#s=` (estados).

| Tela | Arquivo | Destaques |
|---|---|---|
| Início | `home.html` | aviso de primeiro uso, chips, destaques com placa de vidro, escolhas rápidas com motivo, mixes |
| Explorar | `explore.html` | lançamentos, humores e gêneros, em alta |
| Biblioteca | `library.html` | 4 coleções de vidro, Criar playlist, Neste aparelho |
| Álbum | `album.html` | capa grande, Tocar/Aleatório |
| Buscar | `search.html` (`#s=idle\|results\|empty\|loading\|error\|offline`) | campo de vidro com "reconhecer pelo som" |
| Player | `player.html` (`#c=`) | folha cheia, pílula de transporte, qualidade; arrastar fecha |
| Fila | `queue.html` | folha de vidro nível 3, setas de mover |
| Folhas | `sheets.html` (`#s=menu\|playlist\|create`) | menu "mais", adicionar à playlist, nova playlist |
| Ajustes | `settings.html` | YouTube Music ativo, privacidade travada |
| Estados | `states.html` (`#s=`) | seis estados |

Os protótipos usam `backdrop-filter` de verdade (o mesmo efeito do Slint, mais `saturate`).

## 13. Decisões para o autor aprovar

1. **Âmbar** como marca e fallback do palco/acento (alternativa: mudar `brand.ember`).
2. **Matiz do palco vem da capa** (cobre o app inteiro). Alternativa: palco só no player e âmbar fixo no resto.
3. **Vidro escuro mais "esfumaçado" que branco** no tema escuro (fill = tint do matiz da capa, não branco). Branco translúcido dá mais cara de "vidro fosco" mas não deixa texto branco passar de AA; o tint é o jeito de ser vidro **e** legível. Quer ver uma variante mais clara só nas folhas?
4. **Alvo mínimo 48** (era 44) em todo o app.
5. **Início × Explorar separados**; "Para você / Descobrir / Mixes" como chips no Início (UX_PLAN §14.1).
6. **Curtidas e playlists locais** (sem login no YT Music).
7. **Barra inferior com 3 destinos + Buscar circular** (mapa do autor); campo de busca só dentro da tela Buscar.
8. **Sem telemetria travada em ligado**, sem pesquisa de opinião no app.
9. **Player sempre escuro**, mesmo no tema claro.
10. Licença do app (GPL-3.0, hipótese de trabalho) aparece em Ajustes › Sobre.

## 14. O que foi verificado

- `python3 design/tools/gen_tokens.py`: pares de contraste do sistema (inclui vidro 1·2·3 sobre o pior manchão e os sólidos) passam; regenera `tokens.slint` e `tokens.css`.
- **Protótipos renderizados no Chromium (CDP, viewport exato)** em 390×844, 360×740, 768×1024 e 1280×800, claro e escuro, com e sem transparência, com 7 capas diferentes no player e no Início e com rolagem (texto sob a barra).
- **Contraste medido nas capturas** (`tools/check_render.py`, pior pixel de fundo atrás de cada texto): 2.111 textos, **0 reprovados**, menor 4,67 : 1. A primeira rodada achou 33 reprovações (título da faixa em cor de acento sobre o palco, legendas de placas sobre capas claras, "Tocando de" sobre capa pálida); corrigidas (título em `ink` + fundo de linha, placas de vidro escuro sobre véu preto de 50%, véu superior no player).
- **Slint 1.19**: `tokens.slint`, `atoms.slint` (`Stage`, `GlassPanel` de 3 níveis, `Chip`) e `preview.slint` **compilam** (`slint-build 1.19.0`, offline). Renderizados em 390×844, escuro e claro: **FemtoVG com `backdrop-blur` real** (Xvfb + Mesa llvmpipe) em `slint/preview-femtovg-{dark,light}.png` (o vidro deixa os arcos do palco aparecerem desfocados; borda em gradiente e fio especular visíveis) e renderer **software** (fallback sólido, sem blur) em `slint/preview-software-{dark,light}.png`.
- **Não verificado:** Skia (só FemtoVG e software foram renderizados), desempenho do blur em GPU fraca e em Android real, TalkBack, testes com pessoas (roteiro em `UX_PLAN.md` §12).

## 15. Regenerar

```
python3 design/tools/gen_tokens.py        # tokens.slint, tokens.css + checagem AA
python3 design/tools/make_assets.py       # logos, ícone, favicons, 54 ícones, capas, acentos (rsvg-convert, Pillow, magick)
python3 design/tools/build_prototypes.py  # HTML dos protótipos + palco por capa (stage.css)
python3 design/tools/check_render.py      # contraste real em capturas (Chromium + Pillow) -> design/ux/contrast-report.md
python3 design/tools/shoot_all.py         # capturas webp para o artifact do plano de UX
python3 design/tools/build_ux_plan.py     # artifact HTML do plano (precisa de tools/.cache/mermaid.min.js)
```
