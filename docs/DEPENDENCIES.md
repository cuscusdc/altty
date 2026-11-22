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
| `ureq` 3.5 (`default-features = false`, `rustls`) | kazzu-audio | MIT OR Apache-2.0 | Cliente HTTP(S) bloqueante do streaming (`StreamSource::Url`): requisições Range numa thread de rede própria, sem tokio. Ver "HTTP: por que ureq" abaixo |
| `rustls` 0.23 (transitiva, provider `ring`) | kazzu-audio | Apache-2.0 OR ISC OR MIT | TLS em Rust, sem OpenSSL |
| `ring` 0.17 (transitiva) | kazzu-audio | Apache-2.0 AND ISC | Criptografia do rustls (C/asm compilados pelo `cc`; funciona com o NDK do Android) |
| `rustls-webpki` 0.103, `rustls-pki-types` 1, `untrusted` 0.9 (transitivas) | kazzu-audio | ISC / MIT OR Apache-2.0 / ISC | Validação de certificados |
| `webpki-roots` 1 (transitiva) | kazzu-audio | CDLA-Permissive-2.0 | Certificados raiz da Mozilla embutidos (não depende do repositório de certificados do sistema, o que simplifica o Android) |
| `ureq-proto`, `http` 1, `httparse`, `base64`, `percent-encoding` (transitivas) | kazzu-audio | MIT OR Apache-2.0 | Protocolo HTTP/1.1 |
| `hound` 3 (dev) | kazzu-audio | Apache-2.0 | Gerar WAVs de teste |
| `flacenc` 0.5 (dev) | kazzu-audio | Apache-2.0 | Gerar FLACs de teste (16/24 bit) |
| Servidor HTTP de teste (`crates/kazzu-audio/tests/support/server.rs`) | kazzu-audio (testes) | nosso (licença do projeto) | Só `std::net`: Range, latência, quedas, 403, servidor sem Range, queda total. Escrito à mão em vez de `tiny_http` para controlar exatamente onde a conexão cai |
| `tests/fixtures/*.webm`, `*.m4a` | kazzu-audio (testes) | nossos (senos gerados com ffmpeg, ver o README da pasta) | WebM/Opus e MP4/AAC (comum e fragmentado) |
| `slint` 1.19 (`std`, `backend-default`, `renderer-femtovg`, `renderer-software`, `accessibility`, `compat-1-2`) | kazzu-app | **GPL-3.0-only OU Royalty-free-2.0 OU Software-3.0** (ver `docs/research/ui-and-mobile.md`) | Interface. Renderer padrão FemtoVG (OpenGL; `backdrop-blur`); software como fallback sem blur. Skia fica atrás da feature `skia` |
| `slint-build` 1.19 | kazzu-app (build) | idem | Compila `ui/*.slint` no `build.rs` |
| `rfd` 0.15 (`xdg-portal`, `async-std`) | kazzu-app | MIT | Seletor de pasta nativo (portal do XDG no Linux; Windows/macOS usam o nativo) |
| `pollster` 0.4 | kazzu-app | Apache-2.0 OR MIT | Roda o `AsyncFileDialog` do `rfd` numa thread, sem runtime assíncrono |
| `toml` 0.9 | kazzu-app | MIT OR Apache-2.0 | Configuração do app (`config.toml`) |
| `kazzu-ytm` (crate do workspace) | kazzu-app | MIT | Provedor padrão YouTube Music (sem conta): Início, busca, álbuns, rádio, streams, curtidas locais |
| `ureq` 3.5 (`rustls`) | kazzu-app | MIT OR Apache-2.0 | Baixa as capas (miniaturas) dos itens que o usuário vê, só `https` e só de hosts de imagem do YouTube/Google (ver `src/covers.rs`); mesma cópia do lockfile do kazzu-audio/kazzu-ytm |
| `sys-locale` 0.3 | kazzu-app | MIT OR Apache-2.0 | Idioma do sistema para as buscas temáticas do Início (Foco, Treino); mesma cópia do kazzu-ytm |
| `image` 0.25 (+ `webp`) | kazzu-app | MIT OR Apache-2.0 | Decodifica as capas remotas (JPEG/PNG/WebP) fora da thread de UI |
| `serde_json` 1 | kazzu-library | MIT OR Apache-2.0 | Guarda a faixa remota (por referência) em playlists locais |
| `env_logger` 0.11 | kazzu-app | MIT OR Apache-2.0 | Logs no terminal (`RUST_LOG`); nada é enviado |
| `serde` 1, `directories` 6, `image` 0.25 (jpeg, png), `crossbeam-channel` 0.5, `thiserror` 2, `log` 0.4 | kazzu-app | MIT OR Apache-2.0 | Configuração, capas e fundo desfocado, mensagens entre threads, erros e logs |
| `hound` 3, `flacenc` 0.5, `lofty` 0.25 (feature `demo`) | kazzu-app | Apache-2.0 / Apache-2.0 / MIT OR Apache-2.0 | Biblioteca de demonstração gerada por programa (`--demo-library`) |
| `tempfile` 3 (dev) | kazzu-app | MIT OR Apache-2.0 | Diretórios temporários nos testes |
| Hanken Grotesk, Geist Mono (fontes embutidas em `crates/kazzu-app/ui/assets/fonts`) | kazzu-app | SIL OFL 1.1 | Tipografia do app; textos de licença ao lado dos arquivos |
| `ureq` 3.5 (`default-features = false`, `rustls`, `gzip`) | kazzu-ytm | MIT OR Apache-2.0 | Mesmo cliente HTTP/TLS do kazzu-audio (uma cópia só no lockfile). `gzip` só para as respostas JSON do innertube (~8× menores); o googlevideo não comprime áudio, então o Range do streaming não é afetado |
| `flate2` 1, `miniz_oxide`, `crc32fast`, `adler2`, `simd-adler32` (transitivas do `gzip`) | kazzu-ytm | MIT OR Apache-2.0 (miniz_oxide: + Zlib; adler2: + 0BSD) | Descompressão gzip em Rust puro |
| `blocking` 1.7 (+ `async-channel`, `async-task`, `futures-lite`, `piper`, `event-listener`, `concurrent-queue`, `atomic-waker`, `fastrand`, `parking`, `pin-project-lite`, `futures-core`/`-io`, transitivas) | kazzu-ytm | Apache-2.0 OR MIT | Pool de threads que expõe as chamadas HTTP bloqueantes como `Future`, sem runtime (funciona com `pollster`, tokio ou smol) |
| `async-trait` 0.1, `serde` 1, `serde_json` 1, `thiserror` 2, `log` 0.4, `directories` 6 | kazzu-ytm | MIT OR Apache-2.0 | `impl Provider`, (de)serialização do innertube e do cache, erros tipados, logs, diretórios padrão no desktop |
| `sys-locale` 0.3 | kazzu-ytm | MIT OR Apache-2.0 | Idioma/país do sistema (Android, iOS, macOS, Windows, Linux) para `hl`/`gl` do YouTube; nunca `en`/`US` fixo |
| `kazzu-audio`, `pollster` 0.4, `tempfile` 3 (dev) | kazzu-ytm (exemplo/testes) | projeto / Apache-2.0 OR MIT / MIT OR Apache-2.0 | `examples/demo.rs` toca o stream resolvido; testes assíncronos e diretórios temporários |
| `tests/fixtures/*.json` | kazzu-ytm (testes) | respostas do innertube gravadas em 2026-11-22 e sanitizadas (sem IP, sem tracking, visitorData falso) | Testes offline dos parsers; metadados de catálogo do YouTube, só para teste, não redistribuídos no app |

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

