# Kazzu — especificação de UI em Slint (v2, mobile-first)

Para o implementador de UI do `kazzu-app`. **Siga à risca**: nomes, medidas e estados abaixo vêm de `design/prototypes/` (abra a tela ao lado enquanto implementa; todas em 390 e 360 px primeiro). Slint 1.19, renderers FemtoVG/Skia (software = fallback sem blur). Identificadores em inglês; textos visíveis em pt-BR. Planejamento de UX (fluxos, estados, gestos, acessibilidade): `design/ux/UX_PLAN.md`. Receita visual e decisões: `design/DESIGN.md`.

**O que mudou da v1:** (1) o desenho parte do celular (compact) e *acrescenta* para tablet/desktop; (2) o fundo é um **palco vivo** (`Stage`) e o vidro tem **3 níveis**, em vez de painéis escuros opacos sobre quase preto; (3) YouTube Music é a fonte padrão, sem login; (4) alvo mínimo 48; (5) barra inferior e mini player são pílulas flutuantes de vidro; (6) o player e a fila são **folhas**.

## 0. Arquivos prontos (compilam no Slint 1.19; mova para `crates/kazzu-app/ui/`)

| Arquivo | O que é | Regra |
|---|---|---|
| `design/slint/tokens.slint` | `export global Theme` (cores claro/escuro, **palco**, **vidro por nível**, tipografia, espaços, raios, tamanhos, durações) | **Gerado** por `tools/gen_tokens.py`. Não editar à mão; mude `tokens.json` e regenere. |
| `design/slint/atoms.slint` | **`Stage`**, **`GlassPanel` (níveis 1–3)**, **`Chip`**, `CoverImage`, `IconButton`, `FormatBadge`, `KSwitch`, `KSlider` | Usar como estão. Evoluir aqui, não duplicar. Correções da implementação v1 mantidas (`FocusScope` com `focus-on-click: false`; `KSlider.value` como `in`; ver `docs/UI_NOTES.md`). **O `atoms.slint` do app precisa receber o diff do design**: `GlassPanel` ganhou `level`, `Stage` e `Chip` são novos. |
| `design/slint/app-contract.slint` | enums, structs e `global AppState` | Fronteira Rust ↔ UI. **Inalterado na v2**; os acréscimos pedidos estão em §7. |
| `design/slint/preview.slint` | palco + 3 níveis de vidro + chip + dock/mini | Só para `slint-viewer`. |
| `design/fonts/*.ttf` | Hanken Grotesk 400/600/700, Geist Mono 400/500 | `import "…ttf";` e `default-font-family: Theme.font-sans`. |
| `design/assets/icons/*.svg` | **54 ícones** (v2 acrescenta `gear`, `refresh`, `cloud`, `stats`, `timer`, `arrow-up`, `arrow-down`, `minus`, `badge-check`) | `Image { source: @image-url(…); colorize: …; }` |
| `design/assets/kazzu-*.svg` | marca e ícone | wordmark/marca mono: `colorize`. |

No `.slint` raiz: `export { Theme, AppState }`.

## 1. Regras gerais

