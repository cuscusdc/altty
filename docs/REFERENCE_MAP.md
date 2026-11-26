# Mapa de referência — Kazzu (Android + Desktop)

Inventário das referências e relatório da reimplementação no Kazzu em Rust e Slint.

## 1. Fontes e licenças

| Plataforma | Projeto | Licença | Revisão consultada |
|---|---|---|---|
| Android | [Metrolist](https://github.com/MetrolistGroup/Metrolist) | GPL-3.0 | `b55dba3` |
| Android | [BitChord Android](https://github.com/kushagrasinghx/BitChord) | GPL-3.0 | `2135620` |
| Desktop | [BitChord Desktop](https://github.com/efzynx/BitChord-Desktop) | GPL-3.0 | `4d580cc` |
| Desktop | [BitChord Desktop Revived](https://github.com/Bntagaaa/BitChord-Desktop-Revived) | GPL-3.0 | `72da950` |
| Desktop | [Orchard](https://sfg545.dev/orchard/) | AGPL-3.0-or-later (motor Adaptive mix sob PolyForm Shield 1.0.0) | site público e capturas (sem uso de código) |

Os quatro repositórios foram consultados em clones temporários fora da árvore do Kazzu. O BitChord Android está no módulo `app` de `kushagrasinghx/BitChord`, com manifesto em `app/src/main/AndroidManifest.xml`. Para o Orchard, o inventário usa a página pública e suas capturas. A implementação usa código e recursos do Kazzu, com identidade Brasa, Inter e textos em português.

## 2. Síntese Android

### Metrolist
- Material 3 Expressive; cor dinâmica derivada da capa; tema verde (claro `#1B5E20`/`#F1F8E9`,
  escuro `#81C784`/`#101010`, superfície `#121212`); tipografia do sistema.
- Navegação: barra inferior com Início, Busca, Biblioteca e um quarto destino; Ajustes em pilha a partir do
  topo. Telas empilhadas: álbum, artista, playlist, podcast, histórico, estatísticas, reconhecimento.
- Início: chips de humor no topo, prateleiras horizontais (Escolhas rápidas, Continue ouvindo, Favoritas
  esquecidas, Discagem rápida) e cartões grandes com capa.
- Biblioteca: filtros por tipo (playlists, álbuns, artistas, músicas), alternância grade/lista, ordenação.
- Busca: campo fixo no topo, histórico, sugestões instantâneas, filtros por tipo, resultado "melhor match".
- Player: folha que expande da barra inferior; capa grande, letra, fila, vídeo; swipe para fechar.
- Alvos de toque de 48 dp; estados vazios/erro/offline explícitos; conteúdo do provedor com cache.

### BitChord Android
- Estética Apple Music: preto/branco no escuro (`#000`, superfície `#0D0D0F`, `#1C1C1E`, contorno
  `#2C2C2E`), branco/cinza no claro (`#F7F7F9`, `#F2F2F7`, `#E5E5EA`), vermelho `#FA2D48`, fonte SF Pro.
- Cromo de vidro: `FrostedTopBar` (título grande que encolhe ao rolar), `GlassNavBar` com
  `FloatingTabBar` (pílula iOS 26), mini-player embutido na barra que expande para o player.
- Início: saudação, Top Picks, humores, prateleiras, "ouvido recentemente".
- Biblioteca: prateleiras com "ver tudo" em grade, ordenação, seleção.
- Player: capa que encolhe ao pausar, linha Letra/Fila/Saída, controle de volume em cápsula, layout
  paisagem com créditos e transporte.
- Ajustes em pilha (contas/scrobbling, fontes, Listen Together, equalizador, histórico, downloads...).

### Decisões para o Kazzu mobile
- 3 destinos na barra inferior (Início, Biblioteca, Buscar) + Ajustes no topo (um único botão).
- Barra de vidro flutuante única: mini-player + abas na mesma cápsula; tocar na faixa abre o player.
- Barra de topo translúcida que ganha vidro ao rolar; Voltar nas telas empilhadas.
- Detalhe em pilha com herói de capa, ações grandes, lista virtualizada.
- Player em tela cheia, capa central, console inferior, swipe para baixo fecha.
- Ajustes em uma página rolável com busca e categorias (uma única instância do botão de Ajustes).
- Controles principais com alvos de 48 px ou mais, base tipográfica Inter, margens horizontais de 16–32 px e estados de carregamento, erro e offline.

## 3. Síntese Desktop

### BitChord Desktop
- Base escura em cinzas neutros, com fundo #202020, superfícies #0D0D0F e #1C1C1E e realces que seguem a capa. Usa SF Pro Display e superfícies translúcidas no player.
- Barra lateral colapsável (64 px minimizada / 220 px expandida) com Início, Explorar, Biblioteca, Busca,
  Histórico, Downloads, Música local, playlists e Ajustes no rodapé.
- Barra de reprodução inferior estilo Apple Music: capa/título, transporte, linha do tempo, volume, fila.
- Player em tela cheia com capa à esquerda, letra/fila, créditos.
- Atalhos: Espaço, setas, Esc em camadas, Ctrl+W/Ctrl+M; janela redimensionável; título próprio.

### BitChord Desktop Revived

- Barra lateral de 72 dp recolhida ou 220 dp expandida, com efeito inspirado em Mica. Mantém pesquisa, biblioteca e transporte no rodapé. A faixa selecionada abre um player com arte ampliada. Há também um player flutuante redimensionável.
- Temas claro e escuro com desfoque adaptativo, cor de destaque derivada da capa e letra sincronizada. Ctrl+K aciona a busca rápida e Ctrl+/ abre a lista de atalhos.
- O README destaca Spotify, vídeos, equalizador, crossfade e Automix experimental.

### Orchard (página pública e capturas)
- Interface nativa Qt 6 sobre núcleo de áudio em Rust; tema escuro quente, tipografia Inter e acento
  que acompanha a arte.
- Capturas do site: página de artista com cabeçalho grande, último lançamento e músicas populares;
  barra de reprodução inferior e player em tela cheia com capa à esquerda e letra sincronizada à direita.
- Recursos anunciados: player em tela cheia (F, Espaço, Esc), letra sincronizada com tradução opcional,
  arte animada, vídeos musicais, crossfade/gapless, downloads offline, equalizador de 10 bandas, Orchard
  Connect, Android Auto/Chromecast e Hi-Res via Qobuz.
- Licença do projeto: AGPL-3.0-or-later, exceto o motor Adaptive mix (PolyForm Shield 1.0.0). Nenhum
  código, componente QML ou recurso gráfico do repositório foi usado.

### Decisões para o Kazzu desktop
- Barra lateral sólida (240 px) colapsável para 72 px: marca, 3 destinos, playlists, Ajustes no rodapé
  (um único botão, sem gear duplicado no conteúdo).
- Conteúdo em coluna central de até 1240 px, com grades adaptativas, estados de hover e espaçamento por tokens compartilhados. A paleta Brasa usa #FF7345 no escuro e #CC2C05 no claro.
- Barra de reprodução inferior de vidro com faixa, transporte, tempo, volume, curtir e fila.
- Player em tela cheia: capa à esquerda, console à direita.
- Atalhos: F (player), Espaço, ←/→, ↑/↓, Esc, Ctrl+F/Ctrl+K (busca), Ctrl+1/2/3 (destinos), Ctrl+, (Ajustes).
- Janela estreita (< 1100 px) recolhe a barra lateral para 72 px e simplifica o rodapé para faixa atual, progresso e transporte.

## 4. Contrato compartilhado

As duas interfaces usam `AppState` como contrato do Slint. O adaptador Rust publica os modelos e recebe ações por callbacks. Catálogo, biblioteca, fila, reprodução e persistência ficam no Rust. O Slint mantém o estado de apresentação, incluindo abas, rolagem e camadas abertas. `AppState.mobile` (escrito pelo Rust com `cfg!(target_os = "android")`)
escolhe o shell; o desktop adapta o próprio shell à largura da janela.

## 5. Funcionalidades usadas pelas duas interfaces

- Reprodução: tocar/pausar, anterior/próxima, busca na posição, volume, aleatório, repetir, curtir,
  fila (ver, pular, remover, mover, limpar), menu da faixa.
- Início: estados de carga/erro/offline/cache, destaques, escolhas rápidas, mixes, álbuns, lançamentos,
  em alta, recentes locais, primeira execução.
- Biblioteca: categorias, grade/lista, densidade, coleções, playlists, álbuns, artistas, pastas/scan.
- Busca: histórico, sugestões, filtros, melhor resultado, músicas/álbuns/artistas/playlists, erros.
- Detalhe: álbum/artista/playlist/coleção, tocar tudo, aleatório, salvar, faixas, discografia, sobre.
- Ajustes: tema, vidro, atmosfera, transparência/movimento, biblioteca, qualidade, autoplay, fontes,
  privacidade, atalhos, sobre/licenças.

## 6. Entrega e pendências

| Área | Reimplementado no Kazzu | Pendente em relação às referências |
|---|---|---|
| Android | Início, Biblioteca, Busca, Ajustes, detalhes e fila. Barra inferior com mini-player, controles de toque, player com gesto de fechar e retorno de Ajustes à aba de origem. | Letra sincronizada, seletor de saída, downloads, timer e reconhecimento ambiente ainda exibem aviso de implementação futura. Histórico e estatísticas dedicados, vídeo, integrações de carro e transmissão e recursos sociais dos apps de referência ficam pendentes. |
| Desktop | Sidebar expandida ou recolhida, playlists, Ajustes no rodapé, páginas próprias, menus de faixa, barra de reprodução adaptável, player ampliado e atalhos. | A sidebar ainda agrupa descoberta no Início. Destinos dedicados de histórico, downloads e música local, letra sincronizada, vídeo, mix adaptativo e integrações de serviço e dispositivo das referências ficam pendentes. |
| Núcleo | Os dois layouts usam os fluxos Rust de catálogo e biblioteca existentes, reprodução, filtros, favoritos, fila, listas e preferências persistentes. | Conectar provedores adicionais pela UI ainda exibe aviso. A paridade com os catálogos e serviços das referências permanece aberta. |

### Licenças

A adoção dos fluxos preserva a identidade visual do Kazzu. As quatro referências GPL-3.0 e a parte AGPL do Orchard foram usadas como material de consulta. Incorporar futuramente código, fontes, marcas ou arte dessas referências exigiria analisar cada licença e as obrigações de distribuição separadamente. A página do Orchard destaca que o motor Adaptive mix tem licença PolyForm Shield 1.0.0, com restrição para uso em produtos concorrentes. [Licenciamento publicado pelo Orchard](https://sfg545.dev/#download)

O próprio Kazzu declara MIT no manifesto do workspace e GPL-3.0 como intenção no README. A raiz ainda não tem arquivo de licença. Essa divergência fica pendente de decisão do autor antes de uma distribuição pública. [Manifesto](../Cargo.toml) · [README](../README.md)

## 7. Validação

- Android ARM64 compilado com o SDK/NDK local. O script gerou o [APK de teste](../target/android/apk/kazzu-0.1.0-arm64-v8a.apk) com assinatura de debug. A assinatura, o alinhamento de 16 KB e o SHA-256 passaram. A execução em dispositivo ficou pendente porque o ADB não encontrou aparelho conectado e o SDK local não contém emulador.
- O build desktop do workspace passou no Linux x86-64. Para caber na memória disponível, o Cargo foi executado com um job e overrides locais de otimização e símbolos de depuração para o app e a galeria. Nenhum override foi gravado no manifesto.
- Os seis testes de sincronização visual e a matriz de renderização e interação passaram na versão final. A matriz cobre temas claro e escuro, sidebar recolhida, retorno de Ajustes à aba Android de origem, atalho F no desktop e contraste do mini-player com o desfoque desativado.
- A galeria passou na medição de alvos de pelo menos 48×48 px lógicos em 400 combinações de cena e tamanho do shell desktop.
- Os 48 testes unitários do app, 56 do controlador, 7 do worker e 103 das outras crates passaram durante a validação. Os 9 testes de rede marcados como ignorados continuaram fora da execução. Os testes de lógica foram executados antes dos ajustes finais exclusivamente em Slint.
- Os dois exemplos executáveis da documentação do workspace também passaram.