## Android (alvo `aarch64-linux-android` e outros; ver `docs/ANDROID.md`)

Dependências que só entram quando `target_os = "android"` (tabela `[target.'cfg(target_os = "android")'.dependencies]` de `kazzu-app`) e ferramentas de build. Nada disso é baixado nem linkado no desktop.

| Item | Licença | Uso |
|---|---|---|
| `slint` 1.19, feature `backend-android-activity-06` | **GPL-3.0-only OU Royalty-free-2.0 OU Software-3.0** (usado sob GPL-3.0, como no desktop) | Backend Android do Slint (NativeActivity). Traz `i-slint-backend-android-activity`, `android-activity` 0.6 (MIT/Apache-2.0), `ndk` 0.9 (MIT/Apache-2.0) e `jni` 0.22 (MIT/Apache-2.0). O renderer no Android é **sempre Skia** (o backend não usa FemtoVG/software) |
| `skia-safe`/`skia-bindings` 0.157 (transitiva do backend) | MIT (Skia: BSD-3-Clause) | O `build.rs` baixa o Skia **pré-compilado** de `github.com/rust-skia/skia-binaries` (gratuito, sem conta). Esse binário foi compilado com o libc++ do **NDK r30**: com NDK r28/r29 o app não carrega (`__hash_memory` indefinido) |
| `android_logger` 0.15 | MIT OR Apache-2.0 | Logs no logcat (`adb logcat -s kazzu`); nada sai do aparelho |
| `jni` 0.21 | MIT OR Apache-2.0 | `checkSelfPermission`/`requestPermissions` (permissão de ler músicas) |
| `symphonia-adapter-libopus` 0.3 com `bundled` | MIT OR Apache-2.0 (libopus: BSD-3-Clause) | No Android não há libopus do sistema: a feature unifica `bundled` e a libopus é compilada junto (cmake do SDK + NDK) |
| libc++ do NDK (estática) | Apache-2.0 com exceção LLVM | Exigida pelo Skia; linkada dentro de `libkazzu_app.so` |
| Android SDK (cmdline-tools, platform-tools, platform 35, build-tools 35.0.1, cmake 3.31.6), NDK r30 (30.0.16248370) | Termos do Android SDK/NDK (gratuitos, Google); `apksigner`/`zipalign`/`aapt2`: Apache-2.0 | Só ferramentas de build, não vão no APK. Instalação oficial em `$ANDROID_HOME` (ver `docs/ANDROID.md`) |
| JDK 17+ | GPL-2.0 com Classpath Exception (OpenJDK) | `javac`/`d8` (helper de teclado do Slint, compilado no build) e `apksigner`/`keytool` |

