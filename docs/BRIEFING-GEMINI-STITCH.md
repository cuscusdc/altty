# Kazzu — briefing de telas e UI/UX para Gemini Stitch

## Objetivo e contexto

Criar o design de um aplicativo de música chamado **Kazzu**, em português brasileiro, priorizando Android e telas de 360–412 dp. O app reúne streaming e arquivos locais. A pessoa deve conseguir descobrir, buscar e tocar músicas com poucos passos, sem cadastro obrigatório. A experiência deve ser elegante, calma, intuitiva e centrada nas capas dos álbuns.

Este documento sintetiza o brief do produto e a **proposta de UX e direção visual v4**, que ainda aguardava aprovação do autor quando foi escrita. Trate as decisões v4 como direção para os mockups, não como comprovação de que já estão implementadas. As decisões listadas no final permanecem abertas. Os layouts são referências para implementação posterior em Rust + Slint; não é necessário gerar código funcional nesta etapa.

## Direção visual

- **Conceito:** vitrine de discos; interface neutra, com a cor vinda das capas. Atmosfera refinada, Apple-like, sem copiar interfaces ou assets de terceiros.
- **Glassmorphism real:** superfícies flutuantes translúcidas com desfoque do conteúdo atrás, borda fina iluminada, brilho discreto e sombra suave. O vidro precisa revelar formas e cores, não parecer um cartão preto opaco.
- **Fundo:** neutro e liso. Evitar malhas coloridas, neon, manchas, degradês decorativos, excesso de cápsulas e cartões dentro de cartões. A capa e seu reflexo fornecem a cor no player.
- **Tema escuro:** fundo `#07080a`, superfície `#16171a`, texto `#f2f3f6`, secundário `#aeb1b6`, acento laranja Brasa `#ff7345`.
- **Tema claro:** fundo `#f3f4f6`, superfície `#fcfdfe`, texto `#0f1217`, secundário `#4d5057`, acento `#cc2c05`.
- **Acento:** usar com parcimônia para indicar música atual, curtida, estado ligado, links e foco. Ele não colore a interface inteira.
- **Tipografia:** Inter e Inter Display; pesos 400, 600 e 700. Títulos grandes 34/40 dp; títulos de seção 22/28; corpo 17/22; texto secundário 15/20; metadados 13/18; rótulos mínimos de 12/16.
- **Ícones:** próprios, simples, traço uniforme e pontas arredondadas; versão preenchida para estado ativo. Usar os assets existentes em `design/v4/assets/` como referência de marca.
- **Espaçamento:** escala de 4, 8, 12, 16, 20, 24, 32 e 48 dp; margens laterais de 20 dp, ou 16 dp em telas estreitas. Cantos de 12 dp nas capas, 14 dp em campos/grupos, 22 dp em cartões e 32 dp nas folhas; cápsula apenas onde fizer sentido.
- Capas de demonstração devem parecer artes reais e variadas, com contraste e detalhe suficientes para demonstrar o vidro; usar imagens autorizadas ou placeholders, sem copiar capas protegidas.

## Estrutura compartilhada e navegação

A navegação principal no celular fica no rodapé, acessível ao polegar:

1. Cápsula de vidro flutuante com **Início**, **Explorar** e **Biblioteca**.
2. Botão circular de **Buscar** separado, ao lado da cápsula, na mesma linha. Não colocar uma barra fixa de busca no topo das outras telas.
3. **Mini-player** de 60 dp acima da navegação quando houver música: capa, título, artista, tocar/pausar, próxima faixa e progresso discreto na borda inferior. A cápsula é tingida pela capa atual, com texto branco.
4. Marca Kazzu no canto superior esquerdo e engrenagem de **Ajustes** no superior direito. Nas abas internas, título grande abaixo da marca; no Início, a marca ocupa o papel de título.
5. Conteúdo rola por trás dos elementos flutuantes, com espaço inferior para que o último item nunca fique escondido. O cabeçalho vira uma barra compacta de vidro quando a página rola.

A barra principal e o mini-player continuam visíveis em listas e detalhes. O player expandido cobre a tela inteira; fila e menus abrem como folhas sobre a tela atual. Cada aba preserva sua posição e seu contexto. O botão Voltar fecha primeiro a camada mais recente; no player, recolhe para o mini-player; na busca, limpa texto antes de retornar à aba anterior.

## Inventário completo de telas e estados

### 1. Início

**Objetivo:** permitir que a pessoa encontre algo para ouvir imediatamente.

