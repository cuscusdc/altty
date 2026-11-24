# SLINT_SPEC v4: contrato para implementar a UI no `kazzu-app`

Estado: **não implementar antes do aval do autor.** Esta spec descreve o que a prévia em `design/v4/slint/` já faz (compila no Slint 1.19 e está renderizada em `shots/slint/`) e o que o app precisa mudar para chegar lá. `crates/` não foi tocado.

Leia antes: `DIRECTION.md` (a ideia), `DESIGN.md` (o sistema), `ux/` (o comportamento).

## 0. Em uma página

| O que | Onde está | O que o implementador faz |
|---|---|---|
| Tokens | `slint/tokens.slint` (gerado de `tokens.json`) | Substitui `ui/theme/tokens.slint` do app; nunca edita à mão |
| Átomos | `slint/atoms.slint` | Move para `ui/components/`; mantém nomes e propriedades |
| Cromo | `slint/chrome.slint` | Substitui `ui/components/nav.slint` |
| Telas | `slint/screens.slint` | Referência de árvore, medidas e estados; troca os dados fictícios (`Demo`) pelo `AppState` |
| Prévia | `slint/preview.slint` + `tools/slint-shot` | Não vai para o app; serve para conferir |
| Dados fictícios | `slint/data.slint` (gerado) | Só `Icons` vai para o app; `Demo` fica no design e na galeria |

Diferenças grandes para o app de hoje:

1. Sai o "palco" colorido (`Stage`, malha, `stage-*`). O fundo é `Theme.paper`, liso.
2. Sai o acento tirado da capa (`accent-mode`, `accent-dark/light`). O acento é fixo.
3. Saem os chips do Início (`home-section`).
4. O player é outro componente (seção 5.6) e pede três dados novos do Rust (seção 6).
5. O mini-player passa a ser tingido pela capa.
6. O vidro tem uma receita só (`Glass`, `MediaGlass`) e regras de onde pode aparecer (seção 3).

## 1. Compilar e renderizar a prévia

```
df -h /home | tail -1                      # pare se houver menos de 20 GB livres
cd design/v4/tools/slint-shot
CARGO_TARGET_DIR=../../../../target/design-v4 cargo build --offline      # ~45 s com o cache do cargo; ~1 GB em disco
cd ../.. && tools/shoot_slint.sh            # 44 cenas em shots/slint/*.webp (FemtoVG, Xvfb, escala 2)
python3 tools/glass_proof.py                # prova de vidro: sai com 1 se algum caso reprovar
rm -rf ../../target/design-v4               # apague depois
```

Uma cena avulsa: `kazzu-slint-shot-v4 --shot "scene=player+queue cover=3 theme=light size=390x844 safe=32,24 out=/tmp/x.png"`. Chaves: `scene` (`tela[.estado][+folha]`), `size`, `theme`, `rt`, `rm`, `scroll`, `cover`, `safe`, `chrome`, `now`, `out`. `--renderer software` mostra o fallback sem desfoque.

O Cargo.toml da ferramenta liga `image-default-formats` no crate `slint`: sem isso as capas JPEG não decodificam. No app as capas já são decodificadas no Rust, então nada muda lá.

## 2. `Theme` (global)

O Rust escreve cinco propriedades; todo o resto é derivado.

| Propriedade | Tipo | Quem escreve |
|---|---|---|
| `dark` | bool | Rust (tema do sistema ou Ajustes) |
| `reduce-transparency` | bool | Rust e o interruptor de Ajustes (ligação de mão dupla) |
| `reduce-motion` | bool | idem |
| `blur-supported` | bool | Rust: `false` no renderer software |
| `window-w` | length | `AppWindow`, a cada mudança de largura |

Derivados: `glass-on` (= `blur-supported && !reduce-transparency`), `compact` (< 600), `wide` (≥ 1100), `gutter` (16, 20 ou 32).

