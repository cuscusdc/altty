# Som e integrações — pesquisa técnica

Data: 2026-11-21. Versões/licenças: API do crates.io na mesma data. **[verificar]** = conhecimento prévio ou fonte única.

## Resumo executivo

- **Equalizador:** escrever o EQ do Kazzu sobre a crate **`biquad`** (MIT/Apache, 0.6.0, 2026-03) ou ~150 linhas próprias do "RBJ cookbook". `fundsp` é grande demais para isto; `dasp` está parado desde 2020. Perfis de fones: formato **AutoEq `ParametricEQ.txt`** (dados MIT). **Evitar** as crates `autoeq`/`autoeq-iir` (GPL-3.0).
- **Reconhecimento por som ambiente:** Chromaprint/AcoustID **não** serve para microfone com ruído (é para arquivo completo). O que funciona na prática é um cliente do **Shazam** (SongRec/vibra, GPL-3.0, protocolo não oficial) ou um serviço comercial. Recomendo deixar para a etapa 5, atrás de um `trait Recognizer`, opt-in.
- **Discord:** `discord-rich-presence` (MIT, 1.1.0) resolve; **MPRIS/controles de mídia:** `souvlaki` (MIT, 0.9.3) cobre Linux/Windows/macOS.
- **Retrospectiva:** guardar eventos de reprodução **só localmente** num SQLite (esquema abaixo).

## 1. Equalizador

### Candidatas

