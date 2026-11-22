# Slint — UI e mobile

Data: 2026-11-21. **[verificar]** = conhecimento prévio ou fonte única.

## ⚠️ Licença do Slint (destaque para o autor)

O Slint é **triplo-licenciado** (crate `slint` 1.19.0, 2026-11-16): `GPL-3.0-only` **OU** `LicenseRef-Slint-Royalty-free-2.0` **OU** `LicenseRef-Slint-Software-3.0` (comercial). Você escolhe uma:

| Licença | Para quê | Condições principais |
|---|---|---|
| **GPL-3.0** | Software livre em qualquer plataforma (inclui embarcado) | O app distribuído deve ser GPL-3.0 (compatível com código MIT/Apache embutido, mas o **binário** é GPL) |
| **Royalty-free 2.0** | Apps **proprietários** de desktop, mobile e web, sem custo | **Exige atribuição ao Slint** (badge em página pública ou "Sobre"/widget `AboutSlint`); **proibido em sistemas embarcados** |
| **Comercial (Software 3.0)** | Proprietário incl. embarcado | Pago; royalty por sistema embarcado |

**O que isso significa para o Kazzu** (decisão do autor):
- O brief fala em dependências "compatíveis com MIT/Apache". Com Slint, **ou o Kazzu inteiro é distribuído sob GPL-3.0**, **ou** usa-se a licença Royalty-free (que é para apps **proprietários**; se um app MIT de código aberto pode usá-la — não confirmei no texto; **ler o PDF da licença / perguntar à Slint**).
- **Sugestão:** adotar **GPL-3.0** para o Kazzu. Vantagens: licença do Slint simples e gratuita; destrava `rustypipe`, `songrec`, `qobuz-api` e `autoeq` (todas GPL-3.0; ver os outros docs); é coerente com o Metrolist como referência. Desvantagem: o app não pode virar produto proprietário/loja fechada sem rever; a App Store tem histórico de atrito com GPL (distribuição iOS) **[verificar]**.
- Se quiser permissivo (MIT/Apache) no *seu* código e ainda assim distribuir binários: o conjunto fica GPL-3.0 ou exige a Royalty-free com atribuição.

## Estado atual

- **Versão estável: 1.19.0 (2026-11-16)**. Novidades: `backdrop-blur`, **suporte a iOS**, rolagem com inércia nativa em iOS/Android, GIF/PNG/WebP animados, `slint::select_backend()`, arquivo de projeto `slint-project.json` e "editions".
- Plataformas: Linux, Windows, macOS (maduras); **Android** (já existente; ajustes de seleção de texto e árvore de acessibilidade na 1.19); **iOS (novo na 1.19 — tratar como experimental até provar estabilidade; o anúncio não dá rótulo "estável")**. Web/WASM também existe.
- Já há crates comunitárias de apps de música em Slint (abaixo), mas **nenhum exemplo oficial de player**.

## Renderers

| Renderer | Feature | Notas |
|---|---|---|
| **FemtoVG** (OpenGL) | `renderer-femtovg` | Padrão em desktop com winit; leve; suporta `backdrop-blur`, sombras internas e `drop-shadow-spread` |
| **Skia** | `renderer-skia` (Metal/Vulkan/D3D/GL), `renderer-skia-opengl`, `renderer-skia-software` | Melhor qualidade e efeitos; binário maior; **iOS: usar `renderer-skia` (Metal), `renderer-skia-opengl` não é suportado** |
| **Software** | `renderer-software` | Sem GPU (embarcado/fallback); **não** suporta `backdrop-blur` |
| Vello (experimental) | `renderer-vello` | Em evolução **[verificar]** |

Sugestão: Skia no desktop e mobile (efeitos de vidro mais confiáveis), FemtoVG como alternativa leve, software só como fallback de emergência (com visual degradado sem blur).

## Efeitos (glassmorphism)

- **`backdrop-blur` sobre `Rectangle` translúcido** existe desde a 1.19 (renderers Skia e FemtoVG). É o blur do que está **atrás dentro da janela**, não o do desktop (acrílico/Mica/vibrancy do SO).
- **Fundo derivado da capa** (referência Bitchord): robusto e barato → calcular em Rust uma versão reduzida e borrada da capa (crate `image`, ou cor dominante) e exibir como `Image` com camadas de gradiente; funciona em qualquer renderer, inclusive software.
- **Vidro do sistema** (janela translúcida + blur do desktop): requer transparência da janela e APIs específicas por plataforma (ex.: crate `window-vibrancy`, MIT/Apache **[verificar]**) acessando a janela do `winit`; não é nativo do Slint **[verificar nas features `unstable-winit-*`]**. Deixar como "extra" por plataforma.
- Sombras (`drop-shadow-*`), gradientes, bordas finas, `opacity`, clipping de cantos — presentes; para o visual Apple-like, usar bordas de 1 px translúcidas + sombras suaves.
- **Atenção a desempenho:** blur em tela cheia a 60 fps em GPU integrada fraca/Android barato → oferecer "reduzir transparência" nas configurações.

## Animações

Slint tem `animate <propriedade> { duration; easing; }` declarativo (easings `ease`, `ease-in-out`, `cubic-bezier(...)` **[verificar lista completa]**), `states`/`transitions`, `Timer` e animações dirigidas por propriedades; não há física de mola nativa (aproximar com `cubic-bezier`). **Movimento reduzido:** na 1.19 o Slint desliga animações automaticamente conforme a preferência do sistema, **no Windows** (e, segundo outra fonte, no Android) — nas demais plataformas, ler a preferência (D-Bus/portal `ashpd` no Linux, API do macOS) e expor uma propriedade global `reduce-motion` no design system.

