# Fluxos e registro priorizado

Contagens indicam ativações mínimas inferidas do código, partindo do contexto indicado. Digitação, espera, distância percorrida e tentativas não estão incluídas. Não são tempos ou taxas medidos.

| Tarefa | Caminho atual | Consequência |
|---|---|---|
| Ouvir uma escolha existente | Home Android → card com faixa (1 toque) | Caminho curto, mas arte por posição impede reconhecimento pela capa. |
| Encontrar música | Dock → Buscar → digitar → faixa (2 ativações além da digitação se campo focado) | Resultado local pode chegar independentemente da rede. Se o filtro selecionado esvaziar o resultado, desaparece a faixa de filtros. |
| Retomar pela fila | Mini-player Android → fila → faixa (3) | Selecionar fecha a fila. Para continuar examinando a sequência é preciso abri-la novamente. |
| Colocar música a seguir | Mais na linha → Tocar em seguida (2) | O menu distingue próxima de acrescentar ao final. Preservar essa diferença. |
| Guardar em lista existente | Mais → Adicionar à playlist → escolher lista (3) | Contagem aceitável. A complexidade cresce para criar lista durante esse fluxo e para administrar depois. |
| Curtir | Coração no dock Android (1), ou Mais → Curtir (2) | Controlador altera o modelo e publica o estado enquanto persiste. Recuperação em falha e anúncio assistivo não verificados. |
| Explorar artista → álbum → voltar | Abrir artista → discografia → álbum → voltar | Há um detalhe aberto e uma sequência para descartar respostas antigas. O fechamento não reabre o artista anterior. |
| Adicionar pasta desktop | Biblioteca → Adicionar pasta → Procurar → confirmar no sistema | O formulário intercala uma decisão de método que a maioria dos usuários não precisa tomar. |
| Usar como visitante | Iniciar; nenhuma conta exigida | “Usar sem conta” dentro de importação apenas fecha a folha. Não constitui onboarding nem mudança de sessão. |

## Registro de problemas

P0 bloquearia uma tarefa fundamental sem alternativa comprovada. Nenhuma falha desse nível ficou demonstrada apenas pela inspeção. P1 prejudica tarefa principal, P2 exige contorno ou introduz inconsistência, P3 afeta acabamento. Esforço relativo P/M/G representa alteração local, coordenação UI–modelo ou reorganização de fluxo entre plataformas. Os caminhos Slint abaixo são relativos a `crates/kazzu-app/ui`. Rust é relativo a `crates/kazzu-app/src`.

