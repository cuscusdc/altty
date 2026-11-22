# Kazzu — especificação de UI em Slint

Para o implementador de UI do `kazzu-app`. **Siga à risca**: nomes, medidas e estados abaixo vêm de `design/prototypes/` (abra a tela ao lado enquanto implementa). Slint 1.19, renderers Skia/FemtoVG (software = fallback sem blur). Identificadores em inglês; textos visíveis em pt-BR.

## 0. Arquivos prontos (já compilam no Slint 1.19; mova para `crates/kazzu-app/ui/`)

| Arquivo | O que é | Regra |
|---|---|---|
| `design/slint/tokens.slint` | `export global Theme` (cores claro/escuro, tipografia, espaços, raios, tamanhos, durações, vidro) | **Gerado** por `tools/gen_tokens.py`. Não editar à mão; mude `tokens.json` e regenere. |
| `design/slint/atoms.slint` | `GlassPanel`, `CoverImage`, `IconButton`, `FormatBadge`, `KSwitch`, `KSlider` | Usar como estão. Evoluir aqui, não duplicar. Correções feitas na implementação (já aplicadas no arquivo): `FocusScope` com `focus-on-click: false` (senão engole o clique do `TouchArea` irmão) e `KSlider.value` como `in` (senão o arraste quebra a ligação com o progresso); ver `docs/UI_NOTES.md`. |
| `design/slint/app-contract.slint` | enums, structs (`Track`, `Album`…) e `global AppState` (propriedades e callbacks) | É a fronteira Rust ↔ UI. A UI só lê/chama isto. |
| `design/slint/preview.slint` | janela que mostra os átomos | Só para `slint-viewer`. |
| `design/fonts/*.ttf` | Hanken Grotesk 400/600/700, Geist Mono 400/500 | `import "…ttf";` no `.slint` raiz e `default-font-family: Theme.font-sans`. |
| `design/assets/icons/*.svg` | 45 ícones, `colorize` | `Image { source: @image-url(…); colorize: Theme.ink-2; }` |
| `design/assets/kazzu-*.svg`, `icon-*.png` | marca e ícone do app | wordmark/marca mono: `colorize`. |

Estrutura sugerida:

```
crates/kazzu-app/ui/
  app.slint                  // AppWindow (raiz): importa fontes, exporta Theme e AppState
  theme/tokens.slint  theme/atoms.slint  theme/app-contract.slint
  components/                // §3
  screens/                   // §4: home, explore, player, library, queue, search, settings, empty-state
  assets/ (icons, logo, covers fictícias só para preview)
```

No `.slint` raiz: `export { Theme, AppState }` (globais precisam ser exportados para o Rust enxergá-los).

## 1. Regras gerais

1. **Nunca** usar cor, tamanho, raio ou duração literal: sempre `Theme.*`. Exceção: gradientes do véu do player (§5.4).
2. Todo botão de ícone: `IconButton` com `label` em pt-BR. Todo controle interativo tem `accessible-role` e rótulo.
3. Alvo mínimo 44 px. Foco visível: anel de 2 px (`Theme.focus-ring`) a 2 px de distância, **sem animação**.
4. Listas longas = `ListView` (virtualizada). **Nada de vidro dentro de linhas.**
5. No máximo 6 `GlassPanel` com blur visíveis ao mesmo tempo (dock + mini + 4 cartões).
6. Animação só com `animate` em componentes (globais não podem animar). Transformações: `transform-rotation` / `transform-scale` (o antigo `rotation-angle` está obsoleto). Durações `Theme.dur-*`; curvas por literal (copiar de `tokens.slint`): `cubic-bezier(0.22, 0.9, 0.28, 1)` chegadas, `cubic-bezier(0.45, 0, 0.2, 1)` trocas.
7. Sem animação infinita. O indicador "tocando" são 3 barras estáticas (4×10, 4×18, 4×13 px, raio 2, `Theme.accent`, base alinhada, espaço de 3 px).
8. `Theme.window-w` é atualizado no `AppWindow`: `changed width => { Theme.window-w = self.width; }`. Use `Theme.compact` (< 720) e `Theme.wide` (≥ 1100). **Uma única árvore** com `if Theme.compact : …` / `if !Theme.compact : …`; não há telas separadas por plataforma.
9. Rust escreve em `Theme`: `dark`, `reduce-motion`, `reduce-transparency`, `blur-supported` (false no renderer software), `use-dynamic-accent`, `dynamic-accent`, `dynamic-accent-ink`.
10. Textos: `overflow: elide` + `wrap: no-wrap` em títulos de linha; no máximo 2 linhas em descrições.
11. Imagens de capa: `image-fit: cover` dentro de `CoverImage` (tem contorno de 1 px). Capa ausente = `placeholder` (`Theme.surface-2`) + ícone `disc` (24, `Theme.ink-3`).

