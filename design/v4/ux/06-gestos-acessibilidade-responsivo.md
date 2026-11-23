# 06. Gestos, acessibilidade e responsivo

## 1. Gestos e alternativas

Nenhuma função depende só de gesto (WCAG 2.5.1 e 2.5.7). Toda linha da tabela tem um controle visível equivalente.

| Gesto | Onde | Faz | Alternativa visível | Teclado (desktop) |
|---|---|---|---|---|
| Toque | Linha de faixa | Toca; a lista vira a fila | A própria linha | Enter |
| Toque | Mini-player | Abre o player | A própria cápsula | Enter |
| Arrastar para cima | Mini-player | Abre o player | Toque | |
| Arrastar para baixo | Capa do player, alça das folhas | Fecha | Botão de fechar no topo; Voltar | Esc |
| Arrastar na horizontal | Barra de progresso | Busca a posição | Toque na barra; leitor de tela: ajustar | Setas esquerda e direita, 10 s |
| Arrastar na horizontal | Volume | Ajusta | Botões físicos do aparelho | Setas para cima e para baixo |
| Rolagem horizontal | Prateleiras | Passa os cartões | Setas nas pontas a partir de 600 dp | Tab entre cartões |
| Toque longo | Linha de faixa, cartão | Abre o menu | Botão Mais na linha | Menu de contexto, Shift+F10 |
| Arrastar a alça | Fila | Reordena (futuro) | Botão Mover para cima | Alt + setas |
| Deslizar a linha | Fila | Remove (futuro) | Botão Remover na linha | Delete |
| Puxar para baixo no topo | Início, Explorar | Recarrega | "Tentar de novo" nos estados de erro; a carga é automática | F5 ou Ctrl+R |

Atalhos de desktop: Espaço toca e pausa; Ctrl+K ou barra abre a Busca; Ctrl+1, 2, 3 trocam de aba; Ctrl+seta direita e esquerda pulam faixa; Esc segue a ordem do Voltar.

## 2. Acessibilidade (WCAG 2.2 AA)

| Critério | Como o design cumpre | Como se verifica |
|---|---|---|
| 1.1.1 Conteúdo não textual | Todo botão de ícone tem rótulo; capa decorativa não é anunciada; capa que é o único rótulo leva o título | Revisão da árvore de acessibilidade na galeria |
| 1.3.1 Informação e relações | Papéis: aba, botão, controle deslizante, interruptor, lista | Idem |
| 1.4.1 Uso de cor | Estado ativo nunca é só cor: a aba ativa tem fundo e ícone preenchido; curtida tem ícone preenchido; erro tem ícone e texto | Inspeção em escala de cinza |
| 1.4.3 Contraste | Texto 4,5:1; texto grande e ícones 3:1; medido sobre o vidro no pior caso (capa branca e capa preta atrás) | `check_render.py` nas capturas |
| 1.4.4 e 1.4.10 Redimensionar e refluxo | Escala de fonte do sistema até 130% sem cortar ações; títulos quebram em 2 linhas antes de cortar; 320 dp sem rolagem horizontal | Capturas em 320 e com escala 1,3 |
| 1.4.11 Contraste não textual | Bordas de campo e trilhos de controle a 3:1; anel de foco a 3:1 | `check_render.py` e inspeção |
| 1.4.12 Espaçamento de texto | Linhas com altura mínima, não fixa | Capturas com escala |
| 2.1.1 Teclado | Tudo alcançável por Tab; ordem igual à leitura; atalhos acima | `--selftest` |
| 2.4.3 e 2.4.7 Foco | Anel de 2 dp na cor de foco, instantâneo, nunca escondido sob o cromo | Capturas com foco |
| 2.4.11 Foco não encoberto | A reserva de rolagem mantém o item focado acima do mini-player | Limite do Slint: `Flickable` não rola até o foco; registrado como pendência |
| 2.5.7 Arrastar | Toda ação de arrastar tem alternativa de toque | Tabela acima |
| 2.5.8 Tamanho do alvo | Alvos de 48×48 dp, mesmo quando o desenho é menor | `--hit-targets` |
| 2.3.3 e preferência de movimento | "Reduzir movimento": sem deslocamentos, só esmaecer de até 150 ms; nada pisca; nenhuma animação infinita além das barras da faixa atual, que param | Captura e inspeção |
| 3.2 Previsível | Mesma barra, mesma ordem, mesmo Voltar em todas as telas | Roteiro |
| 3.3 Erros | Mensagem ao lado do campo; botão desativado diz a razão | Estados desenhados |
| 4.1.2 Nome, papel, valor | Rótulos em pt-BR vindos do Rust; estado de alternância anunciado | Teste com TalkBack (pendente) |

