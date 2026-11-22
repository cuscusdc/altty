# Dependências

Registro de dependências e licenças. As bibliotecas de apoio são MIT/Apache-2.0 (ou equivalentes permissivos; exceções anotadas: symphonia é MPL-2.0, libopus é BSD-3-Clause). **A exceção que decide a licença do app é o Slint** (`slint`, `slint-build` e as crates `i-slint-*`): é GPL-3.0-only **ou** Royalty-free-2.0 **ou** comercial. Conforme a "Regra de custo" do `docs/BRIEF.md`, o Kazzu usa o Slint sob **GPL-3.0** (hipótese de trabalho, a confirmar pelo autor), o que faz o binário distribuído ser GPL-3.0. Esta tabela só registra; a decisão de licença do projeto é do autor.

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
| `slint` 1.19 (`std`, `backend-default`, `renderer-femtovg`, `renderer-software`, `accessibility`, `compat-1-2`) | kazzu-app | **GPL-3.0-only OU Royalty-free-2.0 OU Software-3.0** (ver `docs/research/ui-and-mobile.md`) | Interface. Renderer padrão FemtoVG (OpenGL; `backdrop-blur`); software como fallback sem blur. Skia fica atrás da feature `skia` |
| `slint-build` 1.19 | kazzu-app (build) | idem | Compila `ui/*.slint` no `build.rs` |
| `rfd` 0.15 (`xdg-portal`, `async-std`) | kazzu-app | MIT | Seletor de pasta nativo (portal do XDG no Linux; Windows/macOS usam o nativo) |
| `pollster` 0.4 | kazzu-app | Apache-2.0 OR MIT | Roda o `AsyncFileDialog` do `rfd` numa thread, sem runtime assíncrono |
| `toml` 0.9 | kazzu-app | MIT OR Apache-2.0 | Configuração do app (`config.toml`) |
| `env_logger` 0.11 | kazzu-app | MIT OR Apache-2.0 | Logs no terminal (`RUST_LOG`); nada é enviado |
| `serde` 1, `directories` 6, `image` 0.25 (jpeg, png), `crossbeam-channel` 0.5, `thiserror` 2, `log` 0.4 | kazzu-app | MIT OR Apache-2.0 | Configuração, capas e fundo desfocado, mensagens entre threads, erros e logs |
| `hound` 3, `flacenc` 0.5, `lofty` 0.25 (feature `demo`) | kazzu-app | Apache-2.0 / Apache-2.0 / MIT OR Apache-2.0 | Biblioteca de demonstração gerada por programa (`--demo-library`) |
| `tempfile` 3 (dev) | kazzu-app | MIT OR Apache-2.0 | Diretórios temporários nos testes |
| Hanken Grotesk, Geist Mono (fontes embutidas em `crates/kazzu-app/ui/assets/fonts`) | kazzu-app | SIL OFL 1.1 | Tipografia do app; textos de licença ao lado dos arquivos |

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
