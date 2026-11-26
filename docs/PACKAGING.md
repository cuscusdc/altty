# Pacotes nativos do Kazzu

A interface, os recursos e o backend são os mesmos no APK e nos AppImages. Os pacotes não incluem o gerador de biblioteca fictícia (`--no-default-features`). As saídas e seus arquivos SHA-256 ficam em `dist/`.

## Android ARM64

`scripts/build-apk.sh` usa o SDK/NDK local, `aapt2` dos build-tools do próprio SDK e o backend NativeActivity do Slint. Exemplo:

```bash
export ANDROID_HOME="$PWD/target/toolchains/android-sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/30.0.16248370"
scripts/build-apk.sh --abi arm64-v8a --out "$PWD/dist" --debug-sign
```

Sem `--debug-sign`, o script exige a chave de release existente, descrita em [ANDROID.md](ANDROID.md). A chave de testes não substitui a assinatura de uma instalação de release. Nenhuma chave ou senha integra os pacotes ou o Git.

## Linux x64 e ARM64

```bash
scripts/build-appimage.sh x86_64
scripts/build-appimage.sh aarch64
```

O script requer `APPIMAGETOOL` (executável da arquitetura do computador de build) e `APPIMAGE_RUNTIME` (runtime da arquitetura de destino). Por padrão, procura esses arquivos em `target/toolchains/`. A arquitetura ARM64 também exige:

- `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER`: compilador/linker cruzado;
- `CC_aarch64_unknown_linux_gnu` e `AR_aarch64_unknown_linux_gnu`: compilador e arquivador de destino;
- `KAZZU_ARM_SYSROOT`: sysroot com ALSA, Opus, Fontconfig e suas dependências;
- variáveis `PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu` e `PKG_CONFIG_LIBDIR_aarch64_unknown_linux_gnu`, apontando somente para bibliotecas de destino.

Quando `target/toolchains/linux-x64` (ou `KAZZU_X64_LIB_ROOT`) contém o runtime de compatibilidade, o empacotador religa apenas o executável com a versão compatível de libm e inclui as bibliotecas x64 desse diretório. Isso evita exigir novas versões de símbolos matemáticos da glibc da máquina de build. A libm e a glibc não são incluídas no pacote.

O empacotador coleta as bibliotecas ELF necessárias: `ldd` no x64 nativo; leitura estática de `NEEDED` no ARM64. Não executa `ldd` em binário estrangeiro. Inclui a marca Kazzu, entrada desktop e `AppRun`. O carregador e a glibc vêm da distribuição do usuário; a compatibilidade depende da versão da glibc usada na compilação. Bibliotecas de GPU e o servidor gráfico continuam sendo os do sistema.

Os empacotadores usam nível de otimização 1 para o código do aplicativo gerado pelo Slint e preservam a otimização de release das dependências de renderização, imagem e áudio. `KAZZU_APP_OPT_LEVEL=3` permite recompilar o aplicativo com otimização máxima.

Os parâmetros opcionais `KAZZU_BUILD_JOBS`, `KAZZU_DIST_DIR`, `CARGO_TARGET_DIR` e os overrides de perfil do Cargo permitem ajustar concorrência, destino e otimização sem alterar o projeto.

## Conferência

Confira os checksums no diretório `dist` com `sha256sum -c <arquivo>.sha256`. Para o APK, use `apksigner verify --verbose`, `zipalign -c -P 16 4` e `aapt2 dump badging`. Para os AppImages, confira a arquitetura do runtime e do executável em `usr/bin/kazzu`; validar somente o arquivo externo não confirma a arquitetura do aplicativo interno.

A execução ARM64 sob QEMU comprova que o executável pode iniciar, mas não substitui testes de áudio, GPU e integração em equipamento ARM64 real.

## Validação desta reconstrução (2026-11-26)

Foram gerados `kazzu-0.4.0-alpha.1-arm64-v8a.apk`, `kazzu-0.4.0-alpha.1-x86_64.AppImage` e `kazzu-0.4.0-alpha.1-aarch64.AppImage`. O APK usa a chave local de testes, com assinatura v2/v3 e alinhamento ZIP/ELF de 16 KiB conferidos. As duas entradas nativas do Android foram verificadas.

A verificação extrai os AppImages, compara seus executáveis com o build e confirma ambas as arquiteturas. O AppImage x64 foi executado com FemtoVG, nos layouts desktop e compacto. O executável extraído do AppImage ARM64 iniciou sob QEMU com as bibliotecas incluídas. A maior versão de símbolos GLIBC referenciada é 2.39 no x64 e 2.36 no ARM64. Isso não constitui uma certificação de todas as distribuições Linux.

O resultado detalhado, incluindo hashes, assinatura e comandos, fica em `dist/validation.txt`, gerado por `python3 scripts/verify-dist.py`. Não houve instalação ou teste em aparelho Android físico nem teste de áudio/GPU em equipamento ARM64 real.

Os nomes dos três arquivos usam o prefixo solicitado `kazzu-0.4.0-alpha.1-`. Esta renomeação não altera a versão interna dos binários já compilados.
