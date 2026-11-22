# Kazzu — design system

Versão 0.1 · Slint 1.19 · pt-BR. Fonte dos números: `design/tokens.json` (gerador: `tools/gen_tokens.py`). Telas navegáveis: `design/prototypes/index.html`. Medidas e árvores de componentes para o implementador: `design/SLINT_SPEC.md`.

## 1. A ideia

> **A capa é a luz.** O Kazzu é uma sala escura e quieta; a única fonte de cor é o disco que está tocando.

Tudo o que não é capa fica em neutros quentes (grafite no escuro, papel no claro). A cor de destaque, o fundo do player e o brilho do ambiente nascem da capa da faixa atual. Quando nada toca, ou a capa é cinza, entra o **âmbar Kazzu** da marca.

O que o app tem de diferente do resto da categoria (e o design mostra, não descreve):

- **Fidelidade visível.** Cada faixa carrega o selo do que ela é de verdade (`FLAC 24/96`, `MP3 320`). O player diz "Original, sem equalizador" quando o caminho é bit-perfect.
- **Vidro só onde flutua.** Barra de navegação, mini player, sidebar, botões do player e cartões de coleção. Listas ficam planas.
- **Quietude.** Sem loops, sem brilho animado, sem confete. O movimento serve para orientar (troca de capa, estado de botão), nunca para decorar.

## 2. Princípios (e o que traduzimos das skills `hallmark` e `taste-skill`)

As skills são de web. O que vale para um app nativo e foi aplicado:

| Princípio | Como aparece no Kazzu |
|---|---|
| **Direção própria, nada de template** | Marca desenhada à mão (K de barras), âmbar + capas fictícias abstratas, par tipográfico Hanken Grotesk + Geist Mono. Nenhum asset ou desenho copiado de Bitchord, Umihi, Intertune ou Metrolist. |
| **Hierarquia por escala e peso, não por enfeite** | 3 pesos (400/600/700), escala de 12 a 36 px, um único botão primário cheio por tela. |
| **Cor com função** | Acento < 5% da tela fora do player: item ativo, foco, botão primário, progresso, coração. Nunca em texto corrido. |
| **Contraste e foco são requisitos** | AA verificado por script em todos os pares (§9.1); foco de 2 px que aparece na hora; alvos de 44 px. |
| **Movimento reduzido de verdade** | Tudo que anima lê `Theme.reduce-motion`; sem nenhum loop infinito (o indicador "tocando" é estático). |
| **Anti "cara de IA"** | Sem gradiente roxo-azul, sem brilho/halo colorido (glow), sem emoji como ícone, sem cartão dentro de cartão, sem rótulos em caixa-alta espaçada, sem "eyebrow", sem strings com ponto-médio (`Artista · Álbum`), sem pontos decorativos de status, sem números inventados na interface (os dados dos protótipos são visivelmente fictícios). |
| **Textos específicos** | Títulos nomeiam coisas ("Álbuns que você ouve"), botões são verbo + objeto ("Criar playlist", "Adicionar pasta"), estados vazios dizem o que falta, por quê, e dão uma ação. |
| **Uma ideia por tela** | Início: o destaque. Player: a capa. Biblioteca: as quatro coleções. Busca: o campo. |

## 3. Cor

Todas as cores vivem em `tokens.json` (OKLCH ou hex) e saem como hex em `tokens.slint` (`Theme.*`) e `tokens.css`. Claro e escuro são dois renderings do mesmo sistema (mesmo matiz quente ~55–85°, luminosidades re-derivadas, nunca inversão).

### 3.1 Neutros

| Token | Escuro | Claro | Uso |
|---|---|---|---|
| `bg` | `#0c0806` | `#f6f3ed` | fundo da janela |
| `surface` | `#17110e` | `#ece9e2` | poços: grupos de configuração, linha em hover |
| `surface-2` | `#211a16` | `#e3dfd8` | chips, campo de busca, botão secundário |
| `surface-3` | `#2c2520` | `#d8d4cc` | trilhos, ícone de linha, estado pressionado |
| `ink` | `#f3efec` | `#1a1511` | texto principal |
| `ink-2` | `#bcb6b1` | `#47413c` | texto secundário, ícones |
| `ink-3` | `#98918b` | `#635c57` | meta (artista em lista, legendas) |
| `line` / `line-strong` | branco 9% / 17% | tinta 10% / 20% | divisores / borda de campo |
| `success` `warning` `danger` | `#61da92` `#edc14d` `#fd736d` | `#027e47` `#8d5e05` `#ba2b2e` | só texto/ícone de estado, nunca como único sinal |

