# Notas da UI (`kazzu-app`)

Etapa 1. Estas notas registram **decisões**, **desvios do design** (com o porquê), **lacunas de API** dos outros crates e **pendências**. O design continua sendo `design/` (fonte visual da verdade); tudo o que mudou na cópia dos arquivos do design está listado abaixo.

## Como rodar e verificar

```
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo      # app real com música de demonstração
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo --null-audio   # sem placa de som
cargo run -p kazzu-app                                                                # biblioteca do usuário (estado vazio na primeira vez)
cargo run -p kazzu-app -- --help
```

- `--demo-library <dir>`: gera em `<dir>/music` 35 faixas (WAV, FLAC 16/44,1, 24/88,2 e 24/96; MP3 320 e Opus 160 se houver `ffmpeg`) com tags e capas fictícias, usa banco e configuração isolados em `<dir>/data` e `<dir>/config.toml`. **Nunca toca na biblioteca do autor.** Fica fora do git (`target/`).
- `--snapshot <dir>` (+ `--renderer femtovg|software`, `KAZZU_SNAPSHOT_TAG=<prefixo>`): percorre todas as telas em 1280×800, 900×700 e 390×844, escuro e claro, e grava PNGs (`gl-*` = FemtoVG com vidro; `sw-*` = software, sem blur). Precisa de um servidor X (`Xvfb :99 -screen 0 1600x1000x24 +extension GLX`) ou Wayland; o GL do Mesa (llvmpipe) basta.
- Testes: `cargo test -p kazzu-app` (lógica sem janela: `Controller` + `Store` reais, motor com `NullSink`, tela falsa) e `cargo clippy --workspace --all-targets -- -D warnings`.

## Renderer: FemtoVG por padrão, software como fallback

| Opção | Veredito |
|---|---|
| **FemtoVG** (`renderer-femtovg`) | **Escolhido.** Tem `backdrop-blur` real (verificado: sidebar, dock, mini, NowBar e placas dos cartões desfocam o conteúdo atrás), compila em ~1 min sem baixar nada, usa OpenGL (Mesa/llvmpipe funciona até sem GPU, o que permite captura em CI) |
| Skia (`renderer-skia`) | Fica atrás da feature `skia` (`cargo build -p kazzu-app --features skia`), **não testada**. Compila o Skia (binários grandes, download no build). Candidata para iOS (Metal) e se o desempenho do FemtoVG decepcionar no Android |
| Software (`renderer-software`) | Fallback: sem blur (o Rust escreve `Theme.blur-supported = false` e todo vidro vira painel sólido, sem `if` na UI). Escolhido por `--renderer software` ou automaticamente se o FemtoVG não puder ser selecionado |

Detecção: `main.rs` pede `winit + femtovg` ao `BackendSelector`; se falhar, seleciona o software e liga `blur-supported = false`. Falha ao **criar o contexto GL** (sem driver) só aparece em `show()`: hoje vira erro de saída (ver Pendências).

## Cópias dos arquivos do design: o que mudou

`ui/theme/tokens.slint` é cópia **idêntica** (gerado). `ui/theme/atoms.slint` e `ui/theme/app-contract.slint` foram alterados:

- **`KSlider` (atoms)**: `value` passou de `in-out` para `in`; o valor mostrado durante o arraste (e por 300 ms depois, até o motor confirmar a posição) é interno (`held`/`pending`). Motivo: com `in-out`, o primeiro toque do usuário **quebra a ligação** `value: AppState.progress` e a barra deixa de acompanhar a faixa. Também: `mouse-cursor: pointer`, cancelamento do ponteiro tratado, e anel de foco que envolve só o trilho (não a área de 44 px).
- **Contrato (`AppState`)**: acréscimos, nada removido. `loading`, `has-library`, `scanning`, `scan-progress`, `scan-label`, `folders`, `library-summary`, `version-label`, `remove-folder`, `rescan`; detalhe (`detail-open/-kind/-title/-subtitle/-empty-text/-tracks/-albums`, `close-detail`, `play-all`, `shuffle-all`) e o enum `DetailKind`; `moods` + `open-genre`; `play-next`, `enqueue` (menu "mais"); `player-error`, `player-loading`, `retry-track`; `toast-*`, `dismiss-toast`; `seek-relative`, `volume-step`; `queue-jump`, `create-playlist-named`, `clear-recent-searches`, `search-text`; `output-note` (texto da nota de qualidade); `now-art` (capa grande; `now.cover` é a miniatura); `accent-dark/-ink`, `accent-light/-ink`; `licenses-text`.
- **`FocusScope` dos átomos (IconButton, KSwitch)**: ganhou `focus-on-click: false`. **Bug do design**: um `FocusScope` irmão e *acima* de um `TouchArea` captura o pressionar do mouse (para focar) e o clique nunca chega ao `TouchArea`: nenhum botão de ícone funcionava com o mouse. Foi pego com um clique real (xdotool) e agora tem teste (`tests/ui_render.rs`). A correção também foi aplicada em `design/slint/atoms.slint` (agora idêntico à cópia do app; `tests/design_sync.rs` garante).
- **Foco global**: o `AppWindow` devolve o foco ao atalho global (Espaço/setas/Esc) quando a tela, o player, a fila ou o detalhe mudam. Sem isso, sair da Busca (cujo campo recebe o foco) deixava o teclado mudo.
- **Cor de destaque**: o Rust não escolhe o tema vigente. Escreve as duas variantes em `AppState.accent-*` e o `AppWindow` aplica a do tema atual (`apply-accent`), inclusive quando o sistema troca de claro para escuro no modo Auto (`Palette.color-scheme` do Slint).

## Desvios da SLINT_SPEC (e por quê)

| Spec | Implementado | Motivo |
|---|---|---|
| `FocusRing` com 2 px de distância | Anel de 2 px **por dentro** da borda do elemento (componentes novos); os átomos do design mantêm o anel externo | Um componente raiz não pode ler `parent`; por dentro também não é cortado por `clip` nem por layouts |
| Grip de reordenar na fila, `queue-move` | **Sem grip**; arrastar/Alt+↑↓ não existem | O motor não tem comando de mover (ver Lacunas). Preferi não mostrar um controle que não funciona. `Delete` na linha e o botão `×` removem |
| Tela "Humores" (Explorar) | **"Gêneros do seu acervo"**: tags de gênero da biblioteca, com a capa do primeiro álbum | Humores é curadoria editorial, que ainda não existe; gêneros são reais e honestos. Toque = detalhe com as faixas do gênero |
| Rótulo "Novidades dos seus provedores" | Mantido; chips YT Music/Spotify mostram o estado vazio "Provedor ainda não conectado" | Provedores só chegam na etapa 2 |
| `open-album`, `open-collection`, `open-playlist` | Abrem a **tela de detalhe** (nova; lista de faixas virtualizada, ou grade de álbuns), com "Tocar" e "Aleatório" | A spec não tinha tela de álbum/coleção; sem ela os cartões ficariam mortos. Reaproveita `TrackRow` |
| Coleção Baixados "214 músicas" | **"Em breve"** | A tabela `downloads` é só placeholder (modo offline na etapa 4); sem números inventados |
| Letra, Saída, "reconhecer som ambiente", conexão de provedores, equalizador, reprodução automática | Botões/linhas existem, mas dão aviso "chega em uma próxima versão" (toast) ou mostram a tag **Em breve**; **o interruptor do EQ e o de "reprodução automática" não aparecem** | Interruptores que não fazem nada seriam mentira. O bloco do EQ (10 faixas) está como o design pede, desabilitado |
| "Saída: Trocar" | Mostra o dispositivo atual + tag "Padrão do sistema" | Sem API para trocar o dispositivo em execução (ver Lacunas) |
| Título grande no desktop 40/46 | Literal `40px` em `PageTop`/Fila | Não há token (`large-title` é 34/40). Sugestão: criar `large-title-desktop` em `tokens.json` |
| Snap por cartão nos carrosséis | Rolagem livre + setas | `Flickable` não tem snap |
| Arrastar para baixo fecha o player (mobile) | Não implementado | Gesto opcional na spec, etapa posterior |
| Menu "mais" (Tocar em seguida, Adicionar à fila, Curtir) | Novo (`RowMenu`, `PopupWindow` sólido); o player usa o mesmo | O botão existe na spec, o menu não |
| Quarta ação "Adicionar à playlist" | **Não há**: é possível criar playlists e abri-las, mas não preenchê-las pela UI | Falta o seletor de playlist (próxima etapa de UI); a biblioteca já tem `add_to_playlist` |