## Acessibilidade

Desktop via **AccessKit** (leitores de tela Windows/macOS/Linux); Android tem árvore de acessibilidade (a 1.19 corrigiu crash/vazamento); iOS **[verificar]**. Boas práticas no design system: `accessible-role`, `accessible-label` em todos os controles (inclusive botões de ícone, como o de busca do Bitchord), foco visível, contraste AA, alvos ≥ 44 px, navegação por teclado, teste com Orca/NVDA/VoiceOver. Há bug corrigido na 1.19 em que "ações não funcionam em widgets desabilitados".

## Empacotamento

| Alvo | Ferramenta sugerida | Notas |
|---|---|---|
| Linux | **AppImage** (linuxdeploy / `cargo-packager` formato appimage), **Flatpak** (manifesto + `flatpak-cargo-generator`, runtime freedesktop; exige dependências vendorizadas, sem rede no build) | Flatpak dá sandbox (portais para microfone e MPRIS: lembrar `--talk-name=org.mpris.MediaPlayer2.*`) |
| Windows | `cargo-packager` (NSIS/MSI) ou `cargo-wix`; **MSIX** via `makeappx`/`msix-pack` **[verificar suporte do cargo-packager]** | Assinatura de código para evitar SmartScreen |
| macOS | `cargo-bundle` (0.12.0) ou `cargo-packager` (.app/.dmg); assinatura + notarização | Precisa conta Apple Developer (US$ 99/ano) |
| Android | `cargo-apk`/`xbuild`/`cargo-ndk` + backend `android-activity` do Slint | Áudio: cpal usa AAudio/Oboe; sessão de mídia e serviço em foreground via JNI |
| iOS | Projeto Xcode linkando a lib estática (Slint 1.19) | Áudio em segundo plano + `MPNowPlayingInfoCenter` via ObjC; App Store e GPL **[verificar]** |
| Releases | `cargo-dist` 0.34.0 (MIT/Apache) | Automatiza artefatos no CI |

`cargo-packager` 0.11.8 (2025-11, MIT/Apache), `cargo-bundle` 0.12.0 (2026-09).

## Exemplos reais de apps de mídia em Slint (todos de terceiros, pequenos — só referência de viabilidade; **não copiar código**)

| App | Link | Observação |
|---|---|---|
| Melodia | [GitHub](https://github.com/KenanSalar/Melodia) | UI na thread principal, runtime Tokio para DB/scan/watcher |
| Slix Player | [GitHub](https://github.com/cetra3/slix-player) | symphonia + rodio + souvlaki |
| p1mplayer | [GitHub](https://github.com/Arian-p1/p1mplayer) | MPRIS/SMTC |
| Janis | [GitHub](https://github.com/obazin/janis) | cpal + symphonia, 10 filtros peaking, sistema de design em 3 camadas |
| Lime Player | [GitHub](https://github.com/zilverjk/lime-player) | FLAC/WavPack/WAV/MP3 bit-perfect, NAS |
| Vivi | [GitHub](https://github.com/Loafer19/vivi) | Slint + libmpv, EQ, letras sincronizadas |
| n_music, Nano Music Player | [n_music](https://github.com/Enn3Developer/n_music), [Nano](https://github.com/huanzichen00/Nano-Music-Player) | multiplataforma / inspirado no iPod nano |

Padrão comum: `slint-build` em `build.rs`; áudio com `symphonia` + `cpal`/`rodio`; `souvlaki` para mídia do SO. Nenhum entrega glassmorphism avançado nem iOS — o Kazzu estaria na vanguarda de efeitos.

## Riscos técnicos para o autor

1. **iOS é novo no Slint** (1.19): planejar iOS por último; Android antes.
2. Blur: só em Skia/FemtoVG; fallback visual necessário.
3. Vidro do SO fora do Slint: custo por plataforma.
4. Tipografia/fontes: embutir uma fonte livre (OFL) para visual Apple-like consistente (SF é proprietária — **não** usar).
5. Binário Skia grande (dezenas de MB) — medir.

## Decisões que cabem ao autor

1. **Licença do Kazzu** (GPL-3.0 recomendada vs. Royalty-free com atribuição) — decide o resto das dependências.
2. Prioridade de plataformas (desktop → Android → iOS) e se publica em lojas (App Store/Play restringem provedores não oficiais e GPL).
3. Aceitar blur "dentro da janela" + fundo derivado da capa, ou investir em vidro nativo do SO por plataforma.

## Fontes (acessadas em 2026-11-21)

- Slint 1.19: https://slint.dev/blog/slint-1.19-released ; https://github.com/slint-ui/slint/discussions/14688
- Licença: https://github.com/slint-ui/slint/blob/master/FAQ.md ; https://github.com/slint-ui/slint/blob/master/LICENSE.md ; https://slint.dev/agreements/slint-royalty-free-license.pdf ; https://slint.dev/terms-and-conditions
- Features/renderers: https://docs.slint.dev/latest/docs/rust/slint/docs/cargo_features/
- crates.io API: slint, slint-build, i-slint-backend-winit (licença tripla confirmada), cargo-packager, cargo-bundle, cargo-dist.
- Apps em Slint: repositórios linkados acima (busca em 2026-11-21).