## 2. Vidro: receita e uso

`GlassPanel` (em `atoms.slint`) já implementa, nesta ordem: `backdrop-blur` → fill `Theme.glass-bg` (translúcido com blur; sólido sem) → gradiente de 50% da altura (`glass-sheen` → transparente) → fio especular de 1 px no topo (inset = raio) → borda 1 px `glass-edge-low` → `drop-shadow` e2 → `clip: true`.

| Instância | Props | Observação |
|---|---|---|
| Dock | `strong: true; radius: Theme.dock-radius` | blur 40 |
| Mini player | `strong: true; radius: Theme.mini-radius` | blur 40 |
| Sidebar | `radius: 24px` | blur 28 |
| NowBar (desktop) | `radius: 22px` | blur 28 |
| Cartão de coleção | `radius: 22px` | blur 28 |
| Placa do cartão de destaque | **não usa GlassPanel**: fill fixo `#0c0907a3` (preto-quente 64%), `backdrop-blur: Theme.glass-blur-radius`; sem `glass-live` → `#1a1511` | texto branco sobre capa clara exige 64% |
| Botão do player (Letra, Saída, Fila) | `light-blur: true; radius: 22px`, fill `Theme.im-control-fill`, borda `Theme.im-control-edge` | sobre fundo escuro do player |

Fallback: `Theme.glass-live` falso ⇒ `glass-bg` sólido e blur 0 (já está nos tokens). A UI **não** tem `if` para isso; é automático. Quando o app detectar o renderer software, `Theme.blur-supported = false`.

Sobre conteúdo rolável, a dock fica sobre um **esvaecimento**: `Rectangle { y: parent.height - 190px; height: 190px; background: @linear-gradient(180deg, transparent 0%, Theme.bg 70%); }` abaixo das barras e acima do conteúdo (no CSS: `.dockwrap::before`).

## 3. Componentes compartilhados

Convenção: `in` = recebido, `callback` = evento. Medidas em px lógicos; `sp-N`, `r-N` = tokens.

### 3.1 `AmbientBackdrop` / `PlayerBackdrop`

- `AmbientBackdrop { in property <image> source; }` — preenche a janela. Camadas: (1) `Rectangle` `Theme.bg`; (2) `Image` `x: -25%·w; y: -25%·w`… na prática: largura 150% da janela, altura 90% da janela, `image-fit: cover`, `opacity: Theme.ambient-opacity`; (3) véu `@linear-gradient(180deg, transparent 0%, Theme.bg.with-alpha(0.7) 40%, Theme.bg 78%)` cobrindo a janela. Cross-fade 480 ms entre duas `Image` ao mudar `source`.
- `PlayerBackdrop { in property <image> source; }` — fundo do player: `Image` cobrindo a janela (`image-fit: cover`, 96×96 já desfocada e escurecida pelo Rust, **sem blur no Slint**) + véu `@linear-gradient(180deg, Theme.im-scrim-top 0%, Theme.im-scrim-mid 30%, Theme.im-scrim-bottom 100%)`. Cross-fade 480 ms.

### 3.2 `AppShell` (raiz)

```
AppWindow
├─ AmbientBackdrop (source: AppState.ambient)         // oculto quando player-open
├─ if Theme.compact: Column { TopBar · ScreenHost }  + FloatingBars (MiniPlayer sobre NavDock) + BottomFade
├─ if !Theme.compact: Sidebar · ScreenHost · NowBar
└─ if AppState.player-open: PlayerScreen (cobre tudo, com PlayerBackdrop)
```

`ScreenHost` mostra a tela de `AppState.tab`; Fila (`queue-open`) e Configurações têm tela própria. Trocar de tela = corte (sem slide); o conteúdo novo entra com opacidade 0→1 em `dur-short`.

### 3.3 `NavDock` (mobile) — "barra principal"

Medidas: `x: 12; width: window − 24; height: Theme.dock-h (64); y: window − 12 − safe-bottom − 64`. `GlassPanel strong`, raio 32, padding 6, `HorizontalLayout spacing: 4`.

