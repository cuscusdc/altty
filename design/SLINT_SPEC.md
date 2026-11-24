# Kazzu — especificação de UI em Slint (v3: Apple-like, mobile-first)

Para o implementador de UI do `kazzu-app`. **Siga à risca**: nomes, medidas e estados abaixo vêm de `design/prototypes/` e de `design/slint/` (abra a tela ao lado enquanto implementa; todas em 390 e 360 px primeiro). Slint 1.19, renderers FemtoVG/Skia (software = fallback sem blur). Identificadores em inglês; textos visíveis em pt-BR. UX (fluxos, estados, gestos, acessibilidade): `design/ux/UX_PLAN.md` (**inalterado**). Receita visual e decisões: `design/DESIGN.md`.

**O que mudou da v2 para a v3 (só visual):** fundo neutro (sem palco colorido); vidro CLARO (tint branco + sombreado neutro) só no cromo flutuante; listas agrupadas opacas; Inter/Inter Display; título grande que encolhe ao rolar; cabeçalho de rolagem em vidro; barra em cápsula com ícone preenchido no acento; player com capa desfocada de fundo; um só acento. Renderizado de verdade no Slint (FemtoVG): `design/slint/*.png`.

## 0. Arquivos prontos (compilam no Slint 1.19; mova para `crates/kazzu-app/ui/`)

| Arquivo | O que é | Regra |
|---|---|---|
| `design/slint/tokens.slint` | `export global Theme` | **Gerado** por `tools/gen_tokens.py`. Não editar; mude `tokens.json` e regenere. |
| `design/slint/atoms.slint` | `Stage` (neutro; capa desfocada só no player), `GlassPanel` (3 materiais), `EdgeFade`, `Divider`, `GroupCard`, `Chip`, `CoverImage`, `IconButton`, `GlassIconButton`, `PillButton`, `FormatBadge`, `SourceTag`, `KSwitch`, `KSlider` | Usar como estão. Correções da implementação v1 mantidas (`FocusScope` com `focus-on-click: false`; `KSlider.value` como `in`). |
| `design/slint/chrome.slint` | `NavDock` (cápsula), `SearchButton`, `MiniPlayer`, `ScrollHeader`, `SectionHeader`, `TrackRow`, `HeroCard`, `Sidebar`, `NowBar` | Moléculas; o app as adapta aos seus dados (`AppState`). |
| `design/slint/app-contract.slint` | enums, structs, `AppState` | Fronteira Rust ↔ UI. **Inalterado.** |
| `design/slint/preview.slint` | 10 cenas (home, home-playing, home-wide, library, settings, player, player-wide, sheet, proof) | Só para o `design/tools/slint-shot` (renderiza PNG). |
| `design/fonts/*.ttf` | **Inter** 400/500/600/700, **Inter Display** 600/700, Geist Mono 400/500 (OFL; licenças ao lado) | `import "…ttf";` e `default-font-family: Theme.font-sans`; títulos `font-family: Theme.font-display`. |
| `design/assets/icons/*.svg` | **57 ícones** (v3 acrescenta `home-fill`, `explore-fill`, `library-fill`) | `Image { source: @image-url(…); colorize: …; }` |

### 0.1 Diferenças para o código atual do app (checklist do implementador)

