# YouTube Music no Kazzu — notas técnicas (`crates/kazzu-ytm`)

Data: 2026-11-22. Tudo o que está marcado como **medido** foi testado neste dia contra a rede real, a partir de um IP residencial no Brasil, sem conta e sem cookies.

## Decisão

**Cliente innertube próprio e mínimo, em Rust puro** (ureq + rustls, sem tokio, sem JS, sem yt-dlp):

| Parte | Como |
|---|---|
| Metadados | cliente `WEB_REMIX` (o mesmo do music.youtube.com), anônimo: `search`, `music/get_search_suggestions`, `browse` (home, paradas, lançamentos, moods, álbum, artista, playlist, letra), `next` (rádio/próximas, metadados de uma faixa) |
| Áudio | endpoint `player` com clientes que devolvem **URL direta, sem cifra de assinatura e sem PO token**: `VISIONOS` (Opus + AAC) e, de reserva, `IOS_PRODUCER` (só AAC) |
| Sessão | um `visitorData` anônimo por execução do app, tirado do `sw.js_data` (2,5 KB), que também traz a versão atual do `WEB_REMIX` |

### Alternativas avaliadas

| Opção | Resultado | Decisão |
|---|---|---|
| `rustypipe` 0.11.4 (GPL-3.0) | Última versão no crates.io em 2025-04-23; último commit de código em 2025-06-18 (o de 2026-08 só troca um identificador de licença). Testado hoje: ver "Teste do rustypipe" abaixo | Descartado: sem manutenção há 17 meses, e o problema central de hoje (PO token / clientes cortados) apareceu depois da última versão |
| `ytmapi-rs` 0.3.4 (MIT) | Ativo (2026-11-07), mas assíncrono sobre tokio + reqwest, não resolve stream, e home/charts/related estão como "Not Planned" | Descartado: traria tokio e ainda exigiria resolver o áudio por outro caminho |
| yt-dlp como subprocesso | Funciona no desktop, mas não existe no Android | Proibido pelo requisito (Android) |
| Implementação própria | ~3,9 mil linhas de Rust (com testes); parsers tolerantes; testes com fixtures | **Escolhida** |

A implementação não copia código de nenhuma referência. As informações de protocolo (nomes e versões de clientes, campos do JSON) vêm da observação das respostas do próprio YouTube e da lista pública de clientes do yt-dlp (`yt_dlp/extractor/youtube/_base.py`, Unlicense), usada só como fonte de fatos.

## O que foi medido hoje (2026-11-22)

### Clientes do endpoint `player`

| Cliente (versão) | Sem `visitorData` | Com `visitorData` | Download completo? |
|---|---|---|---|
| `VISIONOS` 1.02 | `LOGIN_REQUIRED` ("confirm you're not a bot") | OK, URLs diretas: itags 139, 140, 249, 250, **251** | **Sim**: 4,4 MB num único Range em 0,07 s; também o último KiB |
| `IOS_PRODUCER` 0.147.945 | — | OK, só AAC (139, 140) | **Sim** |
| `IOS` 20.10.4 | OK, URLs diretas | — | **Não**: só o 1º ~1 MB (0–999 999 dá 403; 0–99 999 dá 206). É a cobrança de PO token no googlevideo |
| `IOS` 21.26.4 | — | OK, mas sem URL direta | Não |
| `ANDROID_VR` 1.65.10 | `LOGIN_REQUIRED` | OK, URLs diretas | **Não**: corta em ~1 MB (igual ao IOS) |
| `ANDROID_VR` 1.71.26 | — | OK, mas só `signatureCipher` | Não (precisaria do JS do player) |
| `ANDROID_VR` 1.60/1.62 | `LOGIN_REQUIRED` | `LOGIN_REQUIRED` | — |
| `ANDROID_MUSIC` 7.27 | `LOGIN_REQUIRED` | — | — |
| `TVHTML5` 7.x / `TVHTML5_SIMPLY_EMBEDDED_PLAYER` | "a página precisa ser atualizada" / "não é mais compatível" | — | — |
| `WEB_REMIX` | `UNPLAYABLE` (exige PO token + JS) | — | — |

Outros fatos medidos:

- **Sem `Range` o googlevideo estrangula a velocidade**: o mesmo arquivo de 4,4 MB levou mais de 100 s sem Range e 0,07 s com `Range: bytes=0-…`. O kazzu-audio sempre usa Range (pedaços de 4 MiB).
- URLs expiram em ~6 h (`expire=` na URL; `expiresInSeconds` = 21 540) e são **presas ao IP** (`ip=`): trocar de rede (Wi-Fi → 4G) invalida a URL antes de expirar.
- Formatos: itag 251 = WebM/Matroska com Opus 48 kHz estéreo (~110–140 kb/s médios); itag 140 = MP4 **fragmentado** (DASH: `ftyp, moov, sidx` + N × `moof/mdat`) com AAC-LC 44,1 kHz estéreo 128 kb/s. Sem conta, não aparecem os formatos Premium (Opus 256k itag 774, AAC 256k itag 141).
- Todos os endpoints de metadados do `WEB_REMIX` respondem sem login, sem cookie e sem `visitorData`.
- A home anônima é magra (dois carrosséis + um "tastebuilder"; a continuação volta vazia). Por isso a tela Início usa `discover()` = home + lançamentos + paradas do país.
- Os testes de integração (`tests/live.rs`, 8 testes) passaram todos.

### Teste do rustypipe

Projeto descartável no scratchpad com `rustypipe = "=0.11.4"` + tokio (nada disso entrou no repositório):

- `music_search_main("nirvana")`: **funciona**.
- `player("sp86SkWKRQE")` e `player("hTWKbfoikeg")`: **devolvem** o itag 251 (Opus), mas as URLs são do cliente `c=IOS`. Baixando: `0-99999` → 206; o último KiB → **403**; o arquivo inteiro → **403**. Ou seja, a música para de tocar depois de alguns segundos. É exatamente o corte por PO token medido acima, que o rustypipe 0.11.4 (sem atualização desde abril de 2025) não trata.

Conclusão: hoje o rustypipe serve para metadados, mas não para tocar. A sonda de fim de arquivo do `kazzu-ytm` detecta esse caso e passa ao próximo cliente.

## Como o `kazzu-ytm` resolve um stream

1. `visitorData` da sessão (em memória; obtido do `sw.js_data` na 1ª reprodução).
2. Para cada cliente da cadeia (`VISIONOS` → `IOS_PRODUCER`): `POST /youtubei/v1/player` com `hl=en` (para classificar os motivos de erro) e `gl` do usuário.
3. `playabilityStatus` vira um erro tipado (tabela abaixo). Erros definitivos (região, idade) param a cadeia; os outros passam ao próximo cliente.
4. Escolhe o formato: o Opus de maior bitrate (251), senão o AAC (140); ignora variantes DRC (`isDrc`/`xtags=drc`, que não são a qualidade original) e faixas dubladas não padrão. `StreamQuality::Low` pega o Opus mais leve (249).
5. **Sonda**: pede o último KiB do arquivo (`Range: bytes=len-1024-len-1`). Um cliente cortado pelo PO token responde 403 aqui (o começo do arquivo ele serve). Se der 403, o cliente fica marcado como "bloqueado" até o fim da sessão e o próximo é tentado. Custa uma requisição de 1 KiB.
6. Devolve `ResolvedStream { url, headers (User-Agent do cliente), format (codec, taxa, canais, kbps), itag, mime, content_length, duration, expires_at, client, loudness_db }`. `badge()` dá o texto do selo: `Opus · 112 kbps · 48 kHz`.

`Provider::resolve_stream` devolve `StreamSource::Url { url, headers, format }`. Como o `StreamSource` do kazzu-core não tem campo de expiração, a expiração e o selo ficam no `ResolvedStream` (`YtMusic::resolve`).

**Prefetch** (`YtMusic::prefetch`): resolve a próxima faixa da fila antes da hora; o próximo `resolve` a consome se ainda faltarem mais de 20 min para expirar. Depois de um 403/410 no meio da reprodução (rede trocada), o kazzu-audio emite `StreamEvent::Expired { id, position }` em `engine.stream_events()`; o controller chama `invalidate_stream(id)`, resolve de novo e entrega com `engine.replace_stream(id, source)`, que retoma na posição (sem resposta em 30 s, a faixa é pulada).

**Duração:** todo `Track` devolvido (busca, sugestões, home/paradas/lançamentos, álbum, artista, playlist, rádio, `track()`) sai com `duration`. Os que vêm sem ela (as músicas da busca mista e os cartões da home) são completados com **uma** requisição `music/get_queue` por até 50 faixas, antes de gravar no cache; se falhar, segue sem a duração. Além disso, antes de montar o `QueueItem`, chame `ResolvedStream::fill_track(&mut track)`. Ela preenche `Track.duration` quando os metadados não a trazem (as músicas da busca mista vêm sem duração) e também o `format`. O kazzu-audio depende dessa duração para a barra e o seek do MP4 fragmentado (itag 140), cujo cabeçalho só declara o 1º fragmento.

