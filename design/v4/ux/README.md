# Kazzu v4: planejamento de UX (do zero)

Data: 2026-11-22. Autor do plano: líder de UX+UI (agente). Estado: **proposta, aguardando o autor**. Nada disto foi implementado no `kazzu-app`.

Este plano substitui `design/ux/UX_PLAN.md` (v2) como fonte de UX. O material v2/v3 continua no repositório só como estudo e como lista de erros a evitar.

## Resumo em doze linhas

1. O Kazzu é um player de música para celular (Android primeiro) que toca na hora, sem conta, com o YouTube Music já ligado e a música do aparelho ao lado.
2. As versões v2 e v3 falharam por execução: fundo em malha colorida, vidro que parecia cartão marrom, capas sem detalhe atrás do vidro, topo pesado e um player com cara de painel de botões. O diagnóstico está em `01`.
3. A regra de produto da v4: **a interface é neutra e de vidro; toda a cor vem das capas, e as capas passam por trás de tudo que flutua**.
4. Três destinos na barra cápsula (Início, Explorar, Biblioteca) e a Busca num botão redondo próprio, ao lado da barra. Nunca uma barra de busca fixa no topo.
5. O logo fica no canto superior esquerdo de todas as abas; Explorar, Biblioteca e Buscar têm o título grande logo abaixo.
6. A música atual vive num mini-player de vidro, tingido pela capa, acima da barra; um toque abre o player em folha de tela cheia; do player saem a Fila e o menu da faixa.
7. Biblioteca: Baixados, Curtidos, Álbuns e Playlists numa grade 2×2, mais "Criar playlist".
8. "Reconhecer pelo som" mora dentro da Busca. Enquanto não existir, aparece como "Em breve", sem interruptor falso.
9. Cada região que carrega dados tem quatro estados desenhados (carregando, vazio, erro, cheio) e o modo sem conexão.
10. Acessibilidade é requisito: contraste AA medido em captura, alvos de 48 dp, "Reduzir transparência" e "Reduzir movimento" com efeito imediato.
11. Sem telemetria: o sucesso é medido por testes com pessoas, checagens automáticas locais e critérios de aceite, nunca por coleta.
12. O backlog separa o que o produto já faz (redesenhar) do que ainda não faz (desenhado como futuro, marcado).

## Arquivos

| Arquivo | Conteúdo |
|---|---|
| `01-diagnostico-e-benchmark.md` | Por que a v2 e a v3 falharam; convenções de Apple Music, Spotify, YouTube Music, Bitchord, Intertune e Umihi |
| `02-objetivos-pessoas-jtbd.md` | Objetivos, métricas sem telemetria, proto-personas, jobs-to-be-done |
| `03-arquitetura-e-navegacao.md` | Inventário do que o produto faz, árvore do app, navegação mobile, Voltar |
| `04-fluxos.md` | Fluxos ponta a ponta, com diagramas |
| `05-estados-e-microcopy.md` | Matriz de estados e textos em pt-BR |
| `06-gestos-acessibilidade-responsivo.md` | Gestos com alternativas, WCAG 2.2 AA, celular, tablet e desktop |
| `07-backlog-e-aceite.md` | Backlog priorizado e critérios de aceite mensuráveis |

Versão navegável (HTML): artifact `kazzu-ux-v4` no Traycer, gerado a partir destes arquivos.

## Como ler junto com o design

A direção de arte, os tokens, a receita do vidro, os protótipos e os renders em Slint estão em `design/v4/` (`DIRECTION.md`, `DESIGN.md`, `SLINT_SPEC.md`). Este plano diz **o quê e por quê**; o design diz **como fica**.
