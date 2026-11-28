# Estrutura da interface

O ponto de entrada é `crates/kazzu-app/ui/app.slint`. A janela possui o shell e, em camadas acima dele, o player expandido, as folhas e o aviso. Cada página mantém a própria rolagem. O rodapé e a ilha são irmãos da região rolável. No Android o mini-player tem largura, posição e condição de existência próprias. Fechar o player devolve à seção de origem.

| Responsabilidade | Onde alterar | Regra |
|---|---|---|
| Composição | `ui/desktop`, `ui/mobile` | Usar plataforma para escolher a navegação e dimensão útil para compor a tela. |
| Componentes | `ui/components` | DiscoveryFeed recebe o cabeçalho da plataforma; PlayerConsole é dono do seek, transporte, volume e acesso à fila. |
| Estado visual | `theme/app-contract.slint`, `nav-state.slint`, `metrics.slint` | Consulta, tipo, fonte, seleção de biblioteca, visibilidade dos painéis e rolagem pertencem à apresentação. |
| Histórico | `controller/mod.rs` | Cada NavTab guarda visitas tipadas a DetailRef ou RemoteRef e o deslocamento. Pedidos recebem uma sequência para rejeitar respostas antigas. |
| Ligação Rust | `app.rs`, `view.rs`, `slint_view.rs`, `vm.rs` | Os callbacks convertem tipos Slint em comandos. A View publica modelos e estados observáveis. |
| Trabalho de coleção | `worker.rs` | O adaptador chama operações existentes de kazzu-library na thread de trabalho e comunica o resultado. |

**Ciclo de uma visita.** Abrir um álbum registra sua rota. Abrir outra página guarda o deslocamento da anterior. Voltar remove a visita corrente e pede o detalhe precedente ao backend. A resposta válida restaura faixas, ações, origem de reprodução e deslocamento. Trocar de aba invalida respostas pendentes e mantém a pilha de origem. Selecionar novamente a aba ativa retorna à raiz. A pilha é de sessão; não é persistida no disco.

A janela distribui Escape e Voltar primeiro ao painel ativo. Atalhos globais de reprodução ficam suspensos com folha aberta. O guardião de acessibilidade dos controles desabilitados vale também para ativação assistiva. Resize, mudança de tema e chegada de capa são atualizações de apresentação. Não reiniciam o motor.

Os cartões e linhas de navegação enviam uma chave de foco por controle. Ela combina superfície, modelo e ID (e índice para ocorrências numeradas). O controlador associa a chave à seção ou visita corrente. Ao retornar, apenas o controle habilitado com aquela chave solicita foco. Se a origem foi removida do catálogo, permanece o foco padrão da página. Busca continua aceitando digitação ao abrir pelo atalho.
