# Dependências

Registro de dependências e licenças (todas compatíveis com MIT/Apache).

| Crate | Usada por | Licença | Uso |
|---|---|---|---|
| `lofty` 0.25 | kazzu-library | MIT OR Apache-2.0 | Tags e propriedades técnicas (flac/wav/opus/ogg/mp3/m4a/aac/aiff), Rust puro |
| `rusqlite` 0.43 (`bundled`) | kazzu-library | MIT (SQLite embutido: domínio público) | Banco SQLite + FTS5; `bundled` compila o SQLite junto (precisa de compilador C no build, sem libsqlite do sistema) |
| `image` 0.25 (jpeg, png, webp) | kazzu-library | MIT OR Apache-2.0 | Redimensionar capas, versão pré-desfocada e cor dominante |
| `walkdir` 2 | kazzu-library | Unlicense OR MIT | Percorrer pastas de música |
| `rayon` 1 | kazzu-library | MIT OR Apache-2.0 | Leitura de tags e capas em paralelo |
| `directories` 6 | kazzu-library | MIT OR Apache-2.0 | Diretório de dados XDG |
| `thiserror` 2 | kazzu-library | MIT OR Apache-2.0 | Erros |
| `hound` 3 (dev) | kazzu-library | Apache-2.0 | Gerar WAVs de teste |
| `tempfile` 3 (dev) | kazzu-library | MIT OR Apache-2.0 | Diretórios temporários nos testes |
| `serde` 1 (derive) | kazzu-core | MIT OR Apache-2.0 | Serialização dos tipos de domínio, comandos e eventos |
| `async-trait` 0.1 | kazzu-core | MIT OR Apache-2.0 | Trait `Provider` assíncrona e *object safe* (`Arc<dyn Provider>`) |
| `thiserror` 2 | kazzu-core | MIT OR Apache-2.0 | `kazzu_core::Error` |
| `serde_json` 1 (dev) | kazzu-core | MIT OR Apache-2.0 | Testes de serialização |
| `symphonia` 0.6 (`all`) | kazzu-audio | **MPL-2.0** | Demux e decode em Rust puro: FLAC, PCM (WAV/AIFF/CAF), ALAC, MP3, AAC, Vorbis, ADPCM; MP4, Ogg, MKV |
| `symphonia-adapter-libopus` 0.3 | kazzu-audio | MIT OR Apache-2.0 | Decoder Opus (libopus) registrado no symphonia; feature `opus` (padrão) |
| `opusic-sys` 0.7 (transitiva) | kazzu-audio | BSD-3-Clause (libopus: BSD) | Bindings da libopus |
| `cpal` 0.18 | kazzu-audio | Apache-2.0 | Saída de áudio multiplataforma (ALSA/PipeWire/JACK, WASAPI, CoreAudio) |
| `rubato` 5 (sem `fft_resampler`) | kazzu-audio | MIT OR Apache-2.0 | Resample sinc, **só** quando o dispositivo não aceita a taxa do arquivo |
| `ringbuf` 0.6 | kazzu-audio | MIT OR Apache-2.0 | Ring buffer SPSC lock-free entre o decode e o callback |
| `crossbeam-channel` 0.5 | kazzu-audio | MIT OR Apache-2.0 | Canais de comandos e eventos (sem tokio) |
| `log` 0.4 | kazzu-audio | MIT OR Apache-2.0 | Logs (fora do callback de áudio) |
| `hound` 3 (dev) | kazzu-audio | Apache-2.0 | Gerar WAVs de teste |
| `flacenc` 0.5 (dev) | kazzu-audio | Apache-2.0 | Gerar FLACs de teste (16/24 bit) |

## Notas de licença

- **symphonia é MPL-2.0**: é copyleft por arquivo. Usá-la sem modificação como dependência é compatível com um app MIT/Apache. Só modificações nos arquivos *dela* precisariam ser publicadas sob MPL. Não há alternativa Rust pura madura com a mesma cobertura de codecs.
- **libopus** (Xiph, BSD-3-Clause) é uma dependência nativa, aceita pela regra 4 do brief porque não há equivalente Rust puro maduro (ver abaixo).

## Opus: por que libopus via `symphonia-adapter-libopus`

O symphonia 0.6 ainda não tem decoder Opus próprio. Opções avaliadas em 21/11/2026 (dados do crates.io):

| Opção | Situação | Decisão |
|---|---|---|
| `symphonia-adapter-libopus` 0.3 | 14 versões, ~815 mil downloads, MIT/Apache; usa a **libopus de referência** (Xiph), compatível com symphonia 0.6, entra no `CodecRegistry`, então demux Ogg/WebM, seek e gapless continuam do symphonia | **Escolhida** |
| `symphonia-opus` 0.1 | Rust puro, mas publicada em 12/11/2026 (~2 mil downloads, 4 versões), MPL-2.0 | Imatura demais para a qualidade de referência; reavaliar quando amadurecer (seria uma troca local em `decoder.rs`) |
| `opus-decoder` / `opus-pure` / `opus1` | Rust puro, sem integração com o symphonia (precisariam de um adapter próprio) | Mais código nosso e menos testado que a libopus |

Linkagem:
- Feature `opus` (padrão): linka a libopus do **sistema** (pkg-config; `OPUS_LIB_DIR`/`OPUS_LIB_STATIC` para apontar outra). No Linux, a libopus é onipresente (Arch: pacote `opus`; Debian/Ubuntu: `libopus-dev` para compilar).
- Feature `opus-bundled`: compila e linka a libopus estaticamente a partir do fonte (precisa de **cmake** no build). Indicada para builds de release de Windows/macOS.
- Sem as features (`--no-default-features`), o resto funciona e `kazzu_audio::opus_supported()` retorna `false`.