### Transparência

| Modo | O que muda |
|---|---|
| Padrão | Vidro com desfoque real em tudo o que flutua |
| Reduzir transparência (Ajustes ou sistema) | Todo vidro vira superfície sólida da paleta, com a mesma borda; o fundo do player vira a cor média da capa, escurecida |
| Renderer sem desfoque (software) | Igual a Reduzir transparência, automático |
| Aparelho lento | O app oferece uma vez "Deixar mais leve?"; nunca impõe |

### Leitor de tela e ordem

1. Cabeçalho da aba. 2. Conteúdo. 3. Mini-player. 4. Abas. 5. Buscar. Com folha aberta, o foco fica preso nela e volta ao botão que a abriu ao fechar.

## 3. Responsivo: celular, tablet, desktop

O mesmo design cresce; nada é redesenhado. Pontos de quebra pelo tamanho da janela, não pelo aparelho.

| Faixa | Largura | Navegação | Conteúdo | Player |
|---|---|---|---|---|
| Compacta (base) | 320 a 599 | Barra cápsula + busca redonda embaixo; mini-player acima | 1 coluna; prateleiras horizontais; grade 2×2 na Biblioteca | Folha de tela cheia |
| Média | 600 a 1099 | Mesma barra, centralizada, largura máxima de 520; mini-player na mesma largura | Margens de 32; grades de 3 a 4 colunas; detalhe com capa ao lado das faixas a partir de 840 | Folha centralizada de até 520 de largura sobre o fundo desfocado |
| Larga | 1100 ou mais | Barra lateral de vidro à esquerda (logo no topo, Início, Explorar, Biblioteca, Buscar, playlists); barra de reprodução de vidro embaixo, de ponta a ponta | Conteúdo com largura máxima de 1180; grades de 5 a 6 colunas | Painel à direita ou tela cheia; a Fila pode ficar ao lado |

Regras:

- **Logo no canto superior esquerdo** em todas as faixas: no cabeçalho do Início (compacta e média) e no topo da barra lateral (larga).
- **Busca nunca vira barra fixa no topo**: na faixa larga é um item da barra lateral que abre a mesma tela.
- **Alvos de 48 dp** em todas as faixas; com mouse, as linhas ganham realce ao passar.
- **Paisagem no celular**: a barra cápsula continua embaixo; o player põe a capa à esquerda e o console à direita.
- **Áreas seguras**: o cromo soma os recuos do sistema; o conteúdo rola por baixo deles.

## 4. Desempenho como parte da UX

| Orçamento | Valor | Observação |
|---|---|---|
| Painéis com desfoque numa tela | no máximo 4 no celular | Cabeçalho recolhido, mini-player, cápsula, busca. Legendas de cartão usam a capa pré-desfocada, não desfoque ao vivo |
| Primeira pintura com conteúdo do cache | menos de 1 s | Esqueleto imediato |
| Resposta ao toque | menos de 100 ms | Estado de pressionado em todo controle |
| Transições | 180 a 420 ms | Curvas que imitam mola, sem ressalto em estado de UI |

Nenhum destes valores foi medido em aparelho real nesta fase.
