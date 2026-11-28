# Inventário da interface e avaliação dos componentes

## Arquitetura de produção

O build.rs seleciona ui/app.slint. AppWindow instancia um shell pelo alvo informado por app.rs. Cada shell seleciona a página pelo destino e pelo detalhe aberto. Player, fila, folhas e toast são compostos acima do shell. Screen/Flickable cuidam de rolagem e Metrics reserva o espaço do transporte. Nav decide o retorno. AppState é contrato, SlintView publica modelos e o controlador Rust traduz o gesto em operação no motor, catálogo ou worker. Um callback existente comprova o caminho estático, não o sucesso do efeito.

## Índice por arquivo

A relação abaixo vem das declarações e imports Slint do app. O índice inclui componentes internos, globais, tipos do contrato e dependências locais. Rótulos de seção identificam partes da composição; condições e subpáginas são discutidas nas fichas individuais.

### app.slint

Declarações: AppWindow.

Dependências: std-widgets.slint, theme/tokens.slint, theme/metrics.slint, theme/nav-state.slint, theme/app-contract.slint, components/atoms.slint, components/toast.slint, components/sheets.slint, mobile/shell.slint, desktop/shell.slint, mobile/player.slint, desktop/player.slint. [Fonte](../crates/kazzu-app/ui/app.slint)

### components/atoms.slint

Declarações: Icon, Glass, MediaGlass, Pressable, IconButton, MediaIconButton, Button, TextButton, Switch, Segmented, Tag, Cover, Mosaic, EqBars, TrackRow, GroupItem, Group, PickCard, Tile, LibraryTile, MoodTile, SectionHeader, Scrubber, Skeleton, StateCard, Banner, SearchField, Spinner.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint. [Fonte](../crates/kazzu-app/ui/components/atoms.slint)

### components/editorial.slint

Declarações: PageHeading, CollectionShortcut, FeatureArtwork.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, atoms.slint. [Fonte](../crates/kazzu-app/ui/components/editorial.slint)

### components/settings-controls.slint

Declarações: SettingsGroupHead, SettingsChoice.

Dependências: ../theme/tokens.slint, atoms.slint. [Fonte](../crates/kazzu-app/ui/components/settings-controls.slint)

### components/shared.slint

Declarações: Screen, Rail, LargeTitle, Rows, Gutters, Narrow, AlbumGrid, SkeletonRows.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, atoms.slint. [Fonte](../crates/kazzu-app/ui/components/shared.slint)

### components/sheets.slint

Declarações: SheetBase, SheetHeader, SheetAction, SheetRule, QueueSheet, MenuSheet, PlaylistSheet, NewPlaylistSheet, AccountSheet, FolderSheet.

Dependências: std-widgets.slint, ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, atoms.slint. Seções nomeadas: Fila, Adicionar à playlist, Nova playlist. [Fonte](../crates/kazzu-app/ui/components/sheets.slint)

### components/toast.slint

Declarações: Toast.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, atoms.slint. [Fonte](../crates/kazzu-app/ui/components/toast.slint)

### desktop/chrome.slint

Declarações: DesktopCompactBar.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/desktop/chrome.slint)

### desktop/detail.slint

Declarações: DesktopDetail.

Dependências: std-widgets.slint, ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/shared.slint. Seções nomeadas: Discografia, Sobre, Continue explorando. [Fonte](../crates/kazzu-app/ui/desktop/detail.slint)

### desktop/home.slint

Declarações: DesktopHome.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Dê play no seu dia., Continue de onde parou, Uma trilha para cada momento, Feitas para você, Álbuns para explorar, Acabaram de chegar, Em alta agora. [Fonte](../crates/kazzu-app/ui/desktop/home.slint)

### desktop/library.slint

Declarações: DesktopLibrary.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Sua biblioteca, Suas playlists, Álbuns, Artistas do seu acervo, Música neste aparelho. [Fonte](../crates/kazzu-app/ui/desktop/library.slint)

### desktop/nowbar.slint

Declarações: DesktopNowBar.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/desktop/nowbar.slint)

### desktop/player.slint

Declarações: PlayerConsole, DesktopPlayer.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/desktop/player.slint)

### desktop/search.slint

Declarações: TermRow, DesktopSearch.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Seu próximo som., Explore seu acervo, Explore por momentos e gêneros, Novos lançamentos, Buscas recentes, Em alta agora, Melhor resultado, Músicas, Álbuns, Artistas, Playlists. [Fonte](../crates/kazzu-app/ui/desktop/search.slint)

### desktop/settings.slint

Declarações: DesktopSettings.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/settings-controls.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Ajustes, Geral, Reprodução, Aparência, Fontes de música, Privacidade, Atalhos de teclado, Sobre. [Fonte](../crates/kazzu-app/ui/desktop/settings.slint)

