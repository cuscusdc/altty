# Kazzu no Android

Pipeline para compilar, assinar e publicar o APK do Kazzu **sem Android Studio, sem Gradle e sem nenhum serviço pago ou do Google**. A partir da v0.2 a UI do APK é a **v2 mobile-first** (vidro real, YouTube Music padrão; `docs/UI_NOTES.md`).

Estado: **alpha**. Compila, assina, instala e abre num emulador Android 14 (x86_64, KVM). **Não foi testado em aparelho real** (nem em ARM); ver "Verificação".

## Decisões

| Tema | Escolha | Por quê |
|---|---|---|
| Backend de UI | `slint` com feature `backend-android-activity-06` (NativeActivity) | É o backend oficial do Slint para Android; documentado e mantido pelo próprio projeto. **Não** usa winit; o renderer é sempre **Skia** (conferido no código do backend; as features `renderer-femtovg`/`renderer-software` do `Cargo.toml` são inertes lá), que tem `backdrop-blur` real |
| Atividade | `android.app.NativeActivity` do sistema (`hasCode="false"`) | Sem código Java/Kotlin próprio → sem Gradle. O Slint compila um pequeno helper Java (teclado/IME) no build e o embute em `libkazzu_app.so` |
| Empacotamento | `scripts/build-apk.sh`: `cargo rustc --crate-type cdylib` + `aapt2` + `zipalign` + `apksigner` | Só usa ferramentas oficiais do SDK/NDK. `cargo-apk2`/`xbuild`/`cargo-ndk`+Gradle também funcionam com o Slint (a doc do Slint lista os quatro), mas cada um é mais uma ferramenta/versão a manter; o script tem ~120 linhas e roda igual no CI. O crate segue `lib` no desktop: o `cdylib` é pedido só na linha de comando do Android |
| Áudio | `cpal` 0.18, backend **AAudio** nativo | O cpal 0.18 tem backend AAudio próprio (`cpal::host::aaudio`, via `ndk`); **não usa Oboe**, então não precisa de libc++ separada nem de C++ extra. API mínima 26 (Android 8) |
| Opus | libopus compilada junto (`symphonia-adapter-libopus` com `bundled`, ativado só no Android em `kazzu-app/Cargo.toml`) | No Android não há libopus do sistema. Precisa de cmake+ninja (os do SDK) |
| Seletor de pastas | **Sem seletor** nesta versão. O app lê a pasta **`Music`** do armazenamento compartilhado com a permissão `READ_MEDIA_AUDIO` (Android 13+) / `READ_EXTERNAL_STORAGE` (Android 8–12), pedida em tempo de execução via JNI. O botão "Adicionar pasta" adiciona/reescaneia `Music` | O seletor do sistema (SAF, `ACTION_OPEN_DOCUMENT_TREE`) devolve URIs de conteúdo, não caminhos: exigiria trocar o scan por `ContentResolver`/MediaStore. A permissão de mídia dá leitura direta por caminho nos arquivos de áudio, que é o que o scan (`walkdir` + `lofty`) já faz. `rfd` fica só no desktop |
| Dados do app | Banco e configuração em `files/` do app (armazenamento interno, sem permissão). `directories` não funciona no Android, então o `android_main` passa os caminhos em `Options` | Nada da lógica do app precisou mudar |
| Logs | `android_logger` → `adb logcat -s kazzu` | Nada sai do aparelho |
| `targetSdk` | 34 (`minSdk` 26) | A UI v2 já usa as áreas seguras (`Window.safe-area-insets` do backend Slint → `AppState.safe-*`); dá para subir para 35 depois de testar em Android 15 |
| ABIs | `arm64-v8a` (padrão, publicado). `x86_64` só para emulador. `armeabi-v7a` compila mas não é publicado | Celulares atuais são arm64; um APK universal triplicaria o tamanho (o Skia pesa ~20 MB por ABI) |

### Permissões (todas declaradas em `android/AndroidManifest.xml`)

| Permissão | Para quê |
|---|---|
| `INTERNET` | YouTube Music (provedor padrão, sem conta) e as capas dos itens vistos. Só há rede na abertura do Início e por ação do usuário |
| `READ_MEDIA_AUDIO` | Ler as músicas locais (Android 13+) |
| `READ_EXTERNAL_STORAGE` (`maxSdkVersion=32`) | Idem, Android 8 a 12 |