Superfícies sobem por **luminosidade** no escuro (mais claro = mais perto) e descem no claro (poços mais escuros que o fundo). Sem cartões brancos em fundo tinto. Sem preto ou branco puro.

### 3.2 Marca: âmbar Kazzu

`ember` `#ffa154` no escuro; `#ab4b01` no claro (a mesma cor com luminosidade menor para dar 5:1 com o fundo). `accent-ink` (texto sobre acento): `#190c04` no escuro, `#fefbf8` no claro. Aparece no ícone do app, na divisa do K, na cor de destaque padrão e quando a capa não tem cor.

### 3.3 Imersivo (player em tela cheia)

O player é sempre escuro, em qualquer tema, porque o fundo é a capa. Tokens `im-*`: tinta `#faf8f5`, secundária branco 86%, terciária branco 74%, trilho branco 24%, fundo de botão preto 26% com borda branca 18%.

### 3.4 Acento dinâmico (derivado da capa)

Calculado em Rust quando a faixa muda; entra em `Theme.dynamic-accent` e `Theme.dynamic-accent-ink` (com `Theme.use-dynamic-accent = true`). **O algoritmo foi implementado e testado em `tools/make_assets.py` sobre as 10 capas fictícias**; o Rust deve reproduzi-lo igual.

1. Reduzir a capa para 32×32 (filtro bilinear/triangle), converter cada pixel para OKLCH.
2. Ignorar pixels com croma < 0,04, luminosidade < 0,18 ou > 0,97.
3. Peso do pixel `w = C^1,5 · (1 − |L − 0,62|)`. Acumular em 12 baldes de matiz (30° cada): soma de `w`, de `w·L`, de `w·C` e vetor `w·(cos h, sin h)`.
4. Se o peso total < 0,6 (capa quase monocromática): **usar o âmbar da marca**.
5. Balde vencedor = maior `w` somado com metade dos dois vizinhos. `L` e `C` = médias ponderadas do balde; `h` = média circular.
6. **Prender** à faixa do tema: escuro `L ∈ [0,74; 0,86]`, claro `L ∈ [0,44; 0,52]`; croma `C ∈ [0,10; 0,17]`. Reduzir o croma até caber no sRGB.
7. `accent-ink` = o de maior contraste entre `#120f0c` e `#fcfbf8`.

Resultado sobre as capas de teste (acento escuro / claro): Maré Alta `#50bdc9 / #016b74`, Nove Horas `#fa8281 / #a9393d`, Bruma `#ffb494 / #985436`, Quadra 7 `#e8b92f / #816505`, Sal e Cinza `#ffa154 / #ab4b01` (cai no âmbar), Linha de Fuga `#73be91 / #0f623b`, Calor de Março `#fc8457 / #b03f04`, Vidro Líquido `#74aef6 / #2c64a6`, Papel de Seda `#e59680 / #9a523e`, Fita Magnética `#d2a15d / #8c5e12`. Todas passam de 4,5:1 contra o fundo do tema (a faixa de luminosidade garante isso).

**Escopo:** com "Cor de destaque: Da capa" (padrão) o acento vale no app inteiro (dock, botões, foco, progresso). Com "Âmbar" fica fixo na marca. Trocar de faixa corta a cor na hora (globais do Slint não animam); a suavidade vem do cross-fade de 480 ms da capa e do fundo.

### 3.5 Fundo derivado da capa

Dois produtos gerados em Rust a cada faixa, funcionam em qualquer renderer:

- **Fundo do player** (`PlayerBackdrop`): capa reduzida para **96×96**, desfoque gaussiano σ = 4 px nessa resolução, saturação ×1,25, multiplicar por `im-base-darken` (0,46) e somar 1 LSB de ruído (evita faixas de cor em degradê escuro). Exibido com `image-fit: cover`, esticado para a tela inteira, mais um véu: gradiente vertical preto 10% → 0% (em 30%) → 72% (em 100%). Isso garante o contraste do texto sobre a pior capa possível (capa branca): título 7,5:1, artista 6,4:1, tempos 6,1:1.
- **Ambiente das demais telas** (`AmbientBackdrop`): a mesma imagem, mas com opacidade `Theme.ambient-opacity` (0,55 escuro / 0,30 claro), ocupando 90% da altura, e um véu que dissolve para `bg` (transparente em 0%, `bg` 70% em 40%, `bg` sólido em 78%). Serve de "o que está atrás" para o `backdrop-blur` do vidro.

Troca de faixa = cross-fade de 480 ms (duas `Image` sobrepostas alternando opacidade).

## 4. Tipografia

| Papel | Família | Licença | Arquivos (em `design/fonts/`) |
|---|---|---|---|
| Interface | **Hanken Grotesk** | SIL OFL 1.1 | `HankenGrotesk-Regular/SemiBold/Bold.ttf` |
| Dados técnicos | **Geist Mono** | SIL OFL 1.1 | `GeistMono-Regular/Medium.ttf` |

Instâncias estáticas geradas das fontes variáveis oficiais (repositório `google/fonts`) com `fonttools varLib.instancer`, sem alterar glifos. Cobrem pt-BR completo. Textos de licença: `OFL-HankenGrotesk.txt`, `OFL-GeistMono.txt` (devem ir junto do app, também na tela "Licenças"). **Por que não Inter/SF:** SF é proprietária; Inter é a escolha que todo mundo faz. Hanken Grotesk é neutra e limpa como a SF, com números e acentos bem desenhados.

Por que uma mono: o Slint não tem `font-feature-settings`, então não há como pedir números tabulares. Tempos (`1:02`), selos de formato e taxas usam Geist Mono, que mantém colunas alinhadas e dá o ar "instrumento" sem ornamento.

Escala (px lógicos; peso; altura de linha; tracking em px):

| Token | Tamanho/linha | Peso | Tracking | Uso |
|---|---|---|---|---|
| `large-title` | 34/40 (40/46 no desktop) | 700 | −0,6 | título de tela |
| `title-1` | 28/34 | 700 | −0,4 | título do player (mobile) |
| `title-2` | 22/28 | 600 | −0,2 | título de seção, cartão de destaque |
| `headline` | 17/22 | 600 | 0 | linha de configuração |
| `body` | 16/22 | 400 | 0 | texto corrido, título de linha (600) |
| `callout` | 15/20 | 400 | 0 | texto de mini player, botões (600) |
| `subhead` | 14/18 | 400 | 0 | artista em lista, descrição |
| `caption` | 12/16 | 600 | 0,1 | rótulo da dock |
| `player-title-desktop` | 36/42 | 700 | −0,6 | player no desktop |
| `timecode` | 13/16 mono 500 | | 0 | tempos |
| `badge` | 12/14 mono 500 | | 0 | selos de formato |

Regras: três pesos no app inteiro; rótulos em minúsculas/sentence case, **nunca caixa-alta espaçada**; mínimo de 12 px (só na dock e nos selos); títulos com no máximo duas linhas e reticências; corpo nunca abaixo de 14 px.

## 5. Espaçamento, raios, tamanhos

Escala de 4 px: `sp-1..10` = 4, 8, 12, 16, 20, 24, 32, 40, 56, 80. Qualquer valor fora disso precisa de justificativa.

Raios: `xs 6` (selos), `sm 10` (miniaturas de capa), `md 14` (arte de cartão), `lg 20` (poços, cartões), `xl 28` (capa do player), `full` (botões, chips, campo). **Raios concêntricos:** raio interno = raio externo − espaçamento (cartão de 22 com 12 de padding tem arte de 14, igual a `md`).

Tamanhos fixos principais: alvo mínimo 44, ícone 24 (20 e 28 variantes), linha 64 com capa 48, dock 64 (raio 32), mini 64 (raio 22), sidebar 248/80, capa do player 340 (mobile) e 440 (desktop), play 84/72, trilho 6 px (10 ativo), polegar 20. Lista completa: `tokens.json` > `size`.