### desktop/shell.slint

Declarações: DesktopShell.

Dependências: ../theme/tokens.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, sidebar.slint, nowbar.slint, chrome.slint, home.slint, library.slint, search.slint, settings.slint, detail.slint. [Fonte](../crates/kazzu-app/ui/desktop/shell.slint)

### desktop/sidebar.slint

Declarações: SideItem, DesktopSidebar.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/desktop/sidebar.slint)

### mobile/chrome.slint

Declarações: MobileTopBar, IslandTab, MobileDock.

Dependências: ../theme/mobile-reference.slint, ../theme/tokens.slint, ../theme/icons.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/mobile/chrome.slint)

### mobile/detail.slint

Declarações: MobileDetail.

Dependências: std-widgets.slint, ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/shared.slint, chrome.slint. Seções nomeadas: Discografia, Sobre, Continue explorando. [Fonte](../crates/kazzu-app/ui/mobile/detail.slint)

### mobile/home.slint

Declarações: HomeHeading, HomeArt, MobileHome.

Dependências: ../theme/app-contract.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/mobile-reference.slint, ../theme/nav-state.slint, ../components/shared.slint, ../theme/tokens.slint, ../components/atoms.slint. Seções nomeadas: Sua próxima descoberta, Escolha rápida, Mixes de músicas, Explorar, Continue ouvindo. [Fonte](../crates/kazzu-app/ui/mobile/home.slint)

### mobile/library.slint

Declarações: MobileLibrary.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/shared.slint. Seções nomeadas: Playlists, Álbuns, Artistas do seu acervo, Música neste aparelho. [Fonte](../crates/kazzu-app/ui/mobile/library.slint)

### mobile/player.slint

Declarações: PlayerAction, MobileQueueRow, MobilePlayer.

Dependências: std-widgets.slint, ../theme/app-contract.slint, ../theme/icons.slint, ../theme/mobile-reference.slint, ../theme/nav-state.slint, ../theme/tokens.slint, ../components/atoms.slint. [Fonte](../crates/kazzu-app/ui/mobile/player.slint)

### mobile/search.slint

Declarações: TermRow, MobileSearch.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, ../components/atoms.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Pesquisas recentes, Explore seu acervo, Explore por momentos e gêneros, Novos lançamentos, Em alta agora, Melhor resultado, Músicas, Álbuns, Artistas, Playlists. [Fonte](../crates/kazzu-app/ui/mobile/search.slint)

### mobile/settings.slint

Declarações: MobileSettings.

Dependências: ../theme/tokens.slint, ../theme/icons.slint, ../theme/metrics.slint, ../theme/app-contract.slint, ../components/atoms.slint, ../components/settings-controls.slint, ../components/editorial.slint, ../components/shared.slint. Seções nomeadas: Ajustes, Geral, Reprodução, Aparência, Fontes de música, Privacidade, Atalhos de teclado, Sobre. [Fonte](../crates/kazzu-app/ui/mobile/settings.slint)

### mobile/shell.slint

Declarações: MobileShell.

Dependências: ../theme/metrics.slint, ../theme/app-contract.slint, ../theme/nav-state.slint, chrome.slint, home.slint, library.slint, search.slint, settings.slint, detail.slint. [Fonte](../crates/kazzu-app/ui/mobile/shell.slint)

### theme/app-contract.slint

Declarações: Tab, RepeatMode, SearchState, ThemeMode, DetailKind, LoadState, Track, Album, Highlight, Artist, Playlist, Collection, OutputDevice, FolderChoice, ProviderRow, AppState.

Dependências: nenhum import de componente. [Fonte](../crates/kazzu-app/ui/theme/app-contract.slint)

### theme/icons.slint

Declarações: Icons.

Dependências: nenhum import de componente. [Fonte](../crates/kazzu-app/ui/theme/icons.slint)

### theme/metrics.slint

Declarações: Metrics.

Dependências: tokens.slint, app-contract.slint. [Fonte](../crates/kazzu-app/ui/theme/metrics.slint)

### theme/mobile-reference.slint

Declarações: MobilePalette.

Dependências: nenhum import de componente. [Fonte](../crates/kazzu-app/ui/theme/mobile-reference.slint)

### theme/nav-state.slint

Declarações: Nav.

Dependências: app-contract.slint, metrics.slint. [Fonte](../crates/kazzu-app/ui/theme/nav-state.slint)

### theme/tokens.slint

Declarações: Theme.

Dependências: nenhum import de componente. [Fonte](../crates/kazzu-app/ui/theme/tokens.slint)

## Duplicação e separação de responsabilidades

