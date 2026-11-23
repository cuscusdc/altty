# Relatório de uso das skills (v4)

Skills: `hallmark` (v2.0.0) e `design-taste-frontend` (tasteskill 2.5.1), instaladas em `.claude/skills/` e invocadas pela ferramenta Skill. As duas são skills de **sites**; o Kazzu é um **app**. Este relatório diz exatamente o que foi lido, o que foi rodado, o que saiu, o que não se aplica e o que foi corrigido por causa delas. As saídas brutas estão em `skills-logs/`.

Ambiente: Node 26.13.1; Chromium do sistema (`/usr/bin/chromium`) por um script que acrescenta `--no-sandbox` (a sessão roda como root e as skills não passam essa opção); `puppeteer-core` instalado numa pasta temporária fora do repositório, só para a camada de render do hallmark.

## 1. hallmark

### 1.1 O que foi lido

`SKILL.md` inteiro; `references/flow.md` (ramo **App screens**); `direction.md` (direção custom); `gates.md` (seções 1 a 6); `craft.md`; `patterns.md`; `dark-mode.md`; `interaction-and-states.md`; `microinteractions.md`; `fonts.md`; `imagery.md`; e `review-finish.md` (lido pelo revisor, não por mim, como a skill manda).

Não lidos, por não se aplicarem a telas de app: `layout.md` além da seção 2, `macrostructures.md`, `anatomy.md`, os arquivos de gênero e de tema (a direção é custom), `scroll-choreography.md`, `comp.md`, `data-viz.md`, `texture.md`, `redesign.md`.

### 1.2 Passos do fluxo

| Passo | Feito? | Evidência |
|---|---|---|
| 0. Pre-flight | Sim | `prototypes/.hallmark/preflight.json`. `design/DESIGN.md` (v3) NÃO foi tratado como sistema travado, por instrução do autor: é material reprovado |
| 1. Perguntar uma vez | Não perguntei: execução sem o autor na conversa e o brief já responde material, público, uso e tom | Inferências no carimbo do CSS |
| 2. Forma | Ramo de app: sem forma de página, sem hero, sem dobra | Carimbo: `shape: app screen` |
| 3. Direção | Custom (o autor fixa Apple-like, vidro, Inter). `theme.mjs check --face Inter --axes dark/grotesk-sans/warm --hue 38` respondeu: "near: Rim" | `DIRECTION.md`: três direções distantes, uma escolhida |
| 4. Carregar referências | Sim (lista acima) | |
| 5. Bloco de escolhas | Abaixo | |
| 6. Construir | `theme.mjs scaffold --custom --scope app` gerou `tokens.css` e `base.css`; depois `tokens.css` passou a ser gerado de `tokens.json` com os mesmos nomes | `prototypes/assets/` |
| 6. Carimbo e registro | Carimbo nas primeiras linhas de `kazzu.css`; `theme.mjs log --entry` | `prototypes/.hallmark/log.json` |
| 7. Varredura | `sloplint.mjs . --genre atmospheric --scope app --render` | 1.3 |
| 7. Revisão em contexto limpo | Filho Traycer, Sonnet 5.5, effort high, só capturas e brief | Seção 3 |

**Hallmark · v2.0.0**

- **Tela** · app de música: Início, Explorar, Buscar, Biblioteca, detalhe, player, fila, folhas, Ajustes · estados: carregando, vazio, erro, sem conexão, cheio
- **Ideia** · toda a cor vem das capas, vistas através do vidro
- **Aparência** · fundo `oklch(13.5% 0.005 265)` (neutro como os apps de sistema), tinta `oklch(96.5% 0.004 265)`, acento Brasa `oklch(72% 0.185 38)` (continuidade com o K da marca), Inter e Inter Display (pedido do autor)
- **Cromo** · cápsula flutuante de vidro com três abas e uma busca redonda separada; logo no canto superior esquerdo; sem rodapé
- **Assinatura** · a capa vista através do vidro: a capa inteira, o reflexo dela e um console de vidro
- **Material** · real: nomes das telas, recursos e limites do produto, textos em pt-BR · imagens: 22 fotografias CC0 e de domínio público da biblioteca do hallmark como capas substitutas, mais o produto construído · ausente: capas reais, números de uso, depoimentos (nenhum inventado)
- **Varredura** · Floor 0 FAIL · Reflex 0 (1 dispensa registrada: 1, Inter pedido pelo autor) · Finish 4 respondidos (F4, F7, F24) · revisão: subagente, "fix", 8 achados, 5 resolvidos e 3 parciais