- Topo com marca e Ajustes, sem título redundante e sem barra de pesquisa.
- Primeiro uso: aviso discreto e dispensável: **“Pronto para tocar. O YouTube Music já vem ligado. Não precisa de conta.”** Ação: **Entendi**. Sem onboarding obrigatório nem solicitação antecipada de permissões.
- Conteúdo em prateleiras: **Para ouvir agora**, **Faixas para você**, **Mixes**, **Álbuns em destaque** e **Neste aparelho**. Cartões com capas dominantes, títulos claros e, quando fizer sentido, uma justificativa breve como “Em alta hoje”.
- O primeiro conteúdo visual deve aparecer cedo, sem banners ou chips ocupando a maior parte da dobra. Toque em uma faixa ou destaque inicia a reprodução.
- Desenhar também: carregando com esqueletos; conteúdo de cache com aviso **“Sem atualizar”**; sem conexão com acesso à música local; erro com **Tentar de novo** e **Abrir a Biblioteca**.

### 2. Explorar

**Objetivo:** descobrir novidades sem saber exatamente o que buscar.

- Marca, Ajustes e título **Explorar**.
- Seções para **Lançamentos**, **Momentos e gêneros** e **Em alta**, com prateleiras de capas e ladrilhos neutros para categorias.
- Ao tocar em uma categoria, coleção ou lançamento, abrir sua lista ou detalhe, mantendo o contexto da aba.
- Prever conteúdo carregando, vazio, erro e sem conexão, com ações claras quando aplicáveis.

### 3. Buscar

**Objetivo:** localizar rapidamente músicas, artistas, álbuns e playlists.

- A busca é uma tela própria aberta pelo botão circular inferior. Ao entrar, campo de texto recebe foco e teclado abre. Placeholder: **“Músicas, artistas, álbuns”**; incluir ação para limpar.
- Estado inicial: **Buscas recentes**, ação **Limpar**, e linha **Reconhecer pelo som** com descrição e etiqueta **Em breve**.
- Digitando: sugestões relevantes. Resultados: **Melhor resultado**, seguido de seções de músicas, álbuns, artistas e playlists. Filtro segmentado **Tudo / Músicas / Álbuns / Artistas / Playlists** aparece quando há resultados.
- Música toca ao ser selecionada; outros tipos abrem detalhes. Sem resultado: **“Nada para ‘termo’”** e orientação para conferir a grafia. Falha: preservar a consulta e oferecer **Tentar de novo**. Offline: avisar que a busca é apenas no aparelho.

### 4. Biblioteca

**Objetivo:** reunir o que a pessoa salvou ou possui localmente.

- Marca, Ajustes e título **Biblioteca**.
- Grade 2 × 2 com **Baixados**, **Curtidos**, **Álbuns** e **Playlists**, usando arte como elemento principal e contagens quando disponíveis. **Baixados** aparece marcado **Em breve**.
- Botão visível **Criar playlist**. Abaixo, seções **Suas playlists**, **Neste aparelho** e **Tocadas há pouco**.
- Sem pastas locais: estado **“Traga a música do aparelho”**, explicação breve e ação **Adicionar pasta**. Pedir permissão só após a ação da pessoa; usar seletor do sistema. Representar varredura com progresso e contagem, cancelamento e permissão negada com caminho para ajustes do sistema.
- Variantes: biblioteca preenchida e vazia; ausência de capa com placeholder discreto.

### 5. Listas da biblioteca

- **Curtidos:** lista de faixas com miniatura, nome, artista e menu Mais; estado vazio explica que tocar no coração adiciona músicas.
- **Álbuns:** grade ou lista de capas e metadados; toque abre detalhe do álbum.
- **Playlists:** lista ou grade com nome, capa/mosaico, contagem e ação Criar playlist; toque abre detalhe.
- **Baixados:** no escopo atual, apresentar como indisponível com **Em breve**, sem simular downloads funcionais.
- **Neste aparelho:** pastas, músicas ou álbuns locais, origem visível e ação para adicionar/ler pastas.

### 6. Detalhes de álbum, playlist, artista e coleção

Criar uma família visual coerente, com variações conforme o conteúdo:

- Capa ou imagem de destaque; título grande, artista/criador, origem e metadados pertinentes; botões **Tocar** e **Aleatório**.
- Lista de faixas com número ou miniatura, título, artista, duração e botão **Mais**. Música atual claramente identificada. Capa, botões e placas sobre imagem podem usar vidro escuro com texto branco.
- Álbum: artista, ano quando disponível, quantidade e duração, opção de salvar. Playlist: criador/origem e faixas. Artista: identidade visual, músicas e álbuns. Coleção, mix, momento e parada: título contextual e conteúdo correspondente.
- Playlist vazia: **“Esta playlist está vazia”** e **“Toque em Mais numa música e escolha Adicionar à playlist.”**
- Prever carregamento, erro, conteúdo vazio e origem local/remota. Navegação inferior e mini-player permanecem disponíveis.

### 7. Player expandido

**Tela assinatura do produto.** Abrir ao tocar ou arrastar para cima o mini-player.