## 6. Materiais: vidro

### 6.1 O que é e onde entra

O Slint 1.19 tem **`backdrop-blur` em `Rectangle` translúcido** nos renderers **Skia e FemtoVG** (não no software). É o blur do que está atrás *dentro da janela*, não o do desktop. O Kazzu usa isso para:

| Superfície | Variante | Blur |
|---|---|---|
| Dock (barra principal) e mini player | `strong` | 40 px |
| Sidebar (desktop), barra "tocando agora" (desktop) | normal | 28 px |
| Cartões de coleção da Biblioteca (4), placa de texto do cartão de destaque | normal | 28 px |
| Botões do player (Letra, Saída, Fila) | `light` | 16 px |

**Nunca** em linhas de lista, chips, configurações nem dentro de `ListView`. **Teto de 6 camadas com blur visíveis ao mesmo tempo** (token `glass.max-blurred-layers`): mobile típico = dock + mini + 4 cartões. Fora do player, o vidro sempre repousa sobre algo (o ambiente derivado da capa ou o conteúdo), senão o blur não tem o que borrar.

### 6.2 Receita (camadas, de baixo para cima)

Implementada em `design/slint/atoms.slint` > `GlassPanel` (compilada e renderizada); o app deve usá-la, não recriar.

1. **Blur de fundo:** `backdrop-blur: Theme.glass-blur-radius` (28) / `glass-blur-strong-radius` (40) / `glass-blur-light-radius` (16).
2. **Preenchimento translúcido:** `background: Theme.glass-bg` (escuro: grafite `#251e19` 55%; claro: papel `#fcfaf6` 58%). Variante `strong`: escuro 80%, claro 82%. Não existe saturação nem brilho no backdrop do Slint, então a alfa foi calibrada para o blur sozinho.
3. **Gradiente sutil:** `Rectangle` filho na metade superior, `@linear-gradient(180deg, glass-sheen → transparente)` (branco 10% escuro / 55% claro).
4. **Brilho especular:** `Rectangle` de **1 px de altura** no topo, largura = largura − 2×raio, gradiente horizontal transparente → `glass-edge` (branco 16%/85%) em 18% e 82% → transparente. É o "fio de luz" que dá a sensação de borda de vidro.
5. **Borda:** `border-width: 1px; border-color: Theme.glass-edge-low` (branco 6% / tinta 8%).
6. **Sombra:** `drop-shadow-blur: 24px; drop-shadow-offset-y: 8px; drop-shadow-color: Theme.e2-color`. Sombra interna só no FemtoVG, por isso **não é usada** (a borda de 1 px e o filho de gradiente fazem o papel).
7. `clip: true` com o mesmo `border-radius` para o gradiente respeitar os cantos.

### 6.3 Fallback (obrigatório)

`Theme.glass-live = blur-supported && !reduce-transparency`. Quando falso: blur 0 e `glass-bg` vira **sólido** (`glass-solid`: `#221b17` / `#f7f5f1`; `glass-solid-strong`). Gatilhos:

- renderer **software** (o Rust escreve `Theme.blur-supported = false`);
- o usuário liga **Configurações > Aparência > Reduzir transparência** (também lê a preferência do sistema quando existir);
- tela lenta: se o frame médio passar de ~16 ms por 2 s com vidro ativo, o app oferece (não impõe) ligar a opção.

O visual sem blur continua pleno: o sólido passa de 14:1 de contraste, o fundo derivado da capa (pré-desfocado por Rust) não depende de blur, e a hierarquia é a mesma. O protótipo tem chave "Sem transparência" no hub para ver os dois modos.

### 6.4 Limites conhecidos

- Blur só do conteúdo da própria janela. Acrílico/Mica/vibrancy do sistema não faz parte desta versão (extra por plataforma, ver `docs/research/ui-and-mobile.md`).
- GPU fraca/Android barato: blur de 40 px em tela cheia pode custar frames. Por isso o player em tela cheia **não usa backdrop-blur**: o fundo é a imagem já desfocada por Rust; só os 3 botões usam blur leve (16 px).
- Placa de texto sobre capa clara: o fill da placa é preto 64% para dar ≥ 5,5:1 ao texto branco mesmo sobre capa branca (blur não escurece nada).
- Rolagem sob a dock: como o blur é ao vivo, o conteúdo some suavemente sob a barra; um degradê de 190 px (transparente → `bg`) acima das barras garante legibilidade mesmo no fallback.