## Erros para a UI

`YtmError` → `ErrorKind` (`error.rs`). Ao atravessar o trait `Provider`, a mensagem do `kazzu_core::Error` começa com `[ytm:<código>]` e `ErrorKind::of(&erro)` recupera o tipo.

| `ErrorKind` | Quando | Ação na UI |
|---|---|---|
| `network` | sem conexão, DNS, timeout, TLS, HTTP 5xx | "Tentar de novo" |
| `blocked` | HTTP 403, "confirm you're not a bot", googlevideo cortando o cliente | "Tentar de novo" (o app já troca de cliente e de `visitorData` uma vez) |
| `rate_limited` | HTTP 429 (`retry_after` se vier) | "Tentar de novo" mais tarde |
| `region_blocked` | "not available in your country" | "Indisponível no seu país" |
| `age_restricted` | "Sign in to confirm your age" | "Exige conta com idade verificada" (sem login, não há o que fazer) |
| `unavailable` | removido, privado, "This video is unavailable" | "Indisponível" |
| `not_found` | id inválido, página vazia | "Não encontrado" |
| `no_stream` | nenhum cliente deu formato direto | "Tentar de novo"; se persistir, "o YouTube mudou" |
| `upstream_changed` | JSON sem os campos esperados, HTTP 400, "cliente não suportado" | "O YouTube mudou; atualize o app" |
| `offline` | modo offline e nada no cache | "Sem conexão" |
| `storage` | falha ao gravar cache/biblioteca | aviso discreto |

`is_retryable()` diz se faz sentido o botão "tentar de novo".

## O que vai quebrar, como detectar e mitigar

| Risco | Como aparece | Mitigação no código | O que fazer |
|---|---|---|---|
| YouTube passa a cobrar PO token também do `VISIONOS` | sonda do fim do arquivo dá 403 → `blocked`; a cadeia cai para `IOS_PRODUCER` (AAC) | cadeia de clientes + sonda + marca de "bloqueado" na sessão | rodar `cargo test -p kazzu-ytm --test live -- --ignored` (`live_resolve_visionos_opus` falha); ver a lista atual de clientes do yt-dlp; testar novos clientes com `KAZZU_YTM_CLIENTS=...` sem recompilar |
| Versão do cliente aposentada | `ERROR`/`UNPLAYABLE` "no longer supported" → `upstream_changed` | o próximo cliente é tentado | atualizar `version`/`user_agent` em `src/innertube.rs` (um único arquivo) |
| Versão do `WEB_REMIX` velha | HTTP 400 nos metadados | `music_call` busca a versão atual no `sw.js_data` e repete uma vez | atualizar `WEB_REMIX_VERSION` |
| `visitorData` recusado/marcado | `LOGIN_REQUIRED` "not a bot" | descarta e pede outro uma vez por resolução | — |
| Mudança de layout do JSON | campos vazios, seções somem; `upstream_changed` só se faltar a raiz | parsers classificam itens pelo `pageType`/`videoId` do próprio item, procuram renderers em qualquer profundidade e ignoram o que não conhecem | regravar fixtures (ver abaixo) e ajustar `src/parse.rs` |
| Todos os clientes diretos cortados | `no_stream`/`blocked` em tudo | — | caminhos possíveis, em ordem de custo: (a) cliente TV + motor JS (boa ou rquickjs) para decifrar `sig`/`n`; (b) gerar PO token (BotGuard) — exige executar JS do Google; (c) SABR. Todos são trabalho grande; acompanhar yt-dlp/NewPipe |
| IP de datacenter / VPN | `LOGIN_REQUIRED` em tudo | — | sem solução sem conta; documentar na UI |

### Atualizar as fixtures

As fixtures em `crates/kazzu-ytm/tests/fixtures/` são respostas reais gravadas em pt/BR e **sanitizadas**: sem `trackingParams`/`menu`/`overlay`, URLs do googlevideo reescritas para `sn-fixture` com `ip=0.0.0.0` (só `expire`, `itag`, `mime`, `clen`, `dur`, `c`) e `visitorData` falso. Ao regravar, repetir a sanitização: elas não podem conter o IP nem identificadores de quem gravou.

## Privacidade e "sem telemetria"