### 1.3 `sloplint` (saída final em `skills-logs/hallmark-sloplint-render.txt`)

```
render tier: 110 screenshots (1280x800, 1440x900, 1920x1080 folds; 1440 and 375 full pages)
player.html   ANSWERED F4  two filled buttons side by side
queue.html    ANSWERED F4
kazzu.css     ANSWERED F7  the pill radius on 14 non-control selectors
kazzu.css     ANSWERED F24 a hairline border under a wide soft shadow on .glass
sloplint: 0 FAIL, 0 WARN, 4 ANSWERED across 25 file(s).
```

O que ele achou no caminho e o que mudou:

| Rodada | Achado | Correção |
|---|---|---|
| 1 (estática) | 0 FAIL; F7 e F24 em aviso | Respondidos no carimbo: as cápsulas são controles e retratos; borda de luz mais sombra é a receita do vidro |
| 2 (render) | FAIL 26a ×4: o campo de texto não mostrava foco no próprio `input` | O campo foi refeito: o `input` ocupa a caixa inteira e leva o anel de foco por dentro (o mesmo foi levado ao `SearchField` do Slint) |
| 2 | REFLEX 39b: botões de 56 px no player | Reduzidos para 52, depois trocados por dois botões de 48 |
| 3 | FAIL 73 ×3: rótulo escondido pintado sob o campo | O campo passou a ser nomeado pelo título visível "Buscar" (`aria-labelledby`) |
| 4 | F4 em aviso depois da correção do player | Respondido: saída e Fila são dois secundários iguais; o único preenchido da tela é o botão de tocar |

Gates de página que o escopo de app pula por definição da skill (1, 6, 8, 9, 32, 42, 44, 46b, 58, 59, 66, 68, 77 a 79, 81 a 83, 85, F8, F15, F20): não foram avaliados.

### 1.4 Princípios do hallmark levados para o Slint

| Regra | No design |
|---|---|
| Toda cor e toda fonte por token | `tokens.json` gera CSS e Slint; nenhum literal fora dele |
| Nunca preto ou branco absolutos como superfície | Neutros com croma 0,002 a 0,012 |
| OKLCH; acento como sinal, menos de 5% | Postura contida |
| Sem rótulo em caixa alta espaçada, sem "eyebrow", sem ponto do meio | Cabeçalhos de grupo em frase normal; "Álbum, Maré Alta Trio" com vírgula |
| Sem emoji como ícone, sem cartão dentro de cartão | Ícones SVG próprios; ladrilho da Biblioteca sem moldura |
| Oito estados por controle; foco instantâneo | `Pressable` é a base de todo controle |
| Só `transform` e `opacity` em transição; nada de ressalto em estado de UI | Curva de chegada sem ressalto |
| Quatro estados por região de dados | Telas de carregando, vazio, erro e sem conexão |
| Nenhum número, depoimento ou logo inventado | Nenhum |
| Imagens: a coisa real, grande; nunca ilustração desenhada | As capas vetoriais da v3 saíram; entraram fotografias |
| Sucesso silencioso; desfazer em vez de confirmar | Avisos só para efeito que não se vê |

## 2. design-taste-frontend (tasteskill)

### 2.1 O que foi lido

`SKILL.md` inteiro; `references/copy.md`, `copy-voice.md`, `system.md`, `gate.md`. Não lidos por inteiro: `layout-v4.md`, `sections.md`, `sections-library.md`, `motion.md`, `cards.md`, `photos.md`, `moments.md`, `effects-*.md`, `scenes.md`, `backgrounds.md`. A skill pede a leitura deles antes da primeira página; eles descrevem heróis, seções de uma tela, momentos e rodapés de sites de empresa, que não existem numa tela de app. Fica registrado como passo **não cumprido**.