### 6.5 Elevação

Três níveis de sombra, todos com a cor `Theme.shadow`: `e1` blur 8/y 2 (placas pequenas, polegar), `e2` blur 24/y 8 (painéis de vidro, botão play), `e3` blur 48/y 20 (capa do player). No escuro a sombra é suave (preto 25–45%); no claro é quente e mais leve. Superfície elevada nunca é mais escura que o fundo.

## 7. Marca e ícones

**Marca:** o **K de barras** — uma haste e uma divisa `<` separadas por uma fresta, traço de 8/64 com pontas redondas, a divisa em âmbar. Lê-se como K e como barras de nível de áudio. O **wordmark** `kazzu` é desenhado à mão em monolinha (traço 5, x-height 26 de 40); o `k` repete a haste + divisa âmbar. O **ícone do app** é um squircle (superelipse n = 5) grafite com a marca centralizada e um fio de luz no topo; sem glow.

Arquivos (`design/assets/`): `kazzu-icon.svg` (1024), `icon-{256,512,1024}.png`, `favicon.svg` / `favicon.ico` (+ `icon-{16,32,48}.png`), `kazzu-mark-{dark,light,mono}.svg`, `kazzu-wordmark-{dark,light,mono}.svg`, `kazzu-logo-{dark,light,mono}.svg` (ícone + wordmark). Versões `mono` usam `currentColor` (no Slint: `Image` com `colorize`).

Uso: no canto superior esquerdo das telas mobile vai o **wordmark** (28 px de altura); na sidebar larga o wordmark, na sidebar estreita a **marca** (36 px). Mínimos: marca 16 px, wordmark 20 px de altura. Área de proteção = metade da altura da marca em todos os lados. Não recolorir a divisa, não esticar, não pôr sobre foto sem véu.

**Ícones** (`design/assets/icons/*.svg`, 45 arquivos + folha `_sheet.svg`): grade 24, traço 1,75 com pontas e junções redondas; controles de transporte (play, pause, prev, next) e "mais" preenchidos. Desenho próprio, sem emoji. Uso no Slint: `Image { source: @image-url("…/play.svg"); colorize: Theme.ink-2; width: Theme.icon }`. Os SVGs usam `currentColor` e `fill`/`stroke` explícitos, compatíveis com o resvg do Slint.

## 8. Movimento

| Token | Valor | Uso |
|---|---|---|
| `dur-micro` | 120 ms | pressionar, hover de cor, mudança de estado de botão |
| `dur-short` | 200 ms | interruptor, trilho engrossando, hover de cartão (−2 px em `y`), aba ativa |
| `dur-medium` | 320 ms | abrir/fechar fila e folhas |
| `dur-long` | 480 ms | troca de capa e fundo (cross-fade) |
| `ease-out` | `cubic-bezier(0.22, 0.9, 0.28, 1)` | chegadas e hover |
| `ease-in-out` | `cubic-bezier(0.45, 0, 0.2, 1)` | trocas de capa, folhas |
| `ease-in` | `cubic-bezier(0.5, 0, 0.9, 0.4)` | saídas (60–75% da duração de entrada) |

Regras: sem mola nem overshoot; sem animação infinita (o indicador "tocando" são 3 barras **estáticas**); animar só opacidade, cor e posição de elementos soltos (não dentro de layouts). `Theme.reduce-motion = true` zera as quatro durações (o Rust liga isso pela preferência do sistema e pela chave em Configurações; o Slint 1.19 só desliga sozinho no Windows). Com movimento reduzido a troca de capa é corte seco.

## 9. Acessibilidade

