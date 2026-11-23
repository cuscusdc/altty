# Kazzu — design system (v3: Apple-like, neutro, vidro sobre conteúdo)

Versão 0.3 · Slint 1.19 · pt-BR. Fonte dos números: `design/tokens.json` (gerador: `tools/gen_tokens.py`). Telas navegáveis: `design/prototypes/index.html` (390 e 360 px primeiro; depois 768 e 1280). **UX, fluxos, estados, microcopy, acessibilidade e arquitetura de informação não mudaram** (`design/ux/UX_PLAN.md`, aprovado): a v3 troca só o **visual**. Medidas e árvores de componentes: `design/SLINT_SPEC.md`. Prova do vidro: `design/prototypes/glass-proof.html` e `design/slint/proof-*.png`.

**Por que existe a v3.** O autor reprovou o visual da v2: *"está sem glassmorphism"*, *"esse degradê está muito feio"*, *"pedi um design APPLE LIKE"*. O diagnóstico (conferido nas capturas do app e dos protótipos v2) estava certo:

1. o fundo era uma **malha de gradiente marrom/vermelho saturada e turva** (feia e nada Apple);
2. o vidro era **"esfumaçado"**: fill marrom tingido a 40–72% sobre essa malha, que parece cartão opaco;
3. no desktop quase não havia nada vivo atrás dos painéis para desfocar, então o blur não aparecia;
4. a tipografia, as listas e a barra eram de "app Material com cor", não de Apple Music / iOS 26.

## 1. A ideia

> **A capa é o herói; o vidro é o cromo.** O fundo é neutro e quieto. A cor vem das capas. Só o que flutua (barra em cápsula, mini player, cabeçalho de rolagem, folhas, legendas sobre capa, controles do player) é vidro claro, e há **sempre conteúdo colorido passando por baixo dele**.

Inspiração: convenções do Apple Music, do iOS 26 "Liquid Glass" e do macOS Tahoe. **Sem copiar** logotipos, SF Symbols, arte ou código da Apple: tipografia Inter (OFL), ícones próprios, vidro desenhado com blur/borda/sombra comuns.

### 1.1 Antes × depois

| Aspecto | v2 (reprovada) | v3 |
|---|---|---|
| Fundo | malha de 4 manchões saturados + capa em 150% (marrom/vermelho) | **preto verdadeiro** `#000` no escuro; cinza-sistema `#f2f2f7` no claro. Sem gradiente. |
| Cor | fundo inteiro colorido, âmbar por toda parte | cor **só das capas**; **um** acento discreto (âmbar da marca) em estado: aba ativa, curtir, foco, interruptor |
| Vidro | fill tingido do matiz da capa (L 24%, alfa 40–72%): "fumaça" | **sombreado neutro** (preto 32–48%) + **tint branco claro** (10–20% no escuro, 55–75% no claro) + blur 24–40 + saturação 1,8 |
| Borda | 1 px de gradiente, fraca | 1 px de gradiente de luz (highlight especular no topo-esquerda) + aro de 0,5 px + fio especular + sombra neutra |
| Onde há vidro | cartões, banner, grupos de Ajustes, campo… quase tudo | **só o que flutua**; listas, banners e grupos são cartões opacos agrupados (raio 20) |
| O que passa atrás | quase nada no desktop | cabeçalho de rolagem, barra, mini player e NowBar **sobre a lista/capas**; legenda de vidro sobre a capa; borda de rolagem (fade) para contraste |
| Player | malha laranja/vermelha + capa | **capa muito desfocada e escurecida** (blur 70, brilho 0,55, véu neutro 60%); sem capa = grafite |
| Tipografia | Hanken Grotesk, títulos 22–34 | **Inter** (+ Inter Display nos títulos, tracking apertado); título grande 34 que encolhe ao rolar |
| Listas | linhas soltas sobre o fundo | **listas agrupadas** (cartão raio 20, separador fino recuado até o texto) |
| Barra inferior | cápsula marrom translúcida | cápsula de vidro claro, indicador deslizante, ícone **preenchido** no acento quando ativo |

Capturas (antes/depois, lado a lado): `target/ui-screenshots/design-v3/painel-antes-depois.png` (e `design/prototypes/assets/before-after.png`).

## 2. Princípios (e o que traduzimos das skills `hallmark` e `taste-skill`)