Literais de cor/tamanho fora de `Theme` (além de §5.4; `grep '#[0-9a-f]\{6,8\}' ui/` fora de `theme/`): base `#0c0806` do `PlayerBackdrop` (evita piscar claro no tema claro), `#000000a0/#000000b0` dos véus de diálogo, `#ffffff1f` na borda da placa do destaque, `#ffffff` no hover do botão play, `#ffffff14/#00000014` no contorno da capa (já era do átomo), `40px` do título. Sugiro virar tokens.

## Verificação feita (o que foi de fato exercitado)

- **Visual**: 246 capturas reais (FemtoVG com blur, em Xvfb + Mesa llvmpipe; e software) em `target/ui-screenshots/` — `gl-*`, `sw-*`, `empty-gl-*` (primeira execução sem biblioteca). Tamanhos 1280×800, 900×700, 390×844 e 360×740; escuro e claro; Início, Explorar, Biblioteca, Busca (vazia e com resultado), Configurações (inteira), Player (capa laranja, clara e cinza), Fila, Detalhe, foco por teclado, "Reduzir transparência". Comparadas com `design/prototypes/*.html` renderizados no Chromium nos mesmos tamanhos.
- **Contraste do player** (pior caso do aviso do orquestrador): medido nas capturas reais, texto sobre o fundo ao redor, com as cores `im-*`: título ≥ 7,4:1, artista ≥ 6,6:1, tempos ≥ 7,4:1 (capas laranja, rosa-clara, cinza e a de teste), todos acima de 4,5:1. O selo de formato **não sobrepõe** os tempos em 390 nem em 360 px: abaixo de 340 px de coluna o selo troca `24-bit/96 kHz` por `24/96`.
- **Entrada real**: app rodando em Xvfb, clique do mouse numa faixa, Espaço (pausa/retoma), clique no ícone da NowBar, clique na barra de progresso (seek), Esc. Sem placa de som utilizável (ALSA "Host is down" neste ambiente) o erro vira toast em pt-BR e `player-error`, sem travar nada.
- **Testes** (`cargo test -p kazzu-app`): lógica (`Controller` + `Store` reais + motor com `NullSink` + tela falsa; 21 casos), `accent`/`art`/`format`/`config`/`platform` unitários, biblioteca de demonstração escaneável, e `tests/ui_render.rs` (todas as telas, dois tamanhos, dois temas, cliques e teclas, renderer de software). Não medido: desempenho em GPU real.

## Comportamentos decididos na revisão

- **Encerramento**: as closures da UI guardam só `Weak` do controller; `App::run` chama `finish()` ao fechar a janela (WM_DELETE_WINDOW verificado no Xvfb): grava `config.toml`, para o motor e solta o controller (as threads encerram). O volume é gravado 1 s depois do último ajuste e no encerramento.
- **Configuração corrompida**: o arquivo vai para `config.toml.bak` (substitui um `.bak` antigo), o app abre com o padrão e mostra um aviso. Valores impossíveis (`volume = nan`, 7) são corrigidos na leitura; NaN/infinito nunca passam por seek, volume ou eventos.
- **Scan**: um pedido de scan durante outro é registrado e roda de novo assim que o atual termina (adicionar/remover pasta no meio sempre acaba indexado). O scan grava em lotes por pasta (≥ 100 faixas, transações `IMMEDIATE` curtas), relê as pastas dentro de cada transação (pasta removida não tem faixas reinseridas) e o SQLite espera até 30 s por escrita concorrente (curtir/histórico durante o scan não falham). Testes em `kazzu-library/tests/library.rs` e `kazzu-app/tests/worker.rs`.
- **Fila na UI**: mostra as próximas 100 faixas e "e mais N músicas"; view-models da fila e miniaturas ficam em caches LRU limitados (600 e 400).
- **Embaralhar**: tocar uma lista nova **mantém** o embaralhar que o usuário ligou (o motor reembaralha no `Load`); só "Tudo em ordem aleatória"/"Aleatório" ligam o modo.
- **Busca**: debounce de 200 ms só para buscas; os demais jobs (tocar, capa, curtir) não esperam. Digitação validada: no Xvfb (`xdotool type` com a janela em foco; sem foco do X as teclas nem chegam ao app) e por teste (`tests/ui_render.rs` digita no campo e o callback `search` dispara).
- **Detalhe atrasado**: cada pedido tem número de sequência; trocar de aba ou fechar invalida os pendentes.

