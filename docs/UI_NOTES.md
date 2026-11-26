# Notas da UI (`kazzu-app`): v4 (vidro sobre capas) sobre a base v2 (mobile-first, YouTube Music padrão)

> **v4 em vigor.** A UI foi refeita a partir de `design/v4/` (ver a seção "v4: o que mudou" no fim). O que estas notas dizem do visual v2 (palco colorido, acento da capa, chips do Início, `design/` v2/v3) está superado; a lógica (controller, YouTube Music, fila, biblioteca) continua valendo.

Estas notas registram **decisões**, **desvios do design** (com o porquê), **lacunas de API** e **pendências**. O design continua sendo `design/` (fonte visual da verdade). A v1 foi reprovada pelo autor ("não era mobile-first, UX/UI ruim, não ficou glassmorphism"); a v2 refaz a UI e liga o YouTube Music como fonte padrão, sem login. As decisões específicas do Slint (limites da linguagem, vidro, medidas) estão na seção "v2: decisões e limites do Slint", no fim.

## Como rodar e verificar

```
cargo run -p kazzu-app --release -- --demo-library target/fixtures/kazzu-demo      # app real, YouTube Music + música de demonstração
cargo run -p kazzu-app --release -- --no-youtube                                    # só arquivos locais, nenhuma chamada de rede
cargo run -p kazzu-app -- --help
cargo run -p kazzu-ui-gallery -- --shot target/ui-screenshots                       # galeria: dados falsos, todos os estados (Xvfb)
cargo run -p kazzu-ui-gallery -- --glass-proof                                      # prova por pixels de que o vidro desfoca
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
cargo test -p kazzu-app --test live_ytm -- --ignored --nocapture                   # AO VIVO: Início, busca, resolve, motor (rede real)
```

- `--demo-library <dir>`: gera em `<dir>/music` 35 faixas (WAV, FLAC, MP3/Opus com `ffmpeg`) com tags e capas fictícias; banco e configuração isolados em `<dir>/data` e `<dir>/config.toml`. Nunca toca na biblioteca do autor.
- `--snapshot <dir>` (+ `--renderer femtovg|software`, `KAZZU_SNAPSHOT_TAG=<prefixo>`): percorre as telas do app **real** (com o YouTube Music ao vivo) em 390×844, 360×740, 768×1024 e 1280×800, claro e escuro, e grava PNGs. Precisa de um servidor X (`Xvfb :99 -screen 0 1600x1200x24 +extension GLX`).
- Testes: `controller.rs` (lógica sem janela, motor com `NullSink`, **provedor falso sem rede**), `worker.rs`, `ui_render.rs` (todas as telas em 4 tamanhos × 2 temas, clique real, digitação, Voltar), `design_sync.rs`, `live_ytm.rs` (ignorado, rede real).

## Arquitetura v2 (YouTube Music e fila)

```
UI (Slint) ── callbacks ──> Controller (thread de UI, nunca bloqueia) ── Job ──> ThreadedRunner
                                  ▲                                               ├─ thread "kazzu-library": Store (SQLite, capas locais, scan)
                                  └──── Msg::Job / Engine / Stream ───────────────┴─ thread "kazzu-remote": despacha RemoteJob (1 thread por job, debounce 250 ms da busca)
```