```
NavDock
├─ HorizontalLayout (tabs, horizontal-stretch: 1, spacing: 2)
│    └─ NavTab ×3 (Início/home, Explorar/explore, Biblioteca/library)
├─ Rectangle 1×28 (divisor, Theme.line-strong, margem 4 à direita)
└─ SearchButton 52×52
```

- `NavTab { in icon; in label; in active; callback clicked }`: `VerticalLayout` centrado (ícone 24 + label `caption` 12/16 semibold), altura 52, **raio 26**; `active` ⇒ fundo `Theme.accent-soft`, ícone `Theme.accent`, rótulo `Theme.ink`; inativo ⇒ `Theme.ink-3`; hover rótulo `Theme.ink`. Transição de fundo `dur-short`. `accessible-role: tab`, `accessible-label: label`.
- `SearchButton`: círculo 52, fundo `Theme.accent`, ícone `search` 28 com `colorize: Theme.accent-ink`; **ativo** (tab == search) ⇒ fundo `Theme.accent-soft`, ícone `Theme.accent`, borda 1 px `Theme.accent`. `accessible-label: "Buscar"`. Hover: fundo mistura 12% de `Theme.ink`.
- Esta é a **única** entrada da busca; **não há campo de busca no topo de nenhuma tela** exceto dentro da tela Busca.

### 3.4 `MiniPlayer` (mobile)

`GlassPanel strong`, raio 22, `x: 12; width: window − 24; height: 64; y: dock.y − 8 − 64`; toque abre o Player (`AppState.open-player()`).

```
MiniPlayer
├─ CoverImage 48×48 (x: 8, centralizado, raio 12)
├─ VerticalLayout (x: 68, flex): Text título (callout 15/20 semibold, ink) · Text artista (subhead 14/18, ink-3)
├─ IconButton play/pause (44, glyph 28, tint ink)  → toggle-play()
├─ IconButton next (44, glyph 24)                   → next()
└─ Rectangle progresso: x 14, largura (w−28), altura 2, bottom 0, trilho Theme.line, preenchimento Theme.accent (progress)
```

Sem faixa (`has-now == false`) o mini não aparece e o fundo da dock sobe 72 px.

### 3.5 `Sidebar` (desktop)

`GlassPanel`, `x: 12; y: 12; height: window − 24`, `radius: 24`, largura `Theme.sidebar-w-compact` (80, `!wide`) ou `Theme.sidebar-w` (248, `wide`), padding 12 (16/12 no wide), `VerticalLayout spacing: 4`.

```
Sidebar
├─ LogoLink (altura 56)  — wide: wordmark 26 de altura à esquerda (padding-left 12); estreita: marca 36×36 centralizada. Clique = Início.
├─ NavItem ×3 (Início, Explorar, Biblioteca)
├─ Rectangle 1px Theme.line (margem 8)
├─ NavItem search (destacado)
├─ stretch
└─ NavItem settings ("Configurações", ícone sliders)
```

- `NavItem { icon, label, active, kind: normal|search }`: altura 48, raio 16, estreita: ícone centralizado, sem texto (rótulo vira `accessible-label` e tooltip após 800 ms); wide: `HorizontalLayout` ícone 24 + spacing 12 + texto `callout` 15 semibold, padding-horizontal 16.
  - normal ativo: fundo `accent-soft`, ícone `accent`, texto `ink`; inativo `ink-2`; hover fundo `surface-2`.
  - `search` (inativo): fundo `accent-soft`, borda 1 px `accent.with-alpha(0.45)`, ícone `accent`; ativo: fundo `accent`, ícone/texto `accent-ink`.
- Logo: wordmark `kazzu-wordmark-{dark|light}.svg` conforme `Theme.dark` (duas `Image` com `visible`), marca idem. Só no canto superior esquerdo.

### 3.6 `NowBar` (desktop)

`GlassPanel`, `height: 72`, `radius: 22`, `y: window − 16 − 72`, largura `min(760, contentWidth − 48)` centralizada na área de conteúdo, padding `8 12 8 8`, spacing 16.

```
NowBar
├─ CoverImage 56 (raio 14)
├─ VerticalLayout meta (wide: largura fixa 220; senão stretch): título callout semibold + artista subhead ink-3
├─ if Theme.wide: VerticalLayout mid (stretch 1.4): KSlider (immersive:false, altura 32, preenchimento accent) + HorizontalLayout tempos (mono 12, ink-3, extremos)
├─ IconButton prev · IconButton play/pause (glyph 28) · IconButton next
├─ IconButton queue (abre Fila)
└─ IconButton chevron-down (abre Player)
```

