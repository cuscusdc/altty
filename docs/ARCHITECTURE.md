# Arquitetura

Visão geral das crates de base (`kazzu-core`, `kazzu-audio`). Biblioteca local: ver `docs/library.md`.

```
kazzu-app (Slint) ──▶ kazzu-library ──▶ kazzu-core
        │                                   ▲
        └────────────▶ kazzu-audio ─────────┘
```

`kazzu-core` não depende de nenhuma outra crate do projeto e não faz I/O; as demais dependem dele.

## kazzu-core — vocabulário comum

Tudo é reexportado na raiz (`use kazzu_core::*`). Os tipos têm `serde` (Serialize/Deserialize), para servir também à sincronização e ao controle remoto (etapa 4).

| Módulo | Tipos | Notas |
|---|---|---|
| `ids` | `TrackId`, `AlbumId`, `ArtistId`, `PlaylistId`, `QueueItemId` | Os quatro primeiros são newtypes opacos sobre `String` (cada provedor usa seu formato nativo; únicos *dentro* de um provedor). `QueueItemId(u64)` é único por entrada da fila (`QueueItemId::next()`). |
| `model` | `ProviderKind` {Local, Spotify, YtMusic, Tidal, Qobuz}, `Codec`, `AudioFormat`, `TrackSource`, `CoverRef`, `ArtistRef`, `AlbumRef`, `Track`, `Album`, `Artist`, `Playlist` | `TrackSource::Local{path}` ou `Remote{provider, id}`. `AudioFormat` guarda codec, taxa, profundidade, canais e bitrate (campos opcionais: provedores remotos nem sempre sabem tudo). `is_hi_res()` = lossless acima de 16 bit/48 kHz. |
| `provider` | trait `Provider`, `ProviderCapabilities`, `SearchQuery`/`SearchKind`/`SearchResults`, `LibrarySnapshot`, `StreamSource`, `StreamQuality` | `Provider` usa `async-trait` para ser *object safe* (`Arc<dyn Provider>` em um registro). Métodos: `kind`, `capabilities`, `search`, `resolve_stream`, `library`, `track`, `album_tracks`, `playlist_tracks`. Só a interface; implementações na etapa 2. Nenhuma implementação pode fazer chamada de rede não acionada pelo usuário. |
| `playback` | `QueueItem`, `PlaybackCommand`, `PlaybackEvent`, `PlaybackState`, `PlaybackStatus`, `RepeatMode`, `OutputInfo` | `QueueItem = Track + StreamSource` (`QueueItem::local(track)` para faixas locais). Comandos e eventos são enums com tag (`cmd`/`event`) em JSON, prontos para trafegar na rede depois. |
| `error` | `Error`, `Result` | `thiserror`, `#[non_exhaustive]`. |

Por que ids como `String` e não `i64`: o SQLite da biblioteca pode usar o rowid como texto, e os provedores remotos têm ids textuais (URIs do Spotify, video ids do YouTube). Um único tipo evita conversões na UI.

## kazzu-audio — motor de reprodução

### Threads e fluxo de dados

```
 UI ──PlaybackCommand──▶ thread "kazzu-engine" ──f32 intercalado──▶ ring SPSC ──▶ callback do dispositivo
    ◀──PlaybackEvent────  fila, decode (symphonia),               (ringbuf,      (Renderer: volume,
                          resample? (rubato), DSP                   lock-free)     conversão de formato)
```

- **Thread do motor** (`kazzu-engine`): recebe comandos (crossbeam), mantém a fila, decodifica e empurra amostras no ring. Também detecta as fronteiras entre faixas e emite eventos. Não usa tokio. Quando o ring está cheio, dorme em `recv_timeout(5 ms)`, então os comandos são atendidos rápido.
- **Callback de áudio** (`Renderer::render`): só lê do ring, aplica o ganho (pulado quando volume = 1,0) e converte para o formato do dispositivo. Não aloca, não trava, não loga. Toda a comunicação passa por atômicos (`Shared`) e pelo ring.
- O ring guarda cerca de 500 ms (`EngineConfig::buffer`).