- **`catalog.rs`**: trait `Catalog` (síncrono, `Send + Sync`) com o que a UI precisa do provedor: feed do Início, Explorar, busca, sugestões, álbum/artista/playlist, rádio, `resolve` (stream + selo de qualidade), `invalidate`, curtidas e álbuns guardados no aparelho. `Providers` é o registro (o primeiro é o padrão); `YtmCatalog` é o YouTube Music (`kazzu-ytm`), ativo por padrão, sem login. Nos testes entra um `FakeCatalog`. O `kazzu_core::Provider` não tem Início, rádio nem artista, por isso o `Catalog` existe (ver "Lacunas").
- **`remote.rs`**: `RemoteWorker::run(RemoteJob)` faz a rede e devolve `RemoteResult`s. O Início manda primeiro o cache em disco (esqueleto → conteúdo sem esperar a rede) e depois o feed novo.
- **`ytm_vm.rs`**: modelos do `kazzu-ytm` → view-models (funções puras e testadas). Ids remotos na UI levam o prefixo `yt:`.
- **`covers.rs`**: capas remotas só quando a tela as mostra, só `https` de hosts de imagem do YouTube/Google, cache em disco (FNV-1a estável), decodificação e redução fora da thread de UI.
- **Fila sob demanda (`controller/remote.rs`)**: resolver o stream de cada faixa de uma playlist de 200 músicas seria rede demais. O motor recebe só a faixa pedida e **2 à frente**; o resto fica no contexto e entra conforme a fila encolhe. Acabou a lista e "Continuar com músicas parecidas" está ligado: pede o rádio da última faixa (`radio` + `radio_continuation`) e segue sem repetir. Embaralhar e "repetir tudo" de uma fila remota são do contexto (o motor só repete uma faixa). Listas só locais continuam inteiras no motor.
- **URL vencida e rede** (`StreamEvent`, thread "kazzu-stream-events"): `Expired` → `invalidate` + `resolve` + `engine.replace_stream` (retoma na posição); `NetworkFailed` → aviso "Sem conexão, toque para tentar" com **Tentar de novo** (`engine.play()` retoma). Falha de rede ao resolver a *próxima* não martela o servidor: avisa, espera o usuário. Faixa indisponível (região, removida) é pulada com aviso calmo.
- **Erros** (`ErrorKind` → `ytm_vm::error_copy`): texto calmo sem culpa, ação só quando faz sentido (`is_retryable`), código técnico pequeno em fonte mono (`ytm:network`). Erro com conteúdo em cache mantém a tela e mostra a faixa de aviso (`home-stale`).
- **Online/offline**: a UI mostra a barra "Sem conexão" depois de uma falha de rede real e volta sozinha (toast "Conexão de volta" e recarrega o Início). Não há sondagem de rede em segundo plano.
- **Curtidas e playlists**: locais. Curtida remota fica no `ytm-library.json` do `kazzu-ytm`; playlists são do `kazzu-library`, que ganhou a migração v2 (faixas remotas por referência, `playlist_entries`, `add_remote_to_playlist`). Detalhe e tocar de uma playlist mista funcionam.
- **Desfazer**: curtir e "Tocar em seguida/Adicionar à fila" têm **Desfazer** no aviso. "Adicionar à playlist" ainda não tem.
- **Quadros lentos** (`app.rs::watch_frames`): mede o tempo de desenho de cada quadro (local); se por 2 s ≥ 70% dos quadros passam de 16 ms com o vidro ligado, a UI **oferece** "Reduzir transparência" (uma vez; não impõe).
- **Revisão independente (correções)**: (1) o motor parar no fim da fila enquanto a próxima ainda resolve vira "carregando" e **retoma sozinho** quando ela chega (`Context::resume`); (2) fila parada por falha de rede recua (2 s, 4 s … 30 s) e **se recupera sozinha** (quando a conexão volta, na troca de faixa ou a cada `Position` depois do recuo); o aviso de "Conexão de volta" não apaga mais o "Tentar de novo"; (3) URL vencida: no máximo 2 renovações automáticas por faixa em 60 s, depois "Tentar de novo" por ação do usuário; (4) pular para uma faixa ainda não entregue incrementa a geração do contexto e descarta resolução velha; (5) capas pedidas e ausentes (falha de rede ou descartadas do cache de 500) são pedidas de novo depois de 30 s, e só a tela à vista é refeita quando chegam capas; (8) erro de rede no rádio não o esgota; (9)(10) início indisponível segue "carregando" e faixas indisponíveis saem de "repetir tudo"; (11)(12) as ações da Fila recebem o **id da linha** (não o índice) e "mover para cima" também funciona para a primeira linha que ainda não entrou no motor; (15) `Lru` O(1); (16) rede num **pool de 5 trabalhadores** que descarta job de geração velha (busca/Início/detalhe substituídos); (17) o player diz "Original, sem equalizador" só para arquivo local; para o YouTube Music diz "Qualidade do YouTube Music, sem equalizador".
- **Segurança das capas**: `ureq` sem redirecionamentos e só `https` (a lista de hosts só vale para a URL inicial; o mesmo em `kazzu-ytm/transport.rs`), só `Content-Type: image/*`, decodificação com limites (4096×4096 e 64 MiB), só entra no cache o que decodifica, e o diretório `cache/covers` tem teto de 128 MiB (apaga as mais antigas).
- **Banco**: o banco da biblioteca agora é **v2** (playlists com faixas remotas). **Não abre em app antigo** (`SchemaTooNew`): downgrade impossível; faça backup antes de atualizar (ver `docs/library.md`).
- **Cache de streams**: faixas ouvidas até o fim ficam em disco (`cache/streams`, 256 MiB, LRU); base do modo offline. Hoje o controller ainda não consulta `StreamCache::cached` para tocar sem `resolve` quando offline (lacuna).

## Rede e privacidade (AC-10)

O UX_PLAN pede "zero requisição ao abrir e ficar parado"; a decisão do autor de que o Início já funciona no primeiro uso exige **uma** rodada de rede na abertura: cache em disco do Início + `discover()` (home, lançamentos e paradas do país) + as capas das telas vistas. Nada mais acontece parado: sem busca, rádio, resolução de stream ou "prefetch" até o usuário pedir. `--no-youtube` desliga tudo. Sem telemetria, sem identificador do usuário (ver `docs/research/ytm-notes.md`). O idioma vem do sistema (Android: en-US no emulador, portanto o catálogo vem em inglês; a UI do Kazzu é pt-BR).

