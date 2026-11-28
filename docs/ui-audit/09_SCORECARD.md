# Scorecard estrutural

Escala editorial. 0–2 representa tarefa ausente ou apenas anunciada, 3–4 representa tarefa parcial ou expectativa quebrada, 5–6 indica caminho utilizável com fricção importante, 7–8 indica boa estrutura com limitações identificadas, 9–10 exigiria evidência adicional de cobertura e uso. Notas não são medidas de satisfação. Maturidade é juízo global, sem média artificial de telas.

## Telas e subpáginas

|Superfície|Nota|Evidência, impacto e limite|
|---|---:|---|
|Início Android|4|Atalhos disparam reprodução por ID, mas cinco slots sempre desenham gradientes, mesmo sem faixa. A primeira sessão gasta duas linhas sem oferecer música. [mobile/home, linha 108](../crates/kazzu-app/ui/mobile/home.slint#L108)|
|Explorar Android|3|O salto é econômico, mas o destino parece independente e seis gêneros fixos apenas preenchem uma busca textual. [mobile/home, linha 41](../crates/kazzu-app/ui/mobile/home.slint#L41)|
|Início desktop|7|O fallback para álbuns locais é pertinente. Baixados recebe espaço mesmo sem catálogo offline implementado. [desktop/home, linha 20](../crates/kazzu-app/ui/desktop/home.slint#L20)|
|Busca Android|5|A origem é explícita. Os chips de tipo só existem em resultados. Um tipo sem correspondência perde o próprio controle que permitiria voltar a Tudo. [mobile/search, linha 63](../crates/kazzu-app/ui/mobile/search.slint#L63)|
|Busca desktop|6|Foco inicial definido, mas o campo rola junto com os resultados e o histórico fica depois das recomendações. [desktop/search, linha 43](../crates/kazzu-app/ui/desktop/search.slint#L43)|
|Biblioteca Android|5|Há lista e grade, mas minutos ouvidos tomam o primeiro bloco de uma tarefa de localização. [mobile/library, linha 25](../crates/kazzu-app/ui/mobile/library.slint#L25)|
|Biblioteca desktop|6|A janela ampla admite comparação, porém há várias camadas para chegar às faixas e criar tem destaque permanente. [desktop/library, linha 22](../crates/kazzu-app/ui/desktop/library.slint#L22)|
|Playlists da biblioteca|6|O vazio orienta criação. O fluxo público favorece adicionar, sem controles equivalentes de renomear e administrar. [mobile/library, linha 108](../crates/kazzu-app/ui/mobile/library.slint#L108)|
|Álbuns da biblioteca|6|Arte auxilia reconhecimento; a categoria ainda pede Todos os álbuns locais para abrir outra visão do mesmo domínio. [mobile/library, linha 145](../crates/kazzu-app/ui/mobile/library.slint#L145)|
|Faixas locais|5|A ação de ingestão existe, mas selecionar Faixas ainda pode exigir abrir outra coleção para ler a lista completa. [mobile/library, linha 206](../crates/kazzu-app/ui/mobile/library.slint#L206)|
|Artistas da biblioteca|6|O vazio explica dependência de tags. A navegação para discografia depende de mais um detalhe sem histórico visível. [mobile/library, linha 168](../crates/kazzu-app/ui/mobile/library.slint#L168)|
|Detalhe de artista|6|A discografia tem navegação própria, mas fechar um álbum filho só desliga detail-open. [mobile/detail, linha 90](../crates/kazzu-app/ui/mobile/detail.slint#L90)|
|Detalhe de álbum|7|Há estados de carga e falha. No mobile a altura da arte mais 240 px precede as faixas e o título elide. [desktop/detail, linha 48](../crates/kazzu-app/ui/desktop/detail.slint#L48)|
|Detalhe de playlist|5|Há reprodução contextual. Ações do contrato criar/adicionar deixam sem equivalente visual renomear, remover faixa e ordenar manualmente. [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64)|
|Curtidos|6|O controlador publica estado de favorito, mas o tratamento de coleção genérica joga a identificação e o vazio no mesmo hero de outros tipos. [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70)|
|Todos os álbuns|6|Grade apropriada, entrada redundante depois de selecionar Álbuns. [desktop/detail, linha 63](../crates/kazzu-app/ui/desktop/detail.slint#L63)|
|Coleção de músicas|6|Há tocar tudo e aleatório. A página repete a intenção da categoria Faixas. [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70)|
|Detalhe de gênero|5|Tem vazio específico no worker. Os atalhos de gênero da Home Android seguem outro caminho, por busca textual. [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64)|
|Baixados|2|O texto explica indisponibilidade, mas o destino não consegue concluir a tarefa. [mobile/detail, linha 66](../crates/kazzu-app/ui/mobile/detail.slint#L66)|
|Mini-player Android|5|O dock independe do scroll. A arte é um retângulo com brush fixo e quatro controles comprimem o texto. [mobile/chrome, linha 101](../crates/kazzu-app/ui/mobile/chrome.slint#L101)|
|Rodapé desktop amplo|8|Bom uso da largura e semântica de posição. O salto para o modo compacto remove funções de uma só vez. [desktop/nowbar, linha 31](../crates/kazzu-app/ui/desktop/nowbar.slint#L31)|
|Rodapé desktop compacto|5|O texto usa largura restante após 176 px fixos. O progresso de 2 px é indicativo, sem seek. [desktop/chrome, linha 18](../crates/kazzu-app/ui/desktop/chrome.slint#L18)|
|Player Android expandido|4|Play de 64 px, repetição com selo e rótulos são úteis. Palco oliva fixo, controles dentro do Flickable, aba Vídeo sem vídeo e timer sem ação real. [mobile/player, linha 40](../crates/kazzu-app/ui/mobile/player.slint#L40)|
|Player desktop expandido|7|Arte real, origem e erro de reprodução têm espaço. A saída de áudio remete a aviso. Ocupação integral da janela não comprova fullscreen do sistema. [desktop/player, linha 135](../crates/kazzu-app/ui/desktop/player.slint#L135)|
|Fila dentro do player Android|4|O salto usa o ID da linha. Faltam remover, mover e limpar que existem na fila compartilhada; saltar fecha a fila. [mobile/player, linha 247](../crates/kazzu-app/ui/mobile/player.slint#L247)|
|Fila compartilhada|6|Mover é possível com teclado por botões. Limpar aciona diretamente o controlador; se não há próximas, a faixa atual também some do painel. [components/sheets, linha 75](../crates/kazzu-app/ui/components/sheets.slint#L75)|
|Letra Android|2|Há saída explícita. O único corpo é texto constante de indisponibilidade, sem modelo de versos. [mobile/player, linha 274](../crates/kazzu-app/ui/mobile/player.slint#L274)|
|Menu de faixa|6|A identidade do alvo é explícita. Três ações futuras alongam o menu e album/artista fecham o player. [components/sheets, linha 129](../crates/kazzu-app/ui/components/sheets.slint#L129)|
|Adicionar à playlist|6|Reutiliza a faixa selecionada. A transição é coordenada por flags e pode envolver três folhas consecutivas. [components/sheets, linha 174](../crates/kazzu-app/ui/components/sheets.slint#L174)|
|Nova playlist|7|Entrada curta e finalidade clara. O levantamento da folha com teclado depende de geometria heurística. [components/sheets, linha 214](../crates/kazzu-app/ui/components/sheets.slint#L214)|
|Importar Spotify|4|Token mascarado e importação no worker/thread. Formulário fecha e limpa token antes do resultado. Uma faixa importada abre o Spotify. [components/sheets, linha 252](../crates/kazzu-app/ui/components/sheets.slint#L252)|
|Importar YouTube|4|A cópia diferencia reprodução visitante, mas exige token externo e a entrada chama-se Contas. Não há login interativo nessa folha. [components/sheets, linha 268](../crates/kazzu-app/ui/components/sheets.slint#L268)|
|Pasta Android|5|Há pending e erro. Após falha de listagem, o caminho anterior pode continuar selecionável; digitar caminho exige conhecimento de armazenamento. [components/sheets, linha 288](../crates/kazzu-app/ui/components/sheets.slint#L288)|
|Pasta desktop|6|Seletor nativo é adequado, mas exige abrir outra superfície antes de escolher a pasta. [components/sheets, linha 311](../crates/kazzu-app/ui/components/sheets.slint#L311)|
|Ajustes Android|5|Há pesquisa; várias categorias e Tudo competem em largura curta. [mobile/settings, linha 43](../crates/kazzu-app/ui/mobile/settings.slint#L43)|
|Ajustes desktop|6|Categorias separam tarefas, mas parte dos controles replica a Biblioteca. [desktop/settings, linha 38](../crates/kazzu-app/ui/desktop/settings.slint#L38)|
|Geral|6|Valores compartilhados são coerentes, mas duplicam comandos contextuais da Biblioteca. [mobile/settings, linha 62](../crates/kazzu-app/ui/mobile/settings.slint#L62)|
|Reprodução|6|Qualidade e autoplay têm contexto. Volume já está no player e compete com parâmetros persistentes. [desktop/settings, linha 81](../crates/kazzu-app/ui/desktop/settings.slint#L81)|
|Aparência|6|Preferências explícitas e fallback são fortes. Parte do Android usa cores literais independentes do tema. [desktop/settings, linha 98](../crates/kazzu-app/ui/desktop/settings.slint#L98)|
|Fontes e biblioteca|5|Pastas têm remover e reescanear. Capacidade de importar e capacidade de tocar aparecem no mesmo agrupamento. [mobile/settings, linha 120](../crates/kazzu-app/ui/mobile/settings.slint#L120)|
|Privacidade|5|O aviso avisa que a exclusão é definitiva. Duplo toque no mesmo lugar dentro de quatro segundos prejudica leitura pausada; Não há conta conflita com título Contas. [mobile/settings, linha 150](../crates/kazzu-app/ui/mobile/settings.slint#L150)|
|Atalhos|6|Documentação interna útil no desktop. Baixo valor como categoria permanente no telefone sem teclado. [desktop/settings, linha 178](../crates/kazzu-app/ui/desktop/settings.slint#L178)|
|Sobre e licenças|4|Versão vem do estado. UI declara GPL-3.0 enquanto workspace declara MIT, sem decisão demonstrada pelas telas. [mobile/settings, linha 183](../crates/kazzu-app/ui/mobile/settings.slint#L183)|
|Oferta de desempenho|6|A correção proposta é concreta. Um alerta global pode competir com a tarefa atual e não explica como reverter. [app, linha 121](../crates/kazzu-app/ui/app.slint#L121)|

## Dimensões gerais

|Dimensão|Nota|Evidência, impacto e limite|
|---|---:|---|
|UI|5,5|O desktop diferencia arte, lista e transporte. A Home Android destaca slots vazios. [Análise e fontes](01_UI_AUDIT.md)|
|UX|4,5|Tocar uma escolha exige uma ativação. Alterar tipo da Busca após resultado vazio não tem controle no mesmo lugar. [Análise e fontes](02_UX_AUDIT.md)|
|Identidade visual|5|Inter e Brasa são identificáveis. O rosa e o oliva fixos atravessam o tema aplicado nas páginas vizinhas. [Análise e fontes](01_UI_AUDIT.md)|
|Navegação|5|Sidebar e dock são estáveis. O detalhe único não recompõe artista após fechar álbum. [Análise e fontes](02_UX_AUDIT.md)|
|Consistência|4|Mesma fila e mesmo controlador, mas ações disponíveis variam segundo ponto de entrada. [Análise e fontes](02_UX_AUDIT.md)|
|Design system|6|Há primitivas de foco e material. A Home e o player duplicam regras de cor, medida e arte. [Análise e fontes](04_COMPONENT_AUDIT.md)|
|Biblioteca|5,5|Categorias, grade/lista e diretórios têm representação. Gerenciamento posterior à criação de playlist é restrito. [Análise e fontes](03_SCREEN_BY_SCREEN.md)|
|Player|5|Transporte amplo e seek acessível são boa base. Fila móvel perde comandos e a UI anuncia funções futuras. [Análise e fontes](02_UX_AUDIT.md)|
|Pesquisa|5|Agrega local/remoto, cancela logicamente consultas antigas e aceita sugestões. Filtros e vazio não são independentes. [Análise e fontes](02_UX_AUDIT.md)|
|Responsividade estrutural|5,5|Há shell por plataforma e reservas do cromo. Falta consumo de insets laterais e modo por altura no palco Android. [Análise e fontes](01_UI_AUDIT.md)|
|Acessibilidade estrutural|6|Rótulos, focus ring, sliders e anúncios são verificáveis. Foco modal e escrita com teclado ampliado exigem validação. [Análise e fontes](04_COMPONENT_AUDIT.md)|
|Organização dos componentes|6|Separação de shells e camada Rust útil. TermRow duplicado, múltiplos donos de fila e Ajustes paralelos elevam custo de mudança. [Análise e fontes](04_COMPONENT_AUDIT.md)|
|Maturidade geral|4,8|Os principais percursos têm base de produto. Vários controles sem capacidade e a perda de contexto atingem tarefas recorrentes. [Análise e fontes](00_EXECUTIVE_SUMMARY.md)|

## Componentes importantes

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