| Candidata | Link | Licença | Manutenção | Maturidade | Prós | Contras | Risco |
|---|---|---|---|---|---|---|---|
| `biquad` | [GitHub](https://github.com/korken89/biquad-rs) | MIT/Apache | 0.6.0, 2026-03-22; ~525 mil downloads | alta, `no_std` | Coeficientes RBJ (peaking, shelves, LP/HP, notch), estruturas DF1/DF2T, f32/f64 | Só o filtro; preamp/limiter/UI são nossos | Baixo |
| `fundsp` | [GitHub](https://github.com/SamiPerttu/fundsp) | MIT/Apache | 0.23.0, 2026-01-07 | alta (síntese/DSP em grafo) | Muitos filtros e blocos | Pesado e voltado à síntese; overkill para EQ | Baixo (excesso de escopo) |
| `dasp` | [GitHub](https://github.com/rustaudio/dasp) | MIT/Apache | 0.11.0, **2020-05** (parada) | estável mas sem manutenção | Tipos de amostra/conversões | Sem EQ pronto | Baixo |
| `rubato` | [GitHub](https://github.com/HEnquist/rubato) | MIT/Apache | 5.1.0, 2026-11-12 | alta | Resample de alta qualidade (só quando a saída exigir — regra 5 do brief; também útil no "ouvir junto") | — | Baixo |
| `ebur128` | [GitHub](https://github.com/sdroege/ebur128) | MIT | 0.1.11, 2026-10-26 | alta (binding do libebur128 em Rust) | ReplayGain/loudness **opt-in** | O brief veta normalização forçada → só opcional | Baixo |
| `autoeq` / `autoeq-iir` | [crates.io](https://crates.io/crates/autoeq) | **GPL-3.0-or-later** | ativa | — | Calcula PEQ a partir de medições; exporta Equalizer APO | GPL | Alto se app não-GPL |
| AutoEq (dados) | [GitHub](https://github.com/jaakkopasanen/AutoEq) | **MIT** (repo); fontes de medição (oratory1990, Crinacle, Rtings, Innerfidelity) têm termos próprios **[verificar]** | ativa | alta (700+ fones) | Perfis prontos `ParametricEQ.txt` (linhas `Preamp: -x dB` e `Filter n: ON PK Fc … Hz Gain … dB Q …`) | Não embutir tudo no app; baixar sob demanda do usuário | Baixo/Médio |
| Janis, Vivi (apps Slint) | [Janis](https://github.com/obazin/janis), [Vivi](https://github.com/Loafer19/vivi) | — | 2026 | — | Só prova de conceito (Janis: dez filtros *peaking* com cpal+symphonia); **não copiar** | — | — |

### Recomendação

- **Cadeia:** `decode (symphonia) → [EQ] → [preamp/limitador] → saída (cpal)`. O estágio EQ é **omitido por completo** quando desligado (bypass = bit-perfect, regra 5 do brief); nunca "bypass com ganho 0".
- **Tipos de banda:** peaking (PK), low-shelf, high-shelf, LP/HP, notch; modos **gráfico de 10 bandas** (por cima de PK com Q fixo) e **paramétrico** (N bandas). Processar em `f32`/`f64` internamente.
- **Preamp automático:** calcular o ganho de pico da curva e aplicar preamp negativo para evitar clipping (AutoEq já fornece `Preamp:`). Opcional: limitador suave pós-EQ.
- **Presets:** salvar em arquivo (TOML/JSON) versionado; importar/exportar `ParametricEQ.txt` (Equalizer APO) — formato texto simples, fácil de parsear.
- **Perfis de fones:** busca no índice do AutoEq sob demanda (download pelo usuário, com cache); mostrar origem da medição e licença; **não** redistribuir o repositório inteiro.
- **Dither:** só se a saída for de menor profundidade que o fluxo (ex.: f32 → 16-bit).
- **Fase 2 (opcional):** fase linear/FIR e convolução para correções de sala — fora do escopo inicial.

Risco legal: baixo (nenhum ToS de provedor envolvido). Atenção à licença das crates GPL.

## 2. Reconhecimento por som ambiente

### Candidatas

| Candidata | Link | Licença | Manutenção | O que faz | Prós | Contras | Risco |
|---|---|---|---|---|---|---|---|
| `rusty-chromaprint` | [GitHub](https://github.com/darksv/rusty-chromaprint) | MIT | 0.3.0, 2024-10 | Fingerprint Chromaprint em Rust puro | Rust puro; liga com AcoustID | Pensado para faixas completas, **não** para trecho de 10 s com ruído; sem atualização recente | Baixo |
| `chromaprint-rust` | [GitHub](https://github.com/aksiksi/chromaprint) | MIT (binding do C, LGPL-2.1) **[verificar]** | 0.1.3, 2022 | bindings | — | parada | Médio |
| **AcoustID** (serviço) | [acoustid.org/webservice](https://acoustid.org/webservice) | **gratuito só para uso não comercial**; máx. 3 req/s; chave de aplicativo; plano "Open Source" no acoustid.biz | ativa | Identifica faixa a partir do fingerprint | Aberto, MusicBrainz-friendly | Funciona mal com ruído de ambiente | Baixo |
| **SongRec** (cliente Shazam em Rust) | [crate songrec](https://crates.io/crates/songrec) | **GPL-3.0** | 0.7.4 | Reconhece por microfone/áudio do sistema/arquivo; fingerprint feito no cliente e enviado ao Shazam | Funciona de verdade (reconhecimento ambiente); Rust | Protocolo **não oficial** do Shazam; GPL; depende de servidor de terceiros | **Médio-alto** (ToS) |
| `vibra` | [alternativeto](https://alternativeto.net/software/shazam/?license=opensource) | GPL-3.0 | — | cliente Shazam com FFI/WASM | idem | idem | idem |
| Motor próprio estilo `dejavu`/`audfprint`/Panako | — | dejavu MIT, audfprint MIT, Panako AGPL **[verificar todas]** | — | Fingerprint de "constelação" (Wang 2003) contra um banco | Totalmente local/privado | Só reconhece o que **você** indexou (ex.: biblioteca local); não há catálogo aberto grande | Baixo |
| Serviços comerciais (ACRCloud, AudD) | — | pagos | — | API de reconhecimento | Cobertura ampla, suporte oficial | Chave paga, envia trecho de áudio | Médio (privacidade/custo) |

O Metrolist usa Shazam para isso (conceito); o repo guarda "histórico de reconhecimento" (`RecognitionHistory`) — só como ideia de recurso.

### Recomendação

- **Etapa 5, opt-in.** `trait Recognizer { async fn identify(&self, pcm: &[f32], sr: u32) -> Result<Vec<Match>, _> }` com implementações: (1) `LocalLibraryRecognizer` (motor próprio de "constelação", privado, reconhece só a biblioteca do usuário — útil para "qual é essa faixa que estou ouvindo no meu acervo?"); (2) `ShazamRecognizer` (opcional; aviso explícito de que envia a *assinatura* do trecho a terceiros; protocolo não oficial) — escrever cliente próprio, **sem copiar o SongRec**; (3) `AcoustIdRecognizer` apenas para arquivos/trechos limpos.
- **Captura:** `cpal` (entrada de microfone; no Linux também o *monitor* do PipeWire/Pulse para áudio do sistema). Captura **só enquanto o usuário mantém o botão**, ~10–12 s, processa em memória, **não grava em disco**, mostra indicador de microfone. Android/iOS exigem permissão de runtime (código nativo fino).
- **Privacidade:** enviar assinatura (hash), não áudio bruto; sem ID de usuário.
- **Risco legal:** Shazam = protocolo privado, ToS proíbe uso fora do app (risco médio-alto, mas sem alternativa aberta equivalente). Patentes da técnica original de Shazam (anos 2000) provavelmente expiradas **[verificar]**; a técnica é pública (Wang 2003).

## 3. Discord Rich Presence

| Candidata | Link | Licença | Manutenção | Notas |
|---|---|---|---|---|
| `discord-rich-presence` | [GitHub](https://github.com/vionya/discord-rich-presence) | MIT | 1.1.0, 2026-01-26; ~790 mil downloads | IPC local, simples, síncrono; **recomendada** |
| `discord-presence` | [GitHub](https://github.com/jewlexx/discord-presence) | MIT | 3.2.0, 2026-01-20 | Alternativa com mais recursos/eventos |

Pontos: precisa de um **Application ID** registrado pelo autor no Discord (público, não é segredo); a comunicação é **local** (IPC com o cliente Discord instalado) — sem rede do Kazzu. Imagem grande só via **URL pública** ou asset cadastrado no app Discord: capa de arquivo local não pode ser mostrada sem hospedá-la ⇒ usar ícone genérico para faixas locais; para provedores, a URL da capa pública. Limite de atualização (~1 a cada 15 s **[verificar]**) ⇒ atualizar em troca de faixa/pausa. **Opt-in, desligado por padrão**; botão "ouvir junto" só se o usuário escolher expor. Risco legal: baixo (IPC é feature oficial do Discord).

## 4. MPRIS / controles de mídia

| Candidata | Link | Licença | Manutenção | Plataformas |
|---|---|---|---|---|
| `souvlaki` | [GitHub](https://github.com/Sinono3/souvlaki) | MIT | 0.9.3, 2026-10-27 | MPRIS (Linux), SMTC (Windows), MediaRemote (macOS). **Usada pelos apps Slint** Slix Player e p1mplayer |
| `mpris-server` | [GitHub](https://github.com/SeaDve/mpris-server) | MPL-2.0 | 0.11.0, 2026-10-26 | Só Linux, async (zbus), implementação mais completa do MPRIS |
| `mpris` | [GitHub](https://github.com/Mange/mpris-rs) | Apache-2.0 | 2.1.0 | Lado *cliente* (controlar outros players) |
| `zbus` / `ashpd` | — | MIT | ativas | base D-Bus; `ashpd` para portais Flatpak |

Recomendação: `souvlaki` atrás de um `trait MediaSession` no `kazzu-app` (metadados, capa por arquivo temporário/URL, posição e eventos play/pause/next/seek); trocar por `mpris-server` se faltarem recursos no Linux. **Android/iOS:** não coberto por essas crates — exige `MediaSession`/`MPNowPlayingInfoCenter` via JNI/ObjC (código nativo fino). Risco legal: nenhum.

## 5. Retrospectiva (o que armazenar localmente)

Tudo em SQLite local (mesmo do `kazzu-library`); **nunca** sai do dispositivo, exceto exportação manual ou o sync entre *dispositivos do próprio usuário*.

```
play_events(
  id INTEGER PK, started_at_utc INTEGER, local_tz_offset_min INTEGER,
  provider TEXT, track_id TEXT, mbid TEXT NULL, isrc TEXT NULL,
  title TEXT, artist TEXT, album TEXT, genre TEXT NULL,       -- snapshot (o provedor pode mudar/remover)
  duration_ms INTEGER, played_ms INTEGER, completed BOOL, skipped BOOL,
  context TEXT NULL,            -- 'album:…', 'playlist:…', 'radio', 'search'
  source_quality TEXT NULL,     -- 'flac-16-44', 'opus-160'… (para estatísticas de hi-fi)
  device_id TEXT                -- id local; permite somar dispositivos no sync
)
likes(provider, track_id, ts, state)
```

Agregações (calculadas sob demanda ou materializadas por ano): minutos totais, top artistas/faixas/álbuns/gêneros, "dia/hora de pico", maior sequência de dias, taxa de descoberta (artistas novos), ouvintes por provedor, % hi-res. Regras: contar como "play" ≥ 30 s ou ≥ 50 % (definir e documentar); em "ouvir junto" só o **host** registra, para não contar N vezes (padrão sugerido na discussão do app Namida); retenção configurável e botão "apagar tudo"; cartão final renderizado localmente como imagem. Scrobbling (ListenBrainz/Last.fm) é opt-in separado — a API do ListenBrainz é simples (`reqwest`); `listenbrainz` 0.8.1 (MIT, 2024-12) existe, `rustfm-scrobble` está parada desde 2021.

## Decisões que cabem ao autor

1. Aceitar um cliente Shazam não oficial (reconhecimento que funciona, mas ToS/GPL) ou limitar-se a reconhecimento local/AcoustID?
2. Licença do Kazzu define se crates GPL (autoeq, songrec) podem ser usadas — preferir reimplementar a lógica (biquad próprio, cliente próprio) em qualquer caso.
3. Discord: registrar um Application ID do projeto e aceitar a limitação de capas locais.

## Fontes (acessadas em 2026-11-21)

- AutoEq: https://github.com/jaakkopasanen/AutoEq ; https://lib.rs/crates/autoeq-iir ; https://github.com/ronitsingh10/FineTune/blob/main/guide/autoeq.md
- AcoustID: https://acoustid.org/webservice ; https://acoustid.biz/
- SongRec: https://crates.io/crates/songrec ; https://github.com/compensis/SongRec ; https://alternativeto.net/software/shazam/?license=opensource
- Apps Slint de áudio (só conceito): Janis https://github.com/obazin/janis ; Slix https://github.com/cetra3/slix-player ; p1mplayer https://github.com/Arian-p1/p1mplayer
- Sync/escuta em grupo (padrão de contabilização): https://github.com/namidaco/namida/issues/1134
- crates.io API: biquad, fundsp, dasp, rubato, ebur128, rusty-chromaprint, chromaprint-rust, discord-rich-presence, discord-presence, souvlaki, mpris-server, mpris, listenbrainz, rustfm-scrobble.
- Metrolist (só conceito): `.refs/metrolist/README.md`.
