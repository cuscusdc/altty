# Kazzu v4: design system

Direção: `DIRECTION.md`. UX: `ux/`. Contrato do implementador: `SLINT_SPEC.md`. Fonte única dos valores: `tokens.json` (gera `prototypes/assets/tokens.css` e `slint/tokens.slint`).

## 1. Princípios

1. **A cor é das capas.** A interface é neutra. O acento marca estado e nada mais.
2. **Vidro só no que flutua**, e só onde há conteúdo passando por trás.
3. **No vidro, texto é sempre a tinta principal.** Hierarquia por tamanho e peso.
4. **Um componente por função.** Cápsula para a doca e os botões, controle segmentado para escolher uma entre poucas, linha para lista, grupo para ajustes.
5. **Conteúdo primeiro.** A primeira capa do Início começa a 112 dp do topo (13% da tela de 844).
6. **Honestidade.** Recurso que não existe aparece esmaecido com "Em breve"; nunca um controle que finge.

## 2. Cor

Valores em OKLCH no `tokens.json`; abaixo, o sRGB resultante. Matiz neutra 265 com croma de 0,002 a 0,012: nenhum cinza é puro e nenhuma superfície é preto ou branco absolutos.

| Token | Escuro | Claro | Uso |
|---|---|---|---|
| `paper` | `#07080a` | `#f3f4f6` | Fundo da página |
| `paper-2` | `#16171a` | `#fcfdfe` | Superfície elevada: grupos, ladrilho vazio, sólido do vidro |
| `paper-3` | `#25262a` | `#e5e6e9` | Preenchimento de controle, pressionado, esqueleto |
| `ink` | `#f2f3f6` | `#0f1217` | Texto e ícones |
| `ink-2` | `#aeb1b6` | `#4d5057` | Texto secundário, só sobre superfícies sólidas |
| `muted` | `#8a8c91` | `#696c72` | Ícone inativo, borda de campo (3:1) |
| `rule` | `#2b2d30` | `#d8d9dc` | Fio de 1 dp |
| `accent` (Brasa) | `#ff7345` | `#cc2c05` | Faixa tocando, curtido, interruptor ligado, links, foco |
| `accent-ink` | `#1a0a06` | `#fffaf5` | Texto sobre o acento |
| `bubble` | `#595b60` | `#fcfdfe` | Bolha opaca do item ativo dentro do vidro |
| `primary` / `primary-ink` | `#f2f3f6` / `#0b0c0f` | `#0f1217` / `#fafbfc` | Botão principal (um por tela) |
| `error` | tom de vermelho, sempre com ícone e texto | | Erros |

Postura de cor: **contida**. O acento ocupa menos de 5% de qualquer tela. Momentos e gêneros (Explorar) são ladrilhos neutros (`paper-2`): a regra da v4 é que cor só vem de capa.

Sobre capas e no player a paleta é a mesma nos dois temas (`media-*`): tinta quase preta e texto branco puro.

## 3. Tipografia

Inter (OFL), arquivos estáticos em `fonts/`: Regular 400, SemiBold 600, Bold 700 e Inter Display Bold para títulos. Três pesos em uso.

| Papel | Tamanho / linha | Peso | Onde |
|---|---|---|---|
| Título grande | 34 / 40 | 700 Display, -0,02 em | Explorar, Biblioteca, Buscar, Ajustes |
| Título de tela sobre capa | 28 / 34 | 700 Display | Álbum, playlist, artista |
| Título de seção | 22 / 28 | 700 Display | Prateleiras, folhas, título da faixa no player |
| Corpo | 17 / 22 | 400 (600 para destaque) | Título de linha, texto de estado |
| Secundário | 15 / 20 | 400 (600 em botões) | Artista, botões, cabeçalho de grupo |
| Meta | 13 / 18 | 400 | Contagens, tempos, notas |
| Rótulo | 12 / 16 | 600 | Abas, selos |

Regras: nada abaixo de 12; frase normal em tudo (nenhuma caixa alta espaçada); um nome por linha com reticências; números de tempo alinhados por caixa, porque o Slint não expõe numerais tabulares.

## 4. Espaço, forma, tamanho