Cores, medidas e tempos: ver `tokens.slint`. Convenção de nomes: `paper*`, `ink*`, `accent*`, `glass-*` (vidro do cromo), `media-*` (sobre capa e no player; iguais nos dois temas), `s-*` (espaço), `r-*` (raio), `text-*`, `dur-*`.

## 3. Vidro: regras que o código precisa respeitar

1. **Dois componentes.** `Glass` (cromo e folhas; segue o tema) e `MediaGlass` (sobre capa; sempre escuro com conteúdo branco). O console do player e o mini-player montam a mesma receita à mão porque têm tinta variável.
2. **Só no que flutua**, com conteúdo por trás. Grupos, cartões sobre o fundo liso e a barra lateral são `paper-2` sólido.
3. **Nunca vidro dentro de vidro** (o filho borraria o pai) e **nunca dentro de `ListView`**.
4. **No vidro, todo texto é `Theme.ink`** (ou `media-ink` nos painéis de mídia). Nada de `ink-2` nem de `accent` em texto: o contraste deles depende do que passa atrás. `TrackRow` tem `on-glass: true` para isso; `TextButton` recebe `ink: Theme.ink`.
5. **Item ativo dentro do vidro = bolha opaca** (`Theme.bubble`) + ícone preenchido + peso. Nunca só cor.
6. **Sem desfoque** (`!Theme.glass-on`): `Glass` vira `glass-solid` com fio `rule`; `MediaGlass`, o console e o mini-player viram `media-console` sólido; o fundo do player some.
7. **Orçamento no celular**: 4 painéis de cromo com desfoque (cabeçalho recolhido, mini-player, cápsula, busca) + os do conteúdo à vista (2 placas de cartão, ou 3 selos, ou 4 no detalhe, ou 4 no player). Se o Android real não aguentar, a primeira coisa a trocar é a placa do `PickCard`: desenhe ali a capa pré-desfocada recortada em vez de `backdrop-blur` (o visual é o mesmo, porque atrás da placa só existe a própria capa).

Recipiente de `Glass` (resumo): `backdrop-blur` 32 (40 em folha), `background` tinta, `border-width` 1 com `border-color` em degradê vertical (`glass-edge-hi` até `glass-edge-lo` em 42%), brilho (`Rectangle` com degradê `glass-sheen` até transparente nos 46% de cima), `drop-shadow-blur` 32 com deslocamento 12, `clip: true`.

## 4. Casca

Ordem de pintura, de baixo para cima, na `AppWindow`:

1. A tela da aba ou do detalhe (um `Flickable` de janela inteira; o conteúdo rola por baixo de tudo).
2. `Dock` (preso embaixo): esvaecimento leve, mini-player, cápsula, busca.
3. `TopBar` (preso em cima): aparece com `opacity` quando a rolagem passa de 56 dp (no detalhe, quando a placa do título sai: altura da capa menos 110).
4. Aviso temporário (toast): cápsula de `Glass` de 52 dp, 8 dp acima da doca, largura máxima 520; texto `ink`, ação em `TextButton` com `ink: Theme.ink`. Use `accessible-live-region` (existe no Slint 1.19).
5. `PlayerScreen` (cobre tudo, inclusive a doca).
6. Véu (`media-scrim`) e a folha aberta (`QueueSheet`, `MenuSheet`, `NewPlaylistSheet`, lista de playlists).

Medidas da doca (dp): margem lateral 16; cápsula 64 de altura, 12 acima da área segura inferior; busca 64 × 64, 8 à direita da cápsula; mini-player 60, 8 acima da cápsula; largura interna máxima 520, centralizada. `Dock.reserve` devolve a altura que as telas usam como `padding-bottom` (some 16).

Áreas seguras: `safe-top` entra no `padding-top` das telas e na altura da `TopBar`; `safe-bottom` sobe a doca e entra no `padding-bottom` do console e das folhas. A capa do player e a do detalhe ficam POR BAIXO da barra de status (os botões de vidro descem `safe-top`).

