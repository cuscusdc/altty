# Provedores de música — pesquisa técnica

Data da pesquisa: 2026-11-21. Autor: pesquisador técnico (sem código do app). Versões/licenças de crates vêm da API do crates.io na mesma data; o resto vem das fontes citadas no fim. Onde marquei **[verificar]**, a informação veio de conhecimento prévio ou de fonte única e precisa ser confirmada antes de decidir.

## Resumo executivo

- **Nenhum dos quatro provedores oferece, de forma oficial e livre, "áudio de graça para um player de terceiros em Rust".** Cada um tem uma brecha diferente: YT Music = API interna não oficial; Spotify = só via Connect/Soloist (Premium); Tidal = API oficial de metadados, playback só pelo Player SDK; Qobuz = API só para parceiros.
- **Licença importa:** `rustypipe` (melhor crate de YT) é **GPL-3.0**. Se o Kazzu for MIT/Apache, não dá para linkar. Se o Kazzu for GPL-3.0 (que também é o que o Slint pede, ver `ui-and-mobile.md`), o problema some. **Decisão do autor nº 1: licença do Kazzu.**
- **Ordem sugerida:** Local (já na etapa 1) → YT Music → Spotify → Tidal → Qobuz. Tidal/Qobuz como módulos "experimentais, traga sua conta, desligados por padrão".

## 1. YouTube Music

### Candidatas

