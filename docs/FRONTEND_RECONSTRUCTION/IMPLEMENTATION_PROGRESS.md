# Registro de implementação

Atualização da interface do HTML registrada em [HTML_INTEGRATION.md](HTML_INTEGRATION.md). Ela acrescenta as paletas e a rota própria de Explorar ao Kazzu existente.

O diagnóstico de partida inventariou 44 superfícies e 27 problemas, com maturidade estrutural de 4,8/10. A nota é do diagnóstico original. A tabela registra o que mudou e o que ainda precisa de comprovação. A comparação de referências e o diagnóstico precederam as edições. A implementação reutiliza código interno do Kazzu e composição própria em Slint.

| ID | Tratamento |
|---|---|
| A01 | Controles fictícios removidos do player. Letras e sessão têm capacidades desligadas por padrão; dados de saída são informativos. |
| A02 | QueueSheet substitui a segunda fila. O mesmo contrato opera nos dois shells. |
| A03 | Parar e limpar exige confirmação persistente com cancelamento. O efeito atual de parar o motor está no texto. |
| A04 | Pilha por NavTab guarda rota e deslocamento. Resposta antiga é descartada. Restauração precisa de validação adicional de foco assistivo e lista interna. |
| A05 | Os chips ficam fora do ramo Results e permanecem acessíveis durante uma consulta. |
| A06 | Adaptador publica catalog-available; fonte local restringe tipos a músicas e álbuns. Origem indisponível tem explicação. |
| A07 | Listas usam tamanho real do modelo, estados vazios e ação de busca/pasta. Eliminada grade com seis slots artificiais. |
| A08 | Explorar foi incorporado a Início. A rota legada continua compatível com navegação antiga. |
| A09 | Painel distingue token de sessão e informa destino externo. Resultado e falhas permanecem ao reabrir. |
| A10 | Tokens semânticos controlam papel, tinta, foco e acabamento. Mídia conserva contraste independente do tema. |
| A11 | Recentes no seu acervo admite histórico e indexação. Ouvidas recentemente usa consulta exclusiva ao histórico. |
| A12 | Editor de playlists da biblioteca se apoia nas mutações existentes, inclusive playlists mistas de local e remoto. |
| A13 | Faixas abre diretamente a coleção tocável. Álbuns recebe lista integral; segundo botão redundante removido. |
| A14 | Mini-player reduzido a dois transportes. Barra desktop estreita reduz utilidades quando faltar espaço. |
| A15 | Transporte expandido fica fora da rolagem em ambas as plataformas. |
| A16 | Bloqueio explícito no despachante de atalhos; Escape/Voltar permanece ativo. Ordem completa de foco por leitor pendente. |
| A17 | Confirmação de privacidade sem temporizador e com Cancelar. |
| A18 | SheetBase usa viewport rolável para corpo maior que a altura disponível; removido deslocamento estimado por fração ao focar campo. IME físico pendente. |
| A19 | Insets laterais aplicados ao shell e ao player Android. Validar recortes e formulários sobrepostos em aparelho. |
| A20 | Engrenagem explícita na Home e capa real no mini-player com fallback de disco. |
| A21 | Faixa atual e próximas têm condições independentes. |
| A22 | Seleção invalidada antes de cada listagem. Após falha, ação para voltar ao armazenamento. |
| A23 | Sem percentual inventado. Descoberta informa atividade por texto; fase com progresso usa valor do worker. |
| A24 | Removida alça sem gesto do painel. Permanecem fechar, véu e Voltar. |
| A25 | Declaração conflitante retirada de Sobre; licença histórica do projeto requer decisão do mantenedor. |
| A26 | Vitrine local considera a origem selecionada mesmo quando há lançamentos remotos. |
| A27 | Disponibilidade de seek vem da duração real do controlador. Slider desabilita mouse, teclado e ações assistivas. |

Os arquivos de aplicação alterados concentram-se em kazzu-app, modelos de apresentação, adaptadores e testes. O gerador do design e sua saída foram mantidos em sincronia. As crates de áudio, persistência, provedores e suas APIs públicas continuam a cargo do backend.