1. **`Theme` — removido:** `stage-a..d(-dark/-light)`, `glass-tint(-dark/-light)`, `glass-fill*`, `ambient-*`, `scrim-bottom`, `glass-bg`/`glass-bg-strong`/`glass-blur-radius`/`glass-blur-strong-radius` (legado v1). O Rust **deixa de escrever** o palco e o matiz do vidro: o fundo é neutro.
2. **`Theme` — novo:** `glass-tint-1..3`, `glass-shade-1..3`, `glass-bg-1..3`, `glass-tint-on`, `glass-ring`, `ink-glass-2`, `fill`, `fill-2`, `cell`, `cell-hover`, `edge-fade`, `scrim`, `im-veil`, `font-display`, `ff-*` (família por estilo), `fs-title-3`, `fs-footnote`, `header-h`, `group-radius`, `hero-radius`, `tile-radius`, `sheet-radius` 28, `nowbar-h` 76, `row-inset`. Tipografia: corpo 17, subhead 15, footnote 13.
3. **`Stage`:** só `immersive` (player) usa a capa; nas demais telas é `Theme.bg` puro. `AppState.backdrop` continua (capa 96×96 desfocada, σ 4), mas **saturação ×1,15 e sem escurecer no Rust** (o véu é do `Stage`). `AppState.ambient` não existe mais.
4. **`GlassPanel`:** `level` 1–3 mantido, `strong`/`light-blur` legados trocados por `level 2` / `light-blur`. Agora são **duas camadas** (sombreado = `background`; tint = filho) e o fallback sólido apaga o tint (`Theme.glass-tint-on`).
5. **Listas** (`TrackRow`, ajustes, coleções) vão dentro de **`GroupCard`** (opaco, raio 20) com `Divider` recuado em `Theme.row-inset − 8`. Nada de vidro em linha.
6. **Cabeçalho:** `ScrollHeader` (marca à esquerda, Ajustes à direita, título pequeno no centro) sobre o `Flickable`; `p = min(1, content-y / 60px)`; o título grande (`font-display` 34) fica **no fluxo**, com 52 px de altura, logo abaixo de `Theme.header-h`.
7. **Barra:** `NavDock` com indicador deslizante e **ícone preenchido** no destino ativo; `SearchButton` ativo = cápsula neutra + ícone no acento. Reserve `Theme.chrome-reserve` (184) e ponha `EdgeFade` (176 px) sob a barra.
8. **Botão primário** (`PillButton primary`) é **neutro invertido**; o acento não é cor de botão.
9. **Limites do Slint 1.19 que viraram regra:** o backdrop só tem blur (sem saturação: compensado pelas capas vivas); sem cantos contínuos (usar o raio nominal); sem algarismos tabulares (tempos em Geist Mono); `border-color` aceita gradiente no FemtoVG (**conferido**, aparece no vidro; se um renderer ignorar, cai na cor de repouso); `Flickable.viewport-y` está **depreciado**: use `content-y`; uma propriedade chamada `cover` conflita com `image-fit: cover`, use `ImageFit.cover`.
10. **Player:** força `Theme.dark = true` enquanto aberto; `Stage immersive` com a capa; capa 22 de raio com sombra neutra; transporte em `GlassPanel level 2` com o **play como círculo branco** (único opaco); barra fina (`KSlider`, 6 → 10 px, polegar só no hover/foco).

## 1. Regras gerais

1. **Nunca** cor, tamanho, raio ou duração literal: sempre `Theme.*` (exceções em §6.4).
2. Todo botão de ícone: `IconButton` com `label` em pt-BR; todo controle tem `accessible-role` e rótulo; ações que no protótipo são gestos têm **também** botão (UX_PLAN §8).
3. **Alvo mínimo 48 × 48** (`Theme.hit`). Foco: anel de 2 px (`Theme.focus-ring`), sem animação, **nunca coberto** pela barra/mini (reserve `Theme.chrome-reserve` = 184 de padding inferior nas telas de rolagem).
4. Listas longas = `ListView`. **Nada de vidro dentro de linhas.** No máximo **6** `GlassPanel` com blur visíveis ao mesmo tempo (barra, buscar, mini, até 3 cartões).
5. **Nunca aninhar `GlassPanel` dentro de `GlassPanel`** (o filho borraria o pai). Elementos dentro de vidro (chips, campos, botões) usam fill simples `Theme.ink.with-alpha(0.12)` e borda `Theme.ink.with-alpha(0.16)`.
6. Animação só com `animate` em componentes. Sem animação infinita. Durações `Theme.dur-*`; curvas por literal (`cubic-bezier(0.22, 0.9, 0.28, 1)` chegadas, `cubic-bezier(0.45, 0, 0.2, 1)` trocas).
7. `Theme.window-w` atualizado no `AppWindow` (`changed width => { Theme.window-w = self.width; }`). Classes: `Theme.compact` (< 600) **base do design**, médio 600–839 (`!compact && !expanded`), `Theme.expanded` (≥ 840), `Theme.wide` (≥ 1200). **Escreva a árvore compact primeiro**; `if !Theme.compact : …` só acrescenta.
8. Rust escreve em `Theme`: `dark`, `reduce-motion`, `reduce-transparency`, `blur-supported`, `use-dynamic-accent`, `dynamic-accent(-ink)`, (§7; **sem** `stage-*` nem `glass-tint-*` na v3).
9. Textos de linha: `overflow: elide`, uma linha; descrições até 2 linhas. Rótulos de barra 12, mínimo do app 12.
10. Capa: `CoverImage` (`image-fit: cover`, contorno de 1 px). Ausente = placeholder `Theme.surface-2` + ícone `disc`.
11. Texto direto sobre `Theme.bg`/`Theme.surface`: `ink`, `ink-2`, `ink-3`. **Sobre vidro: `ink` e `Theme.ink-glass-2`** (nunca `ink-2`/`ink-3`: não passam AA sobre vidro).