Nada mais: sem serviços Google/Firebase/analytics, sem `WAKE_LOCK`, sem notificações, sem serviço em foreground (ainda; ver Pendências).

## Ambiente de build (o que foi instalado)

Tudo em `/home/sacer/.local/share/android-sdk` (`ANDROID_HOME`), baixado de `dl.google.com`, sem custo. Instalação, a partir do zero:

```bash
export ANDROID_HOME=$HOME/.local/share/android-sdk
mkdir -p $ANDROID_HOME/cmdline-tools && cd /tmp
curl -O https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip   # sha1 e025545c62a8e64c7559119566a569fb1dec5f60
unzip commandlinetools-linux-16111833_latest.zip && mv cmdline-tools $ANDROID_HOME/cmdline-tools/latest
yes | $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --sdk_root=$ANDROID_HOME --licenses
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --sdk_root=$ANDROID_HOME \
  "platform-tools" "platforms;android-35" "build-tools;35.0.1" "ndk;30.0.16248370" "cmake;3.31.6"
# opcional, para testar sem aparelho:
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --sdk_root=$ANDROID_HOME "emulator" "system-images;android-34;default;x86_64"
rustup target add aarch64-linux-android x86_64-linux-android armv7-linux-androideabi
```

- **NDK r30 (30.0.16248370) é obrigatório.** O Skia pré-compilado que o `skia-bindings` baixa foi compilado com o libc++ do r30; com o NDK r28 ou r29 o APK compila, mas **não abre** (`dlopen failed: cannot locate symbol "_ZNSt6__ndk113__hash_memoryEPKvm"`). Foi o que aconteceu na primeira tentativa.
- JDK: serve o do sistema (testado com **OpenJDK 25** localmente e 21 no CI). Com SDK build-tools 35+ o `d8` aceita JDK 21+; com build-tools antigas, usar JDK 17.
- Android Studio **não** é necessário. O `rustup` do sistema deste computador (`/usr/lib/rustup`) deixou instalar os alvos Android normalmente (toolchain em `/root/.rustup`).
- Licenças do SDK: aceitas com `sdkmanager --licenses`, como o autor pediu; os textos estão em `$ANDROID_HOME/licenses`.

## Compilar o APK

```bash
export ANDROID_HOME=$HOME/.local/share/android-sdk
export CARGO_TARGET_DIR=$PWD/target/android      # (padrão do script) não disputa com os builds do desktop
scripts/build-apk.sh                              # arm64-v8a, assinado com a chave de release
scripts/build-apk.sh --abi x86_64 --debug-sign    # para o emulador, com chave descartável
scripts/build-apk.sh --version 0.1.0-alpha.2
# saída: target/android/apk/kazzu-<versão>-<abi>.apk  e  .apk.sha256
```

O script define o `CC`/linker do NDK por ABI, roda `cargo rustc -p kazzu-app --lib --release --no-default-features --crate-type cdylib --target <alvo>` (sem a feature `demo`), remove símbolos, empacota com `aapt2` (manifesto + ícones de `android/res`), grava `libkazzu_app.so` **sem compressão e alinhada a 16 KB** (exigência do Android 15+ em aparelhos com página de 16 KB; o NDK r28+ já liga com esse alinhamento), roda `zipalign` e assina (esquemas v2/v3) com `apksigner`. O `versionCode` vem da versão (`0.1.0-alpha.1` → 10001; versão final tem número maior que a alpha).

Os ícones (`android/res/**`) são gerados de `design/assets/` por `android/gen-icons.sh` e versionados; ícone adaptativo (marca "K" sobre fundo escuro) + ícone monocromático (temas do Android 13+) + PNG legado.

## Assinatura e backup da chave (**leia**)

O APK de release é assinado com uma chave **nova** gerada para o Kazzu:

- keystore: `~/.android/kazzu-release.jks` (PKCS12, RSA 4096, alias `kazzu`, validade de ~100 anos)
- senha: `~/.android/kazzu-release.pass` (arquivo `chmod 600`, fora do git; o script nunca a imprime)

**Essa chave é insubstituível.** O Android só deixa atualizar um app por cima se o novo APK tiver a **mesma** assinatura: perdendo a keystore, ninguém consegue atualizar o Kazzu instalado (só desinstalar e instalar de novo, perdendo os dados). Nunca vai para o git (nem para este repositório, mesmo privado). **Faça backup agora**, por exemplo:

1. copie os dois arquivos para um pendrive/HD externo e/ou para um gerenciador de senhas (anexo seguro, ex.: KeePassXC, gratuito);
2. guarde uma cópia em outro lugar que não seja este SSD (se o SSD falhar, a chave some junto);
3. opcional para o CI: `base64 -w0 ~/.android/kazzu-release.jks | gh secret set KAZZU_KEYSTORE_B64 -R cuscusdc/altty` e `gh secret set KAZZU_KEYSTORE_PASSWORD -R cuscusdc/altty < ~/.android/kazzu-release.pass` (o GitHub guarda cifrado e nunca mostra de volta; não substitui o backup).

Para conferir a impressão digital da chave (sem expor a senha): `keytool -list -v -keystore ~/.android/kazzu-release.jks -storepass:file ~/.android/kazzu-release.pass | grep SHA256`.

## CI (GitHub Actions, gratuito)

`.github/workflows/android.yml`:

- push de tag `v*` → compila, assina e anexa `kazzu-<versão>-arm64-v8a.apk` + `.sha256` à release da tag (prerelease se a tag tiver `-`);
- pull request / execução manual → só compila (artefato do workflow, chave descartável).

Sem os segredos `KAZZU_KEYSTORE_B64` e `KAZZU_KEYSTORE_PASSWORD` o CI assina com chave descartável e **não publica**, de propósito (APK com outra chave não atualiza o instalado). Repositório privado usa os minutos do plano gratuito (2000/mês); um build leva ~10–15 min com cache.

## Publicar uma versão

```bash
scripts/build-apk.sh --version 0.1.0-alpha.2
gh release create v0.1.0-alpha.2 --prerelease --title "Kazzu 0.1.0-alpha.2" --notes-file NOTAS.md \
   target/android/apk/kazzu-0.1.0-alpha.2-arm64-v8a.apk target/android/apk/kazzu-0.1.0-alpha.2-arm64-v8a.apk.sha256
```

(ou só empurrar a tag e deixar o CI fazer, com os segredos cadastrados). O repositório é **privado**: o link da release só abre logado numa conta com acesso (no celular, entre no GitHub pelo navegador antes de baixar o APK).

## Verificação (v0.1.0-alpha.1)

**Emulador** (AVD `kazzu34`: Android 14 / API 34, imagem AOSP x86_64, KVM, GPU por software `swiftshader_indirect`), com o APK `x86_64` (mesmo código do arm64):

- instala, abre e renderiza a UI com Skia (vidro/blur incluso); captura de tela da tela inicial e do estado com músicas feita via `adb exec-out screencap`;
- pede a permissão "acessar músicas e áudio" na primeira execução; ao permitir (evento `Resume`), indexa `/sdcard/Music` (2 álbuns FLAC enviados por `adb push`: 7 faixas, tags e formato "FLAC 16/44,1" corretos);
- reprodução: o stream AAudio abre e inicia (`AAudioStreamBuilder_openStream() returns 0 = AAUDIO_OK`), a barra de progresso e o mini-player avançam. **Não se ouviu o som** (emulador sem áudio, `-no-audio`);
- ícone adaptativo aparece corretamente na tela de recentes.

**Estático, no APK `arm64-v8a` final** (`kazzu-0.1.0-alpha.1-arm64-v8a.apk`, 25.225.519 bytes):

- `aapt2 dump badging`: pacote `app.kazzu.player`, `versionName 0.1.0-alpha.1`, `minSdk 26`, `targetSdk 34`, `native-code: arm64-v8a`, permissões **só** `INTERNET`, `READ_MEDIA_AUDIO`, `READ_EXTERNAL_STORAGE (maxSdk 32)`;
- `apksigner verify`: assinatura v2 e v3 válidas, 1 signatário (`CN=Kazzu`, RSA 4096, certificado SHA-256 `6358e521…2e92`);
- `zipalign -c -P 16`: OK; `llvm-readelf`: `AArch64`, segmentos `LOAD` com alinhamento 0x4000 (16 KB), `NEEDED` só `liblog, libandroid, libaaudio, libdl, libEGL, libGLESv2, libm, libc` (libc++ e libopus estáticas dentro da lib), símbolos `android_main`/`ANativeActivity_onCreate` exportados, nenhum símbolo `__hash_memory` indefinido;
- o desktop continua compilando: `cargo check -p kazzu-app --all-targets` (Linux) passa com as mudanças de `Cargo.toml`/`cfg`.

