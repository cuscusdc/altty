# 07. Backlog priorizado e critérios de aceite

## 1. Backlog (MoSCoW)

"Existe" diz se o `kazzu-app` de hoje já tem a função por baixo (então é redesenho) ou se é trabalho novo.

### Obrigatório para a v4 (M)

| # | Item | Existe? | Depende de |
|---|---|---|---|
| M1 | Tokens v4 e átomos de vidro no app (`Theme`, `Glass`, botões, linhas) | Redesenho | SLINT_SPEC v4 |
| M2 | Cromo: barra cápsula, busca redonda, mini-player, cabeçalho que recolhe | Redesenho | M1 |
| M3 | Início: logo, linha de primeiro uso, prateleiras, estados | Redesenho | M2 |
| M4 | Player novo: capa inteira, reflexo, console de vidro; mini-player tingido | Redesenho | M1; o Rust passa a entregar `backdrop` como capa + reflexo, mais `player-dim` e `mini-dim` |
| M5 | Fila e folhas (menu, adicionar à playlist, nova playlist) | Redesenho | M4 |
| M6 | Buscar: campo na tela, recentes, Reconhecer pelo som (Em breve), resultados | Redesenho | M2 |
| M7 | Biblioteca: grade 2×2, Criar playlist, playlists, aparelho | Redesenho | M2 |
| M8 | Detalhe: álbum, playlist, artista, coleção | Redesenho | M2 |
| M9 | Explorar: lançamentos, momentos, em alta | Redesenho | M2 |
| M10 | Ajustes em listas agrupadas | Redesenho | M1 |
| M11 | Reduzir transparência e movimento com efeito imediato; fallback sólido | Existe | M1 |
| M12 | Estados (carregando, vazio, erro, sem conexão) em todas as telas | Parcial | M3 a M9 |
| M13 | Capas fotográficas nos dados de demonstração e na galeria | Novo | Sem isso o vidro não se prova |
| M14 | Checagens: alvos, contraste em captura, prova de vidro | Existe em parte | Galeria |

### Importante (S)

| # | Item | Existe? |
|---|---|---|
| S1 | Tablet e desktop (faixas média e larga) | Parcial |
| S2 | Desfazer em "Adicionar à playlist" | Novo |
| S3 | Renomear e apagar playlist | Novo (lacuna no app) |
| S4 | Notificação de mídia e tela de bloqueio no Android | Novo |
| S5 | Teste com TalkBack e aparelho físico | Novo |
| S6 | Reordenar a fila arrastando | Novo (precisa de `PlaybackCommand::Move`) |

### Desejável (C)

| # | Item |
|---|---|
| C1 | Deslizar a linha da fila para remover |
| C2 | Puxar para recarregar |
| C3 | Transição da capa do mini-player para o player (elemento compartilhado) |
| C4 | Capa do álbum salva como ícone de atalho |

### Fora desta fase (W), com lugar reservado na UI

| Item | Onde aparece | Como aparece hoje |
|---|---|---|
| Reconhecer pelo som | Buscar | Linha esmaecida, "Em breve" |
| Baixados | Biblioteca | Ladrilho esmaecido, "Em breve" |
| Letras | Menu da faixa e do player | Linha com "Em breve" |
| Escolher saída | Player | O botão mostra a saída atual ("Este aparelho"); a troca chega depois |
| Timer para dormir | Menu da faixa e do player | Linha com "Em breve" |
| Equalizador | Ajustes: Reprodução | Não aparece até existir |
| Outros provedores | Ajustes: Fontes de música | Linhas com "Em breve" |
| Discord, sincronização, ouvir junto, retrospectiva | Não aparecem | |

## 2. Critérios de aceite

Cada critério tem um número, um limite e um jeito de verificar. "Galeria" é `crates/kazzu-ui-gallery`; "captura" é o protótipo ou o Slint renderizado.