### Bit-perfect (regra 5 do brief)

1. O dispositivo é aberto **na taxa e no número de canais nativos do arquivo**. Ao mudar de formato entre faixas, o dispositivo é reaberto.
2. O formato de amostra é escolhido pela profundidade da fonte: 16 bit → I16 > I32 > I24 > F32; 24/32 bit → I32 > I24 > F32 > I16; lossy → F32 primeiro.
3. O caminho interno é `f32`. As conversões int→f32 (symphonia, `x / 2^(n-1)`) e f32→int (nossas, `round(x · 2^(n-1))`) são inversas exatas para fontes de até 24 bit, então 16/24 bit com volume 1,0 chegam ao dispositivo **bit a bit iguais**. Há teste para FLAC 24/96 e WAV 16/44,1. Fontes inteiras de 32 bit (raras) ficam limitadas à precisão de 24 bit.
4. **Resample só se o dispositivo recusar a taxa** (rubato, sinc 256 taps, Blackman-Harris², com o atraso do filtro compensado para não quebrar o gapless). Nesse caso, `OutputInfo.bit_perfect = false` e `resampled_from = Some(taxa)`.
5. DSP padrão = `Bypass`; volume ≠ 1,0, DSP ativo ou resample → `bit_perfect = false`. O evento `OutputChanged` informa à UI (selo "bit-perfect" / "reamostrado 96 → 48 kHz").
6. Atenção, no Linux: o dispositivo `default` normalmente passa pelo PipeWire/Pulse, que pode reamostrar e mixar por conta própria. Para um caminho realmente bit-perfect, escolha um dispositivo `hw:`/`front:` (`CpalSink::with_device`, lista em `CpalSink::devices()`).

### Gapless

Quando o decoder de uma faixa chega ao fim, o motor abre a próxima da fila na hora e continua escrevendo no **mesmo** ring. Com 500 ms de folga, o dispositivo não vê lacuna. Cada faixa ocupa um "segmento" (intervalo de frames no ring). A posição e o fim de faixa vêm da comparação com o contador de frames consumidos pelo callback, que é atômico.

- Delay e padding do encoder (MP3/AAC/Opus) são removidos pelo symphonia (`AudioDecoderOptions::gapless`, padrão).
- O gapless vale entre faixas de mesmo formato (taxa e canais). Se a próxima faixa exigir outro formato, o motor espera o ring esvaziar e reabre o dispositivo. Essa transição tem uma pausa mínima, inevitável em modo bit-perfect.
- Se a fila muda depois que a próxima faixa já foi pré-decodificada (PlayNext, Remove, shuffle, repeat), o motor descarta o que foi decodificado adiante e retoma a faixa atual **exatamente no frame que estava tocando**. O callback registra quantos frames de fato tocou no momento do flush (`flushed_at`), então nada é repetido nem pulado.
- Enfileirar quando a última faixa já terminou de decodificar, mas ainda está tocando, também continua sem lacuna.

### Seek preciso

`SeekMode::Accurate` do symphonia posiciona antes do alvo, e o decoder descarta os frames até o timestamp exato. O ring é esvaziado com um handshake por atômicos (`flush_req`/`flush_ack`, processado pelo callback mesmo em pausa). Testado com WAV e FLAC: o primeiro frame após o seek é exatamente `floor(t · taxa)`. As conversões frame↔`Duration` arredondam para que a ida e volta seja exata.

### Fila

`Queue` (lógica pura, testada isoladamente): itens, ordem de reprodução (shuffle por Fisher–Yates com a faixa atual primeiro, desfeito ao desligar), `RepeatMode` {Off, One, All}. "Próxima" manual ignora o repeat-one. "Anterior" reinicia a faixa se a posição passou de 3 s. Itens que não abrem geram `PlaybackEvent::Error` e são pulados.