**NÃO testado:** aparelho físico (nenhum, ARM ou não); o APK arm64 em si nunca foi executado (o emulador x86_64 não roda código arm64); som real, fones/Bluetooth, rotação, bloqueio de tela, Android 15+ (páginas de 16 KB), desempenho do blur em GPU real.

**Problemas da alpha.1 e o que foi feito na v0.2:**
- desenhava **por baixo** das barras do sistema: a UI v2 usa `safe-area-insets` (o backend Android do Slint os preenche sozinho) e tem fade superior/inferior; verificado no emulador;
- capas em branco: **causa encontrada**, não era render. Com a permissão só de áudio o Android não deixa ler `cover.jpg` da pasta; capas embutidas na tag aparecem. Capa ausente agora mostra o placeholder `disc`. Pedir `READ_MEDIA_IMAGES` fica como decisão do autor;
- "Voltar" não fechava player/detalhe: agora o `FocusScope` raiz recebe `Key.Back` e chama `go-back()` (fecha folha → fila → player → detalhe → volta ao Início → sai).

## Verificação (v0.2.0-alpha.3, emulador x86_64)

Emulador AVD `kazzu34` (Android 14, KVM, `-gpu swiftshader_indirect`, 1080×2400, navegação por 3 botões), APK `--debug-sign`:

- abre e o **Início do YouTube Music carrega** sozinho (rede do emulador funciona): banner, chips, destaques com capas reais, escolhas rápidas;
- **tocar uma faixa remota**: toque na linha → resolve → stream HTTP → `AAudioStreamBuilder_openStream() returns 0 = AAUDIO_OK` → mini player e progresso andando; player com selo `OPUS 48 kHz`/`Do YouTube Music`/"Original, sem equalizador" (ajustado depois: o selo agora usa o formato que o provedor informou);
- **safe areas**: wordmark abaixo da barra de status; barra inferior acima dos botões do sistema;
- **Voltar**: fecha o player, fecha o detalhe de playlist, no Início sai do app;
- biblioteca local (`adb push` em `/sdcard/Music`): indexa; capa embutida aparece, `cover.jpg` não (permissão).
- **Não verificado**: som real (emulador sem áudio), aparelho físico (ARM), TalkBack, teclado sobre a folha Nova playlist, desempenho do blur em GPU real, segundo plano/tela apagada.

## Pendências para o Android (próximas etapas)

- **Áudio em segundo plano**: hoje a música para quando o app vai para segundo plano/tela apagada (o sistema pode matar o processo). Falta *foreground service* (tipo `mediaPlayback`) + notificação com controles + `MediaSession` (fones, tela de bloqueio, Bluetooth) via JNI; exige código Java mínimo (ou `GameActivity`/helper dex) e as permissões `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK` e `POST_NOTIFICATIONS` (Android 13+). Gerenciar foco de áudio (`AudioManager`) e "ruído" (fone desconectado).
- **Seletor de pastas / outras fontes**: SAF (`ACTION_OPEN_DOCUMENT_TREE`) + `ContentResolver`, ou scan por MediaStore, para músicas fora de `Music` (hoje só `Music`).
- **Área segura / borda a borda**: testar em Android 15 (targetSdk 35), navegação por gestos e recorte da câmera; teclado sobre as folhas (`adjustResize`).
- **Desempenho**: medir blur/Skia em aparelho de entrada; a opção "reduzir transparência" já existe na UI.
- **Teste em aparelho real** (arm64), com fones Bluetooth, rotação, bloqueio de tela, 16 KB de página.
- Reprodução em rede (YouTube Music) usa a permissão `INTERNET` declarada; *cleartext* continua bloqueado (HTTPS apenas).
- F-Droid: o APK é reproduzível em tese (sem binários proprietários além do Skia pré-compilado do `rust-skia`, que é baixado no build); para o F-Droid seria preciso compilar o Skia a partir do fonte.
- iOS fica para depois (Slint 1.19 ainda trata como experimental).