|ID|Prioridade · esforço|Local e evidência|Consequência para o ouvinte|Intervenção|
|---|---|---|---|---|
|A01|P1 · M|mobile/player:175–189, 230, 292 e app.rs:416–426. Letras tem corpo constante, timer e saída terminam em coming_soon.|Explorar o próprio player ensina a desconfiar dos controles.|Modelar capacidade por fonte/faixa, ocultar comandos indisponíveis antes do toque.|
|A02|P1 · M|app.slint:147 escolhe entre components/sheets:99 e mobile/player:266. A segunda lista só salta.|Faltam mover/remover no contexto mais natural de edição.|Um modelo de fila com ações equivalentes, duas apresentações.|
|A03|P1 · M|components/sheets:84 e controller/mod.rs:1043. Limpar esvazia cauda e chama engine.clear sem confirmação nem desfazer nesse caminho.|Perda da sequência construída com vários comandos.|Desfazer com instantâneo da sequência e nome claro da operação.|
|A04|P1 · G|theme/nav-state:25–42, app.rs:346 e mobile/detail:96. Abrir álbum a partir do artista substitui o detalhe.|Voltar perde contexto intermediário.|Pilha tipada por destino e restauração de scroll/foco.|
|A05|P1 · M|mobile/search:158, desktop/search:140 e controller/remote.rs:585. Controles de tipo dentro do ramo results.|Ao selecionar tipo sem itens, a recuperação por outro tipo fica indisponível ali.|Chips permanecem nos estados vazio, erro e carregando.|
|A06|P1 · M|controller/remote.rs:573–621. YouTube selecionado desliga busca local, e remoto desligado não cria job; resultado vira Empty. Só músicas e álbuns locais são agregados.|“Nada” pode significar fonte desativada ou tipo não suportado.|Separar fonte indisponível, ausência de resultados e tipos consultáveis por origem.|
|A07|P1 · P|mobile/home:109–151. Slots desabilitados mantêm arte sintética e label vazio.|Primeiro acesso privilegia caixas sem conteúdo que não levam a música.|Gerar a partir do modelo, substituir conjunto vazio por ação útil.|
|A08|P2 · M|mobile/home:41–43, 175–181 e mobile/chrome:155. Explorar é âncora e gêneros são buscas literais.|O usuário espera catálogo próprio e encontra a mesma Home reposicionada.|Absorver seção em Início e manter Buscar como destino.|
|A09|P1 · M|components/sheets:252–282, controller/mod.rs:315 e controller/remote.rs:644. Entrada pede token e fecha antes do resultado.|Cliente espera autenticação e reprodução do Spotify, mas recebe importação de metadados e abertura externa.|Nomear operação, explicar destino antes da ação, manter status e erro persistentes.|
|A10|P2 · M|theme/mobile-reference:1 e tokens:15. Paleta da Home/dock independe do tema.|Preferência clara/escura tem aplicação parcial.|Papéis semânticos únicos, tema de mídia explicitamente delimitado.|
|A11|P2 · P|worker.rs:568 e mobile/home:188. recent_tracks completa lista com adições recentes; texto sugere faixas ouvidas.|Histórico pode incluir música nunca ouvida.|Separar Adicionadas de Continue ouvindo pelo evento real.|
|A12|P1 · G|app.rs:441–452 e menu em components/sheets:157. Criar e acrescentar expostos sem fluxo comparável de remover faixa ou renomear lista.|Organizar playlists passa a ser essencialmente cumulativo.|Ações por propriedade, edição, desfazer e ordenação.|
|A13|P2 · M|mobile/library:165,212, desktop/library:104,131. Categorias direcionam a novas coleções.|Caminho adicional até lista completa e perda de correspondência entre categoria e conteúdo.|Exibir todos os itens diretamente na categoria.|
|A14|P2 · M|desktop/chrome:42 e mobile/chrome:133. Quatro botões no dock móvel; largura do título desktop desconta 176 px.|Risco de identificação insuficiente em janela curta e título longo.|Orçamento de largura por região, overflow de utilidades.|
|A15|P1 · M|mobile/player:47–60, 119–192. Transporte dentro do conteúdo rolável.|Em altura insuficiente, controles essenciais podem sair da área visível.|Fixar transporte, adaptar capa e rolar apenas conteúdo secundário.|
|A16|P2 · M|app.slint:75–99. Apenas F consulta sheet-up, globais de transporte e navegação continuam disponíveis no escopo externo.|Risco de operação atrás de modal quando evento é propagado. Comportamento real de foco não verificado.|Escopo modal, retorno ao disparador e restrição explícita dos atalhos.|
|A17|P2 · P|mobile/settings:35,157–159. Confirmação de histórico expira em 4 s.|Leitura lenta ou tecnologia assistiva pode exigir repetição.|Confirmação persistente com cancelar e lista de dados afetados.|
|A18|P2 · M|components/sheets:25–27,254–255,290–291. Altura fixa com lift proporcional.|Teclado e área útil podem encobrir ação final.|Medir viewport disponível, rolar corpo e manter rodapé alcançável.|
|A19|P2 · M|app.slint:38–51 e contrato. Insets laterais registrados sem consumidor nos layouts.|Risco em paisagem e telas com recorte lateral.|Aplicar insets horizontais no shell e palco.|
|A20|P2 · P|mobile/home:88–95 e mobile/chrome:120–123. C abre Ajustes e mini-capa é gradiente fixo.|A letra parece conta; arte não identifica a faixa.|Engrenagem e capa vinda da faixa, fallback só quando faltar imagem.|
|A21|P2 · P|components/sheets:79,85,93. Quando próximas está vazia o painel suprime Tocando agora.|Enquanto uma música toca, a instrução pede escolher música para começar.|Mostrar atual independente das próximas.|
|A22|P2 · M|components/sheets:315–330 e slint_view.rs em set_folder_listing. Erro atualiza mensagem, pode conservar folder-current anterior.|Selecionar após erro pode adicionar outra pasta.|Invalidar seleção na tentativa e habilitar apenas resultado bem-sucedido.|
|A23|P2 · P|mobile/library:228, desktop/library:136. Progresso desconhecido recebe largura fixa de 35%.|Indicador aparenta medir trabalho inexistente.|Modo indeterminado mais texto de atividade.|
|A24|P3 · P|components/sheets:33. Alça aparece no painel e não declara gesto de arrastar.|Sugestão visual de interação sem implementação local.|Remover ou implementar gesto com alternativa explícita.|
|A25|P2 · P|mobile/settings:186 e Cargo.toml, workspace.package. GPL-3.0 na interface e MIT no manifesto.|Informações institucionais contraditórias.|Solicitar decisão do mantenedor e publicar metadado único.|
|A26|P2 · P|mobile/search:82–108. Na fonte Biblioteca, lançamentos remotos existentes inibem rail local e ficam eles próprios ocultos.|Busca ociosa pode ficar sem vitrine mesmo com álbuns locais.|Calcular vazio e seções após aplicar o filtro de origem.|
|A27|P2 · P|components/atoms:560–565 e controller/mod.rs:878. Slider aceita gesto sem duração; controlador só age com duration.|Busca temporal pode parecer inerte.|Exibir progresso indeterminado e desabilitar seek quando duração é desconhecida.|