TermRow existe nas duas buscas e combina ícone, texto e remoção. Extrair a linha comum e deixar cabeçalho e colocação do campo específicos. Nas duas bibliotecas, seleção de categoria, densidade e ações dependem dos mesmos modelos. Compartilhar descrição dos filtros e estado da coleção. Conservar grids, atalhos e apresentação contextual por plataforma. Nos dois Ajustes se repetem categorias, índices e busca por palavras. Manter um catálogo de preferências com IDs estáveis para que uma nova seção não exija atualizar índices em dois arquivos.

MobilePlayer reúne palco, transporte, fila e letras num único arquivo. Separar apresentação de reprodução e ações do item de fila. SheetBase pode responder por modalidade, foco, safe area e retorno, sem impor a mesma geometria ao desktop e telefone. Glass e MediaGlass podem manter contraste distinto com um contrato comum de material. Cover e Mosaic têm usos diferentes: preservar imagem de obra e composição de coleção como conceitos distintos.

## Avaliação individual das peças de maior impacto

|Componente|Nota|Decisão, impacto e limite|
|---|---:|---|
|Pressable|8|MANTER. Unifica ação, toque, teclado e foco visível. A ação acessível direta merece verificação quando desabilitado. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L70)|
|Scrubber|7|REFINAR. Área de 48 px, trilho de 9 px e ação incremental tornam a posição manipulável. Falta capacidade seekable explícita para duração desconhecida. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L537)|
|TrackRow|8|MANTER. Combina informação e ação secundária sem exigir abrir detalhe. Muitas configurações e botões aninhados exigem verificar foco de linha versus Mais. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L305)|
|Glass|7|REFINAR. Material e fallback centralizados, mas o dock redefine blur e opacidade. Concentrar regras para não divergir com preferência de transparência. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L30)|
|Cover|8|MANTER. Representação reutilizável de imagem e forma. Seu benefício se perde nas superfícies Android que desenham gradiente fixo em seu lugar. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L255)|
|HomeArt|2|SUBSTITUIR. Cor por posição mantém ritmo visual sem identificar faixa. Usar capa e fallback por álbum somente se faltar imagem. [Fonte](../crates/kazzu-app/ui/mobile/home.slint#L18)|
|Segmented|7|REFINAR. Seleção exposta como aba; strings longas disputam largura. Tornar opções compatíveis com fonte e largura disponível. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L205)|
|SearchField|7|REFINAR. Entrada nativa, nome acessível e foco. Token e caminho reaproveitam componente de busca, misturando semântica de pesquisar e preencher formulário. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L631)|
|StateCard|8|MANTER. Título, explicação e CTA permitem recuperação localizada. Falta garantir que cada chamador diferencie fonte desativada de vazio real. [Fonte](../crates/kazzu-app/ui/components/atoms.slint#L575)|
|Toast|6|REFINAR. Live region e ação contextual existem. Mensagem elidida e temporária é insuficiente para importação com erro. [Fonte](../crates/kazzu-app/ui/components/toast.slint#L7)|
|Rail|7|REFINAR. Lista nomeada e navegação horizontal cabem em descoberta. Para catálogo completo obriga percorrer eixo secundário. [Fonte](../crates/kazzu-app/ui/components/shared.slint#L28)|
|SheetBase|5|REDESENHAR o contrato modal. Véu e interceptação interna explícitos são úteis. Altura e lift fixos e alça decorativa fragilizam telas baixas. [Fonte](../crates/kazzu-app/ui/components/sheets.slint#L11)|
|Sidebar|8|MANTER. Destinos e playlists sempre reconhecíveis no modo largo. Modo reduzido exige recuperar rótulos no foco ou hover. [Fonte](../crates/kazzu-app/ui/desktop/sidebar.slint#L34)|
|MobileDock|5|REDESENHAR. Cromo estável e acesso rápido ao transporte. Quatro botões, arte fixa e busca separada competem com título e destinos. [Fonte](../crates/kazzu-app/ui/mobile/chrome.slint#L92)|
|FeatureArtwork|6|REFINAR. Diferencia abrir conteúdo e iniciar reprodução. Espaço editorial excessivo para mostrar apenas um resultado de pesquisa. [Fonte](../crates/kazzu-app/ui/components/editorial.slint#L41)|
|MobileQueueRow|4|SUBSTITUIR. Identifica atual e próximas, mas a composição da fila no player não oferece gerenciamento local. [Fonte](../crates/kazzu-app/ui/mobile/player.slint#L17)|
|FolderSheet|5|REDESENHAR. Combina navegação, input e listagem pendente. Erro não é parte da condição de habilitação de Adicionar. [Fonte](../crates/kazzu-app/ui/components/sheets.slint#L288)|
|AccountSheet|4|REDESENHAR. Mascara segredo, mas fecha antes da conclusão, deixando recuperação fora do formulário. [Fonte](../crates/kazzu-app/ui/components/sheets.slint#L252)|