### 3.7 `TrackRow` (linha de faixa)

`in Track track; in bool show-album; in bool show-format; callback clicked(); callback more();` — altura mínima **64**, padding `8 8`, raio 16; hover fundo `Theme.surface`; pressionado `surface-2` (`dur-micro`).

```
TrackRow  (HorizontalLayout, spacing: 12, alignment center)
├─ CoverImage 48 (raio 10)
├─ VerticalLayout (stretch): Text título (body 16/22 semibold; Theme.accent se track.playing; elide) · Text subtítulo (subhead 14/18 ink-3; track.source == "" ? artista : artista + ", " + source)
├─ (desktop, wide) Text álbum (subhead ink-3, largura 28%, elide)
├─ if track.playing: NowBars (20×18)
├─ FormatBadge (codec, detail, hires) — mostrar sempre que houver codec
└─ IconButton more (44)
```

Subtítulo **sem ponto-médio**: separador é vírgula. Fila: prefixar `grip` (32×44, `ink-3`) à esquerda e substituir `more` por `close` (glyph 20) + tempo mono 13 (largura 40, direita).

### 3.8 Outros

| Componente | Props | Medidas e estados |
|---|---|---|
| `HeroCard` | `Highlight item; callback clicked()` | 300×400 (compact) / 380×460, raio 24, `CoverImage` full-bleed. Placa: `x/right/bottom: 10`, padding 16, raio 18, vertical spacing 2: `caption` (13/18, `im-ink-2`) · `title-2` (22/28, `im-ink`) · `subhead` (`im-ink-2`); `padding-right: 76`. Botão play: círculo 48 em `right/bottom: 14`, `Theme.accent`, glifo `play` 28 `accent-ink`. Hover: `y: -2` em `dur-short`. Clicar toca. |
| `AlbumTile` | `Album item` | 148 (compact) / 176 de largura, capa quadrada raio 16, gap 8, título 15/20 semibold, artista 14/18 `ink-3`. Hover capa `y: -2`. |
| `SectionHeader` | `title; action-label` | `title-2`, à direita `TextButton` (accent 15 semibold, 44 de altura, "Ver tudo"). Margem superior 32 entre seções, 12 até o conteúdo. Desktop: setas `chevron-left/right` (44) em carrosséis. |
| `Chip` | `label; selected; callback clicked` | altura 40 (alvo 44 com 2 px invisíveis acima/abaixo), padding-horizontal 16, raio full, `surface-2`/`ink-2`, `callout` 14 semibold; selecionado: fundo `accent`, texto `accent-ink`. Linha de chips rola horizontal sem barra visível. |
| `Segmented` | `[string] options; int selected` | pílula `surface-2`, padding 4, itens altura 36 raio full, texto 14 semibold `ink-2`; selecionado: fundo `Theme.bg`, texto `ink`, sombra e1. Alvo 44 (4 px invisíveis). `accessible-role: tab`. |
| `SettingRow` | `icon; title; subtitle` + slot de controle | altura mín. 60, padding 10 vertical, ícone num quadrado 36 raio 10 `surface-3` com glifo 20 `ink-2`, título `body` 16 semibold, subtítulo `subhead` `ink-3` (até 2–3 linhas), controle à direita. Divisor 1 px `Theme.line` entre linhas do mesmo grupo. |
| `SettingGroup` | slot | fundo `Theme.surface`, raio 20, padding `4 16` (`4 20` no desktop); grupos separados por 16; título do grupo `title-2` com margem superior 32. |
| `TextButton` | `label` | cor `accent`, `callout` 15 semibold, 44 de altura, largura mín. 44, hover fundo `accent-soft`. |
| `PillButton` | `label; icon; kind: primary\|quiet` | altura 44, padding 16, raio full, `callout` 15 semibold. `primary`: `accent`/`accent-ink` (**um só por tela**); `quiet`: `surface-2`/`ink`. Hover: primário mistura 12% de `ink`; pressionado mistura 24% de `bg`. |
| `EmptyState` | `icon; title; message; action-label; callback action` | coluna centrada, largura máx. 380, padding 56/20: quadrado 96 raio 28 `surface-2` com glifo 28 `ink-3`; título `title-2`; texto `body` `ink-3` (máx. 2 linhas, centralizado); `PillButton`. |
| `Avatar` | `string initial` | círculo 44, `surface-3`, inicial `headline` 16 bold. Toca Configurações. |