- Capa quadrada, nítida e inteira, de ponta a ponta na largura; botões de fechar e Mais sobre ela, além de **“Tocando de Nome”**.
- Abaixo da capa, um reflexo da própria imagem, suavemente desfocado. Um único console de vidro escuro sobe sobre a base da capa e o reflexo, com cantos superiores amplos; a cor do conjunto vem da capa, sem degradê decorativo.
- Dentro do console: título, artista, coração; barra de progresso com tempo decorrido e restante, origem/formato quando conhecido; controles grandes de aleatório, anterior, tocar/pausar, próxima e repetir; volume; botões **Este aparelho** e **Fila**.
- O botão central de tocar é circular, branco e destacado. Texto branco legível mesmo sobre capas claras. Mini-player e player compartilham a cor da capa.
- **Este aparelho** informa a saída atual; não simular troca de dispositivo. Letra e timer ficam no menu com **Em breve**.
- Desenhar reprodução, pausa, carregamento, erro de faixa e sem conexão. Fechar por botão, gesto para baixo ou Voltar.

### 8. Fila de reprodução

- Folha de vidro sobre o player, título **Fila**, alça visual e botão de fechar.
- Grupos **Tocando agora** e **A seguir**, com capa, título, artista e ações por faixa. Tocar em um item pula para ele.
- Ações **Remover**, **Mover para cima** e **Limpar**. Remoção oferece **Desfazer**; limpar exige confirmação no próprio controle.
- Nota: **“Quando a fila acabar, o Kazzu continua com músicas parecidas. Você pode desligar isso em Ajustes.”**
- Estado vazio: **“A fila está vazia. Toque numa música para começar.”** Arrastar para reordenar é futuro; manter alternativa visível.

### 9. Menu de ações da faixa

Folha contextual acessível pelo botão Mais ou toque longo, com título e identificação da música:

- **Tocar em seguida**, **Adicionar à fila**, **Adicionar à playlist**, **Curtir / Tirar dos Curtidos**, **Ir para o álbum**, **Ir para o artista**.
- **Ver a letra**, **Timer para dormir** e **Baixar** identificados como **Em breve**, sem parecerem funcionais.
- Ações concluídas geram avisos breves; oferecer **Desfazer** onde disponível. O desfazer da adição à playlist está previsto como melhoria, não garantido no app atual.

### 10. Adicionar à playlist e criar playlist

- **Adicionar à playlist:** folha com identificação da faixa, lista de playlists existentes e opção **Nova playlist**; seleção confirma o destino com aviso.
- **Nova playlist:** folha com título, campo de nome em foco, ações **Cancelar** e **Criar**. Campo vazio desabilita Criar e explica o motivo; com nome válido, cria e abre a playlist vazia.
- Prever teclado aberto sem ocultar o campo e os botões. A criação também começa diretamente em Biblioteca.

### 11. Ajustes

Tela com título **Ajustes** e listas sólidas agrupadas, sem vidro dentro de vidro:

- **Reprodução:** qualidade do streaming (**Original**, **Alta**, **Econômica**) e opção **Continuar com músicas parecidas**.
- **Aparência:** tema **Automático / Claro / Escuro**, **Reduzir transparência** e **Reduzir movimento**; alterações visíveis imediatamente.
- **Biblioteca:** **Pastas do aparelho** e **Ler pastas de novo**.
- **Fontes de música:** YouTube Music ligado, sem conta; Spotify, Tidal e Qobuz marcados **Em breve**.
- **Privacidade:** explicar ausência de telemetria; apagar histórico de buscas e reprodução com confirmação; relatar problema gerando arquivo local que a pessoa decide enviar.
- **Sobre:** versão e licenças de código aberto.
- Oferecer, quando necessário, **“Deixar o Kazzu mais leve?”**, com **Deixar mais leve** e **Agora não**. A opção de reduzir transparência troca vidro por superfícies sólidas.

## Estados transversais e microinterações

Para cada região dependente de dados, desenhar **carregando, preenchida, vazia, erro e offline**. Preservar conteúdo em cache quando possível, comunicar sua idade ou falta de atualização sem bloquear navegação. Mensagens devem ser curtas, calmas e acompanhadas de uma ação possível. Exemplos: **“Não deu para buscar agora”**, **“Sem conexão. A música do aparelho continua aqui”**, **“Adicionada aos Curtidos”** e **“Removida da fila”**.

Curtida muda de aparência imediatamente e reverte com aviso em caso de falha. Ações destrutivas devem oferecer confirmação ou desfazer. Não criar interruptores falsos para recursos futuros. Toque longo e gestos sempre têm alternativa visível. Animações discretas: aproximadamente 120 ms para resposta, 220 ms para mudança de estado e até 420 ms para folhas e player; com movimento reduzido, usar apenas transição curta de opacidade.

