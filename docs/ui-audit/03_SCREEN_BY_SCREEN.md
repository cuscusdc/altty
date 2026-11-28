# Auditoria por tela e subpágina

Cada ficha indica objetivo e ordem atual, mérito verificável, limite que determina a nota e decisão. Categorias compartilhadas representam o mesmo estado de conteúdo nos dois shells. As diferenças de navegação e de geometria estão nas fichas das páginas hospedeiras.

## Início Android · 4/10

Escolher algo para ouvir. A grade precede mixes, gêneros e recentes.

Atalhos disparam reprodução por ID, mas cinco slots sempre desenham gradientes, mesmo sem faixa. A primeira sessão gasta duas linhas sem oferecer música.

**Decisão.** REDESENHAR. Com histórico, mostrar primeiro a última faixa e sua capa. Sem acervo, oferecer Buscar e Adicionar pasta. Preencher escolhas somente com itens disponíveis. [mobile/home, linha 108](../crates/kazzu-app/ui/mobile/home.slint#L108)

## Explorar Android · 3/10

Descobrir por gênero. A aba muda o scroll da própria Home.

O salto é econômico, mas o destino parece independente e seis gêneros fixos apenas preenchem uma busca textual.

**Decisão.** REMOVER a aba atual. Manter gêneros como atalhos opcionais da descoberta e identificar o termo na Busca. [mobile/home, linha 41](../crates/kazzu-app/ui/mobile/home.slint#L41)

## Início desktop · 7/10

Retomar ou descobrir. Título editorial, destaque, coleções e rails.

O fallback para álbuns locais é pertinente. Baixados recebe espaço mesmo sem catálogo offline implementado.

**Decisão.** REFINAR. Abrir com retomada, uma seleção editorial e biblioteca local. Ocultar coleções indisponíveis. Preservar atualizar com cache visível. [desktop/home, linha 20](../crates/kazzu-app/ui/desktop/home.slint#L20)

## Busca Android · 5/10

Encontrar por fonte e tipo. Campo, seletor de origem, recentes, descoberta e resultados.

A origem é explícita. Os chips de tipo só existem em resultados. Um tipo sem correspondência perde o próprio controle que permitiria voltar a Tudo.

**Decisão.** REDESENHAR. Fixar campo, fonte e tipo acima dos estados. Vazio deve oferecer limpar tipo e buscar em outra origem mantendo o termo. [mobile/search, linha 63](../crates/kazzu-app/ui/mobile/search.slint#L63)

## Busca desktop · 6/10

Pesquisar com teclado. Cabeçalho, campo dentro de Screen, descoberta, recentes e resultados.

Foco inicial definido, mas o campo rola junto com os resultados e o histórico fica depois das recomendações.

**Decisão.** REDESENHAR. Campo persistente com foco por Ctrl+K, histórico imediato, resultados por tipo em lista; destacar melhor resultado sem monopolizar a altura. [desktop/search, linha 43](../crates/kazzu-app/ui/desktop/search.slint#L43)

## Biblioteca Android · 5/10

Organizar acervo. Estatística, criar, visualização, categorias e densidade precedem conteúdo.

Há lista e grade, mas minutos ouvidos tomam o primeiro bloco de uma tarefa de localização.

**Decisão.** REDESENHAR. Começar pelas coleções e último acesso, mover estatísticas para histórico e densidade para menu de apresentação. [mobile/library, linha 25](../crates/kazzu-app/ui/mobile/library.slint#L25)

## Biblioteca desktop · 6/10

Percorrer acervo. Cabeçalho, controles, categorias, coleções e itens locais.

A janela ampla admite comparação, porém há várias camadas para chegar às faixas e criar tem destaque permanente.

**Decisão.** REFINAR. Categoria à esquerda da área de conteúdo, busca na coleção e ordenação junto ao título. Mostrar Criar como primária apenas em Playlists. [desktop/library, linha 22](../crates/kazzu-app/ui/desktop/library.slint#L22)

## Playlists da biblioteca · 6/10

Listar playlists em grade/lista, abrir ou criar.

O vazio orienta criação. O fluxo público favorece adicionar, sem controles equivalentes de renomear e administrar.

**Decisão.** REDESENHAR. Lista ordenável com nome, quantidade, origem, última alteração e ações de gestão apenas para listas editáveis. [mobile/library, linha 108](../crates/kazzu-app/ui/mobile/library.slint#L108)

## Álbuns da biblioteca · 6/10

Escolher álbum pelo artwork e artista.

Arte auxilia reconhecimento; a categoria ainda pede Todos os álbuns locais para abrir outra visão do mesmo domínio.

**Decisão.** REFINAR. Categoria deve conter catálogo completo com paginação e ordenação. Prateleiras resumidas ficam na Home. [mobile/library, linha 145](../crates/kazzu-app/ui/mobile/library.slint#L145)

## Faixas locais · 5/10

Encontrar músicas indexadas. A seção oferece abrir músicas locais e adicionar pasta.

A ação de ingestão existe, mas selecionar Faixas ainda pode exigir abrir outra coleção para ler a lista completa.

**Decisão.** SUBSTITUIR o índice intermediário por lista pesquisável. No vazio oferecer pasta; com dados manter indexação em segundo plano e progresso contextual. [mobile/library, linha 206](../crates/kazzu-app/ui/mobile/library.slint#L206)

## Artistas da biblioteca · 6/10

Escolher intérprete no acervo. Linhas ou tiles abrem detalhe.

O vazio explica dependência de tags. A navegação para discografia depende de mais um detalhe sem histórico visível.

**Decisão.** REFINAR. Ordenar pelo nome, oferecer artista desconhecido para tags incompletas e restaurar a posição ao voltar. [mobile/library, linha 168](../crates/kazzu-app/ui/mobile/library.slint#L168)

## Detalhe de artista · 6/10

Entender catálogo. Retrato, músicas, discografia, descrição e relacionados.

A discografia tem navegação própria, mas fechar um álbum filho só desliga detail-open.

**Decisão.** REFINAR. Cabeçalho compacto com retrato, faixas populares e discografia; empilhar álbum com retorno ao mesmo artista. [mobile/detail, linha 90](../crates/kazzu-app/ui/mobile/detail.slint#L90)

## Detalhe de álbum · 7/10

Ler faixas e tocar álbum. Hero, metadados, tocar/aleatório, linhas.

Há estados de carga e falha. No mobile a altura da arte mais 240 px precede as faixas e o título elide.

**Decisão.** REFINAR. Título em duas linhas, artista navegável, ações agrupadas, ordem do disco e lista; reduzir hero em paisagem. [desktop/detail, linha 48](../crates/kazzu-app/ui/desktop/detail.slint#L48)

## Detalhe de playlist · 5/10

Ouvir e administrar coleção. Cabeçalho e lista compartilham o molde de álbum.

Há reprodução contextual. Ações do contrato criar/adicionar deixam sem equivalente visual renomear, remover faixa e ordenar manualmente.

**Decisão.** REDESENHAR. Identificar proprietária ou importada e fonte externa. Ações de edição persistentes, confirmação ao excluir e desfazer remoção. [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64)

## Curtidos · 6/10

Reouvir favoritos. Coleção única com ações de transporte.

O controlador publica estado de favorito, mas o tratamento de coleção genérica joga a identificação e o vazio no mesmo hero de outros tipos.

**Decisão.** REFINAR. Contagem e filtro por origem. No vazio explicar o coração e ligar à Busca. Descurtir atualiza a lista e oferece desfazer. [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70)

## Todos os álbuns · 6/10

Percorrer grade completa. AlbumGrid sucede o hero.

Grade apropriada, entrada redundante depois de selecionar Álbuns.

**Decisão.** SUBSTITUIR pela própria categoria completa da Biblioteca, preservando posição ao abrir um álbum. [desktop/detail, linha 63](../crates/kazzu-app/ui/desktop/detail.slint#L63)

## Coleção de músicas · 6/10

Tocar biblioteca local em sequência. Cabeçalho e lista de faixas.

Há tocar tudo e aleatório. A página repete a intenção da categoria Faixas.

**Decisão.** UNIFICAR com Faixas locais. Cabeçalho com contagem, tocar, aleatório e filtro. Nome de pasta como dado secundário. [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70)

## Detalhe de gênero · 5/10

Ouvir recorte temático. Worker admite DetailRef::Genre, interface usa lista genérica.

Tem vazio específico no worker. Os atalhos de gênero da Home Android seguem outro caminho, por busca textual.

**Decisão.** REFINAR. Para categorias com metadados usar lista filtrada; rotular como busca os chips que só lançam consulta. Alcance de cada entrada depende dos dados. [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64)

## Baixados · 2/10

Consultar downloads. Worker cria coleção com escopo vazio.

O texto explica indisponibilidade, mas o destino não consegue concluir a tarefa.

**Decisão.** REMOVER a entrada até existir download com estado por faixa. Preservar músicas locais como destino distinto. [mobile/detail, linha 66](../crates/kazzu-app/ui/mobile/detail.slint#L66)

## Mini-player Android · 5/10

Identificar faixa e controlar sem sair da página. Arte circular, título, artista e quatro botões.

O dock independe do scroll. A arte é um retângulo com brush fixo e quatro controles comprimem o texto.

**Decisão.** REDESENHAR. Capa real, título, pausar e avançar no compacto. Levar anterior e favorito ao expandido quando faltar largura. [mobile/chrome, linha 101](../crates/kazzu-app/ui/mobile/chrome.slint#L101)

## Rodapé desktop amplo · 8/10

Controlar transporte sem abandonar a lista. Arte, identificação, tempo, seek, volume e fila.

Bom uso da largura e semântica de posição. O salto para o modo compacto remove funções de uma só vez.

**Decisão.** MANTER. Três regiões, identidade, transporte e utilidades. Colapsar utilidades gradualmente e manter entrada de fila. [desktop/nowbar, linha 31](../crates/kazzu-app/ui/desktop/nowbar.slint#L31)

## Rodapé desktop compacto · 5/10

Transporte em janela estreita. Cápsula com arte, título, play e próxima.

O texto usa largura restante após 176 px fixos. O progresso de 2 px é indicativo, sem seek.

**Decisão.** REDESENHAR. Manter play e título prioritários; menu de utilidades e fila acessíveis por teclado, seek ao expandir. [desktop/chrome, linha 18](../crates/kazzu-app/ui/desktop/chrome.slint#L18)

## Player Android expandido · 4/10

Ouvir e controlar. Cabeçalho, capa, identificação, scrubbers, transporte, fila e utilidades.

Play de 64 px, repetição com selo e rótulos são úteis. Palco oliva fixo, controles dentro do Flickable, aba Vídeo sem vídeo e timer sem ação real.

**Decisão.** REDESENHAR. Fixar transporte e linha de tempo acima do inset inferior; adaptar capa ao espaço restante, manter detalhes secundários roláveis e ocultar capacidades ausentes. [mobile/player, linha 40](../crates/kazzu-app/ui/mobile/player.slint#L40)

## Player desktop expandido · 7/10

Audição imersiva dentro da janela. Palco, arte e console responsivo.

Arte real, origem e erro de reprodução têm espaço. A saída de áudio remete a aviso. Ocupação integral da janela não comprova fullscreen do sistema.

**Decisão.** REFINAR. Capa e identificação à esquerda, console e fila à direita conforme largura; erro com tentar novamente no mesmo lugar. [desktop/player, linha 135](../crates/kazzu-app/ui/desktop/player.slint#L135)

## Fila dentro do player Android · 4/10

Ver sequência e saltar. Folha com 60% da altura, faixa atual e ListView.

O salto usa o ID da linha. Faltam remover, mover e limpar que existem na fila compartilhada; saltar fecha a fila.

**Decisão.** SUBSTITUIR por apresentação touch da fila comum. Separar Tocando e A seguir; mover por alça com alternativa acessível, remover com desfazer. [mobile/player, linha 247](../crates/kazzu-app/ui/mobile/player.slint#L247)

## Fila compartilhada · 6/10

Editar próximas músicas. Painel, atual, lista, subir, remover e limpar.

Mover é possível com teclado por botões. Limpar aciona diretamente o controlador; se não há próximas, a faixa atual também some do painel.

**Decisão.** REFINAR. Atual independente do vazio de próximas. Limpar somente próximas com desfazer. Conservar painel aberto durante edição. [components/sheets, linha 75](../crates/kazzu-app/ui/components/sheets.slint#L75)

## Letra Android · 2/10

Ler letra. Modal com título e fechar.

Há saída explícita. O único corpo é texto constante de indisponibilidade, sem modelo de versos.

**Decisão.** REMOVER entrada atual. CRIAR leitor apenas com fonte, estados de carregamento, ausência por faixa e opção de seguir sincronização. [mobile/player, linha 274](../crates/kazzu-app/ui/mobile/player.slint#L274)

## Menu de faixa · 6/10

Agir sobre uma faixa identificada. Cabeçalho, operações de fila, playlist, curtida e navegação.

A identidade do alvo é explícita. Três ações futuras alongam o menu e album/artista fecham o player.

**Decisão.** REFINAR. Agrupar tocar, organizar e navegar. Filtrar por capacidade da faixa e preservar origem para voltar. [components/sheets, linha 129](../crates/kazzu-app/ui/components/sheets.slint#L129)

## Adicionar à playlist · 6/10

Escolher destino para a faixa. Lista e acesso à criação.

Reutiliza a faixa selecionada. A transição é coordenada por flags e pode envolver três folhas consecutivas.

**Decisão.** REFINAR. Mostrar alvo, destinos recentes e criação inline, confirmação com nome da playlist e retorno à origem. [components/sheets, linha 174](../crates/kazzu-app/ui/components/sheets.slint#L174)

## Nova playlist · 7/10

Criar lista nomeada, eventualmente com faixa inicial. Campo e ação de envio.

Entrada curta e finalidade clara. O levantamento da folha com teclado depende de geometria heurística.

**Decisão.** REFINAR. Foco no nome, erro persistente junto ao campo, envio único e retorno à lista criada ou ao seletor original. [components/sheets, linha 214](../crates/kazzu-app/ui/components/sheets.slint#L214)

## Importar Spotify · 4/10

Importar metadados de playlists e salvas. Alternador, token, enviar ou visitante.

Token mascarado e importação no worker/thread. Formulário fecha e limpa token antes do resultado. Uma faixa importada abre o Spotify.

**Decisão.** REDESENHAR. Identificar importação de metadados antes do envio. Resumo persistente de lidas, ignoradas e erro; rotular Abrir no Spotify. [components/sheets, linha 252](../crates/kazzu-app/ui/components/sheets.slint#L252)

## Importar YouTube · 4/10

Ler playlists autorizadas. Mesma folha com escopo técnico.

A cópia diferencia reprodução visitante, mas exige token externo e a entrada chama-se Contas. Não há login interativo nessa folha.

**Decisão.** REDESENHAR. Separar modo visitante e importar playlists. Descrever fonte, escopo e resultado em linguagem de produto; oferecer autorização guiada somente quando suportada. [components/sheets, linha 268](../crates/kazzu-app/ui/components/sheets.slint#L268)

## Pasta Android · 5/10

Selecionar raiz para indexação. Caminho, pasta Música, subir e lista assíncrona.

Há pending e erro. Após falha de listagem, o caminho anterior pode continuar selecionável; digitar caminho exige conhecimento de armazenamento.

**Decisão.** REDESENHAR. Breadcrumb, permissão e lista primeiro. Habilitar selecionar somente o diretório exibido e válido. Caminho manual como alternativa avançada. [components/sheets, linha 288](../crates/kazzu-app/ui/components/sheets.slint#L288)

## Pasta desktop · 6/10

Adicionar diretório. Formulário intermediário, Procurar e seletor do sistema.

Seletor nativo é adequado, mas exige abrir outra superfície antes de escolher a pasta.

**Decisão.** SUBSTITUIR entrada comum por seletor nativo direto. Reservar caminho manual para ação secundária e confirmar indexação junto à lista. [components/sheets, linha 311](../crates/kazzu-app/ui/components/sheets.slint#L311)

## Ajustes Android · 5/10

Localizar preferências. Busca, categorias horizontais e grupos.

Há pesquisa; várias categorias e Tudo competem em largura curta.

**Decisão.** REDESENHAR. Índice vertical com nome e resumo, busca transversal e seção empilhada com retorno. [mobile/settings, linha 43](../crates/kazzu-app/ui/mobile/settings.slint#L43)

## Ajustes desktop · 6/10

Editar preferências com mouse/teclado. Navegação interna e grupos.

Categorias separam tarefas, mas parte dos controles replica a Biblioteca.

**Decisão.** REFINAR. Índice persistente na largura ampla, detalhe à direita e trilha de retorno na estreita. [desktop/settings, linha 38](../crates/kazzu-app/ui/desktop/settings.slint#L38)

## Geral · 6/10

Escolher grade e densidade. Grupo de opções segmentadas.

Valores compartilhados são coerentes, mas duplicam comandos contextuais da Biblioteca.

**Decisão.** REFINAR. Uma fonte de verdade; em Geral mostrar padrão inicial e link para organização da biblioteca. [mobile/settings, linha 62](../crates/kazzu-app/ui/mobile/settings.slint#L62)

## Reprodução · 6/10

Definir qualidade, volume e continuação.

Qualidade e autoplay têm contexto. Volume já está no player e compete com parâmetros persistentes.

**Decisão.** REFINAR. Priorizar qualidade e continuidade; manter volume da sessão no transporte. [desktop/settings, linha 81](../crates/kazzu-app/ui/desktop/settings.slint#L81)

## Aparência · 6/10

Escolher tema, vidro, movimento e transparência.

Preferências explícitas e fallback são fortes. Parte do Android usa cores literais independentes do tema.

**Decisão.** REFINAR. Tema primeiro, material depois, acessibilidade sempre visível. Aplicar a todas as superfícies de leitura. [desktop/settings, linha 98](../crates/kazzu-app/ui/desktop/settings.slint#L98)

## Fontes e biblioteca · 5/10

Gerir provedores, diretórios e reindexação.

Pastas têm remover e reescanear. Capacidade de importar e capacidade de tocar aparecem no mesmo agrupamento.

**Decisão.** REDESENHAR. Duas listas, Fontes de reprodução e Importações. Diretórios com estado individual e ação de recuperação. [mobile/settings, linha 120](../crates/kazzu-app/ui/mobile/settings.slint#L120)

## Privacidade · 5/10

Entender dados e apagar histórico.

O aviso avisa que a exclusão é definitiva. Duplo toque no mesmo lugar dentro de quatro segundos prejudica leitura pausada; Não há conta conflita com título Contas.

**Decisão.** REFINAR. Confirmar escopo e consequência em diálogo estável com cancelar como saída. [mobile/settings, linha 150](../crates/kazzu-app/ui/mobile/settings.slint#L150)

## Atalhos · 6/10

Descobrir comandos. Lista textual em Ajustes.

Documentação interna útil no desktop. Baixo valor como categoria permanente no telefone sem teclado.

**Decisão.** REFINAR. Ajuda contextual no desktop; no telefone mostrar em assistência a teclado físico. [desktop/settings, linha 178](../crates/kazzu-app/ui/desktop/settings.slint#L178)

## Sobre e licenças · 4/10

Identificar versão e termos. Grupo expansível.

Versão vem do estado. UI declara GPL-3.0 enquanto workspace declara MIT, sem decisão demonstrada pelas telas.

**Decisão.** REFINAR. Fonte única de metadados após decisão do mantenedor. Separar licença do produto e avisos de dependências. [mobile/settings, linha 183](../crates/kazzu-app/ui/mobile/settings.slint#L183)

## Oferta de desempenho · 6/10

Escolher redução de transparência. Cartão com aceitar e adiar.

A correção proposta é concreta. Um alerta global pode competir com a tarefa atual e não explica como reverter.

**Decisão.** REFINAR. Explicar efeito e link para Aparência; preservar escolha e adiar apresentação durante entrada de texto. [app, linha 121](../crates/kazzu-app/ui/app.slint#L121)