## 2. Fundo e vidro (v3)

### 2.1 `Stage` (atrás de tudo, uma vez)

`Stage {}` preenche a janela com `Theme.bg` (preto `#000` / `#f2f2f7`). **Sem manchões, sem malha.** Só o **player** passa `immersive: true; has-cover: true; backdrop: AppState.backdrop`: capa em 150% (imagem 96×96 já desfocada pelo Rust), `Theme.im-veil` (preto 60%) e degradê inferior até preto 72%. Sem capa: `Theme.surface` (grafite). Troca de faixa = cross-fade de 480 ms (corte com movimento reduzido). O fundo é decorativo: fora da árvore de acessibilidade.

### 2.2 `GlassPanel` (3 materiais)

| Nível | Uso | Blur | Tint escuro / claro | Sombreado escuro | Sombra |
|---|---|---|---|---|---|
| **1 · fino** | legenda do cartão-herói, `GlassIconButton`, atalhos do player, barra lateral | 24 | branco 10% / 55% | preto 32% | 24 / y8 |
| **2 · regular** | `NavDock`, `SearchButton`, `MiniPlayer`, `NowBar`, `ScrollHeader`, transporte do player | 32 | 15% / 65% | 40% | 32 / y12 |
| **3 · espesso** | folhas (menu, playlist, nova playlist, fila), diálogos | 40 | 20% / 75% | 48% | 56 / y24 |

Camadas (de baixo): `backdrop-blur` → `background` = sombreado neutro (`Theme.glass-bg-N`) → **tint claro** (filho `Rectangle`, `Theme.glass-tint-N`, some com `glass-tint-on`) → brilho superior (`glass-sheen`, 52% da altura) → **borda de 1 px em gradiente** (`@linear-gradient(155deg, glass-edge 0%, glass-edge-low 34%, glass-edge-low 66%, glass-edge 45%)`; conferido no FemtoVG) → **fio especular** de 1 px → `drop-shadow` neutro → `clip: true`.

**Saturação.** O backdrop do Slint 1.19 só tem blur. Os protótipos usam `saturate(1.8)`; no Slint a cor vem das capas (já vivas). Para ver o mesmo que o Slint, abra qualquer protótipo com `#sat=0`.

**Fallback.** `Theme.glass-live` falso ⇒ blur 0, tint some, fundo `glass-solid*` neutro. Gatilhos: renderer software, **Reduzir transparência**, preferência do sistema, oferta após 2 s com quadro médio > 16 ms. Contraste do sólido ≥ 7:1.

**Regra de ouro:** o vidro só lê como vidro com **conteúdo colorido por baixo**. Mantenha a lista/capas rolando sob `ScrollHeader`, `NavDock`/`MiniPlayer` (com `EdgeFade`) e `NowBar`; legendas de vidro vão **sobre a capa**.

### 2.3 Instâncias

| Instância | Nível | Raio | Observação |
|---|---|---|---|
| Barra inferior (`NavDock`) | 2 | 32 (cápsula) | `x: 12; width: window − 24 − 8 − 56`; 3 abas; indicador deslizante `fill-2`; sobre `EdgeFade` |
| Buscar | 2 | 28 (círculo 56) | ativo: cápsula `fill-2` + ícone no acento |
| Mini player | 2 | 32 (cápsula) | 8 acima da barra; só com faixa; fio de progresso 3 px |
| Cabeçalho de rolagem | 2 | 0 | transparente em repouso; vidro com `p > 0` (blur só com `p > 0,02`) |
| Legenda do cartão-herói | 1 | 18 | sobre degradê preto 60% na capa; texto `ink` / `ink-glass-2` |
| Botões de vidro (voltar, Ajustes, curtir) | 1 (blur 16) | círculo | |
| Folhas, menus | 3 | 28 (topo) | com `Theme.scrim` |
| Trilho (≥ 600) / barra lateral (≥ 1200) | 1 | 24–28 | largura 88 / 248; **discreto** |
| Transporte do player | 2 | 44 | h 88, play branco 72 |
| `NowBar` (≥ 1200) | 2 | 38 (cápsula) | h 76, flutuante 20 do fundo |
| Banner, grupos, coleções, estados | **cartão opaco** (`GroupCard`/`Theme.surface`) | 20–28 | **não** é vidro |