`rfd` e `pollster` (seletor de pastas do desktop) ficam só na tabela `cfg(not(target_os = "android"))`; no Android não há seletor de pastas nesta versão.

## HTTP: por que ureq (streaming do kazzu-audio)

Avaliado em 22/11/2026 para `StreamSource::Url`:

| Opção | Situação | Decisão |
|---|---|---|
| `ureq` 3 + rustls (`ring`) | Bloqueante, sem runtime assíncrono, pouco código; TLS em Rust; o provider `ring` compila com o NDK do Android sem cmake | **Escolhida**. O mesmo cliente/versão é usado pelo provedor YT Music (`kazzu-ytm`), então o Cargo resolve uma cópia só |
| `reqwest` (`blocking`) | Sobe um runtime tokio escondido numa thread por cliente; mais dependências e binário maior | Descartada: o brief pede sem tokio no caminho de áudio |
| `rustls` com `aws-lc-rs` (padrão do rustls) | Exige cmake/NASM para compilar, mais atrito no Android/Windows | Descartada por ora (`ring` basta) |
| OpenSSL / native-tls | Dependência de sistema, difícil no Android | Descartada |

Features: `default-features = false, features = ["rustls"]` (= rustls + `ring` + webpki-roots). `gzip` fica desligado no áudio de propósito: compressão de transporte quebra as requisições Range. Um crate que precise de gzip (respostas JSON) pode ativá-lo no próprio `Cargo.toml`; o Cargo unifica as features.

Android: nenhuma dependência de sistema nova. O `ring` precisa do compilador C do NDK (o mesmo que o `cc` já usa para o libopus `bundled`); as raízes de certificado vêm embutidas (`webpki-roots`), então não é preciso JNI para ler o repositório de certificados do Android. O app precisa da permissão `android.permission.INTERNET` no manifesto.