Voltar: a ordem é a da tabela em `ux/03-arquitetura-e-navegacao.md`, seção 5.4. O `Nav.back()` atual do app já cobre; acrescentar "Busca com texto limpa o texto".

## 5. Telas

Medidas em dp para 390 × 844. Tudo o que não está aqui está em `screens.slint`.

### 5.1 Cabeçalho das abas (`RootHeader`, `LargeTitle`)

Linha de 48: logo (`kazzu-wordmark-{dark,light}.svg`, 26 de altura) em `x = gutter`; Ajustes (`IconButton` tonal de 48, desenho 48) encostado à direita. Em Explorar, Biblioteca e Buscar, logo abaixo, `LargeTitle` de 52 de altura (34, 700, Inter Display, espaçamento -0,68). Em Ajustes e nas telas empilhadas o logo dá lugar ao Voltar.

### 5.2 Início (`HomeScreen`)

| Bloco | Componente | Medidas |
|---|---|---|
| Para ouvir agora | `SectionHeader` + `Rail` de `PickCard` | cartão 264 × 340 (300 × 386 a partir de 600), raio 22, vão 16; placa `MediaGlass` 68 de altura, recuo 8, raio 16; botão de tocar 44 branco |
| Faixas para você | `TrackRow` × 4 | linha 64; capa 48, raio 8; título 17, artista 15 `ink-2`; Mais (48) |
| Mixes, Álbuns em destaque | `Rail` de `Tile` | capa 148 (172), raio 12; título 15/600, apoio 13 |

Vão entre prateleiras: 32. Entre o cabeçalho e a primeira: 12. Estados (`state`): `ready`, `first-run` (um `Banner` no topo), `loading` (esqueletos com a forma dos cartões e das linhas; o cabeçalho continua igual), `offline` (`Banner` + "Neste aparelho"), `error` (`StateCard` com duas ações; sem código técnico na tela).

Dados: `highlights` → cartões; `quick-picks` → linhas; `mixes`, `albums` → ladrilhos; `recents` no modo sem conexão. `home-section` e `Highlight.caption` deixam de ser usados.

### 5.3 Explorar, Buscar, Biblioteca

- **Explorar**: Lançamentos em `PickCard` (os mesmos do Início); Momentos e gêneros em `MoodTile` neutro, 2 colunas (4 a partir de 600), 56 de altura, vão 12; Em alta em `TrackRow` com `number` e `ranked: true`. `Highlight.tint` deixa de ser usado.
- **Buscar**: `SearchField` (48, raio 14, borda `muted`, anel de foco por dentro); linha "Reconhecer pelo som" num `Group`, esmaecida, com `Tag` "Em breve"; "Buscas recentes" com Limpar. Com texto: `Segmented` (Tudo, Músicas, Álbuns, Artistas; Playlists entra rolando se não couber), "Melhor resultado" num `Group` de 72, depois as seções. Sugestões enquanto digita: linhas iguais às recentes, com a lupa no lugar do relógio. O campo ganha foco ao entrar pela busca redonda.
- **Biblioteca**: título grande com "Criar playlist" (`Button` com ícone, 172 de largura) na mesma linha; grade 2 × 2 de `LibraryTile` (vão 16 na horizontal, 20 na vertical; 4 em linha a partir de 600); o ladrilho é a arte (raio 22) com selo `MediaGlass` de 40 no canto e o texto embaixo; Baixados usa `soon: true`. Depois "Suas playlists" (linhas com `Mosaic` de 48) e "Neste aparelho" (`Group`).

### 5.4 Detalhe (`DetailScreen`)