## Acessibilidade e adaptação

- Alvos de toque de pelo menos **48 × 48 dp**, ainda que o ícone visível seja menor. Rótulos acessíveis para todos os ícones e controles; papel e estado anunciados por leitor de tela.
- Contraste WCAG AA: texto normal pelo menos 4,5:1; texto grande e elementos gráficos essenciais pelo menos 3:1, inclusive sobre as capas mais claras e escuras. Nunca depender só de cor para indicar estado.
- Fonte do sistema até 130%, largura mínima de 320 dp sem rolagem horizontal da página; respeitar áreas seguras e teclado. Foco visível e não oculto pela navegação flutuante.
- Reduzir transparência substitui vidro por superfícies sólidas; reduzir movimento remove deslocamentos. Garantir hierarquia e legibilidade em ambos os modos.
- **320–599 dp:** uma coluna, prateleiras horizontais, biblioteca 2 × 2, barra inferior e player em tela cheia.
- **600–1099 dp:** barra inferior centralizada até 520 dp, margens de 32 dp, grades de 3–4 colunas; player em folha central de até 520 dp; detalhe lado a lado a partir de aproximadamente 840 dp.
- **1100 dp ou mais:** navegação lateral com marca, destinos, busca e playlists; barra de reprodução inferior; conteúdo até 1180 dp; player em painel lateral ou tela cheia. Busca continua sendo uma tela, sem barra fixa no topo.

## Fluxos para prototipar

1. Abrir app → Início → tocar destaque → mini-player → player → fechar.
2. Buscar → digitar → filtrar → tocar resultado ou abrir detalhe.
3. Player → curtir → aviso → Biblioteca → Curtidos.
4. Player → Fila → remover uma faixa → desfazer; limpar com confirmação.
5. Biblioteca → Criar playlist → nomear → playlist vazia; menu de uma faixa → adicionar à playlist.
6. Biblioteca → Adicionar pasta → seletor do sistema → varredura → música local.
7. Falha de rede → conteúdo em cache ou biblioteca local → tentar novamente.
8. Ajustes → reduzir transparência e movimento → interface muda imediatamente.

## Escopo e decisões em aberto

A documentação do projeto descreve como disponíveis a busca, reprodução, biblioteca local, curtidas, playlists locais e fila; a interface v4 é uma proposta de redesenho e não deve ser confundida com implementação pronta. Letras, downloads, reconhecimento pelo som, timer, troca de saída e provedores adicionais têm lugar futuro ou **Em breve**. Equalizador, Discord, sincronização, ouvir junto e retrospectiva são objetivos posteriores e não devem virar telas funcionais nesta fase.

As recomendações v4 ainda sujeitas à decisão do autor incluem: remover chips no Início; manter acento fixo; usar player escuro nos dois temas; manter volume no celular; posição do campo dentro da tela Buscar; preservar a marca K; tingir o mini-player pela capa; usar categorias neutras; e manter Letra e Timer apenas no menu. Não tratar essas recomendações como aprovação final.

## Entregáveis para o Stitch

Criar uma família consistente de telas mobile em tema escuro e claro, com componentes reutilizáveis e conteúdo realista em pt-BR. Priorizar: **Início, Explorar, Buscar (inicial/resultados/vazio), Biblioteca (cheia/vazia), listas e detalhes, Player, Fila, menu da faixa, adicionar/criar playlist e Ajustes**. Incluir versões representativas de carregamento, erro, offline e transparência reduzida; demonstrar a continuidade da navegação e do mini-player. Criar exemplos responsivos de tablet e desktop após o mobile.

**Prompt resumido para colar no Stitch:**

> Crie as telas do Kazzu, aplicativo de música mobile-first em português brasileiro, com as especificações deste documento. Estética Apple-like própria: fundos neutros quase pretos ou quase brancos, capas como fonte de cor, vidro translúcido com desfoque real apenas em elementos flutuantes, Inter e detalhes laranja discretos. Navegação inferior com Início, Explorar e Biblioteca em cápsula, busca circular separada e mini-player acima. O player deve ter capa grande de ponta a ponta, reflexo da capa e console de vidro com controles claros. Desenhe todas as telas, estados e fluxos listados, sem inventar funcionalidades prontas; marque recursos futuros como Em breve. Garanta acessibilidade, contraste e versões clara e escura.

## Fontes no repositório

- `docs/BRIEF.md`: visão e etapas do produto.
- `design/v4/ux/03-arquitetura-e-navegacao.md` a `07-backlog-e-aceite.md`: navegação, fluxos, estados e critérios.
- `design/v4/DIRECTION.md`, `design/v4/DESIGN.md` e `design/v4/tokens.json`: direção visual e componentes.
- `design/v4/prototypes/` e `design/v4/shots/`: protótipos e capturas existentes para referência visual.
