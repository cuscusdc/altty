# 05. Estados e microcopy

## 1. Matriz de estados por tela

Toda região que carrega dados tem quatro estados desenhados, mais o modo sem conexão. "Parcial" é quando parte do conteúdo veio do cache.

| Tela | Carregando | Vazio | Erro | Sem conexão | Cheio |
|---|---|---|---|---|---|
| Início | Esqueleto: 2 cartões grandes e 4 linhas | Não acontece com rede (o provedor sempre devolve algo) | Com cache: faixa de aviso + Tentar de novo. Sem cache: estado de erro | Barra "Sem conexão" + "Suas músicas do aparelho" ou "Adicionar pasta" | Prateleiras |
| Explorar | Esqueleto: fileira de capas e grade de momentos | Não acontece | Estado de erro com Tentar de novo | Estado sem conexão com atalho para a Biblioteca | Prateleiras e grade |
| Buscar (sem texto) | Não há | Sem recentes: frase curta de ajuda | Não há | Diz que a busca será só no aparelho | Recentes + Reconhecer pelo som |
| Buscar (com texto) | 6 linhas de esqueleto depois de 300 ms | "Nada para “termo”" + dica | "Não deu para buscar agora" + Tentar de novo | Resultados só do aparelho, com a etiqueta "Neste aparelho" | Seções de resultados |
| Biblioteca | Ladrilhos com esqueleto na primeira consulta | Sem pastas e sem curtidos: cartão "Adicionar pasta" e explicação | Erro do banco: frase + Relatar problema | Igual ao cheio (é local) | Grade 2×2, playlists, aparelho |
| Detalhe | Cabeçalho com esqueleto + 8 linhas | Playlist vazia: como adicionar. Curtidos vazio: como curtir | Estado de erro com Tentar de novo | Faixas remotas esmaecidas; locais normais | Cabeçalho e faixas |
| Player | Botão de tocar vira indicador de espera; barra pulsa | Não se abre sem faixa | Linha de erro no console + Tentar de novo | "Sem conexão" + Tentar de novo; faixas locais tocam | Controles |
| Fila | Linhas ainda resolvendo: esqueleto no lugar do tempo | "A fila está vazia" + o que acontece em seguida | Linha com falha: "Não tocou" + remover | Faixas remotas marcadas | Tocando agora + A seguir |
| Ajustes | Não há | Sem pastas: "Nenhuma pasta" + Adicionar | Falha ao gravar: aviso | Igual | Listas agrupadas |

Regras:

1. **Esqueleto tem a forma do que vem.** Nada de spinner solto; o único indicador giratório fica dentro do botão de tocar.
2. **Vazio** diz o que está vazio, por que importa e oferece uma ação.
3. **Erro** diz o que houve, sem culpar a pessoa, e oferece uma ação quando tentar de novo faz sentido. O código técnico fica pequeno, só para quem vai relatar.
4. **Sem conexão** nunca bloqueia o que é local.
5. **Sucesso é silencioso** quando o efeito está à vista. Aviso só para efeito que não se vê (adicionada à playlist) e para oferecer Desfazer.

## 2. Voz

| Regra | Exemplo |
|---|---|
| Frases curtas, em português do Brasil do dia a dia | "Não deu para carregar." |
| Verbo no botão | "Tentar de novo", "Adicionar pasta", "Criar playlist" |
| Sem culpa e sem exclamação | "Algo falhou do lado do YouTube Music." |
| Diz o que acontece em seguida | "Quando a fila acabar, o Kazzu continua com músicas parecidas." |
| Sem jargão | "Este aparelho", não "saída padrão" |
| Números com unidade | "128 músicas", "3 min 21 s" para leitor de tela |
| Honesto sobre o que não existe | "Em breve" |

## 3. Textos

### Navegação e cabeçalhos

| Onde | Texto |
|---|---|
| Abas | Início, Explorar, Biblioteca |
| Botão de busca (rótulo acessível) | Buscar |
| Engrenagem (rótulo acessível) | Ajustes |
| Títulos grandes | Explorar, Biblioteca, Buscar, Ajustes |

### Início

| Onde | Texto |
|---|---|
| Linha de primeiro uso | **Pronto para tocar.** O YouTube Music já vem ligado. Não precisa de conta. |
| Botão da linha | Entendi |
| Seções | Para ouvir agora, Faixas para você, Mixes, Álbuns em destaque, Neste aparelho |
| Motivo de uma sugestão | Em alta hoje; Porque você ouviu Nome |
| Faixa de aviso com cache | **Sem atualizar.** Mostrando o que já estava salvo. |
| Faixa sem conexão | **Sem conexão.** A música do aparelho continua aqui. |
| Erro sem cache, título | Não deu para carregar o Início |
| Erro sem cache, texto | O YouTube Music não respondeu. Sua música do aparelho continua na Biblioteca. (O código técnico não aparece na tela: vai para o arquivo de "Relatar um problema".) |
| Ações | Tentar de novo; Abrir a Biblioteca |

### Buscar

| Onde | Texto |
|---|---|
| Campo (texto de ajuda) | Músicas, artistas, álbuns |
| Linha de recurso | Reconhecer pelo som. Descubra a música que está tocando perto de você. |
| Estado da linha hoje | Em breve |
| Lista | Buscas recentes |
| Ação da lista | Limpar |
| Sem recentes | Suas buscas aparecem aqui. Nada sai deste aparelho. |
| Filtros | Tudo, Músicas, Álbuns, Artistas, Playlists |
| Seções | Melhor resultado, Músicas, Álbuns, Artistas, Playlists |
| Vazio, título | Nada para “termo” |
| Vazio, texto | Confira a grafia ou tente o nome do artista. |
| Erro | Não deu para buscar agora. |
| Sem conexão | Sem conexão: buscando só neste aparelho. |