## 4. Telas

Margens laterais (gutter): 20 (compact) / 32 (demais). Conteúdo máx. 1180 centralizado. Padding inferior de rolagem: 196 (compact: cobre mini + dock) / 140 (desktop: cobre NowBar). Cada tela é um `Flickable`/`ListView` com `viewport-height` calculado.

### 4.1 Início (`HomeScreen`)

```
HomeScreen
├─ if compact: TopBar (min-height 56): Logo wordmark 28 (clicável, alvo 44) · stretch · Avatar
├─ if !compact: PageHeader: Text "Início" (large-title 40/46) · stretch · Avatar
├─ Section "Destaques": SectionHeader + HeroRail (HorizontalLayout em Flickable, spacing 12, snap por cartão, margens negativas até a borda no compact)  [HeroCard × highlights]
├─ Section "Recentes": SectionHeader("Recentes","Ver tudo") + ListView [TrackRow × recents]
└─ Section "Álbuns que você ouve": SectionHeader + TileRail (spacing 16) [AlbumTile × albums]
```

Propriedades: `AppState.highlights`, `recents`, `albums`. Callbacks: `play-track(id)` (linha e cartão), `open-album(id)`, `navigate(Tab.library)` ("Ver tudo"), `navigate(Tab.settings)` (avatar). Margem superior das seções: 32 (a primeira 16). Estados: carregando = 3 linhas-esqueleto (§4.9); vazio = `EmptyState` de biblioteca vazia.

### 4.2 Explorar (`ExploreScreen`)

Título "Explorar"; seção "Humores": grade de 2 colunas (gap 12) de `MoodTile` (proporção 1,6, raio 20, capa full-bleed + véu `@linear-gradient(180deg, transparent 35%, #000000b3 100%)` + título `title-2` branco no canto inferior esquerdo, padding 14/12); seção "Novidades dos seus provedores": `Chip`s (Todos, Local, YT Music, Spotify; seleção única) + `ListView` de `TrackRow`. Dados: `AppState.recents` filtrado no Rust (etapa 2 traz o resto).

### 4.3 Player (`PlayerScreen`)

Sempre escuro: usa **`Theme.im-*`**. Cobre a janela inteira, sobre `PlayerBackdrop`.

```
PlayerScreen
├─ PlayerBackdrop (source: AppState.backdrop)
├─ TopBar (altura 64 + safe-top): IconButton chevron-down 28 (label "Minimizar player") · Column{ "Tocando de Recentes" 14/18 semibold im-ink-2 ; saída 13/18 im-ink-3 } (centralizada) · IconButton more
└─ Content (compact: coluna; !compact: grade 2 colunas, gap 56, centralizada na vertical, largura máx. 1180, padding-horizontal 40)
   ├─ ArtColumn: CoverImage (lado = min(340, 46% da altura) no compact; min(440, 62% da altura) no desktop; raio 28; sombra e3; contorno branco 12%) — compact centralizada; desktop alinhada à direita da coluna
   └─ ControlsColumn (largura máx. 460, spacing 12/16)
      ├─ MetaRow: Column{ Text título (player-title 28/34 bold im-ink; desktop 36/42; elide) ; Text artista (18/24 im-ink-2; elide) } · IconButton like (44, fill im-control-fill + borda 1 px im-control-edge, ícone heart-fill, ativo = Theme.accent) · IconButton more (idem)
      ├─ KSlider progresso (immersive, accent-fill: true) — accessible-label "Progresso da faixa", value-text "1:02 de 3:21"; released → seek(v)
      ├─ TimesRow (grid 1fr auto 1fr): Text position (mono 13 im-ink-2) · FormatBadge (hires? accent; codec; detail-long; ink: im-ink-2; strong-ink: im-ink) · Text remaining (mono 13, direita)
      ├─ TransportRow (HorizontalLayout, alignment space-between, padding-horizontal 4): shuffle (44, glifo 24, im-ink-3; ativo accent) · prev (56, glifo 32, im-ink) · PlayButton (84 compact / 72 desktop, círculo im-ink, glifo pause/play 40/34 em #15110d, sombra e2; hover fill #fff) · next (56) · repeat (44; ativo accent)
      ├─ VolumeRow: ícone vol-low 24 im-ink-3 · KSlider (stretch) · ícone vol-high 24; set-volume(v)
      ├─ ActionsRow (colunas: auto, 1fr, auto; gap 8): PlayerPill "Letra" (ícone lyrics 20) · PlayerPill saída (ícone output 20 + nome do dispositivo, elide) · PlayerPill "Fila" (ícone queue 20)   — altura 48, raio 22, 14/semibold
      └─ if AppState.bit-perfect: Row centrada: ícone check 20 + "Original, sem equalizador" (13/18, im-ink-3)
```