1. **Nunca** cor, tamanho, raio ou duração literal: sempre `Theme.*` (exceções em §6.4).
2. Todo botão de ícone: `IconButton` com `label` em pt-BR; todo controle tem `accessible-role` e rótulo; ações que no protótipo são gestos têm **também** botão (UX_PLAN §8).
3. **Alvo mínimo 48 × 48** (`Theme.hit`). Foco: anel de 2 px (`Theme.focus-ring`), sem animação, **nunca coberto** pela barra/mini (reserve `Theme.chrome-reserve` = 184 de padding inferior nas telas de rolagem).
4. Listas longas = `ListView`. **Nada de vidro dentro de linhas.** No máximo **6** `GlassPanel` com blur visíveis ao mesmo tempo (barra, buscar, mini, até 3 cartões).
5. **Nunca aninhar `GlassPanel` dentro de `GlassPanel`** (o filho borraria o pai). Elementos dentro de vidro (chips, campos, botões) usam fill simples `Theme.ink.with-alpha(0.12)` e borda `Theme.ink.with-alpha(0.16)`.
6. Animação só com `animate` em componentes. Sem animação infinita. Durações `Theme.dur-*`; curvas por literal (`cubic-bezier(0.22, 0.9, 0.28, 1)` chegadas, `cubic-bezier(0.45, 0, 0.2, 1)` trocas).
7. `Theme.window-w` atualizado no `AppWindow` (`changed width => { Theme.window-w = self.width; }`). Classes: `Theme.compact` (< 600) **base do design**, médio 600–839 (`!compact && !expanded`), `Theme.expanded` (≥ 840), `Theme.wide` (≥ 1200). **Escreva a árvore compact primeiro**; `if !Theme.compact : …` só acrescenta.
8. Rust escreve em `Theme`: `dark`, `reduce-motion`, `reduce-transparency`, `blur-supported`, `use-dynamic-accent`, `dynamic-accent(-ink)`, **`stage-{a,b,c,d}-{dark,light}` e `glass-tint-{dark,light}`** (§7).
9. Textos de linha: `overflow: elide`, uma linha; descrições até 2 linhas. Rótulos de barra 12, mínimo do app 12.
10. Capa: `CoverImage` (`image-fit: cover`, contorno de 1 px). Ausente = placeholder `Theme.surface-2` + ícone `disc`.
11. Texto direto sobre o palco: só `Theme.ink` e `Theme.ink-2`. `Theme.ink-3` **somente sobre vidro** (contraste verificado só aí).

## 2. Palco e vidro

### 2.1 `Stage` (atrás de tudo, uma vez)

`Stage { backdrop: AppState.backdrop; has-cover: AppState.has-now; }` preenche a janela. Camadas: `Theme.stage-base`; 4 manchões `@radial-gradient(circle, stage-X 0%, stage-X.with-alpha(0) 70%)` (A em 12%/8%, B 98%/34%, C 10%/72%, D 88%/96% da janela; diâmetro 1,1–1,3 × o maior lado); a **capa em 150%** desfocada e escurecida pelo Rust, `opacity: Theme.ambient-opacity` (0,50 escuro / 0,35 claro); véu inferior 55%. Troca de faixa = cross-fade de 480 ms entre dois `Stage` ou duas `Image` (corte com movimento reduzido). **O palco é decorativo**: não entra na árvore de acessibilidade.

### 2.2 `GlassPanel` (3 níveis)

| Nível | Uso | Blur | Fill (alfa sobre `glass-tint`) | Sombra |
|---|---|---|---|---|
| **1** | cartões de coleção, banner, placas de destaque, grupos de ajustes | 20 | 0,40 escuro / 0,52 claro | 24 / y8 |
| **2** | barra inferior, Buscar, mini player, campo de busca, trilho lateral | 32 | 0,56 / 0,66 | 32 / y12 |
| **3** | folhas (player, fila, menu, nova playlist), diálogos | 40 | 0,72 / 0,80 | 56 / y24 |

Camadas (de baixo): `backdrop-blur` → fill `glass-tint` com alfa → brilho superior (`glass-sheen`, 52% da altura) → **borda de 1 px em gradiente de luz** (`@linear-gradient(155deg, glass-edge, glass-edge-low 34%, glass-edge-low 66%, glass-edge 40%)`; FemtoVG e Skia aceitam brush em `border-color` — **conferir no primeiro build**; se o renderer ignorar, usar `Theme.glass-edge-mid` sólido) → **fio especular** de 1 px no topo → `drop-shadow` do nível → `clip: true`. O `glass-tint` herda o matiz da capa (escuro: L 24%, C 0,055; claro: L 99%, C 0,012).

**Saturação.** O backdrop do Slint 1.19 só tem blur. Os protótipos usam `saturate(1.45)`; o Slint **compensa no palco** (os manchões e a capa já têm croma alto). Para ver o mesmo que o Slint, abra qualquer protótipo com `#sat=0`.

**Fallback.** `Theme.glass-live` falso ⇒ `glass-bg*` sólidos, blur 0 (já nos tokens). Gatilhos: renderer software, **Reduzir transparência** (Ajustes), preferência do sistema quando existir, e oferta automática após 2 s com quadro médio > 16 ms. O palco continua (estático, sem a camada de capa se o movimento estiver reduzido). Contrastes do sólido ≥ 7:1.