## Dados e contas que a UI faz (para o autor conferir)

- **Destaques** (cartões grandes), sem provedores: álbum mais novo ("Adicionado há pouco"), "Curtidos" (se houver) e "Tudo em ordem aleatória". Ids `album:<n>`, `liked`, `shuffle-all` passam por `play-track`.
- **Recentes** = histórico de reprodução (mais recente primeiro) completado com o que entrou há pouco, até 20. Uma escuta conta no histórico com ≥ 30 s ou ≥ metade da faixa; a lista atualiza na hora.
- **Álbuns que você ouve**: álbuns dos recentes primeiro, depois os demais (24).
- **Selo e "bit-perfect"**: o selo do player passa a vir do `OutputChanged.source` assim que o motor informa; antes usa as tags. A nota "Original, sem equalizador" só aparece com `bit_perfect = true`; com volume < 100% mostra "Volume ajustado: o sinal não está bit-perfect"; com reamostragem, "Reamostrado de 96 para 48 kHz". O motor chama WAV/AIFF de "PCM"; a UI mostra o nome do arquivo (WAV).
- **Acento**: algoritmo de `DESIGN.md` §3.4 portado para Rust (`accent.rs`), com testes de faixa de luminosidade e contraste ≥ 4,5:1 nos dois temas para 5 capas-tipo. Capa cinza/preta/branca cai no âmbar da marca.
- **Fundo do player**: receita §3.5 (`art.rs`): 96×96, σ=4, saturação ×1,25, ×0,46, 1 LSB de ruído. Testado: mesmo com capa branca o brilho máximo fica ≈ 117/255.
- **Busca**: FTS5 da biblioteca (50 resultados), debounce de 200 ms **na thread de trabalho**, resultados atrasados descartados por número de sequência. "Artistas" e "Playlists" mostram "Em breve" (a UI não tem onde listar artistas ainda). Buscas recentes: só neste aparelho, gravadas quando o usuário toca/abre algo a partir da busca (não a cada letra).
- **Preferências** (`config.toml`): tema, destaque, reduzir transparência/movimento, volume, buscas recentes. Pastas de música ficam **só na biblioteca (SQLite)**, uma fonte de verdade. "Reduzir movimento" começa pela preferência do sistema (Linux/GNOME: `enable-animations` via `gsettings`, lida fora da thread de UI); "Reduzir transparência" não tem preferência de sistema no Linux, então é só a chave manual.
- **Privacidade**: nenhuma chamada de rede no app. O seletor de pastas usa o portal do XDG (D-Bus local).

## Lacunas de API nos outros crates (pedidos)