## Player e acessibilidade

O rodapé fixo e o player expandido estão em níveis distintos do scroll das páginas. O problema específico de A15 é o scroll **interno** do player. O controle de progresso comum tem trilho de 9 px, área de 48 px, incremento assistivo de 5% e teclas laterais. O handler de movimento envia seek enquanto o dedo está pressionado; custo real do scrubbing remoto não verificado. Volume usa o mesmo componente com unidade percentual. Shuffle tem realce e rótulo. Repeat-one acrescenta selo 1. Play muda o ícone e o nome. Isso merece preservação. [Scrubber](../crates/kazzu-app/ui/components/atoms.slint#L537) · [Console móvel](../crates/kazzu-app/ui/mobile/player.slint#L119)

O desktop expõe erro, origem, tempo e console; sua barra ampla mantém o acesso à fila. O compacto reduz para identidade, transporte básico e indicador fino. A performance visual do player e a continuidade do áudio entre páginas não podem ser determinadas pela árvore de widgets. O estado de buffering e erro precisa ser tão visível no Android quanto play/pause. O texto fixo “Este celular” não comprova a rota de saída física. [Player desktop](../crates/kazzu-app/ui/desktop/player.slint) · [Compacto](../crates/kazzu-app/ui/desktop/chrome.slint) · [Mobile](../crates/kazzu-app/ui/mobile/player.slint)

O benefício estrutural de Pressable é reunir semântica, Enter/Espaço, foco visível e ponteiro. A ação acessível chama clicked diretamente, enquanto TouchArea verifica enabled; validar o bloqueio assistivo real. O toast tem anúncio polite, mas a elipse visual e desaparecimento exigem alternativa persistente para erros recuperáveis. Testes futuros devem cobrir TalkBack, tecla Tab, Shift+Tab, Escape, teclado físico Android, fonte ampliada e contraste sobre arte clara e escura. [Pressable](../crates/kazzu-app/ui/components/atoms.slint#L70) · [Toast](../crates/kazzu-app/ui/components/toast.slint)