Capa de ponta a ponta (largura da janela, até 520), sem margem, por baixo da barra de status. Sobre ela: Voltar à esquerda, Salvar e Mais à direita (`MediaIconButton`, 48 de toque, 44 de desenho) e a placa do título (`MediaGlass`, raio 22, 88 de altura, `x = gutter`, 16 acima da base): título 28/700, apoio 15. Abaixo: Tocar (`Button primary`) e Aleatório, mesma largura; linhas com número (a atual troca o número pelas barras); rodapé em 13 `ink-2`. Artista e playlist usam a mesma tela (`DetailKind`).

Atenção ao limite do Slint: o detalhe usa `Flickable` + `for`, que instancia todas as linhas. Para listas longas, pagine no Rust.

### 5.5 Mini-player (`MiniPlayer`, em `chrome.slint`)

Cápsula de 60, raio 30, `backdrop-blur` 32. Dentro, de baixo para cima: a capa pré-desfocada (`tint`) a 55% de opacidade cobrindo a cápsula; `media-console` com alfa `dim` (o `mini-dim` do Rust); o conteúdo em branco: capa redonda de 44, título 15/600, artista 13, tocar/pausar e próxima (`IconButton` com `media: true`). A linha de progresso de 2 dp fica na base. Toque na cápsula abre o player.

### 5.6 Player (`PlayerScreen`)

```
y = 0                 capa nítida, quadrada, largura da janela (até 520)
y = 0                 por baixo dela: imagem `backdrop` (192 x 464) desenhada com a largura da capa e altura = largura x 464/192
y = safe-top + 8      fechar | "Tocando de <origem>" | mais        (MediaGlass, 44)
y = capa - 24         console até a base: cantos de 32 em cima, backdrop-blur 24, tinta media-console com alfa `player-dim`
```

Console: `VerticalLayout` com `alignment: space-between`, recuos 24 nas laterais (16 abaixo de 368 de largura), 32 em cima, 20 + `safe-bottom` embaixo. Linhas:

| Linha | Conteúdo | Medidas |
|---|---|---|
| 1 | título 22/700 Display + artista 17; coração (`IconButton media`) | |
| 2 | `Scrubber` (28 de altura aqui; o toque efetivo é a linha de 48 no app) + tempos | decorrido 13 à esquerda (caixa de 40), origem e formato 12/600 no centro, o que falta 13 à direita |
| 3 | aleatório 48, anterior 56 (ícone 36), tocar 72 (círculo `media-ink`, ícone 32 `media-console`), próxima 56, repetir 48 | `alignment: space-between` |
| 4 | volume: ícone 20, `Scrubber`, ícone 20 | |
| 5 | "Este aparelho" e "Fila": dois `Pressable` de 48, raio 24, fundo `media-well`, fio `media-edge-lo`, ícone 20 + texto 15/600 | mesma largura, vão 12 |

Estados que a prévia não mostra e o app precisa: aleatório e repetir ligados (fundo `media-ink` a 22% no botão); carregando (o ícone de tocar vira o indicador de espera; a barra pulsa); erro (uma linha em `media-ink` entre as linhas 1 e 2 com "Tentar de novo" em `TextButton ink: Theme.media-ink`); curtido (`heart-fill`).

Gesto: arrastar a capa para baixo fecha (o app já tem o gesto; o botão de fechar é a alternativa).

### 5.7 Folhas

`Glass` com `sheet: true` e `flat-bottom: true`, largura até 552, raio 32 em cima, alça de 36 × 5. Fila: de `safe-top + 48` até a base. Menu e nova playlist: altura do conteúdo, presas embaixo. Linhas da fila: `TrackRow` com `on-glass: true`, `more: false` e dois `IconButton` (mover para cima, tirar). Menu: `SheetAction` de 52; o que ainda não existe usa `soon: true` (texto a 60% e `Tag` "Em breve").

### 5.8 Ajustes

`GroupHead` (15/600 `ink-2`) + `Group` de `GroupItem`. Linhas com escolha entre poucas usam `SegItem` (título em cima, `Segmented` embaixo, 104 de altura). Interruptores: `Switch` com `checked <=> Theme.reduce-transparency` e `Theme.reduce-motion`.

