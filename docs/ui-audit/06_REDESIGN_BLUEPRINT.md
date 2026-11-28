# Blueprint conceitual por superfície

## Contrato para implementação futura

Cada destino mantém consulta, filtro, item focado e posição. Uma rota de detalhe guarda tipo, ID e origem. Na abertura de uma folha, guardar o disparador para restaurar foco. Voltar fecha a camada mais alta. O transporte continua independente da página. Em Android, folha inferior respeita teclado e os quatro insets. Desktop amplo usa painel lateral para fila e formulário centrado para criação. A largura útil da região determina quando empilhar.

Cada coleção distingue carga inicial, resultado parcial, cache desatualizado, pronto, vazio real e erro com nova tentativa. Durante atualização preservar conteúdo utilizável. Oferecer operações de áudio somente com fonte reproduzível. Ações externas identificam o serviço antes do clique.

### Início Android

REDESENHAR. Com histórico, mostrar primeiro a última faixa e sua capa. Sem acervo, oferecer Buscar e Adicionar pasta. Preencher escolhas somente com itens disponíveis. Origem atual para migração: [mobile/home, linha 108](../crates/kazzu-app/ui/mobile/home.slint#L108).

### Explorar Android

REMOVER a aba atual. Manter gêneros como atalhos opcionais da descoberta e identificar o termo na Busca. Origem atual para migração: [mobile/home, linha 41](../crates/kazzu-app/ui/mobile/home.slint#L41).

### Início desktop

REFINAR. Abrir com retomada, uma seleção editorial e biblioteca local. Ocultar coleções indisponíveis. Preservar atualizar com cache visível. Origem atual para migração: [desktop/home, linha 20](../crates/kazzu-app/ui/desktop/home.slint#L20).

### Busca Android

REDESENHAR. Fixar campo, fonte e tipo acima dos estados. Vazio deve oferecer limpar tipo e buscar em outra origem mantendo o termo. Origem atual para migração: [mobile/search, linha 63](../crates/kazzu-app/ui/mobile/search.slint#L63).

### Busca desktop

REDESENHAR. Campo persistente com foco por Ctrl+K, histórico imediato, resultados por tipo em lista; destacar melhor resultado sem monopolizar a altura. Origem atual para migração: [desktop/search, linha 43](../crates/kazzu-app/ui/desktop/search.slint#L43).

### Biblioteca Android

REDESENHAR. Começar pelas coleções e último acesso, mover estatísticas para histórico e densidade para menu de apresentação. Origem atual para migração: [mobile/library, linha 25](../crates/kazzu-app/ui/mobile/library.slint#L25).

### Biblioteca desktop

REFINAR. Categoria à esquerda da área de conteúdo, busca na coleção e ordenação junto ao título. Mostrar Criar como primária apenas em Playlists. Origem atual para migração: [desktop/library, linha 22](../crates/kazzu-app/ui/desktop/library.slint#L22).

### Playlists da biblioteca

REDESENHAR. Lista ordenável com nome, quantidade, origem, última alteração e ações de gestão apenas para listas editáveis. Origem atual para migração: [mobile/library, linha 108](../crates/kazzu-app/ui/mobile/library.slint#L108).

### Álbuns da biblioteca

REFINAR. Categoria deve conter catálogo completo com paginação e ordenação. Prateleiras resumidas ficam na Home. Origem atual para migração: [mobile/library, linha 145](../crates/kazzu-app/ui/mobile/library.slint#L145).

### Faixas locais

SUBSTITUIR o índice intermediário por lista pesquisável. No vazio oferecer pasta; com dados manter indexação em segundo plano e progresso contextual. Origem atual para migração: [mobile/library, linha 206](../crates/kazzu-app/ui/mobile/library.slint#L206).

### Artistas da biblioteca

REFINAR. Ordenar pelo nome, oferecer artista desconhecido para tags incompletas e restaurar a posição ao voltar. Origem atual para migração: [mobile/library, linha 168](../crates/kazzu-app/ui/mobile/library.slint#L168).

### Detalhe de artista

REFINAR. Cabeçalho compacto com retrato, faixas populares e discografia; empilhar álbum com retorno ao mesmo artista. Origem atual para migração: [mobile/detail, linha 90](../crates/kazzu-app/ui/mobile/detail.slint#L90).

### Detalhe de álbum

REFINAR. Título em duas linhas, artista navegável, ações agrupadas, ordem do disco e lista; reduzir hero em paisagem. Origem atual para migração: [desktop/detail, linha 48](../crates/kazzu-app/ui/desktop/detail.slint#L48).

### Detalhe de playlist

REDESENHAR. Identificar proprietária ou importada e fonte externa. Ações de edição persistentes, confirmação ao excluir e desfazer remoção. Origem atual para migração: [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64).

### Curtidos

REFINAR. Contagem e filtro por origem. No vazio explicar o coração e ligar à Busca. Descurtir atualiza a lista e oferece desfazer. Origem atual para migração: [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70).

### Todos os álbuns

SUBSTITUIR pela própria categoria completa da Biblioteca, preservando posição ao abrir um álbum. Origem atual para migração: [desktop/detail, linha 63](../crates/kazzu-app/ui/desktop/detail.slint#L63).

### Coleção de músicas

UNIFICAR com Faixas locais. Cabeçalho com contagem, tocar, aleatório e filtro. Nome de pasta como dado secundário. Origem atual para migração: [mobile/detail, linha 70](../crates/kazzu-app/ui/mobile/detail.slint#L70).

### Detalhe de gênero

REFINAR. Para categorias com metadados usar lista filtrada; rotular como busca os chips que só lançam consulta. Alcance de cada entrada depende dos dados. Origem atual para migração: [desktop/detail, linha 64](../crates/kazzu-app/ui/desktop/detail.slint#L64).

### Baixados

REMOVER a entrada até existir download com estado por faixa. Preservar músicas locais como destino distinto. Origem atual para migração: [mobile/detail, linha 66](../crates/kazzu-app/ui/mobile/detail.slint#L66).

### Mini-player Android

REDESENHAR. Capa real, título, pausar e avançar no compacto. Levar anterior e favorito ao expandido quando faltar largura. Origem atual para migração: [mobile/chrome, linha 101](../crates/kazzu-app/ui/mobile/chrome.slint#L101).

### Rodapé desktop amplo

MANTER. Três regiões, identidade, transporte e utilidades. Colapsar utilidades gradualmente e manter entrada de fila. Origem atual para migração: [desktop/nowbar, linha 31](../crates/kazzu-app/ui/desktop/nowbar.slint#L31).

### Rodapé desktop compacto

REDESENHAR. Manter play e título prioritários; menu de utilidades e fila acessíveis por teclado, seek ao expandir. Origem atual para migração: [desktop/chrome, linha 18](../crates/kazzu-app/ui/desktop/chrome.slint#L18).

### Player Android expandido

REDESENHAR. Fixar transporte e linha de tempo acima do inset inferior; adaptar capa ao espaço restante, manter detalhes secundários roláveis e ocultar capacidades ausentes. Origem atual para migração: [mobile/player, linha 40](../crates/kazzu-app/ui/mobile/player.slint#L40).

### Player desktop expandido

REFINAR. Capa e identificação à esquerda, console e fila à direita conforme largura; erro com tentar novamente no mesmo lugar. Origem atual para migração: [desktop/player, linha 135](../crates/kazzu-app/ui/desktop/player.slint#L135).

### Fila dentro do player Android

SUBSTITUIR por apresentação touch da fila comum. Separar Tocando e A seguir; mover por alça com alternativa acessível, remover com desfazer. Origem atual para migração: [mobile/player, linha 247](../crates/kazzu-app/ui/mobile/player.slint#L247).

### Fila compartilhada

REFINAR. Atual independente do vazio de próximas. Limpar somente próximas com desfazer. Conservar painel aberto durante edição. Origem atual para migração: [components/sheets, linha 75](../crates/kazzu-app/ui/components/sheets.slint#L75).

### Letra Android

REMOVER entrada atual. CRIAR leitor apenas com fonte, estados de carregamento, ausência por faixa e opção de seguir sincronização. Origem atual para migração: [mobile/player, linha 274](../crates/kazzu-app/ui/mobile/player.slint#L274).

### Menu de faixa

REFINAR. Agrupar tocar, organizar e navegar. Filtrar por capacidade da faixa e preservar origem para voltar. Origem atual para migração: [components/sheets, linha 129](../crates/kazzu-app/ui/components/sheets.slint#L129).

### Adicionar à playlist

REFINAR. Mostrar alvo, destinos recentes e criação inline, confirmação com nome da playlist e retorno à origem. Origem atual para migração: [components/sheets, linha 174](../crates/kazzu-app/ui/components/sheets.slint#L174).

### Nova playlist

REFINAR. Foco no nome, erro persistente junto ao campo, envio único e retorno à lista criada ou ao seletor original. Origem atual para migração: [components/sheets, linha 214](../crates/kazzu-app/ui/components/sheets.slint#L214).

### Importar Spotify

REDESENHAR. Identificar importação de metadados antes do envio. Resumo persistente de lidas, ignoradas e erro; rotular Abrir no Spotify. Origem atual para migração: [components/sheets, linha 252](../crates/kazzu-app/ui/components/sheets.slint#L252).

### Importar YouTube

REDESENHAR. Separar modo visitante e importar playlists. Descrever fonte, escopo e resultado em linguagem de produto; oferecer autorização guiada somente quando suportada. Origem atual para migração: [components/sheets, linha 268](../crates/kazzu-app/ui/components/sheets.slint#L268).

### Pasta Android

REDESENHAR. Breadcrumb, permissão e lista primeiro. Habilitar selecionar somente o diretório exibido e válido. Caminho manual como alternativa avançada. Origem atual para migração: [components/sheets, linha 288](../crates/kazzu-app/ui/components/sheets.slint#L288).

### Pasta desktop

SUBSTITUIR entrada comum por seletor nativo direto. Reservar caminho manual para ação secundária e confirmar indexação junto à lista. Origem atual para migração: [components/sheets, linha 311](../crates/kazzu-app/ui/components/sheets.slint#L311).

### Ajustes Android

REDESENHAR. Índice vertical com nome e resumo, busca transversal e seção empilhada com retorno. Origem atual para migração: [mobile/settings, linha 43](../crates/kazzu-app/ui/mobile/settings.slint#L43).

### Ajustes desktop

REFINAR. Índice persistente na largura ampla, detalhe à direita e trilha de retorno na estreita. Origem atual para migração: [desktop/settings, linha 38](../crates/kazzu-app/ui/desktop/settings.slint#L38).

### Geral

REFINAR. Uma fonte de verdade; em Geral mostrar padrão inicial e link para organização da biblioteca. Origem atual para migração: [mobile/settings, linha 62](../crates/kazzu-app/ui/mobile/settings.slint#L62).

### Reprodução

REFINAR. Priorizar qualidade e continuidade; manter volume da sessão no transporte. Origem atual para migração: [desktop/settings, linha 81](../crates/kazzu-app/ui/desktop/settings.slint#L81).

### Aparência

REFINAR. Tema primeiro, material depois, acessibilidade sempre visível. Aplicar a todas as superfícies de leitura. Origem atual para migração: [desktop/settings, linha 98](../crates/kazzu-app/ui/desktop/settings.slint#L98).

### Fontes e biblioteca

REDESENHAR. Duas listas, Fontes de reprodução e Importações. Diretórios com estado individual e ação de recuperação. Origem atual para migração: [mobile/settings, linha 120](../crates/kazzu-app/ui/mobile/settings.slint#L120).

### Privacidade

REFINAR. Confirmar escopo e consequência em diálogo estável com cancelar como saída. Origem atual para migração: [mobile/settings, linha 150](../crates/kazzu-app/ui/mobile/settings.slint#L150).

### Atalhos

REFINAR. Ajuda contextual no desktop; no telefone mostrar em assistência a teclado físico. Origem atual para migração: [desktop/settings, linha 178](../crates/kazzu-app/ui/desktop/settings.slint#L178).

### Sobre e licenças

REFINAR. Fonte única de metadados após decisão do mantenedor. Separar licença do produto e avisos de dependências. Origem atual para migração: [mobile/settings, linha 183](../crates/kazzu-app/ui/mobile/settings.slint#L183).

### Oferta de desempenho

REFINAR. Explicar efeito e link para Aparência; preservar escolha e adiar apresentação durante entrada de texto. Origem atual para migração: [app, linha 121](../crates/kazzu-app/ui/app.slint#L121).

## Especificação das transições mais sensíveis

| Origem e intenção | Transição proposta | Estado a preservar | Saída ou recuperação |
|---|---|---|---|
| Busca por “Ana”, tipo Artistas, origem Biblioteca | Mostrar tipos disponíveis nessa fonte antes da consulta. Se houver busca apenas por faixa e álbum, explicar e oferecer essas duas opções. | Texto, teclado, origem. | Trocar para Tudo ou outra fonte sem redigitar. Resultado vazio mantém o controle visível. |
| Busca → artista → disco → faixa | Empilhar artista e disco sob Busca. Tocar cria contexto do disco. | Scroll dos resultados, item da discografia, faixa atual. | Voltar do disco chega ao artista. Voltar do artista chega ao mesmo resultado. Pausar não muda a página. |
| Mini-player → player → fila → editar | O player abre sobre a aba. Fila mostra atual, próximas e total. Remover mantém posição e oferece desfazer. | Aba original, posição da lista de próximas, modo de repetição e transporte. | Voltar fecha fila, depois player. Se acabou a lista, mostrar “Nenhuma próxima faixa” com a atual ainda identificada. |
| Biblioteca → playlist importada → faixa Spotify | Mostrar origem ao lado de cada item externo relevante. Abrir no Spotify tem ícone e nome explícitos. | Seleção e scroll da playlist. | Cancelamento ou indisponibilidade do manipulador não deve trocar Tocando agora. Mostrar recuperação no detalhe. |
| Uma faixa → criar playlist durante adição | Uma folha com duas etapas e resumo da faixa fixo no topo. Seleção de existente tem um envio; criar pede nome. | Identidade da faixa e entrada que chamou o menu. | Erro preserva nome. Sucesso confirma playlist e volta à faixa. Fechar não altera a coleção. |
| Conta → importação longa | Após autorização, mostrar origem, operação atual e, quando disponíveis, contagens. Ocultar segredo após submissão. | Resultado acessível em Fontes, mesmo após sair da folha. | Sem acesso a uma lista distingue importação parcial de falha total. Credencial expirada retorna à entrada. |
| Pasta → diretório sem permissão | Suspender seleção enquanto lista. Identificar diretório da tentativa e permissão necessária. | Última localização válida como destino explícito de Voltar. | “Adicionar esta pasta” só age sobre o caminho válido que a pessoa vê. Depois, progresso no item da biblioteca. |
| Aparência → reduzir transparência | Aplicar material sólido em dock, menu, toast e player com contraste equivalente. | Destino, posição, foco e ação em curso. | Preferência continua inteligível ao mudar tema do sistema. |
| Privacidade → limpar histórico | Resumo separa histórico de busca, escuta, arquivos e playlists. Confirmar ou cancelar em diálogo estável. | Foco no controle de origem. | Sucesso atualiza estatísticas e histórico. Vazio de primeira execução não sugere perda de arquivos. |

## Contrato de composição por espaço disponível

- **Telefone em retrato.** Cabeçalho, corpo rolável e dock são regiões irmãs. Medir reserva pelo dock real. No player, reservar primeiro transporte, tempo e área segura; distribuir o restante entre capa e identificação. Se ainda faltar altura, recolher volume para controle secundário antes de exigir scroll para pausar.
- **Telefone em paisagem.** Arte e identificação em uma coluna, transporte na outra. Aplicar inset dos dois lados. Quando há teclado, formulário ocupa o espaço restante, com ação principal após o campo e sempre alcançável. O dock cede prioridade à entrada de texto.
- **Tablet Android.** Manter gesto Voltar e alvos de toque móveis. Usar espaço adicional em grade de biblioteca e painel de fila quando couberem títulos e controles. A troca de orientação preserva rota e posição de reprodução.
- **Desktop estreito.** Sidebar reduzida mantém foco, nome acessível e ajuda ao hover. O ponto de redução do rodapé considera largura após a sidebar. Mover primeiro volume e utilidades para Mais; manter identidade da faixa, play e entrada de fila.
- **Desktop amplo.** Sidebar, conteúdo e possíveis painéis têm larguras independentes. Fila pode ocupar lateral sem cobrir toda a coleção. Atalhos de transporte não se aplicam a campo editável; Esc fecha o painel superior e repõe o foco. Expansão do player ocupa a janela; fullscreen do sistema seria decisão separada.

## Critérios para a futura revisão de design

A primeira sessão sem rede deve terminar em uma ação possível. Uma busca com zero correspondências deve permitir desfazer o último filtro em uma ação. Reordenar três músicas deve manter aberta a fila e anunciar a nova posição. Um título longo deve continuar identificável em texto completo ao expandir. A ausência de capa nunca deve produzir um card clicável sem nome. A soma de linhas, teclado e inset não pode remover a ação principal de uma folha. Cada critério descreve uma validação futura, sem alegar aprovação em execução.
