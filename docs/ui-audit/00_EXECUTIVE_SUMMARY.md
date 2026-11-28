# Kazzu · Diagnóstico executivo

**4,8/10 em maturidade estrutural. O Kazzu é um projeto em desenvolvimento com um núcleo de player plausível e uma interface que apresenta ações, estados e destinos com significados inconsistentes.** O redesign deve começar pelo que acontece ao buscar, tocar, voltar e editar a fila. Trocar o acabamento antes disso preservaria os problemas centrais.

## Escopo e confiança

Revisão `c2a2d0239e4e6dd77dd357596f6d91a8f33fa78d`. Inspeção estática do app compilado a partir de [app.slint](../crates/kazzu-app/ui/app.slint), seus vínculos [Rust](../crates/kazzu-app/src/app.rs), modelos, controlador e worker. Comparação com documentação pública, código/repositórios e descrições de capturas das referências. Nenhum app foi executado, nenhum build ou teste foi rodado. Os diretórios `design/`, a galeria e o comando de captura são material auxiliar, fora do inventário de superfícies de produção. Os resultados históricos de testes citados em outros documentos do repositório ficaram fora da avaliação.

- **Confirmado no código** significa ligação, ausência de ligação ou decisão de layout verificável por leitura.
- **Risco estrutural** significa consequência plausível da composição, dependente de renderer, dispositivo, dados ou foco.
- **Não verificado** inclui contraste da composição real, rolagem e hit testing em execução, leitores de tela, teclado virtual, permissões, áudio, rede, latência, fluidez, tamanhos renderizados e experiência com contas reais.
- A severidade abrange problemas de experiência. O dossiê não atribui P0 a falhas de execução sem evidência. Há vários P1 que, em conjunto, impedem recomendar a interface como produto acabado.

## Respostas às 12 perguntas

| Pergunta | Resposta |
|---|---|
| 1. Estado atual | Os fluxos básicos têm caminhos de código, mas Busca, fila e biblioteca perdem contexto ou controle em cenários comuns. Os dois shells compartilham o backend, porém a experiência diverge inclusive no significado de ações. |
| 2. Produto profissional? | Ainda em desenvolvimento. Há decisões profissionais pontuais, como cromo fixo e adaptação por plataforma. O conjunto expõe controles sem entrega e não sustenta uma sessão previsível de ponta a ponta. |
| 3. Cinco maiores problemas | **Ações sem capacidade** (A01), **fila com contrato variável e limpeza imediata** (A02/A03), **histórico de navegação raso** (A04), **Busca sem recuperação previsível por tipo e fonte** (A05/A06), **Biblioteca sem administração de playlists e com acervo fragmentado** (A12/A13). |
| 4. Reconstruir primeiro | Player Android e fila, Home/Explorar Android, Busca nas duas plataformas e Biblioteca. Ajustes e detalhes precisam de reorganização interna e divisão por propósito. |
| 5. Preservar | Base de `Pressable`, `TrackRow`, `Cover`, `StateCard`, `Screen`, reservas do cromo, tokens Inter/Brasa e callbacks de reprodução. Preservar a separação entre as composições desktop e Android. |
| 6. Remover | Blocos de arte sem ação, avatar C de Ajustes, tab Vídeo sem vídeo, estatísticas sem tela, letras/timer/saída/compartilhamento enquanto indisponíveis, entrada de Baixados sem entrega e descoberta duplicada na Busca. Remover a falsa alça de gesto das folhas sem arraste. |
| 7. Navegação | Reconstruir o modelo de navegação e preservar a topologia física adequada de cada plataforma. Três destinos estáveis, detalhes com histórico e overlays com foco e retorno explícitos. |
| 8. Player moderno? | A barra ampla do desktop é uma base funcional. No conjunto, ainda não. A fila e a barra recolhida retiram controles, o Android esconde a arte e exige rolagem dentro do player em condições previsíveis. Parte dos comandos secundários termina em aviso ou painel vazio. |
| 9. Direção visual | Inter, Brasa semântica, arte real dominante, base opaca de alto contraste e vidro reservado a cromo/camadas com conteúdo atrás. A cor da capa fica na atmosfera. |
| 10. Quanto redesenhar? | Estimativa de produto, não contagem de código: cerca de **dois terços das superfícies compostas** merecem reestruturação. Primitivas e motor ficam como base. Home, Busca, Biblioteca, player Android e sistema de overlays têm maior escopo. |
| 11. Ordem ideal | Visibilidade de capacidades e perdas de contexto → rotas/painéis → tokens e componentes → telas → player/fila → ergonomia Android → polimento. O player e a fila recebem correções críticas já na primeira fase. |
| 12. Recomendação | Suspenda novos efeitos e atalhos cosméticos até que ouvir, encontrar, organizar e voltar tenham comportamento coerente. Defina qualidade pelo fluxo completo com rede ausente, fila curta, janela pequena, título longo e ausência de capa. |

## Índice

[01 UI](01_UI_AUDIT.md) · [02 UX e registro de problemas](02_UX_AUDIT.md) · [03 Telas](03_SCREEN_BY_SCREEN.md) · [04 Componentes e inventário](04_COMPONENT_AUDIT.md) · [05 Referências](05_REFERENCE_COMPARISON.md) · [06 Blueprint](06_REDESIGN_BLUEPRINT.md) · [07 Direção visual](07_DESIGN_DIRECTION.md) · [08 Roadmap](08_REDESIGN_ROADMAP.md) · [09 Notas](09_SCORECARD.md)