| Princípio | Como aparece no Kazzu |
|---|---|
| **Conteúdo é o herói** | fundo neutro; capas grandes com sombra suave; texto curto; uma ideia por tela |
| **Neutro e contido** | sem malha, sem *glow*, sem gradiente roxo-azul; **um** acento discreto, nunca texto corrido nem botão |
| **Vidro com função** | cromo flutuante só; nunca aninhado; nunca em linha de lista; no máx. 6 painéis com blur |
| **Hierarquia por escala e peso** | 4 pesos (400/500/600/700), título grande 34, **um** botão primário (neutro invertido) por tela |
| **Contraste e foco são requisitos** | AA **medido em captura real** (`ux/contrast-report.md`), foco de 2 px imediato, alvos de 48 |
| **Movimento reduzido de verdade** | tudo que anima lê `Theme.reduce-motion`; nenhum loop |
| **Anti "cara de IA"** | sem emoji como ícone, sem cartão dentro de cartão, sem rótulos em caixa-alta espaçada, sem *eyebrow*, sem `Artista · Álbum` (vírgula), sem pontos decorativos, sem números inventados |
| **Textos específicos** | títulos nomeiam coisas; botões = verbo + objeto; vazio diz o que falta, por quê e dá uma ação (UX_PLAN §7.3) |

Skills rodadas (resultado em `design/README.md`): `hallmark` (leitura completa; `sloplint` nos protótipos HTML) e `taste-skill` (leitura completa; princípios traduzidos: uma seção por tela, sem legenda sob imagem, hover/pressão visíveis, bordas iguais, ícones reais). O que é de site (hero, deal de temas, gate do Chrome) **não se aplica** a um app e foi só traduzido.

## 3. Cor

Cores em `tokens.json` (hex ou OKLCH) → hex em `tokens.slint` (`Theme.*`) e `tokens.css`. Claro e escuro são o mesmo sistema.

### 3.1 Neutros (sem tom de cor)

| Token | Escuro | Claro | Uso |
|---|---|---|---|
| `bg` | `#000000` | `#f2f2f7` | fundo da janela (sem palco) |
| `surface` / `cell` | `#1c1c1e` | `#ffffff` | cartões agrupados, banners, coleções |
| `surface-2` / `surface-3` | `#2c2c2e` / `#3a3a3c` | `#ebebf0` / `#e0e0e6` | estados e poços |
| `ink` | `#ffffff` | `#000000` | texto principal |
| `ink-2` / `ink-3` | `#b5b5bb` / `#98989e` | `#505056` / `#68686e` | secundário / meta (sobre `bg` e `surface`) |
| `ink-glass-2` | `#fffffff5` | `#44444a` | **secundário sobre vidro** (mais claro que `ink-2` para manter 4,5:1) |
| `fill` / `fill-2` | branco 12% / 20% | cinza-sistema 16% / 24% | chips, trilhos, estados |
| `line` / `line-strong` | branco 12% / 25% | tinta 20% / 30% | separadores finos / bordas |
| `edge-fade` | `#000000` | `#f2f2f7` | cor do degradê de borda de rolagem |

### 3.2 Acento (um só, discreto)

Âmbar da marca: `#ffa154` (escuro) / `#ab4b01` (claro); tinta sobre acento `#190c04` / `#fefbf8`. **Onde aparece:** ícone preenchido da aba/destino ativo, coração de curtir, barrinhas de "tocando", interruptor ligado, anel de foco, "Ver mais". **Onde não aparece:** botão primário (é neutro invertido), fundo, texto corrido. O acento **dinâmico da capa** (`use-dynamic-accent`) continua opcional em Ajustes (padrão desligado) e usa o algoritmo da v2 (reduzir a 32×32, OKLCH, 12 baldes de matiz).

### 3.3 Player (sempre escuro)

Fundo = a **capa muito desfocada**: CSS `blur(70px) saturate(1,2) brightness(.55)` em 160% da tela, véu preto 60% e degradê inferior até preto 72%; Slint: imagem 96×96 já desfocada pelo Rust (σ 4, saturação ×1,15) + `Theme.im-veil` + degradê. Sem capa: `surface` (grafite). A luminosidade é **controlada** (o texto branco passa 7:1 até no pior caso, capa branca) e a cor nunca é forçada: é a da capa.

## 4. Tipografia