## 6. Contrato (`AppState`): o que muda

### Novos

| Propriedade | Tipo | Conteúdo |
|---|---|---|
| `backdrop` (muda de formato) | image | 192 × 464: a capa (192 × 192) em cima, a capa espelhada na vertical logo abaixo, e os 80 px do topo da capa de novo; `GaussianBlur` de raio 3 na imagem inteira; saturação × 1,2 |
| `backdrop-wide` | image | o `backdrop` antigo: capa 96 × 96 com desfoque forte (raio 9) e saturação × 1,35. Usado no mini-player e atrás do player em janelas largas |
| `player-dim` | float | escurecimento do console, 0,24 a 0,64 (regra abaixo) |
| `mini-dim` | float | escurecimento do mini-player, 0,30 a 0,70 |
| `now-source-line` | string | "YouTube Music, Opus 160 kbps" ou "Neste aparelho, FLAC 24-bit/96 kHz" (hoje dá para montar de `now-source` + `quality-label`) |

Regra dos escurecimentos (referência em `tools/make_covers.py`), com a capa reduzida a 12 × 12 por média de caixa:

```
L  = maior luminância relativa (WCAG) entre as 144 células
V  = 255 * gama_sRGB(L)                       # cinza equivalente, 0..255
player-dim = clamp((V - 112) / (V - 12) + 0.02, 0.24, 0.64)      # 0 se V <= 112, antes do clamp
X  = 0.45 * 255 + 0.55 * V                    # mini-player: 45% do que rola atrás (pior caso: branco), 55% da capa
mini-dim   = clamp((X - 112) / (X - 12) + 0.02, 0.30, 0.70)
```

112 é o cinza sobre o qual texto branco dá 5:1. Sem capa: `player-dim` 0,64 e `backdrop` vazio (o console fica sólido).

### Deixam de ser usados (podem sair do contrato)

`home-section`, `home-section-changed`, `accent-mode`, `accent-mode-changed`, `accent-dark`, `accent-dark-ink`, `accent-light`, `accent-light-ink`, `home-error-code` (vai só para o arquivo de relatar problema), `Highlight.caption`, `Highlight.tint`, `search-ideas`.

### Mudam de lugar na tela

| Hoje | Na v4 |
|---|---|
| `open-lyrics`, `set-sleep-timer`, `download-track` | Linhas "Em breve" do menu; os callbacks só voltam a ser chamados quando os recursos existirem |
| `output-name` | Texto do botão de saída no console ("Este aparelho") |
| `output-note` ("Original, sem equalizador") | Descrição acessível da linha de origem e formato; visível ao tocar nela (aviso). Decisão do autor: ver o relatório |
| `Track.why` | Não aparece na linha; pode entrar no rótulo acessível |
| `first-run-banner` | `Banner` com título "Pronto para tocar" |

## 7. O que aprendi do Slint 1.19 nesta prévia

| Assunto | O que acontece | O que fazer |
|---|---|---|
| `image-fit: cover` | Se o componente tem uma propriedade chamada `cover`, o compilador entende a propriedade | Escreva sempre `ImageFit.cover` |
| Propriedade chamada `color` | "Cannot override property 'color'" em quem herda de `Rectangle` ou `Text` | Use outro nome (`ink`, `tint`) |
| `min-height` explícito num componente com layout dentro | Substitui o mínimo calculado: a linha não cresce com texto de duas linhas | Ponha um espaçador de largura 0 e `min-height` dentro do layout |
| `width: parent.width - ...` num `Text` dentro de layout dentro de `Flickable` | Laço de vínculos | Embrulhe num `HorizontalLayout` com `padding` |
| `viewport-y` | Depreciado | `content-y` |
| Elementos de largura fixa dentro do layout de um `Flickable` vertical | Alargam o conteúdo e empurram o cabeçalho para fora | Posicione-os num `Rectangle` com `clip` |
| `accessible-selected` | Não existe | `accessible-item-selected`; há também `accessible-live-region` |
| `backdrop-blur` com `background` totalmente transparente | Funciona (usado no mini-player) | |
| Borda em degradê | Funciona no FemtoVG | No software cai para sólido |
| `transform-scale`, `inner-shadow-*`, `drop-shadow-spread` | Existem na 1.19; não foram usados aqui | Servem para o estado pressionado e para a transição do player |
| Numerais tabulares, `saturate` no desfoque, cantos contínuos, encaixe em `Flickable` | Não existem | O design não depende deles |