Callbacks: `close-player`, `toggle-play`, `previous`, `next`, `seek`, `set-volume`, `toggle-shuffle`, `cycle-repeat`, `toggle-like(now.id)`, `open-lyrics`, `open-output-picker`, `queue-open = true`. Estados: tocando/pausado (glifo do play), curtido, shuffle/repeat ativos, carregando faixa (progresso indeterminado: trilho estático sem animação + título em `ink-3`), erro de reprodução (placa de vidro `light` no lugar do `bit-perfect` com texto do erro e botão "Tentar de novo"). Sem selo (`codec == ""`): oculta o `FormatBadge` e centraliza os tempos.

Gestos (mobile): arrastar para baixo fecha (`close-player`); deslizar a capa para os lados = próxima/anterior (opcional, etapa posterior).

### 4.4 Biblioteca (`LibraryScreen`)

```
LibraryScreen
├─ TopBar (compact) · logo + avatar
├─ TitleRow: Text "Biblioteca" (large-title) · stretch · PillButton primary "Criar playlist" (ícone playlist-add 20)   → create-playlist()
├─ CollectionGrid (compact 2 col / desktop 4 col; gap 12 / 16): CollectionCard × 4 (AppState.collections: Baixados, Curtidos, Álbuns, Playlists)
└─ Section "Suas playlists": SectionHeader + PlaylistGrid (compact 2 col gap 16/12; desktop colunas de ≥ 200, gap 24/16): PlaylistCard × playlists
```

- `CollectionCard`: `GlassPanel radius 22`, padding 12, spacing 12; arte quadrada (largura − 24) raio 14; título `body` 16/22 semibold; contagem `subhead` `ink-3`. Arte por tipo: **Baixados** = fundo `surface-3` + ícone `download` 56 `ink-2`; **Curtidos** = degradê `@linear-gradient(160deg, Theme.accent 0%, Theme.accent.darker(0.4) 100%)` + `heart-fill` 56 `accent-ink`; **Álbuns** = fundo `surface-3` + 3 `CoverImage` sobrepostas (56% da arte, raio 10, sombra e1), em `x` 8%/22%/36% e `y` 16%/20%/24% da arte, com `transform-rotation` −8°/0°/7° (use `transform-*`; `rotation-angle` está obsoleto no 1.19); **Playlists** = mosaico 2×2 (gap 2, raio 14) das 4 primeiras capas. Hover: `y: -2`.
- `PlaylistCard`: mosaico 2×2 (gap 2, raio 14, quadrado) + nome `body` 16 semibold + contagem `subhead` `ink-3`; sem vidro; hover `y: -2`. Menos de 4 capas: células vazias `surface-3`.

Callbacks: `open-collection(id)`, `open-playlist(id)`, `create-playlist()`. Vazio: `EmptyState` "Sua biblioteca está vazia" (ação `add-folder()`).

### 4.5 Fila (`QueueScreen`)

Aberta de `queue-open`; mobile: tela cheia com botão voltar; desktop: tela de conteúdo (largura máx. 860).

```
QueueScreen
├─ TitleRow: IconButton back (44) · "Fila" (large-title) · stretch · PillButton quiet "Limpar" (ícone trash 20) → queue-clear()
├─ "Tocando agora" (headline 17/22 semibold ink-2) + NowCard: padding 12, raio 20, fundo Theme.accent-soft, CoverImage 64 (raio 14), Column{ título 18/24 bold ; artista callout ink-2 }, FormatBadge, NowBars
├─ Section: Text queue-summary ("A seguir, 6 músicas, 24 min") + ListView [QueueRow × queue]  // TrackRow com grip, tempo mono e botão close
└─ SettingGroup: SettingRow "Reprodução automática" ("Quando a fila acabar, o Kazzu toca músicas parecidas.") + KSwitch → set-autoplay(bool)
```

Reordenar: arrastar pelo `grip` (callback `queue-move(from, to)`); teclado: Alt+↑/↓ move a linha em foco. Remover: botão `close` (label "Remover <título> da fila") → `queue-remove(index)`. Estado vazio: `EmptyState` "A fila está vazia".