- Nenhuma requisição acontece sem uma chamada de método disparada pelo usuário (criar o `YtMusic` não toca a rede; teste `no_network_until_asked`).
- **Nenhum identificador do usuário é enviado**: sem cookie, sem conta, sem id de dispositivo. O `User-Agent` é o do cliente imitado (genérico) e nunca contém o nome do app.
- O único identificador é o `visitorData`, um token **anônimo emitido pelo próprio YouTube**, exigido pelo endpoint `player`. Fica só em memória, um novo por execução do app, e é enviado **só** no `player` (os metadados não levam nenhum: teste `resolves_best_opus_stream_with_visionos` verifica). Ele codifica o país do IP; o YouTube consegue correlacionar as reproduções de uma mesma sessão, como faria com um navegador anônimo.
- Consentimento/cookies (UE): a API JSON do innertube não passa pela página de consentimento; não guardamos nem enviamos cookie `CONSENT`/`SOCS`. Se um dia for exigido, aceitar só o mínimo ("rejeitar tudo") e documentar aqui.
- Cache em disco: só metadados de catálogo (álbuns, artistas, playlists, home, paradas, letras), com TTL e limpeza após 30 dias. **Não** guardamos URLs de stream (contêm o IP), `visitorData`, buscas nem sugestões (seriam histórico de busca no disco).
- Biblioteca (curtidos, álbuns/artistas/playlists salvos) é **local** (`ytm-library.json`); nada vai para o YouTube.
- Educação com o servidor: no máximo ~10 requisições/s (intervalo mínimo de 100 ms), uma repetição em erro de rede/5xx, nada de pré-carregamento em massa.

## Locale

`Locale::system()` usa o idioma do sistema (`sys-locale`; no Android, a propriedade do sistema) e converte para os códigos do YouTube (`pt-BR` → `hl=pt`, `gl=BR`; `pt-PT` → `pt-PT`; `es-MX` → `es-419`; `zh-Hant-TW` → `zh-TW`; `he` → `iw`…). Sem país conhecido, `gl` é omitido e o YouTube usa o país do IP. Nunca há `en/US` fixo. O `player` usa `hl=en` só para poder classificar os motivos de erro; a UI mostra as próprias mensagens.

## API pública (resumo)

```rust
let ytm = YtMusic::new(Config::with_data_dir(app_dir))?;   // Android/iOS; Config::desktop() no PC
ytm.discover().await?;                     // Início: home + lançamentos + paradas
ytm.charts(Some("BR".into())).await?; ytm.new_releases().await?; ytm.categories().await?; ytm.browse(tile.browse).await?;
ytm.search_items("nirvana".into(), None, 0).await?;              // misto, já classificado
ytm.search_items(q, Some(SearchFilter::Songs), 50).await?;        // filtrado, com paginação
ytm.suggestions("nirv".into()).await?;
ytm.album(id).await?; ytm.artist(id).await?; ytm.playlist(id).await?; ytm.playlist_continuation(tok).await?;
ytm.radio(video_id).await?; ytm.radio_continuation(radio).await?;  // auto-fila infinita
ytm.lyrics(video_id).await?;                                       // texto simples + fonte
ytm.resolve(video_id, StreamQuality::Original).await?;             // ResolvedStream (selo, expiração)
ytm.prefetch(next_id, q).await?; ytm.invalidate_stream(&id);
ytm.library_store().like(&track)?;                                 // biblioteca local
ytm.client().set_offline(true); ytm.client().peek_home();          // offline / primeira pintura instantânea
// + impl kazzu_core::Provider (search, resolve_stream, library, track, album_tracks, playlist_tracks)
```

Exemplo: `cargo run -p kazzu-ytm --example demo -- search 'nirvana'` e `… -- play <videoId> [segundos] [--cpal]`.

## Pendências

- Android: validado em 2026-11-22 com o NDK r30 (`cargo build -p kazzu-ytm --target aarch64-linux-android`, CC/AR/linker `aarch64-linux-android26-clang` como em `scripts/build-apk.sh`); o executável de testes do crate linka (ELF aarch64, `/system/bin/linker64`). Falta rodar num aparelho.
- Letras sincronizadas (timed lyrics) não existem no `WEB_REMIX`; só texto simples.
- O `kazzu_core::Error` não tem variantes estruturadas para provedores; o tipo do erro viaja no prefixo `[ytm:…]`. Sugestão para o kazzu-core: `Error::Provider { provider, kind, retryable, message }`.
- O `StreamSource` não tem expiração; o controller precisa reagir ao `StreamEvent::Expired` do kazzu-audio (`stream_events()` / `replace_stream`).
- Podcasts e episódios são ignorados de propósito.
- Paginação de playlists com mais de 100 faixas está implementada (continuação da prateleira), mas as fixtures não a cobrem.