### Saídas (`AudioSink`)

- `CpalSink`: placa de som (ALSA/PipeWire/JACK no Linux, WASAPI, CoreAudio). Abre na negociação (`open(request, make_renderer)`), detecta perda do dispositivo e reabre.
- `NullSink`: sem dispositivo, para testes/CI. Uma thread simula o callback a `speed`× o tempo real, pode restringir as taxas aceitas (para testar o resample) e pode gravar a saída (`Capture`) para verificar exatidão. O mutex da captura é aceitável só porque essa thread não é um callback real.

### Estágio de DSP (etapa 3)

Trait `Dsp { configure, process, reset, is_bypassed }`. Roda na thread do motor, depois do resample e antes do ring: não precisa ser realtime-safe, mas a mudança de parâmetros só se ouve depois da latência do ring (~0,5 s; reduzível via `EngineConfig::buffer`). `DspChain` encadeia estágios. Troca em execução: `EngineHandle::set_dsp`.

### API pública

```rust
let engine = Engine::spawn()?;                              // cpal, dispositivo padrão
let engine = Engine::builder().sink(NullSink::new()).dsp(Bypass)
    .config(EngineConfig { volume: 1.0, ..Default::default() }).spawn()?;
engine.load(items, 0); engine.enqueue(items); engine.play_next(items);
engine.play(); engine.pause(); engine.toggle_pause(); engine.stop();
engine.seek(Duration::from_secs(30)); engine.set_volume(0.8);
engine.next(); engine.previous(); engine.jump_to(id); engine.remove(id); engine.clear();
engine.set_repeat(RepeatMode::All); engine.set_shuffle(true);
engine.send(PlaybackCommand::...);                          // equivalente genérico
engine.events();                                            // crossbeam Receiver<PlaybackEvent>
engine.state();                                             // Option<PlaybackState> (snapshot)
engine.set_dsp(meu_eq);
```

Streaming (ver abaixo): `engine.stream_events()` (`Receiver<StreamEvent>`), `engine.replace_stream(id, stream)`, `engine.cache()`.

Eventos: `StatusChanged`, `TrackChanged`, `Position` (~10 Hz tocando), `TrackEnded` (só fim natural), `QueueChanged`, `VolumeChanged`, `RepeatChanged`, `ShuffleChanged`, `OutputChanged`, `QueueFinished`, `Error`. O canal de eventos é limitado (4096): se ninguém ler, os eventos são descartados em vez de acumular memória.

Exemplo manual em placa real: `cargo run -p kazzu-audio --example play -- arquivo.flac outro.opus` (`--list` lista dispositivos; `KAZZU_DEVICE=<id>` escolhe um).

### Streaming HTTP (`StreamSource::Url`)

```
 thread "kazzu-net" (1 por stream)   thread "kazzu-stream" (1 por stream)     thread "kazzu-engine"
 GET Range: bytes=a-b (ureq) ──▶ blocos de 64 KiB ──▶ HttpSource (Read+Seek) ──▶ symphonia ──▶ PCM ──canal limitado──▶ Decoding
 retry/backoff, limite de banda    (memória limitada)    bloqueia só esta thread                       try_recv: nunca bloqueia
 grava o .part do cache
```