**Inter** (OFL, `design/fonts/`) 400/500/600/700 para texto; **Inter Display** 600/700 para títulos ≥ 20; **Geist Mono** 400/500 só para tempos e selos (Slint não tem algarismos tabulares). Escala (px lógicos; peso; linha; tracking): `large-title` 34/41 700 −0,8; `title-1` 28/34 700 −0,6; `title-2` 22/28 700 −0,45; `title-3` 20/25 600 −0,4; `headline` 17/22 600 −0,34; `body` 17/22 400 −0,34; `callout` 16/21 −0,25; `subhead` 15/20 −0,2; `footnote` 13/18; `caption` 12/16 500; `player-title` 24/30 700 (celular), 34/41 (desktop). Hierarquia por **peso e tamanho**, não só por cinza. Texto mínimo 12 (rótulos); corpo ≥ 15. Respeita a escala de texto do sistema até 200%. **Título grande:** em cada aba o título (34, Inter Display bold) fica no fluxo, abaixo do cabeçalho; ao rolar ele sai e o **título pequeno (17 semibold) entra no centro do cabeçalho**, que vira vidro.

## 5. Forma, espaçamento, tamanhos

Escala de 4 px. Raios: `xs 6`, `sm 10`, `md 14`, `lg 20`, `xl 26`, `full`. Slint não tem cantos contínuos (squircle): usar o raio nominal (a diferença é invisível a 20+). Raios concêntricos (interno = externo − padding).

| Token | Valor | Nota |
|---|---|---|
| `hit` | 48 | alvo mínimo; `hit-min` 44 só para chips (alvo estendido a 48) |
| `gutter-mobile` | 16 (24 ≥ 600; 32 ≥ 1200) | |
| `group-radius` | 20 | listas agrupadas, banners, coleções |
| `hero-radius` / `tile-radius` / `thumb-radius` | 26 / 14 / 10 | cartão-herói / tile / miniatura |
| `player-cover-radius` / `sheet-radius` | 22 / 28 | |
| `header-h` | 56 | cabeçalho de rolagem |
| `dock-h` / `dock-radius` / `dock-search` | 64 / 32 / 56 | cápsula e botão Buscar |
| `mini-h` | 64 (cápsula, raio 32) | |
| `nowbar-h` | 76 (cápsula) | ≥ 1200 |
| `row-h` / `row-cover` / `row-inset` | 64 (+4) / 52 / 80 | separador recuado até o texto |
| `play-main` | 72 (68 no desktop) | círculo branco |
| `progress-h` / ativo / `slider-thumb` | 6 / 10 / 18 | barra fina; polegar só com hover/foco |
| `rail-w` / `rail-w-wide` | 88 / 248 | trilho e barra lateral |

## 6. Materiais: vidro

### 6.1 Três materiais

| Nível | Quem | Blur | Tint (branco) escuro / claro | Sombreado (preto) escuro | Sombra |
|---|---|---|---|---|---|
| **1 · fino** | legenda sobre capa, botões de vidro, atalhos do player, barra lateral | 24 | 10% / 55% | 32% | 24, y8 |
| **2 · regular** | barra em cápsula, Buscar, mini player, NowBar, cabeçalho de rolagem, controle do player | 32 | 15% / 65% | 40% | 32, y12 |
| **3 · espesso** | folhas (menu, playlist, fila), diálogos | 40 | 20% / 75% | 48% | 56, y24 |

Só o **escuro** tem sombreado: o preto translúcido que garante o contraste do texto branco sem tingir. No claro o próprio branco 55–75% já dá contraste (texto preto).

### 6.2 Receita (de baixo para cima)

1. **Backdrop blur** forte (24/32/40) + **saturate(1,8)** no CSS (no Slint 1.19 só existe blur: sem saturação; as capas já são vivas).
2. **Sombreado neutro** (preto, só no escuro).
3. **Tint claro translúcido** (branco): é o que faz ler "vidro" e não "fumaça".
4. **Brilho especular** em gradiente (branco 18% → 0) nos 52% de cima.
5. **Borda de luz de 1 px em gradiente** a 155°: forte no canto superior esquerdo (`glass-edge`), fraca no meio (`glass-edge-low`), 45% no canto oposto; mais um **aro externo de 0,5 px** (`glass-ring`).
6. **Fio especular** de 1 px no topo (inset = raio, esvaece nas pontas).
7. **Sombra neutra** do nível.

**Regras:** nunca aninhar vidro (filhos usam `fill`); no máximo 6 painéis com blur visíveis; nada de vidro em linhas ou dentro de `ListView`; **cada tela mantém conteúdo colorido sob o cromo** (é isso que prova o vidro). Banners, listas e grupos são **cartões opacos** (`cell`), não vidro.