## 3. Componentes compartilhados

### 3.1 `AppShell`

```
AppWindow
├─ Stage (backdrop: AppState.backdrop)
├─ if Theme.compact : Content(ScreenHost) · Chrome { MiniPlayer · Row { NavDock · SearchButton } } · BottomFade(190)
├─ if !Theme.compact : NavRail (88)  (wide: Sidebar 248) · Content · NowBar (flutuante, inferior)
└─ if AppState.player-open : PlayerSheet (compact: folha cheia; ≥ 840: duas colunas) · Scrim + QueueSheet / MenuSheet / NewPlaylistSheet (nível 3)
```

Trocar de aba = corte com `opacity` 0→1 em `dur-short`; **cada aba guarda a posição de rolagem**; tocar na aba ativa volta ao topo. `ScreenHost` reserva `Theme.chrome-reserve` embaixo (compact 184; ≥ 600: 112; ≥ 1200: 120).

### 3.2 `NavDock` (compact) e `SearchButton`

`NavDock` (`GlassPanel level 2, radius 32, height 64`), padding 6, **3 `NavTab`** iguais. `NavTab { icon; icon-fill; label; active }`: coluna ícone 24 + rótulo `caption` 12; **ativo** = cápsula `Theme.fill-2` que **desliza** atrás (320 ms), ícone **preenchido** (`*-fill`) no `Theme.accent`, rótulo `Theme.ink` em 700 (três sinais, nunca só cor); inativo = ícone/rótulo `Theme.ink-glass-2`. `accessible-role: tab`, `accessible-checked: active`. `SearchButton`: 56 × 56, `GlassPanel level 2` redondo ao lado (gap 8); ativo preenchido `Theme.accent`. `accessible-label: "Buscar"`. **Única entrada de busca**; nenhuma tela tem campo de busca no topo exceto a própria Busca.

### 3.3 `MiniPlayer` (compact; ≥ 600 flutuante com largura máx. 640)

`MiniPlayer` (`GlassPanel level 2`, cápsula raio 32, altura 64), toque abre o player. Capa **redonda** 48 · título `callout` 15 semibold + artista `subhead` 14 (`ink-2`) · `IconButton` play/pause (48, glifo 28) · `IconButton` próxima. Fio de progresso de 3 px a 20 de cada lado, a 5 do fundo (`ink` 22% / `accent`). Sem faixa: não aparece e a barra desce. **Pausado/Tocando** muda `accessible-label`. Em ≥ 1200 vira `NowBar`: 72 de altura, adiciona `KSlider` (altura 32) com tempos mono, e volume.

### 3.4 `NavRail` (≥ 600) e `Sidebar` (≥ 1200)

`Sidebar` (`GlassPanel level 1`, radius 24, **material fino e discreto**), `x: 12; y: 12; height: window − 24`, largura `Theme.rail-w` (88) ou `Theme.rail-w-wide` (248). Marca (36) / wordmark (26) no topo, **só aqui** no canto superior esquerdo. Itens 64 de altura no trilho (ícone sobre rótulo) / 52 na lateral (ícone à esquerda, `callout`); **Buscar** é o último item do grupo (mesma regra de destaque do botão circular); **Ajustes** no pé.

### 3.5 `TrackRow`

`in Track track; in bool show-format; callback clicked(); callback more();` — altura **68** (`row-h` + 4) **dentro de `GroupCard`** (cartão opaco raio 20), capa 52 em x 14, texto em x 72, separador `Divider` recuado em `row-inset − 8`, hover `Theme.cell-hover`. Capa 52 (raio 10) · título `body` 17 semibold (**700 se `playing`**, sem trocar de cor) · artista `subhead` 15 `ink-2` · **motivo** opcional (`why`, `ink-2`, ex.: "Em alta hoje") · barras "tocando" estáticas · `FormatBadge` (some em < 380 px) · `IconButton more` (48). Separador entre partes: vírgula (nunca ponto-médio). Sem vidro.

### 3.6 Outros