| Escala | Valores (dp) |
|---|---|
| Espaço | 2, 4, 8, 12, 16, 20, 24, 32, 48, 64 |
| Raio | 8 miniatura, 12 capa, 14 grupo e campo, 16 placa, 22 cartão, 32 folha e console, cápsula |
| Alvo de toque | 48 no mínimo, mesmo quando o desenho tem 32, 40 ou 44 |
| Linha de lista | 64 |
| Doca | cápsula 64, busca 64, mini-player 60, vãos de 8, 12 acima da área segura |
| Margem lateral | 20 (16 abaixo de 368 de largura; 32 a partir de 600) |

Raios são concêntricos: placa de 16 dentro de cartão de 22 com recuo de 8; bolha de 28 dentro de cápsula de 32 com recuo de 4.

## 5. Vidro

### 5.1 Receita (de baixo para cima)

1. **Desfoque de fundo real**: 32 dp no cromo, 40 dp em folhas, 24 dp no console do player.
2. **Tinta** que garante AA no pior caso possível atrás do painel.
3. **Brilho**: degradê branco que some antes do meio do painel.
4. **Fio de luz** de 1 dp, forte em cima (branco a 46% no escuro) e fraco embaixo.
5. **Sombra** do painel sobre o conteúdo.

| Painel | Tinta escuro | Tinta claro | Desfoque | Pior caso (texto `ink`) |
|---|---|---|---|---|
| Cromo: cápsula, busca, cabeçalho recolhido | cinza fosco `#343539` a 74% | branco a 54% | 32 | 4,9:1 com branco puro atrás; 5,3:1 com preto puro atrás |
| Folhas: fila, menu, playlists | cinza fosco `#313336` a 76% | branco a 62% | 40 | 5,4:1; 6,7:1 |
| Placa sobre capa e botões sobre capa | quase preto a 60% | igual | 32 | 4,9:1 com branco puro atrás |
| Mini-player | capa da faixa desfocada (55%) + o que rola atrás (45%), sob quase preto a `mini-dim` (0,30 a 0,70, por capa) | igual | 32 | texto branco a 5:1 com branco puro atrás |
| Console do player | quase preto a `dim` (0,24 a 0,64, por capa) | igual | 24 | 5,9:1 no `dim` máximo com branco puro atrás |

Os números exatos e todos os pares estão em `contrast-tokens.md` (gerado). A composição é calculada em sRGB, como o Slint e o navegador fazem.

**Por que estas tintas.** Para o texto branco passar de 4,5:1 sobre QUALQUER coisa, o que atravessa um vidro escuro não pode passar de cerca de 42% da luz: é física, não gosto. Dentro desse limite há duas escolhas, e a v4 usa as duas:

- **No cromo e nas folhas, um cinza fosco mais claro que o fundo.** Sobre o preto da página o painel ainda aparece como uma lâmina (`#2b2c2f` sobre `#07080a`), com o fio de luz desenhando a borda. Atravessa menos cor (26%), mas nunca vira um buraco preto. Foi a correção pedida pela revisão de contexto limpo: a primeira tentativa usava tinta quase preta e, sobre listas escuras, lia como cartão opaco.
- **No player e no mini-player, a própria capa dá a cor.** O console é quase transparente quando a capa deixa (`dim` 0,24) e o mini-player carrega a capa desfocada dentro dele, então os dois têm cor mesmo quando nada passa por trás.

### 5.2 Onde o vidro é obrigatório (e o que precisa estar atrás)

| Painel | O que passa por trás | Tela |
|---|---|---|
| Cápsula de abas e busca redonda | Idem | Idem |
| Cabeçalho recolhido | O topo da lista que rolou | Abas, detalhe, Ajustes |
| Placa do cartão grande | A própria capa do cartão | Início |
| Selos da Biblioteca | A arte do ladrilho | Biblioteca |
| Botões e placa do título | A capa de ponta a ponta | Álbum, playlist, artista |
| Botões e contexto do topo | A capa | Player |
| Console | A base da capa e o reflexo dela | Player |
| Mini-player | A capa da faixa (dentro dele) e a lista que rola | Todas as abas e o detalhe |
| Folhas | O player ou a tela de onde vieram | Fila, menu, playlists, nova playlist |

### 5.3 Onde o vidro é proibido

Dentro de outro vidro; dentro de `ListView`; em grupos de Ajustes; na barra lateral do desktop; em cartões sobre o fundo liso. Nesses lugares a superfície é `paper-2` sólida.

### 5.4 Orçamento