| # | Critério | Limite | Verificação |
|---|---|---|---|
| AC-01 | Do ícone do app à música tocando, no primeiro uso | 2 toques | Roteiro; `--selftest` |
| AC-02 | Buscar e tocar uma música conhecida | 3 toques, fora a digitação | Roteiro |
| AC-03 | Curtir a faixa atual a partir de qualquer tela | 2 toques | Roteiro |
| AC-04 | Texto abaixo do contraste AA, em claro e escuro, com capa clara e capa escura atrás do vidro | 0 | `check_render.py` |
| AC-05 | Alvos de toque abaixo de 48×48 dp | 0 | `--hit-targets` |
| AC-06 | Requisições de rede com o app parado por 60 s após a carga inicial | 0 | Registro de rede |
| AC-07 | O vidro desfoca a capa: a faixa sob o painel difere da versão sólida e guarda variação de cor da capa | passa em claro e escuro | `glass_proof.py` |
| AC-08 | Voltar fecha a camada de cima e nunca sai do app com camada aberta | 100% dos casos da tabela | `--selftest` |
| AC-09 | Estados desenhados por região de dados | 4 estados + sem conexão | Galeria: cenas por estado |
| AC-10 | Menor texto na tela | 12 dp; texto de leitura 15 dp ou mais | Tokens |
| AC-11 | Reduzir movimento | nenhum deslocamento; esmaecer de até 150 ms | Inspeção |
| AC-12 | Reduzir transparência | 0 painéis com desfoque; mesmo contraste | Captura |
| AC-13 | Ações destrutivas | Desfazer ou confirmação em 100% | Roteiro |
| AC-14 | Primeira capa visível no Início | começa acima de 20% da altura da tela | Captura 390×844 (hoje: 112 de 844, 13%) |
| AC-15 | Rolagem horizontal da página entre 320 e 1920 dp | nenhuma | Capturas |
| AC-16 | Painéis com desfoque ao vivo por tela no celular | no máximo 4 | Contagem na SLINT_SPEC |
| AC-17 | Recurso inexistente | nunca um interruptor ou botão que finge funcionar | Revisão |
| AC-18 | Teste com 5 pessoas: 6 tarefas sem ajuda | 4 de 5 ou mais; SUS 80 ou mais | Teste moderado |
| AC-19 | Fundo sem degradê decorativo | 0 malhas ou manchas de cor; a única cor de fundo vem de capa | Revisão das capturas |
| AC-20 | Logo no canto superior esquerdo do Início; busca fora do topo | sempre | Capturas |

## 3. Roteiro de teste com pessoas (sem telemetria)

Cinco pessoas, 20 minutos cada, aparelho de teste, anotação em papel. Ninguém é gravado sem pedir.

| # | Tarefa | Sucesso |
|---|---|---|
| T1 | "Ponha uma música para tocar." | Tocando em até 2 toques |
| T2 | "Ache a música Tal e toque." | Usa a busca redonda sem procurar no topo |
| T3 | "Guarde essa música para ouvir depois." | Curte ou põe em playlist |
| T4 | "Veja o que vai tocar depois e tire uma música." | Abre a Fila e remove |
| T5 | "Crie uma playlist chamada Estrada." | Biblioteca, Criar playlist |
| T6 | "O efeito de vidro está atrapalhando. Desligue." | Ajustes, Reduzir transparência |

Depois: questionário SUS em papel e duas perguntas abertas ("o que te confundiu?", "o que você tiraria?").

## 4. Decisões que dependem do autor

| # | Decisão | Recomendação | Alternativa |
|---|---|---|---|
| A1 | Remover os chips do Início (Para você, Descobrir, Mixes, Foco, Treino) | Remover | Manter como controle segmentado sob o logo |
| A2 | Acento fixo da marca, sem "acento da capa" | Fixo | Manter a opção em Ajustes |
| A3 | Player sempre escuro, mesmo no tema claro | Sempre escuro | Seguir o tema |
| A4 | Volume dentro do player no Android | Manter (pedido do mapa) | Tirar no celular e deixar no desktop |
| A5 | Campo de busca na tela de Buscar, no alto | No alto da tela de Buscar | Campo na barra de baixo, colado ao teclado (precisa testar o teclado no Slint Android) |
| A6 | Capas de demonstração fotográficas (domínio público) no app de demonstração | Adotar | Manter as vetoriais |
| A7 | Marca: manter o K atual, redesenhado para a v4 | Manter e refinar | Encomendar marca nova |
| A8 | Mini-player tingido pela capa da faixa atual (escuro, texto branco, nos dois temas) | Tingido | Vidro neutro como a barra |
| A9 | Momentos e gêneros como ladrilhos neutros, sem a cor do provedor | Neutros | Ladrilhos com a cor que o YouTube Music manda |
| A10 | Letra e Timer só no menu, com "Em breve", até existirem | Só no menu | Ícones esmaecidos no player |
| A11 | Nomes das prateleiras do Início ("Para ouvir agora", "Faixas para você") em vez dos nomes do YouTube Music | Nomes próprios | Manter os do provedor |