| Componente | Props | Medidas e estados |
|---|---|---|
| `HeroCard` | `Highlight item` | compact `min(300, 78% da janela)` × proporção 300:380, raio 26, capa full-bleed + degradê preto 60% nos 58% de baixo; **legenda de vidro nível 1** (`left/right/bottom 10`, h 76, raio 20): `caption` 12 `ink-glass-2` · `title-3` 20 (Inter Display) · `footnote` `ink-glass-2`; botão play 44 em círculo **branco** com triângulo preto. ≥ 840: 400×440; ≥ 1200: 460. Rola com snap por cartão. |
| `AlbumTile` | `Album item` | compact 2 colunas (gap 12/16) ou trilho de 148; capa quadrada raio 24; título 16 semibold; artista 14 `ink-2`. |
| `SectionHeader` | `title; source-tag?; action?` | `title-2` 22; à direita **`SourceTag`** ("Do YouTube Music", pílula 28 de altura, ícone `cloud` 16, `caption` 12) ou ação de texto ("Ver mais" + chevron, 48 de altura). Entre seções 28. |
| `Chip` | `text; selected` | **40 de altura** (alvo 48 com 4 px invisíveis acima e abaixo), padding 16; selecionado: fundo `Theme.ink`, texto `Theme.bg`. Linha de chips rola na horizontal sem barra. Seção do Início: Para você, Descobrir, Mixes, Foco, Treino. |
| `Banner` | `title; text; dismissible` | vidro nível 1, raio 24, ícone `badge-check` em círculo 44 `accent`; "O YouTube Music já está ativo". Dispensar grava em config e **nunca volta**. |
| `CollectionCard` | `Collection` | **cartão opaco** `Theme.surface`, raio 20, padding 12/14; arte quadrada raio 14: **Baixados** = `Theme.fill` + ícone `download` 56; **Curtidas** = fundo `Theme.accent` + `heart-fill` em `accent-ink`; **Álbuns / Playlists** = mosaico 2×2; título `headline` 17 semibold; contagem `subhead` `ink-2`. |
| `SettingGroup` | slot | `GlassPanel level 1`, raio 24, linhas de 64 com divisor 1 px `ink` 10%; título do grupo fora do vidro, `subhead` 14 semibold `ink-2`. `SettingRow`: ícone em quadrado 40 raio 12 `ink` 10%, título 16 semibold, descrição `subhead` `ink-2`, controle à direita. |
| `Segmented` | `options; selected` | pílula `ink` 12%, itens de 40, selecionado `Theme.ink`/`Theme.bg`. |
| `Sheet` | `title?; level 3` | `GlassPanel level 3`, raio superior 32, **grabber** 36×4 `ink` 45% a 8 do topo; compact: encostada embaixo, largura da janela; ≥ 600: centralizada, largura 560, flutuante a 16 do fundo. `Scrim`: `Theme.scrim` (preto 38%) clicável fecha. `accessible-role: dialog`. |
| `MenuItem` | `icon; label; hint?` | altura 52, ícone 24 `ink-2`, texto 16 semibold, `hint` `ink-3` (ex.: "Wi-Fi"). ≤ 7 itens por menu. |
| `Toast` | `text; action?` | vidro nível 2, altura 52, raio 26, 8 acima da reserva da barra; **6 s** (ou 12 s com ação); `role=status`; ação ("Desfazer"). |
| `EmptyState` | `icon; title; message; action-label` | vidro nível 1, raio 28, centralizado, padding 28/20; ícone num quadrado 88 raio 28 `ink` 12%; título `title-2`; texto `body` `ink-2` (máx. 34 caracteres de medida); **uma** ação. |
| `SkeletonRow` | — | geometria da `TrackRow` (64): capa 52 raio 12 e duas barras `ink` 14%, **estático**. Spinner só após 300 ms e ≥ 300 ms visível. |
| `OfflineBar` | — | vidro nível 2, altura 44, raio 22: ícone `cloud-off` + "Sem conexão. Tocando só o que está no aparelho." |

## 4. Telas (compact primeiro)

Gutter: **16** (compact) / 24 (≥ 600) / 32 (≥ 1200). Conteúdo máx. 1240 centrado. Topo: wordmark 28 à esquerda (alvo 48) e **Ajustes** (`gear`, botão de vidro 48) à direita; o título grande aparece só a partir de 600 (no compact o rótulo da aba já diz onde está; título presente para leitor de tela).

### 4.1 Início (`HomeScreen`)