1. **Contraste AA** conferido por `tools/gen_tokens.py` em 51 pares (rodar após qualquer mudança em `tokens.json`): corpo ≥ 4,5:1 (texto principal ≥ 7:1 no fundo), acento sobre fundo ≥ 4,5:1, ícones e foco ≥ 3:1, sólido do vidro ≥ 7:1, e o pior caso do player (capa branca) ≥ 4,5:1 para todo texto.
2. **Alvos de toque 44 px** em tudo que é tocável (chips de 40 px e interruptor de 32 px ganham área invisível de 44; sliders têm 44 de altura útil). Em desktop com ponteiro o mínimo aceito é 32 px (barra de progresso da NowBar).
3. **Foco visível:** anel de 2 px na cor de destaque, deslocado 2 px, aparece instantaneamente (sem animação), nunca coberto por vidro. Ordem de tab segue a leitura. Atalhos: Espaço/Enter ativam, setas movem sliders (±5%).
4. **Leitores de tela (AccessKit):** todo botão de ícone tem `accessible-label` em pt-BR (inclusive o de busca: "Buscar"); sliders expõem `accessible-value` ("1:02 de 3:21"); interruptores expõem estado; indicador "tocando" tem rótulo.
5. **Não depender só de cor:** favorito = ícone preenchido + cor; item ativo da dock = fundo + cor + rótulo; hi-res = selo com texto.
6. **Movimento e transparência reduzidos:** duas chaves independentes em Configurações > Aparência.
7. **Texto escalável:** o app respeita o fator de escala do sistema; layouts usam `preferred-width` e reticências, não larguras fixas de texto.

## 10. Estados vazios, carregando e erro

Estrutura única: **ícone do sistema** num quadrado de 96 px (`surface-2`), título (`title-2`), uma ou duas frases de apoio (`ink-3`, máx. 2 linhas), **uma** ação. Sem ilustração decorativa. Protótipo: `prototypes/states.html`.

| Estado | Título | Texto | Ação |
|---|---|---|---|
| Biblioteca vazia | Sua biblioteca está vazia | Adicione uma pasta de músicas ou conecte um provedor. O Kazzu encontra capas e tags sozinho. | Adicionar pasta |
| Sem resultados | Nada para "…" | Confira a grafia ou busque só o nome do artista. | Limpar filtros |
| Fila vazia | A fila está vazia | Toque em uma música e ela aparece aqui, junto com as próximas. | Ir para o Início |
| Offline | Sem conexão | Suas músicas baixadas continuam tocando. O resto volta quando a rede voltar. | Ver baixados |

Carregando: linhas-esqueleto estáticas (capa + duas barras em `surface-3`/`surface-2`); spinner só depois de 300 ms e visível por pelo menos 300 ms. Erros: o que aconteceu, por quê (se souber), o que fazer; sem "Ops", sem exclamação. Provedor indisponível mostra a linha com tag "Em breve" ou "Sem conexão", nunca some.

## 11. Responsividade (desktop ↔ mobile)

| Classe | Largura | Navegação | Mini player |
|---|---|---|---|
| `compact` | < 720 | dock flutuante (3 abas + botão de busca dentro dela) | mini flutuante acima da dock |
| `regular` | 720–1099 | sidebar de vidro estreita (80 px, só ícones) | barra "tocando agora" (sem progresso) |
| `wide` | ≥ 1100 | sidebar de 248 px (wordmark, rótulos) | barra com progresso e tempos |

`Theme.window-w` alimenta `Theme.compact` / `Theme.wide`. Gutter 20 px (compact) / 32 px (demais); conteúdo limitado a 1180 px e centralizado (em 1920 só as laterais crescem). Carrossel: 300×400 (mobile) / 380×460 (desktop), com setas só no desktop. Tabelas de linhas ganham coluna de álbum (28%) no desktop. Grades: coleções 2 colunas → 4; playlists 2 colunas → `auto-fill 200`. Player: coluna única → duas colunas (capa à esquerda, controles em ≤ 460 px à direita). A busca é um **botão com ícone próprio dentro da barra principal** em todos os tamanhos (no desktop: último item da sidebar, separado por um divisor); a barra de campo só existe dentro da tela Busca.

## 12. Telas (protótipos)

