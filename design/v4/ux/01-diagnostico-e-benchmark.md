# 01. Diagnóstico e benchmark

## 1. O que o autor disse

| Versão | Veredito do autor | Leitura |
|---|---|---|
| v1 | "não era mobile-first, UX/UI ruim, não ficou glassmorphism" | Layout de desktop encolhido; painéis opacos |
| v2 | "sem glassmorphism", "esse degradê está muito feio", "pedi um design APPLE LIKE" | Malha colorida no fundo; vidro escuro sem nada atrás; excesso de pílulas |
| v3 | "ainda com fundo/vidro do player fracos" | O fundo ficou neutro, mas o player manteve um degradê marrom e uma cápsula escura |

## 2. Diagnóstico por evidência

Analisei as capturas do que o autor vê hoje (`target/ui-screenshots/v3/*.png`, o app real) e os renders do design v3 (`design/slint/v3-*.png`). Cada linha cita a tela onde o problema aparece.

| # | Problema | Onde se vê | Causa | O que a v4 faz |
|---|---|---|---|---|
| D1 | Fundo em malha saturada (vermelho, laranja, verde) | app: todas as telas (`v3-gl-mobile-dark-home-playing`) | O "palco" colorido era a fonte de cor para o vidro | Fundo neutro e liso. A cor vem só das capas |
| D2 | O vidro parece cartão marrom | app: banner, mini-player, cartões da Biblioteca | Tinta escura a 60% ou mais sobre um fundo quase uniforme: o desfoque não tem o que mostrar | Vidro só onde há conteúdo passando por trás; tinta mais clara, borda de luz, brilho no topo |
| D3 | Nada com detalhe atrás do vidro | design v3: capas vetoriais chapadas | Capas fictícias em formas simples; desfocadas viram uma cor só | Capas fotográficas nos protótipos e na galeria, como as capas reais |
| D4 | Topo pesado: a primeira capa só aparece a 43% da altura | app: Início (logo, banner de primeiro uso, fileira de chips, título de seção) | Quatro faixas de cromo antes do conteúdo | Logo e um botão; a primeira capa começa em 112 dp (13% da altura) |
| D5 | Player com cara de painel | app e v3: cápsula com 5 controles + 4 botões-ladrilho | Cada grupo ganhou um contêiner | Capa inteira e um único console de vidro; transporte sem caixas |
| D6 | Fundo do player fraco | v3: degradê marrom de cima a baixo | Cor média da capa virou um degradê | O fundo é a própria capa e o reflexo dela, vistos através do console |
| D7 | Cartão dentro de cartão na Biblioteca | app e v3 | Ladrilho com moldura e outro quadro dentro | Ladrilho é a arte; título e contagem ficam embaixo, sem moldura |
| D8 | "Baixados: Em breve" ocupa um quarto da tela | app: Biblioteca | Recurso inexistente com o mesmo peso dos demais | Continua no lugar pedido pelo autor, mas esmaecido e com o estado dito em uma linha |
| D9 | Pílulas por toda parte | app: chips do Início, duas fileiras na Busca, selos | Um só formato para filtro, sugestão, selo e botão | Um controle segmentado para filtros; sugestões viram linhas de lista |
| D10 | Hierarquia tipográfica rasa | app: títulos de 20, 22 e 28 parecidos | Escala sem saltos | Escala com saltos claros: 34, 22, 17, 15, 13 |
| D11 | Capas ausentes na busca | app: `search-results` (discos cinza) | Capas remotas não carregadas na captura | Estado de capa ausente desenhado; esqueleto no formato da capa |
| D12 | Aviso de primeiro uso no lugar nobre | app: Início | Mensagem de sistema acima do conteúdo | Uma linha discreta, só na primeira abertura |

**Conclusão.** A ideia "vidro sobre conteúdo colorido" estava certa nas três versões. O que faltou foi o conteúdo (capas com detalhe), a disciplina (vidro só no que flutua) e o acabamento Apple (hierarquia, listas, transporte sem caixas). A v4 muda a execução, e muda o player por inteiro.

## 3. Benchmark de convenções

Não copiamos telas, ícones, textos nem código. O que segue são convenções de uso que as pessoas já conhecem.