## 8. Movimento

| Transição | Como |
|---|---|
| Abrir o player | O `PlayerScreen` sobe da base (`y` de altura da janela até 0) em `dur-long`, curva `cubic-bezier(0.2, 0.9, 0.2, 1)`; o mini-player esmaece em `dur-short` |
| Folhas | Sobem da base em `dur-long`; o véu esmaece junto |
| Bolha da aba | `animate background` em `dur-short`; quando houver tempo, deslizar a bolha (uma só, com `x` animado) |
| Cabeçalho recolhido | `opacity` em `dur-short` |
| Pressionado | `opacity` 0,82 a 0,9 em `dur-micro` (ou `transform-scale: 0.97`) |
| Reduzir movimento | `Theme.dur-short` e `dur-long` já caem para 120 ms; troque os deslocamentos por `opacity`; `EqBars` para |

## 9. Responsivo

| Faixa | Feito na prévia Slint | O que falta |
|---|---|---|
| Compacta (até 599) | Tudo | |
| Média (600 a 1099) | Doca centralizada (máx. 520), cartões maiores, Biblioteca 4 em linha, momentos em 4 colunas, player em coluna central sobre a capa desfocada e escurecida | Detalhe com a capa ao lado das faixas a partir de 840 |
| Larga (1100 ou mais) | **Não construída no Slint.** Referência: `prototypes/*.html` em 1280 e `shots/proto/*-1280-*.webp` | Barra lateral sólida de 264 (logo no topo, 4 destinos, playlists), barra de reprodução de vidro embaixo (72), player com a capa inteira ao lado do console |

## 10. Acessibilidade: o que conferir ao implementar

1. `--hit-targets` da galeria com os componentes novos: todo `Pressable` com 48 × 48 ou mais (os `IconButton` de 56 do transporte, o `Switch` de 64 × 48, os itens do `Segmented` de 40 + folga).
2. Contraste: rode `tools/check_render.py` nos protótipos e compare com as capturas do app; no app, confira o player com uma capa branca e uma preta.
3. Foco: o anel de `Pressable` é instantâneo; sobre capa e console use `ring-color: Theme.media-ink`.
4. Rótulos em pt-BR vindos do Rust; capas decorativas com `accessible-role: none`.
5. TalkBack: não testado.

## 11. Ordem sugerida de implementação

1. Tokens, `Icons`, átomos, `Glass`. Galeria: cena de átomos.
2. Cromo (`Dock`, `TopBar`) com o Início. Rode a prova de vidro no app.
3. Player e mini-player (precisa dos dados da seção 6).
4. Folhas (fila, menu, playlists, nova playlist).
5. Buscar, Biblioteca, Detalhe, Explorar, Ajustes.
6. Estados e sem conexão em todas.
7. Faixa média; depois a larga.

Em cada passo: cenas na galeria (claro, escuro, sem transparência, 360 e 390), `--hit-targets`, `--glass-proof`.

## 12. Fora desta spec

Equalizador, letras, downloads, timer, troca de saída, reconhecer pelo som, outros provedores, notificação de mídia do Android, gestos de deslizar na fila, transição de elemento compartilhado da capa. Onde tocam a UI de hoje, aparecem como "Em breve".