`design/prototypes/index.html` mostra todas em 390×844 e 1280×800, com chaves de tema e de transparência. Cada tela abre sozinha e aceita `#theme=light|dark`, `#rt=1` (sem transparência) e, no player, `#c=01..10` (capa).

| Tela | Arquivo | Destaques |
|---|---|---|
| Início | `home.html` | logo à esquerda, carrossel de destaques com placa de vidro, recentes com selo de formato, álbuns |
| Explorar | `explore.html` | humores, novidades por provedor |
| Player | `player.html` | capa grande, fundo da capa, progresso, selo `FLAC 24-bit/96 kHz`, controles grandes, volume, Letra/Saída/Fila; seletor de capa para ver o acento mudar |
| Biblioteca | `library.html` | Baixados, Curtidos, Álbuns, Playlists, "Criar playlist" |
| Fila | `queue.html` | tocando agora, a seguir com arrastar e remover |
| Busca | `search.html` (`#s=idle\|results\|empty`) | campo, filtros, botão "som ambiente", resultados agrupados |
| Configurações | `settings.html` | provedores, áudio (EQ placeholder), aparência, privacidade, sobre |
| Estados vazios | `states.html` | os quatro estados + carregando |

Os protótipos usam `backdrop-filter: blur()` de verdade (o mesmo efeito do Slint) e **não** usam nada que o Slint não faça: sem `saturate`, sem sombra interna, sem `mask`, sem animação infinita.

## 13. Decisões para o autor aprovar

1. **Âmbar** como cor de marca e fallback do acento (não é azul/verde/vermelho de provedor nenhum). Alternativa: outra cor, é só mudar `brand.ember` em `tokens.json`.
2. **Acento dinâmico no app inteiro** (padrão) vs. só no player/mini. Hoje: app inteiro, com chave "Âmbar" para fixar.
3. **Busca dentro da dock** como botão circular de acento (mobile) e item destacado da sidebar (desktop), em vez de aba. Mantém "não fazer search bar no topo".
4. **"Sem telemetria" como chave travada em ligado** (não dá para desligar), com texto explicando. A alternativa (chave livre) permitiria religar telemetria que o produto promete não ter.
5. **Marca K de barras + wordmark monolinha** (não imita os ícones de referência). O app icon é squircle grafite com K claro/âmbar.
6. **Fontes Hanken Grotesk + Geist Mono** (OFL). Substituir SF/Inter.
7. **Vidro também nos cartões de coleção da Biblioteca** (pedido): são 4 cartões com blur de 28 px; se medir lento, a opção é achatar só esses.
8. **Player sempre escuro**, mesmo no tema claro.
9. **Tema padrão escuro**; "Auto" segue o sistema.
10. Licença do app (GPL-3.0) afeta o texto de "Sobre" e a tela de licenças; os protótipos mostram GPL-3.0 como hipótese.

## 14. O que foi verificado

- `python3 design/tools/gen_tokens.py`: 51 pares de contraste, todos passam; regenera `tokens.slint` e `tokens.css`.
- `tokens.slint`, `atoms.slint`, `app-contract.slint` e `preview.slint` **compilam com Slint 1.19** e foram renderizados em PNG pelo renderer de software (que mostra o fallback sólido). Foram descobertos pelo compilador e já contornados: globais não podem ter `animate` (o acento corta), nomes de propriedades não podem colidir com o `focus-ring` do tema (renomeado `focus-ring-w`).
- Protótipos renderizados no Chromium em 390×844, 720×900 e 1280×800, claro e escuro, vidro e sem transparência: sem rolagem horizontal, sem erro de console, fontes e imagens carregadas, alvos de toque ≥ 44 px (exceto: seletor de capa do protótipo, bandas desabilitadas do EQ placeholder, barra de progresso da NowBar com 32 px).
- Não verificado: `backdrop-blur` em tela (o renderer disponível aqui foi o de software), e a medição de desempenho do blur em GPU fraca.

## 15. Regenerar

```
python3 design/tools/gen_tokens.py        # tokens.slint, tokens.css + checagem AA
python3 design/tools/make_assets.py       # logos, ícone do app, favicons, 45 ícones, capas, acentos (rsvg-convert, Pillow, magick)
python3 design/tools/build_prototypes.py  # HTML dos protótipos
```