### 4.6 Busca (`SearchScreen`)

```
SearchScreen
├─ Title "Buscar" (large-title; no desktop dentro do PageHeader)
├─ SearchField (altura 52, raio 26, fundo surface-2, padding-left 16, borda 1 px transparente → Theme.accent com foco)
│    ├─ ícone search 24 ink-3 · TextInput (17/22, ink; placeholder "Músicas, álbuns, artistas" ink-3) · IconButton close (visível se houver texto; label "Limpar busca") · IconButton wave (accent-soft/accent; label "Reconhecer música pelo som ambiente") → recognize-ambient()
├─ Chips tipo (Tudo, Músicas, Álbuns, Artistas, Playlists — seleção única) + Chips fonte (Local, YT Music, Spotify — múltipla), 8 px entre as linhas
└─ switch AppState.search-state:
     idle    → Section "Buscas recentes": ListView de linhas (ícone clock, texto 16, botão close) ; "Limpar" (TextButton)
     results → Section "Músicas" ListView [TrackRow × search-songs] + Section "Álbuns" TileRail [AlbumTile × search-albums]
     empty   → EmptyState "Nada para “…”" (ação "Limpar filtros")
```

Foco automático no campo ao abrir a tela (teclado em foco no desktop; no mobile ao tocar). `search(text)` com debounce de 200 ms feito no Rust. Digitar `edited => { AppState.search(self.text) }`.

### 4.7 Configurações (`SettingsScreen`)

Largura máx. 760. Título "Configurações" (large-title). Grupos (`title-2` + `SettingGroup`):

| Grupo | Linhas |
|---|---|
| Provedores | Biblioteca local ("3 pastas, 1.284 músicas", tag "Ativa") · YT Music (conta; `TextButton` "Sair") · Spotify (`PillButton quiet` "Conectar") · Tidal e Qobuz (tag "Em breve", desabilitado). Dados: `AppState.providers` |
| Áudio | Saída (nome + `TextButton` "Trocar" → `open-output-picker`) · Qualidade original (tag "Sempre"; texto: "Sem normalizar e sem reamostrar. Só convertemos a taxa se o dispositivo de saída exigir.") · Equalizador (`KSwitch` → `eq-toggled`; desligado = bit-perfect) · **placeholder do EQ**: bloco desabilitado (opacidade .55, sem foco) com legenda "Equalizador de 10 bandas, com curvas por fone" + tag "Em breve" + 10 sliders verticais (largura 28, altura 84, trilho 6, polegar `ink-3`) rotulados 31, 62, 125, 250, 500, 1k, 2k, 4k, 8k, 16k em mono 12 `ink-3` |
| Aparência | Tema (`Segmented` Auto/Claro/Escuro → `theme-mode-changed`) · Cor de destaque (`Segmented` "Da capa"/"Âmbar" → `accent-mode-changed`) · **Reduzir transparência** (`KSwitch` → `Theme.reduce-transparency`; texto "Troca o vidro desfocado por painéis sólidos. Ajuda em telas lentas.") · Reduzir movimento (`KSwitch` → `Theme.reduce-motion`) |
| Privacidade | **Sem telemetria**: `KSwitch { locked: true }` (sempre ligado, não editável; `accessible-label: "Sem telemetria, sempre ativado"`) + texto "O Kazzu só se conecta quando você pede algo. Nenhum dado de uso sai do seu aparelho, e esta opção não pode ser desligada." Sem callback. |
| Sobre | "Kazzu 0.1.0, código aberto" + `TextButton` "Licenças" (lista inclui Slint, Hanken Grotesk e Geist Mono, OFL) |

Rótulos "Provedores/Áudio/…" não usam caixa-alta. Avatar no topo (compact) leva a esta tela.

### 4.8 Estados vazios (`EmptyState`)

Quatro instâncias fixas (texto exato em `DESIGN.md` §10): biblioteca vazia (`library`, ação `add-folder`), sem resultados (`search`), fila vazia (`queue`, ação "Ir para o Início" → `navigate(Tab.home)`), offline (`cloud-off`, ação "Ver baixados" → `open-collection("downloads")`).

### 4.9 Carregando

`SkeletonRow`: mesma geometria da `TrackRow` (64): capa 48 raio 10 `surface-3`; duas barras (altura 14, largura 55%, raio 7, `surface-3`; altura 12, largura 34%, raio 6, `surface-2`). **Estático.** Mostrar 3 linhas. O indicador de progresso só aparece após 300 ms e fica ≥ 300 ms.