## Android

- O backend Slint Android usa **só Skia** (as features `renderer-femtovg`/`renderer-software` do `Cargo.toml` são inertes lá) e já tem `backdrop-blur`. Preenche `Window.safe-area-insets`, que o `AppWindow` copia para `AppState.safe-*`: nada de JNI nosso. O Voltar chega como `Key.Back` no `FocusScope` raiz: `go-back()` aceita só se algo foi fechado, senão o sistema sai (verificado no emulador: folha → detalhe → Início → sai).
- **Capas em branco (causa)**: com a permissão só de **áudio** (`READ_MEDIA_AUDIO`) o Android não deixa ler `cover.jpg` da pasta (arquivo de imagem); capas **embutidas** na tag aparecem normalmente (verificado: o mesmo álbum com capa embutida mostra a imagem). Capa ausente é caso normal: placeholder `disc`. Pedir `READ_MEDIA_IMAGES` resolveria, ao custo de uma permissão de "fotos" (decisão do autor, ver abaixo).
- Verificação feita: ver `docs/ANDROID.md`.

## Decisões para o autor

1. **Permissão de imagens no Android** (`READ_MEDIA_IMAGES`) só para ler `cover.jpg` de pastas: hoje **não** pedimos (privacidade); álbuns sem capa embutida ficam com o placeholder.
2. **Rede na abertura** (item acima): confirma que o Início carregar sozinho é o desejado.
3. **Cache de streams de 256 MiB** em disco (offline futuro) ligado por padrão.
4. **Idioma do catálogo** segue o sistema; no emulador/aparelhos em inglês o YouTube devolve títulos de seção em inglês.
5. `Provider` do `kazzu-core` é pequeno demais para a UI; sugiro promover o `Catalog` (ou partes) ao core antes de entrar Spotify/Tidal.

## Lacunas de API nos outros crates

| Crate | Pedido | Por quê |
|---|---|---|
| `kazzu-core` | Estender `Provider` (Início/Explorar, artista, rádio, curtidas) ou adotar o `Catalog` | Hoje a UI usa um trait próprio no app |
| `kazzu-audio` | `PlaybackCommand::Move { id, to }` | A seta "Mover para cima" da fila emula (remove e devolve); o gesto de arrastar depende disso |
| `kazzu-audio` | Trocar/listar dispositivo; `OutputInfo.device` com nome legível | "Saída" mostra só o padrão ("Este celular" no Android) |
| `kazzu-audio` | `StreamCache::cached(&track)` usado pelo controller | Tocar offline sem `resolve` |
| `kazzu-library` | `genres()`, `playlist_covers(id, n)`, `home_recents(limit)` | Evitar consultas extras (já pedidos na v1) |
| `kazzu-ytm` | Indicar que o resultado veio do cache (stale) | Hoje o Início deduz `stale` pelo erro depois de mostrar o cache |

## Pendências conhecidas

1. Letra, Timer de sono, Saída, Baixar, equalizador: botões presentes com aviso "chega em uma próxima versão" (honesto; nenhum interruptor falso). **Reconhecer pelo som**: botão presente, "Em breve".
2. Reordenar a fila por arrastar; "Adicionar à playlist" com Desfazer; renomear/apagar playlists.
3. Foreground service/notificação de mídia no Android (a música para com a tela apagada); seletor de pastas (SAF).
4. Teste com TalkBack e aparelho físico; medir 60 fps em GPU/Android de entrada.

## v2: decisões e limites do Slint

A UI foi reescrita mobile-first (SLINT_SPEC v2). Arquivos: `ui/app.slint` (uma árvore; `Theme.compact` é a base, `if !Theme.compact` só acrescenta), `ui/components/{basics,media,nav,sheets,layout}.slint`, `ui/screens/{home,explore,library,search,detail,settings,player}.slint`, `ui/theme/{metrics,nav-state}.slint` (novos). `tokens.slint` não foi tocado. `atoms.slint` ganhou duas correções (copiadas também para `design/slint/atoms.slint`, o teste `design_sync` continua válido): `Chip` agora tem `FocusScope` + anel de foco, e `FormatBadge` não soma 16 px de margem em cima do `padding` que já tem.

### Como verificar (galeria)