- **O motor nunca toca na rede.** Abrir, sondar o formato, fazer seek e decodificar um stream acontecem na thread do stream; o motor só faz `try_recv` num canal (cerca de 3 s de PCM decodificado à frente). O callback continua lendo só o ring. Teste: com 1,5 s de latência no servidor, `engine.state()` responde em menos de 200 ms.
- **Requisições Range limitadas** (`StreamConfig::request_chunk`, padrão 4 MiB, nunca abertas): o googlevideo estrangula pedidos sem limite, e uma queda custa no máximo um pedaço. Os `headers` do `StreamSource::Url` vão em todas as requisições (o YT Music exige o mesmo User-Agent do cliente innertube).
- **Leitura adiante** de até `read_ahead` (16 MiB) à frente do decoder, com `memory_limit` (32 MiB) por stream: blocos longe da posição de leitura saem primeiro e são baixados de novo se o usuário voltar.
- **Seek remoto**: o leitor diz ao downloader onde está; se o alvo estiver fora do que foi baixado, a conexão atual é largada e um novo Range começa no bloco do alvo. Alvos até 512 KiB à frente são alcançados lendo a conexão aberta. Servidor que ignora Range (responde 200): o arquivo é lido em sequência e o seek para frente espera chegar lá.
- **Quedas de conexão**: retomadas a partir do último byte recebido, com backoff exponencial (`retry_base` 250 ms, dobrando até `retry_max` 8 s, `max_retries` 8 tentativas seguidas sem nenhum byte). `read_timeout` (20 s) limita o recebimento do cabeçalho e de cada pedaço; um pedaço lento é retomado, não recomeçado. 429/5xx/timeouts são repetidos; 404 e outros 4xx viram erro e a faixa é pulada.
- **Formato**: dica pelo Content-Type (`audio/webm` → webm, `audio/mp4` → m4a etc.), pela extensão da URL e por `StreamSource::Url::format`; o symphonia também reconhece os bytes mágicos, então a dica só ordena as tentativas.
- **Estados**: ao abrir um stream (ou quando a rede fica para trás com o ring abaixo de 20%), o status vira `Buffering` e o dispositivo é pausado sem perder amostras; volta a `Playing` com o ring acima de 60% (ou no fim da faixa). Pausar durante o `Buffering` guarda a intenção (`Paused`); `play()` volta a `Buffering` até haver dados. Nada é descartado: o resultado é bit a bit igual ao arquivo (testado com WAV via HTTP, com e sem quedas).
- **Gapless remoto**: `StreamConfig::prebuffer` (15 s) antes do fim da faixa atual, o motor já abre a próxima faixa remota (sonda, seek, decodifica à frente); a junção usa o mesmo mecanismo de segmentos dos arquivos locais. Se a próxima ainda não abriu quando a atual acaba, o dispositivo espera em `Buffering` com o finzinho da faixa anterior no ring, e a junção continua sem lacuna nas amostras.
- **Cache de disco opcional** (`StreamConfig::cache = Some(CacheConfig { dir, max_bytes })`, base do offline): enquanto baixa, o stream é gravado em `<chave>.part`; quando todos os bytes chegaram, vira `<chave>.<ext>` e os arquivos menos usados recentemente (mtime, atualizado a cada uso) são apagados até caber em `max_bytes`. Lacunas deixadas por seeks são completadas em segundo plano enquanto a faixa toca. A chave é o `TrackSource::Remote { provider, id }` da faixa (a URL assinada muda a cada resolve). Uma faixa no cache abre como arquivo local, **sem nenhuma requisição**. `StreamCache::cached(&track)` deixa o controller tocar offline sem chamar `resolve_stream`. Arquivos maiores que metade do limite não entram.
- **Limite de banda** (`max_bandwidth`, bytes/s): aplicado na thread de rede; o TCP segura o servidor.
- **Sem telemetria**: só há requisição para a faixa que o usuário mandou tocar e para a próxima da fila dele (pré-buffer). Nenhum HEAD de verificação, nenhuma chamada de fundo.

#### Contrato de URL expirada (`StreamEvent`)

`StreamEvent` é um canal separado (`engine.stream_events()`), de propósito fora de `PlaybackEvent`: o `match` do controller sobre `PlaybackEvent` é exaustivo, e esses eventos são locais (não trafegam para controle remoto).