### 2.3 Instâncias

| Instância | Nível | Raio | Observação |
|---|---|---|---|
| Barra inferior | 2 | 32 | `x: 12; width: window − 24 − 64`; 3 abas; sobre fade de 190 |
| Buscar (botão) | 2 | 28 (círculo 56) | ativo: fill `Theme.accent` + ícone `accent-ink` |
| Mini player | 2 | 24 | 8 acima da barra; só com faixa |
| Cartão de coleção | 1 | 24 | 4 na Biblioteca |
| Placa do destaque | 1 | 20 | sobre a capa do cartão; texto `ink` |
| Banner de aviso | 1 | 24 | dispensável |
| Campo de busca | 2 | 28 | **não** é barra fixa: só dentro da tela Buscar |
| Folhas, menus | 3 | 32 (topo) | com `Scrim` (preto 38%) |
| Trilho (≥ 600) | 2 | 32 | 88 de largura |
| Barra lateral (≥ 1200) | 2 | 32 | 248 |
| Pílula de transporte do player | 2 | 46 | h 92 |

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

`GlassPanel level 2, radius 32, height 64`, padding 6, `HorizontalLayout` de **3 `NavTab`** (flex igual, spacing 4). `NavTab { icon; label; active }`: coluna ícone 24 + rótulo `caption` 12 semibold, altura 52, raio 26; **ativo** = fundo `Theme.ink.with-alpha(0.20)`, ícone `Theme.accent`, rótulo `Theme.ink` em 700 (três sinais, nunca só cor); inativo = `Theme.ink-2`. `accessible-role: tab`, `accessible-checked: active`. `SearchButton`: 56 × 56, `GlassPanel level 2` redondo ao lado (gap 8); ativo preenchido `Theme.accent`. `accessible-label: "Buscar"`. **Única entrada de busca**; nenhuma tela tem campo de busca no topo exceto a própria Busca.

### 3.3 `MiniPlayer` (compact; ≥ 600 flutuante com largura máx. 640)

`GlassPanel level 2, radius 24, height 64`, toque abre o player. `Grid`: capa 48 (raio 16) · título `callout` 15 semibold + artista `subhead` 14 (`ink-2`) · `IconButton` play/pause (48, glifo 28) · `IconButton` próxima. Fio de progresso de 3 px a 20 de cada lado, a 5 do fundo (`ink` 22% / `accent`). Sem faixa: não aparece e a barra desce. **Pausado/Tocando** muda `accessible-label`. Em ≥ 1200 vira `NowBar`: 72 de altura, adiciona `KSlider` (altura 32) com tempos mono, e volume.

### 3.4 `NavRail` (≥ 600) e `Sidebar` (≥ 1200)

`GlassPanel level 2, radius 32`, `x: 12; y: 12; height: window − 24`, largura `Theme.rail-w` (88) ou `Theme.rail-w-wide` (248). Marca (36) / wordmark (26) no topo, **só aqui** no canto superior esquerdo. Itens 64 de altura no trilho (ícone sobre rótulo) / 52 na lateral (ícone à esquerda, `callout`); **Buscar** é o último item do grupo (mesma regra de destaque do botão circular); **Ajustes** no pé.

### 3.5 `TrackRow`

`in Track track; in bool show-format; callback clicked(); callback more();` — altura mín. **64**, padding `6 4 6 8`, raio 18, hover fundo `Theme.ink.with-alpha(0.08)`. Capa 52 (raio 12) · título `body` 16 semibold (`Theme.accent` se `playing`) · artista `subhead` 14 `ink-2` · **motivo** opcional (`why`, `ink-2`, ex.: "Em alta hoje") · barras "tocando" estáticas · `FormatBadge` (some em < 380 px) · `IconButton more` (48). Separador entre partes: vírgula (nunca ponto-médio). Sem vidro.

### 3.6 Outros