### 2.2 Passos da corrida

| Passo | Feito? | Observação |
|---|---|---|
| 0. Pilha | Não usei o `starter.html` nem o kit `assets/` | O kit monta páginas de empresa (nav fixa opaca, hero, seções, rodapé). Uma tela de app não cabe nele |
| 1. Tipo | `software` (metas `page-type` e `site-kind` nas páginas) | |
| 1. Armar o portão | `gate.mjs --start home.html --kind software --category "app de música" --primary "Tocar" --rounds 1` | Uma rodada só, porque as reprovações são estruturais |
| 2. Texto primeiro | Sim: `ux/05-estados-e-microcopy.md` antes das telas | |
| 3. Planejar em telas | Sim, mas como telas de app (`ux/03`, `ux/04`), não seções de uma tela |  |
| 4. Sortear (herói, paleta, momentos) | **Não rodei** `deal.mjs`, `start.mjs deal`, `moments.mjs` | A paleta e a forma são fixadas pelo autor (Apple-like, neutro, vidro); sortear contrariaria o brief |
| 5. Construir com o kit | Não | Idem passo 0 |
| 6. Checar | `copy-lint.mjs`, `consistency.mjs`, `perf.mjs`, `assets/layout/check.mjs`, `gate.mjs` | 2.3 |
| 7. Relatório | Este arquivo | |

Voz (cartão): direta e doméstica, em pt-BR do dia a dia; "você"; verbo no botão; sem exclamação; honesta sobre o que não existe ("Em breve").

### 2.3 Verificadores

**`copy-lint.mjs *.html --kind software`** (`skills-logs/taste-copy-lint.txt`): 22 páginas, **0 FAIL**, 1 WARN. O aviso é `copy-stat-strip` em `artist.html`: o verificador leu as durações das faixas e os anos dos álbuns como uma "faixa de números". Mantido: são dados de lista, não indicadores inventados.

**`consistency.mjs --quick --widths 1440,390`** em Início, Player e Biblioteca (`skills-logs/taste-consistency-*.txt`): 52, 17 e 35 FAIL. Todos os tipos de reprovação:

| Verificações | O que cobram | Por que não se aplicam a uma tela de app |
|---|---|---|
| `hero-screen`, `hero-fill`, `cta`, `copy-headline`, `copy-fact`, `copy-headings` | Um herói que ocupa a primeira tela, com título de categoria, um fato e um botão | Não há herói: o Início é uma lista de prateleiras |
| `section-height`, `one-section`, `one-heading`, `section-air`, `flush`, `no-visual` | Uma seção por tela, 128 px de ar entre seções, um visual por seção | Um app é denso por desenho |
| `container-edges`, `container`, `row-lines`, `card-layout` | Um contêiner `.wrap`, nenhuma fileira que rola de lado, no máximo dois cartões iguais | Prateleiras horizontais e cartões repetidos são a convenção de app de música |
| `nav`, `moment-none` | Barra de navegação no topo; um "momento" de site premiado | A navegação é embaixo por decisão do autor |
| `photo-upscale` | Imagem menor que a área em que aparece | É o fundo do player, pequeno e desfocado de propósito |

O que o mesmo verificador **aprovou** e vale para o app: `contrast`, `text-contrast`, `tap-size`, `a11y-name`, `img-alt`, `heading-order`, `focus-ring`, `focus-visible`, `lang`, `overlap-text`, `collide`, `spill`, `body-size`, `button-styles`, `button-shape`, `nesting`, `tilt`.

O que ele achou de **real** e foi corrigido:

| Achado | Correção |
|---|---|
| `collide` / `overlap-text`: o apoio dos mixes ("Serra Mansa, Nina Pomar e mais") avançava sobre o ladrilho vizinho | O ladrilho ganhou coluna `minmax(0, 1fr)` e os textos foram encurtados |
| `off-viewport` em 320 px: o console do player passava 24 px da tela | Recuo lateral de 16 abaixo de 368 px (no CSS e no Slint) |
| `button-contrast`: item ativo da barra lateral a 1,18:1 do fundo | Passou a usar a bolha opaca |
| `focus-hidden`: controle focado escondido sob a doca | `scroll-padding-bottom` nos protótipos; no Slint fica como pendência (o `Flickable` não rola até o foco) |

**`perf.mjs`** (`skills-logs/taste-perf-*.txt`): Início 5 FAIL, Player 2 FAIL.

| Linha | Situação |
|---|---|
| `font bytes` 1,2 MB | Mantido: são os TTF estáticos que o Slint precisa; o protótipo não é um site publicado |
| `oversized images` nos ladrilhos | Corrigido: capas de 344 px para ladrilhos |
| `oversized images` nos cartões grandes (640 para 264) | Mantido: é o tamanho certo em telas 2x |
| `below-fold images not lazy` | Mantido: com `loading="lazy"` as capturas automáticas saíam com imagens pela metade |
| Tarefas longas, bloqueio, deslocamento de layout, WebGL | PASS |

**`assets/layout/check.mjs`**: terminou com erro (`TypeError: r.sections is not iterable`): ele espera as seções do kit. Não aplicável.

**`gate.mjs home.html`** (`skills-logs/taste-gate.txt`): `GATE: OPEN 34 (rounds used up)`. Os 34 abertos: `container-edges` 6, `row-lines` 5, `one-section` 3, `card-layout` 3, `section-air` 2, `one-heading` 2, `hero-screen` 2, `cta` 2, e um de cada: `section-height`, `no-visual`, `moment-none`, `hero-fill`, `flush`, `copy-headline`, `copy-headings`, `copy-fact`, `container`. Nenhuma dispensa foi escrita em `waivers.json` (o limite é 5 e são regras de outra espécie de página): ficam abertos e declarados aqui. **O portão do tasteskill não passa, e não vai passar, numa tela de app.**

### 2.4 Princípios do tasteskill levados para o design

| Regra da casa | No design |
|---|---|
| Fotografias reais onde cabem | As capas de demonstração são fotografias |
| Interface 90 a 95% neutra; a cor vem do conteúdo; um acento | É a ideia da v4 |
| Nunca violeta | Nenhum tom entre 270° e 330° |
| Margens iguais dos dois lados | `gutter` único |
| Um sistema: um par de botões, uma escala de raio | `Button` principal e secundário; raios concêntricos |
| Micro-interações discretas; nada inclina | Pressionado por escala ou opacidade |
| Texto: dizer o que é; botão = verbo e objeto; nada inventado | "Criar playlist", "Adicionar pasta", "Tentar de novo" |
| Sem legenda sob imagem | **Não seguido**: título e artista sob a capa são o rótulo do item num app de música |
| Uma seção por tela; herói; momentos | Não se aplicam |

## 3. Revisão em contexto limpo

Quem: agente Traycer filho "Revisor de contexto limpo (Kazzu v4)", Claude Sonnet 5.5, effort high, criado com `traycer_create_agent`. Recebeu só: o brief do autor, a linha da ideia, 26 capturas do Slint, 10 dos protótipos em tablet e desktop, 5 capturas das versões reprovadas e a instrução de seguir `review-finish.md` (ramos custom e app). Não leu `design/` nem o código.

Primeira palavra: **fix**. "O vidro dos cartões grandes funciona. Mas o vidro que o autor mais vai tocar (barra de baixo, mini-player, sheets) parece cartão preto no escuro. Isso fica a um passo de rebuild."