### 6.3 Borda de rolagem (scroll edge)

Sob a barra e o mini player há um degradê da cor do fundo (preto / `#f2f2f7`) de 0 a 55% em 176 px, e sob o cabeçalho de rolagem o vidro regular. Isso garante que o texto de vidro mantenha contraste sobre capas claras e dá o efeito Apple de o conteúdo "sumir" sob o cromo. Cabeçalho de rolagem: em repouso transparente; ao rolar > 4 px o vidro aparece (200 ms) com fio inferior.

### 6.4 Contraste (calculado + medido)

- **Calculado** (`gen_tokens.py`): `ink` e `ink-glass-2` sobre vidro 1·2·3 passam ≥ 7:1 (ink, sobre o fundo) e ≥ 4,5:1 nos casos "conteúdo médio (L 0,20)" e "branco sob o fade". O vidro escuro tolera atrás dele até luminância relativa **0,25–0,28** com `ink-glass-2` a 4,5:1 (a borda de rolagem mantém o conteúdo abaixo disso); o claro tolera qualquer fundo. Sólidos (reduzir transparência) ≥ 7:1.
- **Medido em captura real** (`tools/check_render.py` → `ux/contrast-report.md`): **2.122 textos em 117 capturas, 0 abaixo de AA, menor contraste 4,83:1** (10 telas × 390/360/768/1280 × claro/escuro, com e sem transparência, rolagem sob o cromo, 7 capas no Início e no player). A primeira rodada achou 127 reprovações; corrigidas: título pequeno invisível contado como texto (ferramenta), legendas de capa em tema claro (a legenda agora é sempre vidro escuro `scope-dark` sobre degradê preto 60% + sombreado 60%), cabeçalho de rolagem sobre capa clara (acrescentado fade da cor do fundo sob o vidro), folha da fila sobre o player (scrim 80%).

### 6.5 Fallback ("Reduzir transparência")

`Theme.glass-live = blur-supported && !reduce-transparency`. Falso ⇒ blur 0, tint some, fundo sólido neutro (`glass-solid`, `-strong`, `-3`; ≥ 7:1). Gatilhos: renderer software; chave em Ajustes › Aparência; preferência do sistema; oferta automática após 2 s acima de 16 ms/quadro. Protótipo: `#rt=1`.

### 6.6 Limites conhecidos

- Slint 1.19: **sem saturate/brightness** no backdrop; **sem cantos contínuos**; **sem algarismos tabulares** (usa Geist Mono nos tempos); blur de 40 em tela cheia custa em GPU fraca: a folha do player usa a capa **pré-desfocada pelo Rust** e só a cápsula de transporte tem blur.
- Cabeçalho de rolagem: o blur só liga com `p > 0,02` (evita pagar blur em repouso).
- Blur só do conteúdo da própria janela (sem acrílico do SO).

## 7. Marca e ícones

Marca (K de barras), wordmark, ícone do app e favicons **inalterados** (`design/assets/`); wordmark no canto superior esquerdo (28 de altura) em todas as telas compactas; trilho: marca 36; lateral ≥ 1200: wordmark. **57 ícones** de grade 24, traço 1,75, pontas redondas, estilo uniforme (desenho próprio, não SF Symbols). A v3 acrescenta **`home-fill`, `explore-fill`, `library-fill`**: o destino ativo da barra troca para a versão preenchida no acento (como as abas da Apple). "Do YouTube Music" usa o ícone genérico `cloud`: nada do logotipo do YouTube.

## 8. Movimento

`dur-micro` 120, `dur-short` 200, `dur-medium` 320, `dur-long` 480. Curvas `ease-out cubic-bezier(.22,.9,.28,1)` (chegadas, mola suave) e `ease-in-out (.45,0,.2,1)`. Indicador da barra desliza (320 ms); cabeçalho de rolagem faz fade do vidro (200 ms); folhas sobem em `dur-medium` e **seguem o dedo** ao arrastar (fecham passando de 140 px); troca de capa = cross-fade 480 ms. `reduce-motion` zera tudo (corte seco, sem camada de capa se preferido). Sem loops.

## 9. Acessibilidade

Detalhe em `UX_PLAN.md` §9. Resumo: AA medido em captura; foco visível e não obscurecido (reserva de 184 dp); alvos de 48; TalkBack com rótulos, estados e ações no lugar de gestos; WCAG 2.5.7; **Reduzir transparência** e **Reduzir movimento** independentes; texto até 200%; cor nunca é o único sinal (aba ativa = cápsula + ícone preenchido + rótulo em negrito).