| Candidata | Link | Licença | Manutenção | Maturidade | Prós | Contras | Risco |
|---|---|---|---|---|---|---|---|
| `ytmapi-rs` | [crates.io](https://crates.io/crates/ytmapi-rs) / [GitHub](https://github.com/nick42d/youtui) | MIT | 0.3.4, 2026-11-07 (ativa); monorepo do `youtui` | média; README diz "heavy development, interface pode mudar" | Async (tokio+reqwest), licença permissiva, busca/biblioteca/playlists/letras; auth por cookie ou OAuth (TV client) | **Não documenta stream URLs** (só tracking URL) **[verificar]**; `GetHome`/`GetSongRelated`/`GetCharts` "Not Planned"; vários endpoints incompletos | Médio |
| `rustypipe` | [Codeberg](https://codeberg.org/ThetaDev/rustypipe) | **GPL-3.0** | 0.11.4 (2025-04) no crates.io, mas commits em 2026-08; 45 releases; CI com falhas na data | alta em escopo (YT + YTM, player, radio, charts, moods, lyrics) | Devolve streams de áudio; tem radio/recomendações; trata erros salvando "reports" | **GPL**; PO token exige binário separado `rustypipe-botguard` (simula navegador); hospedado no Codeberg | Médio-alto (quebra a cada mudança do YT + licença) |
| `yt-dlp` (processo externo) | [GitHub](https://github.com/yt-dlp/yt-dlp) | Unlicense **[verificar]** | Release estável 2026.10.26; muito ativa | a mais robusta contra mudanças do YT | Resolve stream URL de forma resiliente; atualização rápida; sem contaminação de licença se rodado como **subprocesso** | Dependência externa (Python/binário ~30 MB); latência de ~1–3 s por resolução; PO token via plugin (`bgutil-ytdlp-pot-provider`) ou `player_client=tv` | Baixo técnico / alto ToS |
| Crate `yt-dlp` (wrapper) | [GitHub](https://github.com/boul2gom/yt-dlp) | GPL-3.0-only | 2.10.0, 2026-11-02 | wrapper fino | Baixa/atualiza o binário sozinho | GPL; só embrulha o processo | — |
| `rusty_ytdl` | [GitHub](https://github.com/Mithronn/rusty-ytdl) | MIT/Apache | 0.7.4, **2024-08** (parada) | defasada | Rust puro | Sem PO token/SABR → provavelmente quebrada | Alto |
| `youtube_dl` (crate wrapper) | [GitHub](https://github.com/GyrosOfWar/youtube-dl-rs) | MIT/Apache | 0.10.0, 2024-04 | wrapper do binário | simples | parada | Médio |
| Invidious / Piped | [Invidious](https://github.com/iv-org/invidious) / [Piped](https://github.com/TeamPiped/Piped) | AGPL-3.0 (servidores) | Invidious 2.20261022.0 (2026-10-23) | instâncias públicas fracas: ~3 do Invidious e ~15 do Piped em mar/2026 (blog de self-hosting; tratar como retrato, não número oficial) | Fallback com API HTTP simples; sem login | IPs de datacenter bloqueados pelo YT; SABR ainda não suportado no Invidious (issue aberta #5263); YouTube já enviou notificação jurídica ao Invidious em 2023 | Alto como fallback "padrão"; ok como **opção do usuário auto-hospedada em casa** |

**Sobre o ecossistema (contexto de 2026):** o YouTube exige **PO tokens** para clientes web (incl. `web_music`); sem token os URLs dão HTTP 403; tokens são ligados ao vídeo. O **SABR** (streaming adaptativo próprio) ainda não está no yt-dlp estável, que contorna usando outros clientes (`tv`, `tv_samsung`, `web_embedded`, `visionos`). Ou seja: **qualquer solução nativa exige manutenção contínua**. O Metrolist (Kotlin, GPL-3.0; usado só como inspiração conceitual) tem o mesmo aviso no README: "o uso pode violar os ToS do YouTube".

**Qualidade:** YT Music entrega Opus ~160 kbps (itag 251) e AAC ~128–256 kbps; não é lossless. Alinhado ao brief ("qualidade original" = o que o provedor oferece, sem reprocessar).

### Recomendação (YT Music)

1. **Metadados/biblioteca:** `ytmapi-rs` (MIT) atrás do `trait Provider`.
2. **Resolução de stream:** um sub-trait `StreamResolver` com implementações plugáveis: (a) **yt-dlp como subprocesso** (padrão; atualização automática do binário pelo app, opt-in); (b) `rustypipe` atrás de uma *feature flag* (só habilitável se o Kazzu for GPL-3.0); (c) URL de instância Piped/Invidious **configurada pelo usuário** (self-host).
3. Login **opcional**; padrão = sem conta (menor risco de ban). Cookies só se o usuário pedir, guardados no cofre do SO (`keyring`).
4. Cache de URLs com expiração (os URLs do YT expiram em ~6 h), pré-resolução da próxima faixa para gapless.

**Risco legal/ToS: alto.** Não existe API pública para áudio do YT Music; o acesso por innertube/yt-dlp contraria os Termos do YouTube. É a mesma zona cinzenta de NewPipe/Metrolist/youtui. Mitigações: sem telemetria, não hospedar nada, não redistribuir áudio, aviso claro no primeiro uso, nunca embutir cookies/chaves. O autor deve decidir se aceita essa exposição (sobretudo se distribuir em lojas — Play Store/App Store provavelmente recusariam).

## 2. Spotify

### O que mudou em 2026 (importante)

- **Web API (Dev Mode):** desde 2026-02-11 (apps novos) e 2026-03-09 (migração dos existentes), o dono do app precisa de **Premium**, **máx. 5 usuários** por app; endpoints removidos/alterados (top tracks do artista, new releases, buscas em lote, `limit` de busca capado em 10, gêneros ausentes…); redirect só `127.0.0.1`. Extended Quota só para organização registrada com 250 mil MAU. ⇒ **um app distribuído ao público não consegue usar a Web API com um client_id próprio do Kazzu**; o usuário teria de registrar o próprio app (muito atrito).
- **`librespot` (0.8.0, 2025-11-10, MIT):** o login quebrou várias vezes em 2026: o `login5` passou a rejeitar tokens de client IDs que não sejam o do desktop; o caminho access-token/login5 foi fechado em ago/2026 (pré-lançamento do Soloist); o login Zeroconf quebrou em 2026-10-29 (go-librespot#433); contas novas recebem `audio key error 0 1` (issue #1649, sem workaround). Há fluxo *device authorization* funcionando em forks (go-librespot 0.10.5).
- **Spotify Soloist (oficial, lançado 2026-08-13):** cliente Spotify Connect headless (Linux/RPi; macOS e Windows desde out/2026). **Lossless até 24-bit/44.1 kHz (só Premium)**, API local por **WebSocket** (play/pause/seek/volume/shuffle/repeat, eventos, fila de até 80 faixas com metadados). Exige conta de desenvolvedor para baixar e emitir a `SOLOIST_API_KEY`; relatos dizem que **os builds expiram em 90 dias e não podem ser redistribuídos [verificar nos termos oficiais]**. É a primeira via *sancionada* para tocar Spotify fora do app oficial — mas é um **processo separado que toca o áudio sozinho**: o Kazzu só o controla (sem PCM ⇒ sem EQ, sem visualizador).

### Candidatas

| Candidata | Link | Licença | Manutenção | Prós | Contras | Risco |
|---|---|---|---|---|---|---|
| `librespot` + `librespot-playback`/`-connect` | [GitHub](https://github.com/librespot-org/librespot) | MIT | 0.8.0 (2025-11); 99 issues e 110 PRs abertos; mantido por voluntários | PCM acessível via *sink* próprio (dá para passar pelo EQ do Kazzu); Connect receiver; Rust puro | Ogg Vorbis até **320 kbps — não é lossless**; login instável em 2026; README: conectar "provavelmente é proibido" pelo Spotify; só Premium | **Alto** (quebra sem aviso, risco de ban de conta) |
| Spotify **Soloist** | [blog](https://developer.spotify.com/blog/2026-08-13-introducing-spotify-soloist) / docs `developer.spotify.com/documentation/soloist` | proprietária **[verificar]** | oficial, nova | Lossless 24/44.1; sancionado; WebSocket simples; Jam/Connect | Redistribuição/expiração de 90 d; exige dev account + API key por usuário; sem PCM; ainda sem histórico de estabilidade | Médio (política pode mudar) |
| `rspotify` | [GitHub](https://github.com/ramsayleung/rspotify) | MIT | 0.16.1, 2026-04-25 (issue #550 acompanha a migração de fev/2026) | Boa cobertura da Web API; async | Limites do Dev Mode acima; recomendações/related removidos para apps novos **[verificar]** | Médio |

### Recomendação (Spotify)

- **Fase A (metadados + controle):** `SpotifyProvider` via `rspotify` com *client_id do próprio usuário* (BYO app) — só para biblioteca/playlists/busca; ou pular a Web API e usar os metadados que o Soloist devolve.
- **Fase B (áudio):** backend **Soloist** (controle por WebSocket) como caminho recomendado; `librespot` como backend alternativo experimental atrás de feature flag (permite EQ/visualização, mas 320 kbps e frágil).
- **Qualidade:** lossless só com Soloist; com librespot, Vorbis 320.
- **Limitação "1 dispositivo":** é regra de conta do Spotify (Connect = um stream ativo por conta) e **o Kazzu não pode contorná-la** para Spotify. O diferencial do brief (vários dispositivos tocando/controlando livremente) só vale para fontes locais e provedores sem essa trava; documentar isso na UI.
- **ToS:** `librespot` = zona cinzenta/proibida (risco à conta). Soloist = permitido, dentro dos termos de dev. Web API = Dev Mode restrito. O autor deve decidir se o Kazzu **distribui** o Soloist (provavelmente proibido) ou apenas **detecta/orienta** o usuário a instalá-lo.

## 3. Tidal

| Candidata | Link | Licença | Manutenção | Notas |
|---|---|---|---|---|
| API oficial (OpenAPI v2, JSON:API, OAuth2+PKCE) | [developer.tidal.com](https://developer.tidal.com/documentation/api-sdk/api-sdk-overview) | termos do portal | ativa | Cadastro aberto e gratuito; metadados, playlists, coleções, manifests. **Playback de terceiros só pelo *Player SDK* oficial (Web/iOS/Android)**; na prática, prévias. Não serve para um player Rust nativo tocar faixas completas. |
| `tidalrs` | [GitHub](https://github.com/phayes/tidalrs) | MIT | 0.6.2, 2026-10-16; 74 commits, 20 estrelas | API **v1 (privada)**; device flow/PKCE; qualidades Low/High/Lossless/**HiResLossless** via MPEG-DASH; "não afiliado, use por sua conta e risco" |
| `tidalv2` / `prawn` | [GitHub](https://github.com/phayes/tidalv2) / [GitHub](https://github.com/khreezy/prawn) | Unlicense / MIT | 0.3.0 (2026-09) / 0.1.0 (2026-01) | Clientes da API v2 oficial (metadados); imaturos |
| `moosicbox_tidal` | [GitHub](https://github.com/MoosicBox/MoosicBox) | MPL-2.0 | 0.5.0, 2026-10-16 | Device flow, favoritos, FLAC/Hi-Res; acoplado ao framework MoosicBox (trait `MusicApi`) |
| `tidalapi` (Python) | [docs](https://tidalapi.readthedocs.io/) | **[verificar]** (LGPL-3.0 segundo memória) | 0.7.x | Referência de comportamento; não usar (Python) |

**Recomendação:** metadados via API oficial v2 (`tidalv2`/`prawn` ou HTTP direto com `reqwest`) e **playback via `tidalrs`** (MIT, hi-res FLAC) como **módulo experimental, opt-in**, avisando que usa a API não pública. **Risco legal/ToS: alto** — playback fora do Player SDK não é autorizado; risco de revogação de client ID e de ban. Alternativa 100% sancionada: **não tocar** Tidal, só importar playlists/metadados (menos valor).

## 4. Qobuz

| Candidata | Link | Licença | Manutenção | Notas |
|---|---|---|---|---|
| API oficial | `api@qobuz.com` | Termos de 2011 ([PDF](http://static.qobuz.com/apps/api/QobuzAPI-TermsofUse.pdf)) | — | **Sem portal público**: solicita-se `app_id`+`secret` por e-mail, aprovação em semanas e não garantida; termos proíbem compartilhar o secret e limitam a "dispositivos aprovados". Não existe SDK. |
| `qobuz-api` (loxoron218) | [GitHub](https://github.com/loxoron218/qobuz-api-rust) | **GPL-3.0-or-later** | 4.0.0, 2026-10-17; ~250 downloads | Auth, busca, favoritos, URL de stream assinada, extração de credenciais do web player (scraping do bundle.js) |
| `moosicbox_qobuz` | [crates.io](https://crates.io/crates/moosicbox_qobuz) | MPL-2.0 **[verificar]** | desatualizado (~489 dias na pesquisa) | FLAC até 24/192; login usuário/senha |
| `qobuz` (sr.ht) | [sr.ht](https://git.sr.ht/~tarneo/qobuz.rs) | GPL-2.0-only | 0.1.4, 2026-03-28 | pequena |
| Go: `go-qobuz` (GPL-3.0), `qobuz-dl-go` (várias) | [GitLab](https://gitlab.com/dunn.dev/go-qobuz) | GPL | 2026 | só referência de protocolo |

Formatos: 5 = MP3 320; 6 = FLAC 16/44.1; 7 = FLAC até 24/96; 27 = FLAC até 24/192 (hi-res de verdade).

**Recomendação:** é o provedor de **melhor qualidade**, mas o **mais fechado**. Reimplementar o cliente (não copiar) a partir da documentação pública do formato de requisição, **sem** embutir `app_id/secret` do web player, é zona cinzenta; escrever um cliente próprio MIT pequeno (login, search, `track/getFileUrl` com assinatura) é viável. **Risco legal/ToS: alto.** Primeiro passo mais seguro: **pedir parceria oficial** (api@qobuz.com) em paralelo e só implementar o módulo se houver resposta ou se o autor aceitar o risco "traga sua conta".

## 5. Esboço de integração — `trait Provider`

O ponto-chave descoberto: provedores **não** entregam todos "uma URL de áudio". Há três tipos de reprodução, e o `trait` precisa expressá-los.

```rust
// crates/kazzu-core (esboço, não é código do app)
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;                 // "ytm", "spotify", "tidal", "qobuz", "local"
    fn capabilities(&self) -> Capabilities;     // search, library, lyrics, recommendations, lossless, offline...
    async fn authenticate(&self, step: AuthStep) -> Result<AuthState, ProviderError>; // device-code/cookie/oauth/none
    async fn search(&self, q: &str, kinds: KindSet, page: Page) -> Result<Paged<Item>, ProviderError>;
    async fn library(&self, kind: LibraryKind, page: Page) -> Result<Paged<Item>, ProviderError>;
    async fn track(&self, id: &TrackId) -> Result<Track, ProviderError>;
    async fn resolve(&self, id: &TrackId, q: QualityPref) -> Result<Playback, ProviderError>;
    async fn related(&self, id: &TrackId) -> Result<Vec<Track>, ProviderError> { Err(Unsupported) }
    async fn lyrics(&self, id: &TrackId) -> Result<Option<Lyrics>, ProviderError> { Ok(None) }
}

pub enum Playback {
    // O Kazzu decodifica (symphonia): EQ, gapless, visualizador funcionam. YT Music, Tidal, Qobuz.
    Stream { url: Url, headers: HeaderMap, codec: Codec, bitrate_kbps: u32,
            sample_rate: u32, bit_depth: Option<u8>, expires_at: Option<Instant> },
    // PCM entregue por um sink do provedor (librespot). EQ ok, qualidade limitada.
    PcmSource(Box<dyn PcmSource>),
    // O provedor toca sozinho; o Kazzu só controla (Soloist/Connect). Sem EQ, sem multi-dispositivo.
    Delegated(Box<dyn RemotePlayer>),
}
```

Regras de integração:
- `capabilities()` deixa a UI esconder o que não existe (ex.: EQ desabilitado quando `Playback::Delegated`).
- Erros tipados (`Unsupported`, `AuthRequired`, `RateLimited{retry_after}`, `RegionBlocked`, `UpstreamChanged` — este último dispara aviso "o provedor mudou, atualize o app").
- Cada provedor vive em uma crate/feature própria (`kazzu-provider-ytm`, …) para o autor poder **não distribuir** os de maior risco jurídico.
- Rate limit e cache (SQLite) por provedor, no próprio `Provider`; nenhuma chamada fora de ação do usuário (regra "sem telemetria").
- Para o "ouvir junto": o par recebe `TrackRef` (`provider+id` ou MBID/ISRC) e resolve **com a própria conta**; nunca retransmitir áudio (ver `sync-and-listen-together.md`).

## 6. Ordem de implementação sugerida

1. `LocalProvider` (já planejado; valida o trait).
2. **YT Music**: `ytmapi-rs` + resolver yt-dlp (maior catálogo, sem conta).
3. **Spotify**: metadados (BYO client_id ou do Soloist) + backend Soloist; librespot só experimental.
4. **Tidal**: `tidalrs` opt-in (hi-res).
5. **Qobuz**: só após parceria ou decisão explícita de risco.

## Decisões que cabem ao autor

1. Licença do Kazzu (GPL-3.0 destrava `rustypipe`, crates GPL de Qobuz/AutoEQ e é compatível com o Slint GPL; MIT/Apache mantém o app permissivo, mas exige descartá-los e usar a licença Royalty-free do Slint).
2. Aceita distribuir módulos que contrariam ToS (YT, Tidal, Qobuz)? Sugestão: sim, como módulos separados, desligados por padrão, com aviso; lojas de apps móveis não os aceitarão.
3. Spotify: depender do Soloist (oficial, mas com expiração e API key por usuário) ou do librespot (frágil, arriscado)?

## Fontes (acessadas em 2026-11-21)

- crates.io API (versões/licenças): rustypipe, ytmapi-rs, librespot, rspotify, tidalrs, tidalv2, prawn, moosicbox_tidal, qobuz-api, qobuz, rusty_ytdl, youtube_dl, yt-dlp.
- RustyPipe (README): https://codeberg.org/ThetaDev/rustypipe
- youtui/ytmapi-rs: https://github.com/nick42d/youtui
- librespot: https://github.com/librespot-org/librespot ; issues [#1465](https://github.com/librespot-org/librespot/issues/1465), [#1649](https://github.com/librespot-org/librespot/issues/1649); [go-librespot#433](https://github.com/devgianlu/go-librespot/issues/433); [sonn-audio/core#333](https://github.com/sonn-audio/core/issues/333)
- Spotify Soloist: https://developer.spotify.com/blog/2026-08-13-introducing-spotify-soloist
- Spotify Dev Mode: https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide ; https://techcrunch.com/2026/02/06/spotify-changes-developer-mode-api-to-require-premium-accounts-limits-test-users/ ; https://github.com/ramsayleung/rspotify/issues/550
- yt-dlp PO Token Guide: https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide ; Invidious SABR: https://github.com/iv-org/invidious/issues/5263 ; https://selfhosting.sh/compare/invidious-vs-piped/
- Tidal: https://developer.tidal.com/documentation/api-sdk/api-sdk-overview ; https://github.com/phayes/tidalrs ; https://lib.rs/crates/moosicbox_tidal
- Qobuz: http://static.qobuz.com/apps/api/QobuzAPI-TermsofUse.pdf ; https://musicapi.com/blog/qobuz-app-id-and-secret ; https://github.com/loxoron218/qobuz-api-rust
- Metrolist (só conceito): `.refs/metrolist/README.md` (aviso de ToS do YouTube; lista de recursos).