```
cargo run -p kazzu-ui-gallery -- --interactive [--state player-orange] [--size 390x844] [--light] [--rt] [--safe]
Xvfb :99 -screen 0 1600x1200x24 +extension GLX &  export DISPLAY=:99
cargo run -p kazzu-ui-gallery -- --shot target/ui-screenshots [--quick] [--only 390x844-dark]   # matriz de capturas
cargo run -p kazzu-ui-gallery -- --renderer software --shot target/ui-screenshots --only 390x844 # sem blur, sufixo -sw
cargo run -p kazzu-ui-gallery -- --glass-proof     # prova por pixels do vidro; sai ≠ 0 se falhar
cargo run -p kazzu-ui-gallery -- --selftest        # ponteiro/teclado reais: arrastar o player, Voltar, abas, rolagem por aba, área segura, alvos de 48 fora do visual
cargo run -p kazzu-ui-gallery -- --hit-targets     # mede a ÁREA DE TOQUE (≥ 48×48) de todo controle em todas as cenas; sem Xvfb; sai ≠ 0 se falhar
crates/kazzu-ui-gallery/scripts/compare.sh         # folhas comparativas protótipo × Slint (Chromium + ImageMagick)
```

A galeria compila o `ui/app.slint` real e só escreve em `AppState`/`Theme`, como o Rust do app faz. Nomes: `<tela>-<estado>-<LxA>-<tema>[-rt][-safe][-sw].png`.

### Decisões

| Tema | Decisão | Motivo |
|---|---|---|
| Player "sempre escuro" | Não há `Theme.dark` escopado (o global é único). O `PlayerScreen` usa paleta própria: variantes `stage-*-dark`, `glass-tint-dark`, `im-*` e o acento escuro (`AppState.accent-dark`), com `DarkStage`/`DarkGlass` locais | Forçar `Theme.dark = true` ao abrir faria o app inteiro por baixo piscar escuro no tema claro e brigaria com o Rust |
| Folhas sobre o player (Fila, Menu) | Seguem o tema vigente (no claro, vidro claro sobre o player escuro) | Os itens usam `Theme.ink`; trocar a paleta dentro do escopo exigiria duplicar todos os itens |
| Navegação | Global `Nav` (`ui/theme/nav-state.slint`): `go(tab)`, `open-player`, `close-player`, `close-detail`, `back()`. `AppWindow.go-back()` chama `Nav.back()`; Esc e `Key.Back` (Android) também | Um único lugar para a ordem do Voltar (menus, fila, player, detalhe, raiz → Início, Busca limpa o texto) |
| Rolagem por aba | `Metrics.scroll-*` guarda o `content-y` de cada aba; a tela restaura no `init`; tocar na aba ativa chama `Metrics.reset-scroll` | Telas são instanciadas por `if` (nada fica vivo fora da aba) |
| Áreas seguras | O `AppWindow` copia `Window.safe-area-insets.*` (struct `Edges` no Slint 1.19) para `AppState.safe-*` (`in-out`); a UI só lê `AppState`. A galeria força 48/24 | O backend Android já preenche `safe-area-insets` |
| Áreas seguras: esvaecimentos | Com `safe-top > 0` há um esvaecimento superior de `safe-top + 24` px (`stage-base` 70% → 0, sem TouchArea) acima das telas; o inferior vai até a borda da janela (`190 + safe-bottom`): 55% atrás da barra, ~95% na faixa dos botões do sistema | O conteúdo não pode aparecer legível sob a barra de status nem sob os 3 botões do Android; o vidro da barra continua vendo o conteúdo (a prova de vidro cobre isso) |
| Capa ausente | `Cover`, `Mosaic`, `HeroCard`, `MoodTile` detectam `source.width == 0` e mostram `Theme.surface-2` + ícone `disc` (mosaico: célula a célula; sem nenhuma capa, um disco só). Cenas `*-nocover` na galeria | Com a permissão só de áudio o Android não lê `cover.jpg`; só capas embutidas na tag |
| Teclado na folha Nova playlist | Com o campo em foco (compact) a folha sobe 40% da altura da janela (`SheetFrame.lift`); ≥ 600 não muda | Evitar o campo sob o teclado virtual |
| Palco | `Stage` do atoms atrás de tudo (decorativo, sem acessibilidade) + esvaecimento inferior com `Theme.stage-base` a 55% (mais forte escondia o conteúdo que o vidro deve desfocar) | Prova de vidro e SLINT_SPEC §8.2 |
| Skeleton | Só estáticos; **nenhum spinner** (o spinner "após 300 ms" não existe) | Sem animação infinita; o esqueleto cobre a espera |
| Faixa de formato | Aparece em larguras ≥ 380 px, como a spec; os dados curtos (`detail` = "160", "24/96") vêm do Rust | Cabe sem esmagar o título |
| Redução de movimento/transparência em Ajustes | `KSwitch checked <=> Theme.reduce-*` e o callback `reduce-*-changed` | Efeito imediato e o Rust persiste |
| Apagar histórico | Confirmação em duas etapas no próprio botão ("Apagar" → "Confirmar", volta sozinho em 4 s) | Sem diálogo extra |
| Timer de soneca | O botão chama `set-sleep-timer(0)`; o Rust decide o que fazer (alternar/abrir seletor) | O contrato só tem esse callback |

### Limites do Slint 1.19 encontrados