### Biblioteca

| Onde | Texto |
|---|---|
| Ladrilhos | Baixados, Curtidos, Álbuns, Playlists |
| Contagens | 128 músicas; 24 álbuns; 6 playlists |
| Baixados hoje | Em breve |
| Botão | Criar playlist |
| Seções | Suas playlists, Neste aparelho, Tocadas há pouco |
| Sem pastas, título | Traga a música do aparelho |
| Sem pastas, texto | Escolha uma pasta e o Kazzu toca FLAC, WAV, Opus e MP3 como são, sem mexer no som. |
| Ação | Adicionar pasta |
| Varredura | Lendo a pasta: 312 músicas |
| Antes da permissão | O Kazzu precisa ler os arquivos de áudio da pasta que você escolher. Só áudio. |

### Detalhe

| Onde | Texto |
|---|---|
| Botões | Tocar, Aleatório |
| Rodapé do álbum | 12 músicas, 43 min |
| Origem | Do YouTube Music; Neste aparelho |
| Playlist vazia, título | Esta playlist está vazia |
| Playlist vazia, texto | Toque em Mais numa música e escolha Adicionar à playlist. |
| Curtidos vazio | Toque no coração de uma música e ela aparece aqui. |

### Player e fila

| Onde | Texto |
|---|---|
| Contexto | Tocando de Nome |
| Nota da saída | Original, sem equalizador; Qualidade do YouTube Music, sem equalizador |
| Tempos | decorrido à esquerda (1:16), o que falta à direita (-2:05) |
| Origem e formato | YouTube Music, Opus 160 kbps; Neste aparelho, FLAC 24-bit/96 kHz |
| Botões | Este aparelho (a saída atual), Fila |
| Escolher saída hoje | Mostra a saída atual; a troca chega depois |
| Erro | Não deu para tocar esta música. |
| Rede | Sem conexão. Toque para tentar. |
| Fila, título | Fila |
| Fila, grupos | Tocando agora; A seguir, de Nome |
| Fila, ações | Limpar; Confirmar limpar (segundo toque) |
| Fila, nota | Quando a fila acabar, o Kazzu continua com músicas parecidas. Você pode desligar isso em Ajustes. |
| Fila vazia | A fila está vazia. Toque numa música para começar. |

### Menu da faixa

Tocar em seguida; Adicionar à fila; Adicionar à playlist; Curtir ou Tirar dos Curtidos; Ir para o álbum; Ir para o artista; Ver a letra (Em breve); Timer para dormir (Em breve); Baixar (Em breve).

Tudo o que ainda não existe mora aqui, com a etiqueta "Em breve". Nenhum recurso futuro ocupa lugar de controle nas telas principais.

### Avisos

| Situação | Texto | Ação |
|---|---|---|
| Curtiu | Adicionada aos Curtidos. | Desfazer |
| Tirou a curtida | Removida dos Curtidos. | Desfazer |
| Tocar em seguida | Toca em seguida. | Desfazer |
| Adicionou à fila | Adicionada à fila. | Desfazer |
| Adicionou à playlist | Adicionada a Nome. | Desfazer |
| Removeu da fila | Removida da fila. | Desfazer |
| Criou playlist | Playlist criada. | Abrir |
| Faixa indisponível | Esta música não está disponível. Pulamos para a próxima. | |
| Rede caiu | Sem conexão. | Tentar de novo |
| Rede voltou | Conexão de volta. | |
| Recurso futuro | Ainda não chegou. Vem numa próxima versão. | |

### Ajustes

| Grupo | Linhas |
|---|---|
| Reprodução | Qualidade do streaming (Original, Alta, Econômica); Continuar com músicas parecidas |
| Aparência | Tema (Automático, Claro, Escuro); Reduzir transparência; Reduzir movimento |
| Biblioteca | Pastas do aparelho; Ler pastas de novo |
| Fontes de música | YouTube Music: Ligado, sem conta. Spotify, Tidal, Qobuz: Em breve |
| Privacidade | Sem telemetria: o Kazzu não coleta nada; Apagar histórico de buscas e de reprodução; Relatar um problema (gera um arquivo neste aparelho; você decide se envia) |
| Sobre | Versão; Licenças de código aberto |

Texto da oferta de desempenho: "Deixar o Kazzu mais leve? Reduzir transparência troca o vidro por superfícies sólidas." com "Deixar mais leve" e "Agora não".

## 4. Leitor de tela

| Elemento | Anúncio |
|---|---|
| Aba | "Início, aba, 1 de 3, selecionada" |
| Busca | "Buscar, botão" |
| Mini-player | "Tocando: Título, de Artista. Toque duas vezes para abrir o player" |
| Linha de faixa | "Título, Artista, 3 minutos e 21 segundos. Ações disponíveis: Mais" |
| Barra de progresso | "Posição, 1 minuto e 2 segundos de 3 minutos e 21 segundos, ajustável" |
| Coração | "Curtir, botão" ou "Curtida, botão, selecionado" |
| Capa decorativa | Não anunciada |
| Aviso | Anunciado como região de status, com a ação |