| # | Achado | O que mudou | Nota (minha, contra a recaptura) |
|---|---|---|---|
| 1 | Vidro escuro lê como cartão preto: barra, mini-player, folhas, cabeçalho | Tinta do cromo e das folhas trocada de quase preto para um cinza fosco mais claro que o fundo; fio de luz de 30% para 46%; véu das folhas de 42% para 30%; **mini-player tingido pela capa** | **Parcial.** Mini-player e folhas: resolvido. A cápsula sobre uma lista preta continua com pouca cor atrás dela (transmissão medida 0,11): é o teto físico do contraste AA com texto branco |
| 2 | O console do player lê como degradê tingido; a capa é cortada; nada reconhecível atravessa | Player refeito: capa **inteira**; reflexo (capa espelhada) com desfoque leve; desfoque do console de 40 para 24; sobreposição de 64 para 24 | **Parcial.** Capa inteira e reflexo reconhecível: resolvido (ver `player-white`, `player-delta`). Numa capa de uma cor só (Pimenta) o console continua um campo dessa cor, porque é o que a capa é |
| 3 | Espaço morto e rótulos fracos no console | Saída e Fila viraram dois botões de 48 com caixa; "Letra" saiu do player e foi para o menu com "Em breve"; entraram o tempo que falta e a linha de origem e formato | Resolvido |
| 4 | Logo só no Início | Logo no canto superior esquerdo de todas as abas; marca à esquerda no cabeçalho recolhido | Resolvido |
| 5 | Ladrilhos de gênero coloridos e vazios | Ladrilhos neutros; os lançamentos de Explorar viraram cartões grandes com placa de vidro | Resolvido |
| 6 | Estados: código cru no erro, faixa sem conexão apertada, engrenagem some ao carregar, texto ambíguo no primeiro uso | Código fora da tela; faixa com título e apoio; esqueleto não empurra mais o cabeçalho; "Pronto para tocar" | Resolvido |
| 7 | Costuras: divisórias passando da margem, rótulos desalinhados em Ajustes, placa do álbum fora da margem, texto no vão da doca, 17% de cromo | Divisórias, Ajustes e placa corrigidos | **Parcial.** O vão entre mini-player e cápsula e os 17% de cromo ficaram: é a barra que o mapa mental pede |
| 8 | Tablet e desktop não resolvem a ideia | Protótipo largo: capa inteira ao lado do console, fundo escurecido | **Parcial.** A faixa larga não foi construída no Slint |

Também acatado dos textos: "Descubra" no lugar de "Descobre"; "Para ouvir agora" e "Faixas para você" no lugar dos nomes do YouTube Music; "Este aparelho" para casar com "Neste aparelho"; "Você pode desligar isso em Ajustes"; interruptor com polegar claro; ícone da Biblioteca redesenhado (estantes, não camadas).

Não acatado, e por quê: tirar o ladrilho "Baixados" (o autor pede os quatro); juntar mini-player e barra (o autor pede a barra do Bitchord); mudar a barra com busca redonda por parecer com a do iOS 26 (é a decisão do mapa mental; fica como alerta para o autor).

Segundo a skill, o revisor só recebe as recapturas de volta quando a primeira palavra é `rebuild` ou quando cita um gate de Floor. Não foi o caso: não houve segunda rodada, e as notas acima são minhas.

## 4. O que as skills mudaram de fato

1. As capas vetoriais deram lugar a fotografias (hallmark `imagery.md`, tasteskill regra 6). É a mudança que faz o vidro existir.
2. Neutros com croma e tokens em OKLCH; nenhum literal de cor fora de `tokens.json`.
3. Nenhuma caixa alta espaçada, nenhum ponto do meio, nenhum pré-título, nenhum emoji.
4. O campo de busca com foco visível no próprio campo.
5. Quatro estados por região e textos de erro sem código.
6. As correções de sobreposição e de 320 px que só os verificadores viram.
7. O lote inteiro da seção 3, que veio da revisão em contexto limpo exigida pelo hallmark.

## 5. Limites desta aplicação

- O portão do tasteskill ficou **aberto** (34 itens de estrutura de site). Um site de apresentação do Kazzu seria o lugar certo para rodá-lo por inteiro.
- Os sorteios do tasteskill e parte das leituras dele não foram feitos (2.1 e 2.2).
- O contraste do modo escuro, que a camada de render do hallmark não vê, foi coberto por `tools/check_render.py`, não pelo `sloplint`.
- A revisão teve uma rodada só.