| Convenção | Quem usa | Adotamos? | Como |
|---|---|---|---|
| Barra de abas flutuante com busca separada à direita | Apple Music (iOS 26), Bitchord | Sim | Cápsula com 3 abas e um botão redondo de busca. É a decisão do mapa mental |
| Mini-player acima da barra | Apple Music, Spotify, YouTube Music, Bitchord, Intertune | Sim | Cápsula própria de vidro; toque abre o player |
| Título grande que recolhe ao rolar | Apple Music, apps do iOS | Sim | Título de 34 que vira título de 17 numa barra de vidro |
| Player em folha de tela cheia, fechado ao arrastar para baixo | Todos | Sim | Com botão de fechar como alternativa ao gesto |
| Fundo do player derivado da capa | Apple Music, Bitchord, Intertune | Sim | A capa desfocada, sem degradê desenhado |
| Tocar e Aleatório lado a lado no topo de um álbum | Apple Music | Sim | Dois botões de mesma largura |
| Linhas de faixa com capa, título, artista e "mais" | Todos | Sim | 64 dp de altura; menu em folha |
| Busca como destino com recentes e categorias | Spotify, Apple Music | Sim | Recentes e "Reconhecer pelo som"; a navegação por gêneros fica em Explorar |
| Chips de humor no topo do Início | YouTube Music, Intertune | Não | Duplicam Explorar e empurram o conteúdo para baixo (D4) |
| Grade 2×2 na Biblioteca com botão de criar playlist | Umihi | Sim | Pedido do autor |
| Selo de qualidade no player | Apple Music (Lossless), Tidal | Sim | Linha de formato real: códec, bits, taxa |
| Fila com "a seguir" e continuação automática | YouTube Music, Spotify | Sim | Com a nota de onde vêm as próximas |
| Gestos de deslizar em linhas | Apple Music, Spotify | Adiado | O Slint não tem o gesto pronto; fica como melhoria, sempre com alternativa no menu |

## 4. Heurísticas de Nielsen: onde a v2/v3 feriam e o que a v4 exige

| Heurística | Falha observada | Exigência na v4 |
|---|---|---|
| Visibilidade do estado | Faixa atual pouco destacada na fila; "carregando" sem forma | Linha atual com barras animadas e cor; esqueleto com o formato do conteúdo |
| Correspondência com o mundo real | "Saída simulada", "Palco" | Palavras do dia a dia: "Este celular", "Fila", "Curtidos" |
| Controle e liberdade | Voltar inconsistente em folhas | Uma ordem única do Voltar; Desfazer em curtir, fila e playlist |
| Consistência | Pílula para tudo | Um componente por função |
| Prevenção de erro | "Limpar fila" a um toque | Limpar pede confirmação no próprio botão; remover tem Desfazer |
| Reconhecer em vez de lembrar | Buscas recentes escondidas atrás de chips | Recentes como lista ao abrir a Busca |
| Flexibilidade | Sem atalhos de teclado no desktop | Espaço, setas, Ctrl+K e Esc (seção 06) |
| Estética minimalista | Quatro faixas de cromo no topo | Um cabeçalho, conteúdo logo abaixo |
| Recuperação de erro | Código técnico em destaque | Frase calma, uma ação, código pequeno só para relatar |
| Ajuda | Nenhuma | Textos dos estados vazios explicam o próximo passo |

## 5. Restrições que moldam a UX

- **Slint 1.19**: há `backdrop-blur` (FemtoVG e Skia), não há no renderer software, não há `saturate` no desfoque de fundo, não há cantos contínuos, fontes variáveis nem numerais tabulares por recurso OpenType. O `ListView` aceita um só `for`; `Flickable` não tem encaixe (snap). O design evita depender do que não existe.
- **Android**: Voltar do sistema, áreas seguras, teclado virtual, permissão só de áudio (capas de pasta podem faltar).
- **Produto**: 100% grátis, sem telemetria, YouTube Music sem login, licença GPL-3.0 como hipótese.
- **Honestidade**: recurso que não existe aparece como "Em breve" em texto, nunca como interruptor que não faz nada.