- **`ListView` aceita um único `for`** (nada de cabeçalho/rodapé que role junto). A Fila usa cabeçalho fixo ("Tocando agora", "A seguir") + `ListView` + nota fixa embaixo. O **Detalhe** usa `Flickable` + `for` (instancia todas as linhas; para coleções com milhares de faixas, limitar no Rust ou paginar).
- **Sem snap em `Flickable`**: os carrosséis rolam livres (setas ≥ 600 como alternativa, ver `HRail`). Sem layout "flow": "Para experimentar" é uma única linha rolável (o protótipo quebra em linhas).
- **`accessible-role`** não tem `dialog`/`heading`/status: folhas e a faixa da fila usam `groupbox`, títulos usam `text`; o Toast não é uma região "polida" (live region) de verdade, só `text` com o rótulo.
- **`Flickable` não rola até o foco**: a reserva `Metrics.chrome-reserve` garante que o último item não fique sob a barra, mas navegar por Tab não traz o item para a vista.
- **Borda em gradiente** (`border-color` com `@linear-gradient`) funciona no FemtoVG (conferido nas capturas); no software cai para sólido.
- **`drop-shadow` precisa de `background` ou borda** no próprio elemento (a capa do detalhe e do player ganharam um fundo para a sombra aparecer).
- `Window.safe-area-insets` é uma struct, e `PopupWindow` não foi usado (as folhas são elementos normais com `Scrim`).
- O campo de texto não tem `placeholder`: o texto de ajuda é um `Text` por cima, escondido quando há conteúdo.
- Textos de `Text` com `accessible-role: text` dentro de componentes que têm uma propriedade `text` precisam de `AccessibleRole.text` (o nome colide).

### Vidro

- Tinta e matiz vêm de `Theme.glass-tint`; o filho de um vidro usa `Theme.ink.with-alpha(0.12)` + borda `0.16` (chips, campos, botões). Os botões redondos do topo (Ajustes, Voltar) são fill simples (não há conteúdo rolando atrás: o topo rola junto com a página).
- Contagem de painéis com blur por tela (compact, com faixa tocando): Início 6 (barra, Buscar, mini + 2–3 placas dos destaques), Explorar até 9 (as 3 + placas de humor), Biblioteca 7 (as 3 + 4 cartões). A spec fala em 6; **não medi** o custo em GPU fraca. Se pesar, o caminho é "Reduzir transparência" (oferta `perf-offer`) ou tirar o blur das placas dos humores.
- `--glass-proof`: janela baixa (390×520) para a barra cair sobre a capa do destaque; compara a faixa de vidro da barra com a mesma região sem barra (janela alta), com o sólido (`blur-supported = false`) e com Reduzir transparência.

### Alvos de toque (≥ 48×48) e `--hit-targets`

- **O que mede**: `--hit-targets` usa o backend de testes do Slint (`i-slint-backend-testing` 1.19, dependência **só da galeria**; licença `GPL-3.0-only OR royalty-free`, a mesma do Slint, e o Kazzu é GPL-3.0). Ele não desenha, só calcula o layout (roda sem Xvfb, ~1 min em debug). Para cada cena (todas as de `scenes::SCENES`) × 5 tamanhos (390×844, 360×740, 320×640, 768×1024, 1280×800) **e** a mesma largura com altura 6000 (o Slint só visita os itens dentro da área visível das listas rolantes; a janela alta traz o resto da tela para a medição) faz dois passes: (1) todo elemento com `accessible-role` button/tab/switch/checkbox/radio-button/slider/combobox: a **área** é a união dos `TouchArea` dentro dele (é ela que recebe o toque, mesmo maior que o desenho, como o Chip de 40 com área de 48), ou o próprio elemento se não houver `TouchArea`; (2) todo `TouchArea` da janela, com ou sem papel. Falha (exit ≠ 0) se largura ou altura < 48. A galeria compila a UI com informação de depuração (`with_debug_info`) por causa desta API.
- **`--selftest` confere por pixel** três casos do mesmo problema (aba "Alta" do `Segmented`, interruptor, chip da Busca): o toque a 22 px do centro, fora do desenho de 40/32 e dentro dos 48, aciona o controle.
- **Antes (baseline desta revisão, 8 tipos de violação)**: `Logo` (marca da barra lateral, 44×48), `KSlider` (volume do NowBar e do player largo, 112×32), `KSwitch` (64×44), `Segmented` (item de 40: abas "Original/Alta/Baixa", "Automático/Claro/Escuro", "Da capa/Marca", 73–316×40) e os `TouchArea` desses quatro. **Depois: 0**. Correções (sem mudar o desenho visível): `Logo.width = max(48, …)` com a marca centralizada; `KSlider` com `TouchArea` de 48 de altura centrada (o `mouse-x` não muda); `KSwitch` com `TouchArea` de 64×48; `Segmented` com `TouchArea` de 4 px a mais acima e abaixo de cada item (os itens continuam com 40 de desenho). `atoms.slint` mudou em `design/slint/` também (idêntico).
- **Não são alvos (itens apontados pela revisão)**: o quadrado de 40 px do ícone na linha de busca recente (`search.slint`), o de 40 px do ícone da linha de ajuste (`settings.slint`) e o círculo de 44 px do ícone do `Banner` (`basics.slint`) são desenhos decorativos, sem `TouchArea`. O alvo da linha de busca recente é a linha toda (64) e o × é um `IconButton` de 48; o `Banner` tem o × de 48; a linha de ajuste não é clicável, só o controle à direita (≥ 48 pelas regras acima).
- **Exceções legítimas**: `TouchArea` de tela cheia (`Scrim`, arrastar do player) e a barra de rolagem do `ListView` (`ScrollBar::touch-area`, de std-widgets, 14 px: é arrasto de rolagem; a lista rola por gesto em qualquer ponto). Chips (40 + área de 48), itens de `Segmented` (40 + 48) e interruptores (32 + 48) **passam** porque o teste mede a área de toque, não o desenho.
- **Limite do teste**: mede o retângulo do `TouchArea`, sem recortar por ancestrais com `clip: true` (o Slint não expõe o pai pelo `ElementHandle`). Onde a área extrapola o desenho (chip, aba, interruptor, volume) conferi à mão que o ancestral é maior (trilho de 48, linha de ≥ 64, NowBar de 72) e o `--selftest` prova os casos por pixel.