## 10. Estados vazios, carregando e erro

Estrutura única: cartão opaco (raio 28), ícone de sistema (88, raio 26), título, até 2 frases, **uma** ação. Seis estados em `prototypes/states.html`. Textos: `UX_PLAN.md` §7.3.

## 11. Responsividade (mobile-first)

| Classe | Largura | Navegação | Mini player | Conteúdo |
|---|---|---|---|---|
| **compact** (base) | 360–599 | cápsula + Buscar circular | cápsula acima da barra | 1 coluna; coleções 2×2 |
| **medium** | 600–839 | trilho 88 (vidro fino) | flutuante, até 640 | 2–3 colunas |
| **expanded** | 840–1199 | trilho 88 | idem | coleções 4×1; player em 2 colunas |
| **large** | ≥ 1200 | barra lateral 248 (vidro fino) | **NowBar** cápsula de vidro com progresso e volume | até 1240 centrado; o carrossel rola **por baixo** da barra lateral |

## 12. Telas (protótipos)

`design/prototypes/index.html` (hub) mostra 390, 360, 768 e 1280; cada tela abre sozinha e aceita `#theme=light|dark`, `#rt=1`, `#sat=0`, `#c=01..10`, `#s=`, `#scroll=N`. **Novo:** `glass-proof.html` (prova do vidro: três materiais em escuro e claro sobre capas, antes × depois, fallback sólido).

| Tela | Arquivo | Destaques v3 |
|---|---|---|
| Início | `home.html` | título grande, aviso em cartão, chips neutros, destaques com legenda de vidro, escolhas rápidas em **lista agrupada** |
| Explorar | `explore.html` | tiles com sombra neutra, humores com legenda de vidro |
| Biblioteca | `library.html` | título grande + Criar playlist (primário neutro), 4 coleções em cartões opacos |
| Álbum / Buscar / Fila / Folhas / Ajustes / Estados | `album`, `search`, `queue`, `sheets`, `settings`, `states` | listas e grupos agrupados; folhas de vidro nível 3 sobre conteúdo |
| Player | `player.html` | capa desfocada de fundo, controles de vidro, barra fina |

## 13. Decisões para o autor aprovar

1. **Âmbar** como único acento (alternativa: mudar `brand.ember`; ou ligar o acento da capa por padrão).
2. **Preto verdadeiro** no escuro (como Apple Music OLED) e `#f2f2f7` no claro. Alternativa: grafite `#111113`.
3. **Vidro escuro leva sombreado neutro** (preto 32–48%) sob o tint branco para garantir contraste AA sobre capas claras; sem ele o vidro seria mais "cristal" mas reprovaria. O tint branco continua 10–20%.
4. **Início sem palco colorido**, cor só nas capas (a v2 tinha a decisão inversa).
5. **Inter** no lugar de Hanken Grotesk (e Inter Display nos títulos).
6. **Barra lateral desktop** em vidro fino e discreto (a cor atrás depende do carrossel que rola por baixo); sem vibrancy do SO.
7. Alvo mínimo 48; Início × Explorar separados; curtidas e playlists locais; barra com 3 destinos + Buscar circular; player sempre escuro (decisões v2 mantidas).

## 14. O que foi verificado

Ver `design/README.md` (lista completa de comandos e resultados) e o relatório de entrega. Resumo: `gen_tokens.py` (pares AA), protótipos renderizados no Chromium (390/360/768/1280, claro/escuro, com e sem transparência), contraste medido em captura (`check_render.py`), `sloplint` do hallmark nos protótipos, **renderização real em Slint 1.19 com FemtoVG (backdrop-blur real, Xvfb)**: `design/slint/*.png`. **Não verificado:** Skia (só FemtoVG e software), desempenho do blur em GPU fraca e em Android real, TalkBack, testes com pessoas.

## 15. Regenerar

```
python3 design/tools/gen_tokens.py        # tokens.slint, tokens.css + checagem AA
python3 design/tools/make_assets.py       # ícones (57), capas fictícias, sprite (NÃO altera a marca: ver README)
python3 design/tools/build_prototypes.py  # HTML dos protótipos
python3 design/tools/check_render.py      # contraste real em capturas (Chromium + Pillow) -> design/ux/contrast-report.md
design/tools/slint-shot                   # renderiza design/slint/preview.slint (FemtoVG); ver design/README.md
```
