# Roadmap de redesign

Esforço relativo para produto e implementação futura. P corresponde a alterações locais, M a uma família de telas ou modelos, G a coordenação transversal. Não é estimativa de calendário.

| Fase | Impacto / esforço | Dependência | Entrega e critério de saída futuro |
|---|---|---|---|
| 1. Problemas críticos de experiência | Muito alto / M | Inventário de capacidades por fornecedor e faixa | Remover comandos sem efeito, preservar filtro após vazio, impedir perda acidental de fila e seleção de pasta antiga. Uma conta visitante consegue identificar o que toca, o que é externo e o que está indisponível. A01–06, A09, A22. |
| 2. Arquitetura de navegação | Muito alto / G | Acordo sobre destinos, estados e retorno | Três destinos principais, pilha de detalhes por aba e um dono de overlays. Artista → álbum → voltar retorna ao artista e à posição. Busca conserva consulta. A04, A08, A16. |
| 3. Design system | Alto / M | Papéis funcionais definidos | Consolidar tema, estados, foco, componente de fila e semântica de ações. Contraste e fallback em tema claro, escuro e sem blur entram nos critérios futuros de validação. A10, A14, A20. |
| 4. Telas | Alto / G | Fases 2 e 3 | Home por dados disponíveis, Busca com recuperação, Biblioteca administrável, detalhes por tipo e importação com relatório. Verificar primeiro uso, cache e coleções longas. A07, A11–13, A23, A26. |
| 5. Player e fila | Muito alto / G | Correções de fase 1, estado de reprodução e componentes da fase 3 | Transporte, origem, seek, erro e fila equivalentes nos dois shells. Próximas vazia preserva Tocando agora. Editar duas linhas consecutivas é possível sem reabrir painel. A02, A03, A15, A21, A27. |
| 6. Android dedicado | Alto / M–G | Fluxos estabilizados | Tratar altura curta, teclado, insets, rotação e tablet. Foco assistivo e Voltar são critérios de aceitação. Origem inválida oferece recuperação sem perder seleção correta. A14, A17–19, A22. |
| 7. Polimento | Médio / M | Jornadas completas disponíveis | Refinar microinterações, hover, transições e textos. Revisar licença exibida com mantenedor. Validar com teclado, leitor de tela e reprodução prolongada fora desta auditoria. A20, A24–25. |

**Primeira fatia recomendada.** Escolher uma faixa local → tocar → abrir fila → alterar sequência → visitar álbum → voltar sem perder o estado. Essa fatia atravessa contrato, navegação e transporte com pouca dependência externa. Depois repetir com origem remota, sem rede e com metadados externos. Só promover mudanças visuais quando a jornada inteira puder ser avaliada.