### Início offline: cartão "Suas músicas do aparelho" (desvio do protótipo)

O protótipo (`design/prototypes/states.html`, estado offline; `compare-home-offline`) mostra o cartão "Seus baixados estão aqui" com a ação "Ver baixados". Como **Baixados ainda é "Em breve"** (não há dado), o Início offline mostra, abaixo da `OfflineBar` e acima dos recentes, um cartão de vidro nível 1 (`EmptyState` com ação `quiet`): título "Suas músicas do aparelho", subtítulo `AppState.library-summary` e a ação "Ver biblioteca" (vai à aba Biblioteca). Sem biblioteca (`!has-library`): "Nenhuma música no aparelho ainda. Adicione uma pasta" e a ação "Adicionar pasta" (`add-folder()`). A seção "Neste aparelho" (recentes) só aparece com recentes, e o antigo estado vazio "Sem conexão" saiu (o cartão já cobre). Quando Baixados existir, troque o texto e a ação do cartão por "Seus baixados estão aqui" / "Ver baixados". Cenas `home-offline` e `home-offline-nolib` na galeria.

### Fila e nota da saída

- `queue-remove`, `queue-jump` e `queue-move-up` recebem o `id` da faixa da **linha** (`t.id`), não o índice. A seta "Mover para cima" some só na primeira linha de "A seguir" (`if i > 0`; um espaçador de 48 mantém a coluna do ×).
- `output-note` do Rust ("Qualidade do YouTube Music, sem equalizador") quebra em até 2 linhas e termina em reticências; o que passa de uma linha é tirado da capa do player (`PlayerControls.note-extra`) para a folha não passar da janela em 360×740. Cena `player-longnote`.


## v4: o que mudou (implementação do `design/v4`)

Fonte visual da verdade: `design/v4/` (`README`, `DESIGN`, `SLINT_SPEC`, `ux/`, protótipos e `shots/`). Onde o app diverge de uma captura, o protótipo vence.

