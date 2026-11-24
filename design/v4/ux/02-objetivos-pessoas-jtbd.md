# 02. Objetivos, pessoas e jobs-to-be-done

## 1. Objetivos de UX

| # | Objetivo | Por que importa |
|---|---|---|
| O1 | Tocar música em segundos, no primeiro uso, sem conta | É a promessa do produto: YouTube Music já ligado |
| O2 | Achar uma música específica sem pensar | A busca é a tarefa mais frequente depois de tocar |
| O3 | Guardar o que agrada sem sair de onde está | Curtir e playlists dão motivo para voltar |
| O4 | Ouvir sem internet o que está no aparelho | Diferencial "offline/local" do mapa |
| O5 | Confiar: saber o que toca, de onde vem e que nada é coletado | Diferenciais "qualidade original" e "sem telemetria" |
| O6 | Parecer um app da Apple feito para o Kazzu: neutro, legível, vidro de verdade | Requisito do autor, reprovado duas vezes |
| O7 | Funcionar para quem usa fonte grande, leitor de tela, pouca transparência ou pouco movimento | WCAG 2.2 AA é requisito |

## 2. Métricas de sucesso sem telemetria

Nada é coletado do usuário. Medimos de quatro formas, todas locais.

| Métrica | Meta | Como medir |
|---|---|---|
| Toques do ícone do app até a primeira música tocando (primeiro uso) | no máximo 2 | Roteiro de teste e `--selftest` da galeria |
| Toques para tocar uma música conhecida pela busca (fora a digitação) | no máximo 3 | Roteiro de teste |
| Toques para curtir a faixa atual, de qualquer tela | no máximo 2 | Roteiro de teste |
| Sucesso nas 6 tarefas do roteiro, sem ajuda | 4 de 5 pessoas ou mais | Teste moderado com 5 pessoas, em papel |
| Escala SUS (questionário em papel) | 80 ou mais | Mesmo teste |
| Texto abaixo de AA nas capturas | 0 | `design/v4/tools/check_render.py` |
| Alvos de toque menores que 48×48 dp | 0 | `kazzu-ui-gallery --hit-targets` |
| Prova de vidro (o painel desfoca a capa de verdade) | passa em claro e escuro | `design/v4/tools/glass_proof.py` nos renders do Slint |
| Requisições de rede com o app parado por 60 s, depois da carga inicial | 0 | Teste manual com registro de rede |
| Quadros acima de 16 ms durante a rolagem do Início num aparelho de referência | menos de 10% | Contador local de quadros do app (já existe em `watch_frames`) |

Retorno do usuário sem coleta: "Relatar um problema" gera um arquivo local que a pessoa decide enviar ou não.

## 3. Proto-personas

São hipóteses, não pesquisa. Servem para decidir prioridades e precisam ser confirmadas nos testes com pessoas.

| | Lia, 24 | Caio, 33 | Bete, 58 |
|---|---|---|---|
| Aparelho | Android intermediário, 6,5" | Android topo de linha, fones com fio e DAC | Android de entrada, fonte grande |
| Hoje usa | YouTube Music grátis, playlists dos outros | Arquivos FLAC numa pasta, app de player local | O que o filho instalou |
| Quer | Abrir e tocar; curtir; montar playlist | Qualidade original, ver o formato, nada de reamostragem | Achar a música pelo nome e dar play |
| Incomoda | Login, anúncio de assinatura, telas cheias | Apps que mexem no som sem avisar | Letras pequenas, coisas que se mexem, vidro que atrapalha a leitura |
| Sucesso para ela/ele | Música tocando em dois toques | Selo "FLAC 24-bit/96 kHz, sem equalizador" | Tudo legível e botões grandes |

## 4. Jobs-to-be-done

| # | Quando... | quero... | para... | Telas |
|---|---|---|---|---|
| J1 | abro o app sem saber o que ouvir | ver boas opções e tocar com um toque | ter música em segundos | Início |
| J2 | lembro de uma música ou artista | achar e tocar em poucos passos | não perder a vontade | Busca |
| J3 | uma música está tocando perto de mim | descobrir qual é | guardar | Busca: Reconhecer pelo som (futuro) |
| J4 | gosto do que está tocando | curtir ou pôr numa playlist sem sair da tela | ouvir de novo | Player, mini-player, menu da faixa |
| J5 | quero continuar o que ouvia | retomar álbuns, playlists e curtidos | não procurar de novo | Biblioteca, Início |
| J6 | estou sem internet | ouvir o que está no aparelho | não ficar em silêncio | Biblioteca, Início sem conexão |
| J7 | a fila não é o que eu quero | ver, pular, remover e reordenar | controlar a sessão | Fila |
| J8 | me importo com o som | saber o formato real e a saída | confiar na qualidade | Player, Ajustes |
| J9 | o vidro ou o movimento me atrapalham | desligar com efeito imediato | usar com conforto | Ajustes |
| J10 | desconfio de apps | ver que não há conta nem coleta | usar sem receio | Ajustes: Privacidade |
| J11 | quero explorar | navegar por lançamentos, momentos e paradas | descobrir | Explorar |

## 5. O que fica fora desta fase

Equalizador, letras, downloads, timer, troca de saída, outros provedores, Discord, sincronização, ouvir junto e retrospectiva são do roteiro (Etapas 3 a 5 do brief). Onde encostam na UI de hoje, aparecem como "Em breve". O desenho deles fica para quando cada um entrar.