| Componente | Props | Medidas e estados |
|---|---|---|
| `HeroCard` | `Highlight item` | compact `min(300, 78% da janela)` × proporção 300:380, raio 28, capa full-bleed; placa de vidro nível 1 (`left/right/bottom 10`, raio 20): `caption` 12 `ink-2` · `headline` 17 · `subhead` `ink-2`; botão play 52 em círculo `accent`. ≥ 840: 400×440; ≥ 1200: 460. Rola com snap por cartão. |
| `AlbumTile` | `Album item` | compact 2 colunas (gap 12/16) ou trilho de 148; capa quadrada raio 24; título 16 semibold; artista 14 `ink-2`. |
| `SectionHeader` | `title; source-tag?; action?` | `title-2` 22; à direita **`SourceTag`** ("Do YouTube Music", pílula 28 de altura, ícone `cloud` 16, `caption` 12) ou ação de texto ("Ver mais" + chevron, 48 de altura). Entre seções 28. |
| `Chip` | `text; selected` | **40 de altura** (alvo 48 com 4 px invisíveis acima e abaixo), padding 16; selecionado: fundo `Theme.ink`, texto `Theme.bg`. Linha de chips rola na horizontal sem barra. Seção do Início: Para você, Descobrir, Mixes, Foco, Treino. |
| `Banner` | `title; text; dismissible` | vidro nível 1, raio 24, ícone `badge-check` em círculo 44 `accent`; "O YouTube Music já está ativo". Dispensar grava em config e **nunca volta**. |
| `CollectionCard` | `Collection` | vidro nível 1, raio 24, padding 12/14; arte quadrada raio 16: **Baixados** = `Theme.ink.with-alpha(0.10)` + ícone `download` 56; **Curtidas** = `@linear-gradient(150deg, accent mais claro, accent misturado ao stage-b)` + `heart-fill`; **Álbuns / Playlists** = mosaico 2×2; título `headline` 17 semibold; contagem `subhead` `ink-2`. |
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
├─ Stage (capa; palco escuro mesmo com tema claro)
├─ Grabber · TopRow: IconButton chevron-down (label "Fechar o player") · "Tocando de" + origem · IconButton more
├─ Cover: lado = min(largura − 32, altura − 500), raio 28, sombra colorida (cor stage-a) e contorno 12%
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

Somente: `@radial-gradient` do `Stage` (usa tokens), `Scrim` (preto 38%), contorno da capa (`ink` 10%) e o gradiente de Curtidas (deriva de `Theme.accent`). Qualquer outro literal é erro de revisão.

## 7. Dados do Rust que a UI espera

- **Palco.** A cada faixa, calcular o acento (como na v1, `DESIGN.md` §3.4) e, a partir do **matiz** `h` (ou 58° quando cai no âmbar), escrever as variantes escuras e claras: `stage-X = oklch(L, C, h + dh)` com `[L, C, dh]` de `tokens.json > stage` (escuro `a [0,50 0,19 0] b [0,45 0,18 −36] c [0,41 0,14 147] d [0,44 0,15 37]`; claro `a [0,86 0,09 0] b [0,86 0,085 −36] c [0,89 0,06 147] d [0,90 0,08 37]`), `stage-base` (escuro `[0,20 0,05]`, claro `[0,94 0,03]` no mesmo matiz) e `glass-tint` (escuro `oklch(24% 0,055 h)`, claro `oklch(99% 0,012 h)`). Funções puras em `accent.rs`, testadas: L máximo do palco escuro ≤ 0,50 (garante o contraste AA do texto no vidro).
- **`backdrop`** (capa para o palco): `Image::from_rgba8` 96×96, σ 4, saturação ×1,35, **brilho ×0,72**, 1 LSB de ruído; `AppState.ambient` deixa de existir como imagem separada (é o mesmo `backdrop`).
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
5. Contraste: pior caso do vidro conferido com capas clara, escura, saturada e cinza (palco com `h` 40°, 205°, 150° e âmbar); texto ≥ 4,5:1 (referência: `design/ux/contrast-report.md`).
6. `Theme.reduce-motion = true` ⇒ nenhuma transição visível; sem timers de animação.
7. Medir: Início com vidro em GPU fraca e no Android de referência ≥ 55 fps ao rolar; se falhar, reportar e oferecer "Reduzir transparência" (não remover o vidro sem o autor).
8. Zero requisição de rede ao abrir e ficar parado (UX_PLAN AC-10).