| Crate | Pedido | Por quê |
|---|---|---|
| `kazzu-audio` | `PlaybackCommand::Move { id, to }` (reordenar a fila) | Reordenar por arrastar (spec §4.5) |
| `kazzu-audio` | Trocar/listar dispositivo em execução (`set_device`, `OutputInfo.device` com nome legível, `CpalSink::devices()` acessível pelo `EngineHandle`) | "Saída: Trocar" e o seletor do player |
| `kazzu-audio` | `OutputInfo.device` para o `default` devolve `"default"`; a UI traduz para "Padrão do sistema". Seria melhor o nome real do dispositivo | Texto do player |
| `kazzu-audio` | Evento de fim de fila deixa `QueueFinished` mas a faixa atual continua em `TrackChanged`; documentar a semântica | A UI mantém a última faixa no mini player |
| `kazzu-library` | Consulta de gêneros (`genres() -> Vec<(String, count)>`); hoje a UI lê todas as faixas para agrupar | Explorar |
| `kazzu-library` | `playlist_covers(id, n)` ou `Playlist.covers` | Mosaico das playlists (hoje 1 consulta por playlist) |
| `kazzu-library` | `recent()` só devolve o histórico; a UI completa com `recently_added`. Um `home_recents(limit)` único evitaria duas consultas | Início |
| `kazzu-core` | `RepeatMode` do core e do Slint têm ordem diferente (`Off, One, All` × `off, all, one`); a UI mapeia | Evitar confusão |

## Desempenho: o que medir

- **`Image::from_rgba8` não entra no cache de texturas do FemtoVG** (`ImageCacheKey::Invalid`): toda capa visível é reenviada à GPU a cada quadro desenhado. Mitiguei com miniaturas de ≤ 160 px nas listas, capa de ≤ 640 px só no player e miniatura de 192 px no mini/NowBar. Lista de 20 capas de 160 px ≈ 2 MB/quadro. **Falta medir** em GPU integrada fraca e no Android. Plano B: `Image::load_from_path` (cache por caminho, mas lê/decodifica na thread de UI) ou pré-aquecer com um `Image` por capa guardado no modelo.
- Vidro: 6 painéis com blur no máximo por tela (dock + mini + 4 cartões na Biblioteca). A meta da spec (≥ 55 fps ao rolar em iGPU fraca) **não foi medida aqui** (só llvmpipe, que é software).
- Listas longas (Fila, detalhe de faixas) usam `ListView` (virtualizado). Grades de álbuns/playlists e o carrossel instanciam todos os itens (ok até algumas centenas).
- A thread de UI nunca faz I/O: capas, SQLite, scan e decodificação rodam em `kazzu-library`/`kazzu-scan`; o seletor de pasta roda em thread própria; eventos do motor chegam por uma thread que agrupa `Position` (10 Hz) antes de acordar a UI.

## Acessibilidade e teclado

- Todo controle tem `accessible-role` e rótulo em pt-BR; sliders expõem `accessible-value` ("1:02 de 3:21"); selo de formato é `text` com rótulo "FLAC 24/96"; indicador "tocando" tem rótulo.
- Atalhos (janela em foco, fora de campos de texto): **Espaço** play/pause, **←/→** ±5 s, **↑/↓** volume ±5%, **Esc** fecha player → detalhe → fila. Tab percorre os controles; Enter/Espaço ativam; anel de foco de 2 px sem animação. Com um slider em foco, ←/→ movem ±5% do próprio slider (o slider aceita o evento antes do atalho global).
- **Reduzir movimento** zera as durações (`Theme.dur-*`). Reduzir transparência troca o vidro por sólido (`Theme.glass-live`).

## Pendências conhecidas

1. Seletor "Adicionar à playlist" e edição de playlists (renomear/apagar) na UI.
2. Reordenar a fila (depende do motor); seletor de dispositivo de saída; letras; EQ (etapa 3).
3. Arrastar para baixo para fechar o player e gesto de trocar faixa na capa (mobile).
4. (resolvido) Se o FemtoVG não puder criar o contexto GL, `App::build` descobre isso mostrando a janela **antes** de subir motor, worker e banco e o `main` reabre em modo software.
5. Mensagem de erro e "Tentar de novo" do player testados por unidade (controller) e por captura; falta provar com um arquivo corrompido em placa real.
6. Sem testes de leitor de tela (Orca/NVDA). A árvore de acessibilidade existe (AccessKit), não foi exercitada.
7. Barra de rolagem: desligada (`always-off`) nas listas virtualizadas, no estilo do design. Em desktop sem roda/trackpad pode ser preciso um indicador.
8. `Theme.window-w` é a largura da **janela**; telas no desktop usam a largura da área de conteúdo para gutter e colunas (ok), mas `Theme.compact/wide` ignoram a largura da sidebar.