- **Arquivos.** `ui/theme/tokens.slint` é **cópia exata** de `design/v4/slint/tokens.slint` (teste `design_sync`); `ui/fonts/` = Inter e Inter Display (OFL) de `design/v4/fonts`; `ui/theme/icons.slint` = lista de ícones do design. `ui/components/atoms.slint` e `chrome.slint` são os átomos e o cromo do design **adaptados** ao `AppState` (a linha de faixa recebe `Track`, não `TrackItem`; `IconButton` ganhou `on` e `badge`; entrou `SearchField`, `Spinner`; o cromo ganhou `Toast`, `Sidebar` e `NowBar`). `components/shared.slint` tem a rolagem por aba (`Screen`), a prateleira (`Rail`, com setas fora do celular), a grade de álbuns e os esqueletos. `components/sheets.slint` tem as quatro folhas. `screens/*.slint` seguem a árvore de `design/v4/slint/screens.slint`.
- **Decisões A1 a A11** adotadas como recomendado: sem chips no Início; acento **fixo** Brasa (`Theme.accent`; o acento da capa e a opção em Ajustes saíram, e `SlintView::set_accent` não faz nada); player **sempre escuro**; mini-player tingido pela capa; Letra, Timer e Baixar só no menu da faixa, como "Em breve"; "Reconhecer pelo som" como linha esmaecida na Busca; prateleiras com nomes próprios ("Para ouvir agora", "Faixas para você", "Mixes", "Álbuns em destaque"); momentos e gêneros em ladrilhos neutros.
- **Contrato (`AppState`).** Entraram `backdrop-wide`, `player-dim`, `mini-dim`, `now-source-line`; `backdrop` mudou para 192×464 (capa, reflexo, topo da capa; σ3, saturação ×1,2) e quem o faz é `art::backdrop`; `art::backdrop_wide` faz a 96×96 (σ9, ×1,35); `art::glass_dims` calcula os escurecimentos pela regra de `make_covers.py` (12×12, 112 de cinza, 5:1). Saíram `home-section`, `accent-*`, `home-error-code`, `search-ideas`, os enums `HomeSection` e `AccentMode` e as propriedades de palco de `Theme`. `Highlight.caption/tint` e os callbacks `open-lyrics`, `set-sleep-timer`, `download-track`, `recognize-ambient`, `connect-provider` continuam no contrato, mas a UI não os chama (os recursos são "Em breve").
- **Faixa larga (>= 1100) e média.** O design só prototipou a faixa larga em HTML; no Slint ela foi construída aqui: barra lateral **sólida** de 264 (`paper-2`, logo, quatro destinos, playlists), `NowBar` de vidro de 72, player em duas colunas (capa à esquerda, console à direita, sobre a capa muito desfocada), detalhe com a capa ao lado das faixas (>= 840 de largura de conteúdo). Abaixo de 1100 é o layout de celular (doca de vidro centralizada, até 520).
- **Biblioteca de demonstração.** `--demo-library` agora gera 22 álbuns com as 22 fotos CC0 de `assets/demo-covers` (créditos no `CREDITS.md`). Sem YouTube Music (`--no-youtube`) o Início mostra os álbuns e as músicas do aparelho e o Explorar mostra o estado vazio; o detalhe local ganhou capa grande.
- **Acessibilidade mantida.** Alvos de 48 (os botões do transporte têm 56 e 72), anel de foco por `Pressable`, nomes acessíveis em pt-BR, Reduzir transparência (`Theme.glass-on` falso: o vidro vira `paper-2` com fio; o player perde o fundo da capa) e Reduzir movimento (`dur-*` caem para 120 ms; o player entra só por esmaecer), renderer software sem desfoque, áreas seguras e Voltar (`Nav.back`, inalterado).

## Reconstrução nativa de novembro de 2026

A UI de produção continua em Rust + Slint, com as cores Brasa/Ink/Paper e as fontes Inter existentes. `components/editorial.slint` reúne os novos cabeçalhos, atalhos de coleção e destaques de arte. As composições de Home, Biblioteca, detalhes e player foram substituídas; Busca, ajustes, folhas, linhas e navegação usam a mesma linguagem visual.

A ilha principal contém Início, Biblioteca e Buscar, no celular e no desktop. A rota antiga Explore resolve para Home. O feed Explore é solicitado na inicialização e alimenta lançamentos/em alta na Home e gêneros/lançamentos na Busca. A atualização de capas considera os três destinos consumidores, inclusive ao retornar à tela.

A Biblioteca salva `library_view` e `library_density` no TOML com defaults para arquivos antigos. Categorias ficam no estado global Slint ao navegar. Playlists usam mosaicos; o acervo de músicas e os artistas locais consultam o banco existente. Detalhes de artistas locais mostram músicas e discografia. Descrições e conteúdo relacionado de álbum, playlist e artista remotos são publicados quando fornecidos pelo catálogo, incluindo capas pelo cache existente. A lista de faixas dos detalhes é virtualizada. Em celular, a grade padrão tem duas colunas; a densidade permite três colunas a partir de 390 px ou uma capa grande. A mesma preferência adapta tablet/desktop.

O player usa capa inteira com margem, console em coluna no celular e em duas colunas em janelas largas. Telas curtas rolam para manter volume e fila acessíveis. Mostra álbum, tempo atual, duração total, qualidade real, favorito, shuffle/repeat, fila e ações existentes. Fila vira painel lateral no desktop; demais folhas ficam centralizadas. Movimento e transparência seguem as preferências existentes.

Referências examinadas: screenshots e componentes Home/Player/Library de `efzynx/BitChord-Desktop`; desktop chrome, drawer e screenshots de `Bntagaaa/BitChord-Desktop-Revived`; página pública e capturas de Orchard (somente o que o site mostra; nenhum código do repositório foi usado). Foram aproveitadas estruturas de apresentação, sem portar seus frameworks nem copiar seu código.

A árvore única do design v4 foi dividida em `ui/mobile` e `ui/desktop`. O teste `design_sync` registra o mapa de equivalências (`HomeScreen` → `MobileHome`/`DesktopHome`, `TopBar` → `MobileTopBar`/`DesktopCompactHeader`, etc.) e confere que todo nome do design continua coberto nas duas famílias; Explorar foi dobrado no Início.