```
HomeScreen
├─ TopBar (wordmark · gear)
├─ Banner (só no 1º uso; dispensável)
├─ Chips: Para você · Descobrir · Mixes · Foco · Treino     → AppState.home-section
├─ Section "Continue por aqui" + SourceTag → HeroRail [HeroCard × highlights]
├─ Section "Escolhas rápidas" + "Ver mais" → [TrackRow × quick-picks]   (com `why`)
├─ Section "Mixes para você" + SourceTag → TileRail
└─ Section "Álbuns em destaque" + SourceTag → AlbumGrid
```
Primeiro uso, offline e erro: UX_PLAN §6.1, §6.8, §6.9 (esqueleto → conteúdo; `OfflineBar` + Baixados/locais; erro com cache mantém o conteúdo).

### 4.2 Explorar

Rolagem: "Novos lançamentos" (TileRail + SourceTag) · "Humores e gêneros" (grade 2 colunas de `MoodTile`: proporção 1,55, raio 22, capa full-bleed, **placa de vidro nível 1** no pé com o nome, `headline`) · "Em alta" (linhas numeradas, sem selo de formato).

### 4.3 Player (`PlayerSheet`, **sempre escuro**: `Theme.dark` forçado dentro dele)

```
PlayerSheet (compact: coluna única, padding 16)
├─ Stage immersive (capa muito desfocada + véu; escuro mesmo com tema claro)
├─ Grabber · TopRow: IconButton chevron-down (label "Fechar o player") · "Tocando de" + origem · IconButton more
├─ Cover: lado = min(largura − 32, altura − 500), raio 22, sombra **neutra** (preto 60%, blur 48, y 24) e aro de 0,5 px
├─ MetaRow: Column{ título player-title 26/32 bold (elide) ; artista 16 ink-2 } · IconButton like (vidro, 48)
├─ KSlider progresso (altura 48, trilho 8 → 12) + tempos (mono 13 ink-2: posição | −restante)
├─ Transport: GlassPanel level 2 (h 92, raio 46) { shuffle · prev (52) · PlayButton 76 (círculo `Theme.ink`, glifo `Theme.bg`, sombra) · next (52) · repeat }
├─ Pills (4 colunas, h 56, raio 20, fill simples): Fila · Letra · Timer · Saída ("Este celular")   — ícone 24 + rótulo 12 semibold
└─ Quality: FormatBadge (codec + taxa) · "Do YouTube Music" · "Original, sem equalizador"
```
Gesto: **arrastar o grabber/cabeçalho para baixo** acompanha o dedo; passando de **140 px** fecha (senão volta). **Alternativas**: botão de fechar e Voltar do sistema. ≥ 840: duas colunas (capa à esquerda até 520, controles à direita até 440), volume visível, folha deixa de ser cheia. Callbacks como na v1 (`toggle-play`, `seek`, …) + `open-queue`, `open-output-picker`, `set-sleep-timer`.

### 4.4 Biblioteca

```
LibraryScreen
├─ TopBar
├─ TitleRow: "Biblioteca" (large 34/40) · PillButton primary "Criar playlist" (playlist-add)   → create-playlist()
├─ CollectionGrid (2 col compact / 4 col ≥ 840): CollectionCard × 4
├─ LocalCard (vidro 1): "Neste aparelho" · "2 músicas locais…" · Chip-botão "Adicionar pasta"   → add-folder()
└─ Section "Tocadas recentemente" → [TrackRow]
```
Curtidas e playlists são **locais** (sem login). Vazio e sem permissão: UX_PLAN §7; `EmptyState` "Sua biblioteca está vazia" (ação `add-folder`).

### 4.5 Coleção / álbum

Topo: voltar (vidro) · mais. Capa 64% da largura (máx. 280, raio 28), título `title-1`, subtítulo, `SourceTag`, **Tocar** (primary) · **Aleatório** (quiet) · Curtir (vidro). Depois `TrackRow`s.

### 4.6 Busca (`SearchScreen`)

```
SearchScreen
├─ TopBar
├─ SearchField (GlassPanel level 2, h 56, raio 28): ícone search · TextInput · IconButton wave ("Reconhecer música pelo som ambiente")
├─ Chips tipo (Tudo, Músicas, Álbuns, Artistas, Playlists)
└─ switch AppState.search-state: idle | results | empty | loading | error | offline
     idle    → "Buscas recentes" (clock + texto + ×) · "Para experimentar" (chips de humor)
     results → BestResult (vidro 1: capa 96, "Melhor resultado") · "Músicas" + SourceTag · "Álbuns" TileRail
     empty / loading (5 SkeletonRow) / error / offline → EmptyState (textos em UX_PLAN §7.3)
```
Debounce 200 ms no Rust; foco no campo ao abrir.