## 5. Medidas e estados transversais

### 5.1 Quadro de medidas

| Item | Compact (< 720) | Regular (720–1099) | Wide (≥ 1100) |
|---|---|---|---|
| Gutter | 20 | 32 | 32 |
| Navegação | dock 64 (raio 32) | sidebar 80 | sidebar 248 |
| Mini/NowBar | mini 64 (raio 22) | NowBar 72 sem progresso | NowBar 72 com progresso |
| Carrossel | cartão 300×400 | 380×460 | 380×460 |
| Linha | 64, capa 48 | 64 + coluna álbum 28% | idem |
| Coleções | 2 colunas | 4 | 4 |
| Capa do player | 340 | 440 | 440 |
| Botão play | 84 | 72 | 72 |
| Padding inferior | 196 | 140 | 140 |

### 5.2 Estados de controle

| Controle | Normal | Hover | Pressionado | Foco | Desabilitado |
|---|---|---|---|---|---|
| `IconButton` | tint `ink-2` | fundo `accent-soft`, tint mantém | fundo `surface-3` | anel 2 px | opacidade .5, sem eventos |
| `PillButton primary` | `accent` | mistura 12% `ink` | mistura 24% `bg` | anel | opacidade .5 |
| `Chip` | `surface-2` | `surface-3` | `surface-3` | anel | .5 |
| `TrackRow` | transparente | `surface` | `surface-2` | anel arredondado | n/a |
| `KSwitch` | `surface-3` / `accent` | n/a | n/a | anel | `locked`: ligado, sem eventos |
| `KSlider` | trilho 6 | trilho 10 + polegar | idem | idem + polegar | .45 |

### 5.3 Teclado e acessibilidade (desktop)

Tab percorre na ordem visual. Espaço/Enter ativam. Setas movem sliders (±5%). Esc fecha o Player e a Fila. Atalhos globais (etapa 3, MPRIS/mídia): Espaço play/pause com a janela em foco fora de campos de texto. Todo item da `Sidebar` é `accessible-role: tab` (ou `button`) com rótulo; o selo de formato é `text` com rótulo "FLAC 24/96".

### 5.4 Pontos de cor que não passam por `Theme`

Somente: véu do player (3 paradas de `Theme.im-scrim-*`, já tokens), gradiente do mood tile (`#000000b3`), placa do cartão de destaque (`#0c0907a3`; sólida `#1a1511`) e fundo/ícone do play do player (`Theme.im-ink` e `#15110d`). Qualquer outro literal é erro de revisão.

## 6. Dados do Rust que a UI espera

- `backdrop` e `ambient`: `Image::from_rgba8` 96×96 (receita em `DESIGN.md` §3.5), atualizados na troca de faixa; `ambient` pode ser a mesma imagem.
- `Theme.dynamic-accent*`: algoritmo de `DESIGN.md` §3.4 (testado em `tools/make_assets.py`).
- `Track.detail` curto para lista (`24/96`, `320`), `detail-long` para o player (`24-bit/96 kHz`); `hires` = profundidade ≥ 24 bits.
- Rótulos de contagem e duração já formatados em pt-BR ("214 músicas", "3:21", "−2:19" com sinal de menos real `−` ou hífen, escolher um só em todo o app).
- Detectar o renderer escolhido (Skia/FemtoVG × software) e escrever `Theme.blur-supported`. **[verificar a API exata no Slint 1.19]**.
- Preferências do sistema de movimento e transparência reduzidos → valores iniciais de `Theme.reduce-motion` / `reduce-transparency`; as chaves de Configurações mandam depois.

## 7. Critérios de aceite de UI (por etapa)

1. Visual conferido lado a lado com `design/prototypes/*.html` em 390×844 e 1280×800, claro e escuro.
2. Nenhum literal de cor/tamanho fora de `Theme` (grep em `ui/`), exceto §5.4.
3. Com `Theme.reduce-transparency = true` e com o renderer software, nenhuma tela perde legibilidade nem hierarquia.
4. Navegação inteira por teclado, anel de foco visível, todo botão de ícone com `accessible-label`.
5. `Theme.reduce-motion = true` ⇒ nenhuma transição visível; nenhum timer de animação em execução.
6. Medir: abrir Início com vidro ativo em GPU integrada fraca ≥ 55 fps ao rolar; se falhar, reportar (não remover o vidro sem aprovação do autor).