Limites do backend continuam explícitos: downloads gerenciados, letras, timer e troca de saída não ganharam implementações fictícias. Os controles existentes mantêm seus avisos de disponibilidade. Android/iOS e dispositivos físicos exigem seus respectivos toolchains para validação; testes de largura em desktop verificam composição responsiva, não equivalem a certificação em dispositivos.

### Verificação da reconstrução

Validado em Linux com `cargo check -j 1 --workspace` e `git diff --check`. A suíte do app passou com 115 testes (47 unitários, 54 de controller, 6 de consistência do design, 7 de worker e a matriz de renderização). O teste inalterado de síntese da biblioteca de demonstração foi filtrado; a execução nativa usou cinco arquivos WAV temporários indexados pelo backend real. Após os últimos ajustes de viewport, `cargo test -j 1 -p kazzu-app --test ui_render` foi executado novamente e passou.

A aplicação também foi executada com FemtoVG/X11 e saída de áudio simulada. Capturas confirmaram 390×844, 768×1024 e 1280×800; o renderer de software cobriu ainda 320 e 360 px, categorias da biblioteca, modos grade/lista, temas e interações reais de clique/teclado. O ambiente usa Niri: a janela de captura foi colocada em modo flutuante para permitir as medidas solicitadas. O canvas **Kazzu — interface nativa implementada** reúne dez dessas capturas. Android/iOS não foram compilados nem testados em dispositivos neste ambiente.

### Ajuste visual após uso no desktop

O cabeçalho agora oculta Ajustes em janelas largas: o único acesso fica na lateral; em celular e tablet, continua no cabeçalho. O cromo usa vidro mais transmissivo sobre atmosfera da capa (o fundo do player em cache quando há reprodução), com fallback sólido para reduzir transparência/software. A lateral passa a ser um painel flutuante único, sem empilhar dois blurs. As chamadas editoriais usam uma placa de vidro sobre a arte; os recentes ocupam até três colunas no desktop, seguindo a organização musical do BitChord. A paleta de marca permanece intacta.

Validação deste refinamento: build nativo aprovado; seis testes de design e a matriz de renderização/interações em ambos os temas aprovados (7 testes, sem falhas). Capturas FemtoVG em 1280×800 conferidas com reprodução ativa. O comparativo está no artefato `kazzu-glass-refinement`. A captura desktop manteve o tamanho pelo IPC do Niri, pois o compositor altera a geometria após a captura de Ajustes alta. Para iterar mais rápido nesta validação, o segundo build usou override local de `profile.dev.package.kazzu-app.opt-level=0`; nenhum perfil do repositório foi alterado.

### Configurações inspiradas nas três referências

Foram estudados `SettingsSheet.kt` dos dois BitChord, `DesktopPlayerThemeSettings.kt` do Revived e os componentes `SettingsPanel`, `AppearanceSectionForm` e `PlaybackSectionForm` do Orchard. A implementação em Slint usa categorias laterais no desktop, navegação horizontal no celular e busca global que ignora caixa e acentos. As seções são Geral, Reprodução, Aparência, Fontes e biblioteca, Privacidade, Atalhos e Sobre.

Os controles de qualidade, rádio automático, volume, tema, movimento/transparência, pastas, histórico e diagnóstico continuam ligados ao backend. Biblioteca, acabamento do vidro e atmosfera de fundo são preferências persistidas em TOML com valores padrão para configurações antigas. Atalhos exibidos correspondem ao `FocusScope` real do aplicativo. Não foram importados controles falsos para equalizador, normalização, crossfade ou serviços que o Kazzu ainda não implementa.

Na validação nativa, a seção Geral revelou uma largura preferida nula quando continha apenas escolhas segmentadas. `Narrow` agora fixa explicitamente a largura disponível, até 720 px, e calcula a altura pelo conteúdo; a regressão tem verificação de pixels na matriz visual. O menu de categorias também rola em janelas baixas. As capturas do AppImage atualizado estão no artefato `kazzu-settings-native` (janelas compactas Linux, não dispositivo Android).

Os testes de fila passaram a selecionar um item com sucessores na ordem efetivamente apresentada, sem presumir ordem de término do escaneamento. O teste de retomada aceita a faixa tanto na fila quanto já em reprodução, pois o relógio de teste acelera os arquivos de três segundos em 8×. As verificações de dados, origem, formato, continuidade e ausência de tentativas repetidas permanecem.

Validação final das configurações: 56 testes de controle/integração, 6 de sincronização visual e 1 matriz de renderização/interação aprovados (63 no total). A região de amostragem da regressão Geral inclui os rótulos e os controles, evitando o espaço vazio entre linhas no tema claro. A matriz final passou em 125,87 s. Os três pacotes tiveram arquitetura, conteúdo e checksums verificados; detalhes em `dist/validation.txt`.