### 4.7 Fila (`QueueSheet`, vidro nível 3, quase tela cheia)

Título "Fila" + "Limpar fila" + fechar. "Tocando agora" (linha com fundo `ink` 12%) · "A seguir" + origem · linhas com **seta "Mover para cima"** (alternativa ao arrastar) e menu. Arrastar pela alça é opcional (etapa 2, depende do motor). Texto final: "Quando a fila acabar, o Kazzu continua com músicas parecidas do YouTube Music."

### 4.8 Folhas contextuais (`MenuSheet`, `PlaylistSheet`, `NewPlaylistSheet`)

Menu da faixa: cabeçalho (capa 56 + título + artista) + itens (Tocar em seguida, Adicionar à fila, Curtir, Adicionar à playlist, Baixar, Ir para o álbum, Ir para o artista). "Adicionar à playlist": "Nova playlist" + existentes. "Nova playlist": campo de nome com sugestão, **Criar playlist** (primary).

### 4.9 Ajustes

Voltar + "Ajustes". Grupos (vidro nível 1): **Fontes de música** (YouTube Music: tag "Ativo", "Ativo e padrão. Sem login." · Arquivos neste aparelho + "Adicionar pasta" · Outras fontes: "Em breve", só se o usuário ligar) · **Áudio** (Qualidade "Original", Equalizador "Em breve", Baixar só no Wi-Fi) · **Aparência** (Tema, Cor de destaque, **Reduzir transparência**, Reduzir movimento) · **Privacidade** (**Sem telemetria**: `KSwitch locked`, texto "O Kazzu só recebe dados…"; Histórico de escuta "Apagar") · **Sobre** (versão, GPL-3.0, "Relatar um problema" gera arquivo local).

### 4.10 Estados

Seis variantes (`prototypes/states.html`): vazio, carregando, erro de provedor, offline, sem permissão, sem resultados. Estrutura única em §3.6 (`EmptyState`).

## 5. Gestos e alternativas (implementação)

| Gesto | Slint | Alternativa obrigatória |
|---|---|---|
| Arrastar o player para baixo | `TouchArea` no cabeçalho/grabber, `moved` atualiza `y`; `pointer-event` `up`: fecha se `dy > 140px` | `IconButton chevron-down`; Voltar |
| Seek | `KSlider` (arraste, toque, setas ±5%) | toque no ponto; ±15 s (etapa 2) |
| Reordenar fila | alça `grip` com `TouchArea` | `arrow-up` na linha; "Mover" no menu |
| Deslizar linha | `TouchArea` horizontal | `more` → Remover / Curtir |
| Carrossel | `Flickable` (sem snap nativo; aproximar com `released`) | setas ≥ 840; teclado ←/→ |
Leitor de tela: ações personalizadas (`accessible-action-*`) equivalentes aos gestos; **nenhum gesto é necessário** (WCAG 2.5.7).

## 6. Medidas, estados e exceções

### 6.1 Quadro de medidas

| Item | Compact < 600 | Medium 600–839 | Expanded 840–1199 | Large ≥ 1200 |
|---|---|---|---|---|
| Gutter | 16 | 24 | 24 | 32 |
| Navegação | barra 64 + Buscar 56 | trilho 88 | trilho 88 | lateral 248 |
| Mini / NowBar | mini 64 | mini flutuante ≤ 640 | idem | NowBar 72 com progresso e volume |
| Destaque | 300×380 | 340 | 400×440 | 460 |
| Coleções | 2 col | 2 col | 4 col | 4 col |
| Linha | 64, capa 52 | idem | + coluna de álbum | idem |
| Player | folha cheia, capa ≤ 358 | folha 560 | 2 colunas | 2 colunas |
| Reserva inferior | 184 | 112 | 112 | 120 |

### 6.2 Estados de controle

| Controle | Normal | Hover | Pressionado | Foco | Desabilitado |
|---|---|---|---|---|---|
| `IconButton` | tint `ink` | fundo `ink` 8% | `ink` 20%, escala .96 | anel 2 px | .5, sem eventos |
| `NavTab` | `ink-2` | `ink` | idem | anel | n/a |
| `Chip` | fill `ink` 12% | `ink` 20% | idem | anel | .5 |
| `PillButton primary` | `accent` | +6% brilho | escala .97 | anel | .5 |
| `KSwitch` | `ink` 24% / `accent` | n/a | n/a | anel | `locked`: ligado, sem eventos |
| `KSlider` | trilho 8 | trilho 12 + polegar 22 | idem | idem | .45 |