1. Uma resposta 401/403/410 (URL assinada vencida) gera `StreamEvent::Expired { id, position }`. O áudio já decodificado continua tocando; quando acaba, o status fica `Buffering`.
2. O controller chama `Provider::resolve_stream(&item.track, quality)` de novo (fora da thread da UI) e entrega o resultado com `engine.replace_stream(id, novo_stream)`.
3. O motor retoma em `position`: no meio da faixa, abre a URL nova a partir dali e continua no **mesmo** segmento, sem flush (gapless: o teste confere a saída bit a bit). Vale também para a próxima faixa (pré-buffer) e para uma faixa que já falhou ao abrir.
4. Sem resposta em `expired_wait` (30 s), ou se a URL nova falhar de novo no mesmo ponto, a faixa recebe `PlaybackEvent::Error` e é pulada.

`StreamEvent::NetworkFailed { id, position, message }` (e um `PlaybackEvent::Error`) sai quando as tentativas se esgotam: o áudio no buffer termina, o status vira `Paused`, e `play()` tenta de novo a partir de `position` com a mesma URL ("Sem conexão, toque para tentar de novo").

```rust
// no controller (pseudocódigo)
for ev in engine.stream_events().try_iter() {
    if let StreamEvent::Expired { id, .. } = ev {
        let item = fila.get(id);
        let novo = provider.resolve_stream(&item.track, quality).await?;
        engine.replace_stream(id, novo);
    }
}
```

#### Limites conhecidos

- Editar a fila (PlayNext/Remove/shuffle) nos últimos segundos de uma faixa remota, depois que a próxima já foi pré-decodificada, reabre a faixa atual pela rede na posição exata (o mesmo mecanismo dos arquivos locais); pode haver um breve `Buffering`. Com o cache, a reabertura é local.
- O cache só guarda arquivos completos; uma faixa interrompida (pulada no meio) não entra.
- Sem HTTP/2 nem conexões persistentes entre pedaços quando o servidor fecha a conexão (o `ureq` reaproveita conexões keep-alive quando o servidor permite).
- Sem seleção adaptativa de qualidade: o provedor escolhe a URL.

### Formatos

FLAC, WAV/AIFF/CAF (PCM), ALAC, MP3 (e MP1/MP2), AAC (M4A/MP4), Vorbis, ADPCM via symphonia; Opus via libopus (feature `opus`, padrão). Contêineres: MP4/M4A, Ogg, Matroska/WebM, WAV, AIFF, CAF. Verificados pelo caminho cpal real (dispositivo `alsa:null`): FLAC 24/96, Opus (Ogg), MP3, AAC (M4A), Vorbis e ALAC. WAV e FLAC também são cobertos pelos testes automáticos. WebM/Opus (itag 251 do YouTube) e MP4/AAC comum e **fragmentado** (DASH, itag 140) têm testes automáticos (`tests/formats.rs`, `tests/streaming.rs`), por arquivo e por HTTP. AIFF, CAF e ADPCM ainda não foram testados.

MP4 fragmentado declara só a duração do primeiro fragmento. `TrackDecoder::reconcile_duration` usa a duração dos metadados da faixa (`Track.duration`, vinda do provedor) quando ela é bem maior, o que também libera o seek depois do primeiro fragmento. Os provedores devem preencher `Track.duration`.

### Pendências conhecidas

- Streaming: ver "Limites conhecidos" em "Streaming HTTP" acima.
- Na reabertura por mudança de formato, os últimos milissegundos no buffer interno do dispositivo podem ser cortados (esperar o *drain* do cpal).
- A latência do dispositivo não é descontada da posição reportada (dá para usar os timestamps de `OutputCallbackInfo`).
- Modo exclusivo (WASAPI exclusive / ALSA `hw:` automático) e seleção de dispositivo na UI.
- Arquivos gerados pelo `flacenc` 0.5 com o último bloco curto são rejeitados pelo symphonia (o `flac -t` também avisa). Arquivos do libFLAC funcionam. Nos testes, os FLACs têm comprimento múltiplo de 4096.