No celular, no máximo 4 painéis com desfoque ao vivo vistos ao mesmo tempo no cromo (cabeçalho, mini-player, cápsula, busca), mais os do conteúdo à vista: 2 placas no Início e em Explorar, 3 selos na Biblioteca, 4 no detalhe, 4 no player. Se o aparelho não aguentar, o app oferece Reduzir transparência; as placas dos cartões são as primeiras candidatas a virar imagem pré-desfocada (ver `SLINT_SPEC.md`).

### 5.5 Sem vidro

Com **Reduzir transparência**, ou num renderer sem desfoque (software), todo vidro vira `paper-2` com fio `rule`, e o player perde o fundo da capa (console sólido). O contraste só melhora. Renders: `shots/slint/home-rt.webp`, `player-rt.webp`, `home-software.webp`.

### 5.6 Prova

`shots/glass-proof.md`: onze casos medidos no Slint real (mini-player, cápsula, cabeçalho, console com três capas, folha da fila, folha de ações, placa do cartão). Em cada um, a região do painel difere do sólido, a cor dela segue a cor do que está atrás (cosseno de 0,90 a 1,00) e o detalhe de trás cai para menos de um terço. A transmissão medida vai de 0,11 (cápsula e folha no escuro, que são foscas de propósito) a 0,86 (console sobre capa escura).

## 6. O player e o mini-player

| Camada | O que é |
|---|---|
| 1 | Fundo: uma imagem de 192 × 464 que o Rust prepara (a capa em cima, a capa **espelhada** embaixo, desfoque leve e 20% a mais de saturação), desenhada na largura da tela a partir do topo |
| 2 | A capa nítida, quadrada e **inteira**, de ponta a ponta, por baixo da barra de status |
| 3 | Sobre a capa, em vidro: fechar, "Tocando de Nome", mais |
| 4 | O console: sobe 24 dp sobre a base da capa, cantos de 32 em cima, vai até a base da tela, sobre o reflexo |

O reflexo é a ideia da mesa de vidro: a capa aparece de cabeça para baixo logo abaixo dela mesma, e o console é o vidro por cima. O desfoque do console é moderado (24 dp) de propósito, para as formas grandes da capa continuarem reconhecíveis através dele: o disco de vinil vira um disco borrado, a tigela azul vira um arco azul. Não há degradê desenhado no player: tudo o que tem cor é a capa.

No console, de cima para baixo: título (22, 700) e artista (17) com o coração; barra de posição com o tempo decorrido, a origem e o formato ("YouTube Music, Opus 160 kbps") e o tempo que falta; transporte (aleatório, anterior, tocar em círculo branco de 72, próxima, repetir); volume; dois botões com caixa, a saída atual ("Este aparelho") e Fila. Letra e Timer ficam no menu, com "Em breve", até existirem.

Todo texto do console é branco puro. `dim`, o quanto o console escurece, sai da própria capa: pega-se a região mais clara dela (grade 12 × 12) e escolhe-se o menor escurecimento que mantém o branco a 5:1. Capa escura dá console quase transparente (0,24); capa branca dá console cinza (0,61 a 0,64). Os botões de baixo são poços mais escuros dentro do vidro (nunca vidro dentro de vidro, nunca um preenchimento claro, que roubaria contraste).

O **mini-player** é o console em miniatura: a mesma capa desfocada tinge a cápsula, com texto branco nos dois temas. Ao abrir, ele vira o console da mesma cor.

Em telas mais largas que a capa (tablet), a capa pré-desfocada cobre a janela bem escurecida (60%) e a coluna do player fica no centro. No desktop a capa inteira fica ao lado do console.

## 7. Componentes

Todos existem em `slint/atoms.slint` e `slint/chrome.slint`, compilam no Slint 1.19 e aparecem nos renders.