### 6.3 Teclado (desktop e acessibilidade)

Tab segue a ordem visual; Espaço/Enter ativam; setas movem sliders (±5%); Esc fecha folha → player → fila; Espaço play/pause fora de campos de texto. Itens de navegação `accessible-role: tab`.

### 6.4 Pontos de cor fora de `Theme`

Somente: o brilho/aro dos `GlassPanel` (tokens), `#ffffff`/`#000000` do botão tocar sobre capa e do polegar do `KSwitch` (cores fixas sobre mídia: `media-*`), degradês pretos de legibilidade sobre capas (`#00000099`), contorno da capa (`#ffffff1a`/`#0000001a`) e o gradiente de Curtidas (deriva de `Theme.accent`). Qualquer outro literal é erro de revisão.

## 7. Dados do Rust que a UI espera

- **Fundo neutro.** O Rust **não calcula mais palco nem matiz de vidro**: removidos `stage-*` e `glass-tint-*`. Continua escrevendo `dark`, `reduce-motion`, `reduce-transparency`, `blur-supported`, `window-w` e, só se o usuário ligar "Cor da capa", `use-dynamic-accent`/`dynamic-accent(-ink)` (algoritmo da v2, DESIGN.md §3.2).
- **`backdrop`** (fundo do **player**): `Image::from_rgba8` 96×96, σ 4, saturação ×1,15, **sem escurecer** (o véu é do `Stage`), 1 LSB de ruído. Só é lido pelo player.
- **Contrato novo** (acréscimos a fazer em `app-contract.slint`; **ainda não aplicados** no arquivo do design, o implementador os cria junto com as telas): `home-section` (enum Para você/Descobrir/Mixes/Foco/Treino), `quick-picks` (`[Track]` com `why`), `source-label` por seção ("Do YouTube Music"), `online: bool`, `first-run-banner: bool` + `dismiss-first-run-banner()`, `quality-label` ("OPUS 160 kbps"), `output-name`, `sleep-timer-*`, `toast-*` (com `action-label`), `search-state` com `loading/error/offline`, `track.why`. Campos antigos intocados.
- **Provedores:** `AppState.providers[0]` é o YouTube Music com `connected: true, active-default: true, requires-login: false`.
- **Rótulos** já formatados em pt-BR ("214 músicas", "3:21", restante com sinal `−` real, um só padrão em todo o app).
- **Renderer:** escrever `Theme.blur-supported`; **[verificar a API no Slint 1.19]**.
- **Preferências do sistema** de movimento/transparência reduzidos → valores iniciais; as chaves de Ajustes mandam depois.

## 8. Critérios de aceite de UI (por etapa)

1. Visual conferido lado a lado com `design/prototypes/*.html` em **390×844 e 360×740 primeiro**, depois 768 e 1280; claro e escuro; com e sem `#rt=1`.
2. **Teste de vidro:** com `Stage` ligado, a lista que passa atrás da barra e do mini player fica **visível e desfocada**; nenhum painel de vidro parece cartão escuro opaco. Com `Theme.reduce-transparency`, todos viram sólidos sem perder hierarquia.
3. Nenhum literal de cor/tamanho fora de `Theme` (grep em `ui/`), exceto §6.4.
4. Alvos ≥ 48 (teste automático nos componentes), foco visível e nunca encoberto, todo botão de ícone com `accessible-label`, TalkBack percorre o roteiro T1–T9 do UX_PLAN.
5. Contraste: pior caso do vidro conferido com capas clara, escura, saturada e cinza (capas de matiz 40°, 205°, 150°, cinza e branca no fundo do player e sob o cromo); texto ≥ 4,5:1 (referência: `design/ux/contrast-report.md`).
6. `Theme.reduce-motion = true` ⇒ nenhuma transição visível; sem timers de animação.
7. Medir: Início com vidro em GPU fraca e no Android de referência ≥ 55 fps ao rolar; se falhar, reportar e oferecer "Reduzir transparência" (não remover o vidro sem o autor).
8. Zero requisição de rede ao abrir e ficar parado (UX_PLAN AC-10).