| Componente | Função | Estados |
|---|---|---|
| `Glass`, `MediaGlass` | O material | com e sem desfoque |
| `Pressable` | Base de todo controle: papel, rótulo, foco, pressionado | padrão, sobre, foco, pressionado, desativado |
| `IconButton`, `MediaIconButton` | Botão de ícone (48 de toque) | idem; tonal; sobre capa |
| `Button`, `TextButton` | Ação com texto; principal preenchido, secundário tonal, link | idem |
| `Switch`, `Segmented` | Liga e desliga; uma entre poucas | marcado; bolha opaca no ativo |
| `SearchField` | Campo com lupa, limpar e anel de foco por dentro | vazio, com texto, foco |
| `Cover`, `Mosaic` | Capa (com disco quando falta) e mosaico 2 × 2 | com e sem imagem |
| `TrackRow` | Linha de música: capa ou número, título, artista, tempo, mais | atual, sobre vidro, com ações extras |
| `GroupItem`, `Group` | Linha e bloco de Ajustes | com nota, valor, seta, esmaecido |
| `PickCard`, `Tile`, `LibraryTile`, `MoodTile` | Cartões; o de momento é neutro | pressionado; "Em breve" |
| `SectionHeader` | Título de prateleira com "Ver tudo" | |
| `Scrubber` | Posição e volume: desenho de 6, toque de 48, cresce ao arrastar | foco, arrastando |
| `StateCard`, `Banner`, `Skeleton`, `Tag` | Estados e avisos | |
| `MiniPlayer`, `TabBar`, `SearchOrb`, `Dock`, `TopBar` | Cromo | aba ativa, busca ativa, sem faixa |

## 8. Telas

| Tela | Protótipo | Render no Slint |
|---|---|---|
| Início | `prototypes/home.html` | `shots/slint/home.webp`, `home-scrolled`, `home-light`, `home-360`, `home-768` |
| Explorar | `explore.html` | `explore` |
| Buscar | `search.html`, `search-results.html`, `search-empty.html` | `search`, `search-results`, `search-empty` |
| Biblioteca | `library.html`, `library-empty.html` | `library`, `library-light`, `library-empty`, `library-768` |
| Álbum, playlist, artista | `album.html`, `playlist.html`, `artist.html`, `playlist-empty.html` | `album`, `album-scrolled`, `album-light` |
| Player | `player.html` (use `#c=03` para trocar a capa) | `player`, `player-delta`, `player-white`, `player-dark`, `player-768` |
| Fila | `queue.html` | `queue`, `queue-light` |
| Folhas | `sheet-menu.html`, `sheet-playlists.html`, `sheet-new-playlist.html` | `menu`, `menu-light`, `new-playlist` |
| Ajustes | `settings.html` | `settings`, `settings-light` |
| Estados | `home-first-run.html`, `home-loading.html`, `home-offline.html`, `home-error.html` | `home-first-run`, `home-loading`, `home-offline`, `home-error` |
| Sem transparência | qualquer tela com `#rt=1` | `home-rt`, `home-rt-light`, `player-rt`, `queue-rt`, `home-software`, `player-software` |
| Áreas seguras | | `home-safe`, `player-safe` |

Tablet e desktop: os protótipos são a referência (`shots/proto/*-768-dark.webp`, `*-1280-*.webp`). No Slint só as faixas compacta e média foram renderizadas; a barra lateral do desktop está especificada, não construída.

## 9. Movimento

| Coisa | Duração | Curva |
|---|---|---|
| Pressionar, cor, foco | 120 ms (o anel de foco é instantâneo) | saída |
| Bolha da aba, interruptor, cabeçalho recolhido | 220 ms | entra e sai |
| Abrir o player, folhas | 420 ms | `cubic-bezier(0.2, 0.9, 0.2, 1)`: chegada tipo mola, sem ressalto |
| Barras da faixa atual | 1,1 s, contínua | única animação infinita; para ao pausar |

Com Reduzir movimento: sem deslocamentos; tudo vira esmaecer de 120 ms; as barras param.

## 10. Acessibilidade

- Contraste medido: `contrast-report.md` (cerca de 3500 textos em 188 capturas, 0 abaixo de AA; o número exato está no relatório).
- Alvos de 48 dp: garantidos por `Pressable` e pelos tamanhos dos átomos; falta rodar `--hit-targets` da galeria com os componentes novos.
- Foco: anel de 2 dp na cor de foco (branco sobre capa), instantâneo.
- Estado nunca só por cor: aba ativa tem bolha e ícone preenchido; faixa atual tem barras e peso; curtido tem ícone preenchido.
- Leitor de tela: papéis e rótulos em pt-BR em todo controle; capas decorativas fora da árvore.

## 11. O que NÃO foi verificado

Desempenho do desfoque em aparelho Android real e no renderer Skia; TalkBack; teste com pessoas; a barra lateral do desktop no Slint; gestos (arrastar o player, reordenar a fila); o teclado virtual sobre o campo de busca e sobre a folha de nova playlist.
