# 04. Fluxos ponta a ponta

Cada fluxo lista o caminho feliz, os desvios e o critério de aceite ligado a ele (ver `07`). Os diagramas usam Mermaid.

## F1. Primeiro uso: abrir e tocar

```mermaid
flowchart LR
  A[Toca no ícone do Kazzu] --> B{Há cache do Início?}
  B -- não --> C[Esqueleto do Início]
  B -- sim --> D[Início com conteúdo]
  C --> E{Rede respondeu?}
  E -- sim --> D
  E -- não --> F[Início sem conexão: músicas do aparelho ou Adicionar pasta]
  D --> G[Toca no botão de tocar de um destaque]
  G --> H[Mini-player aparece, carregando]
  H --> I[Música tocando]
```

- Sem tela de boas-vindas, sem conta, sem permissão pedida na abertura.
- Uma linha discreta sob o cabeçalho, só na primeira abertura: "O YouTube Music já está ligado. Sem conta." com o botão de dispensar.
- Aceite: AC-01 (2 toques até tocar), AC-06 (rede só na carga inicial).

## F2. Buscar e tocar

```mermaid
flowchart LR
  A[Toca na busca redonda] --> B[Tela Buscar, campo em foco, teclado aberto]
  B --> C{Digitou?}
  C -- não --> D[Recentes e Reconhecer pelo som]
  D --> E[Toca num recente]
  C -- sim --> F[Sugestões enquanto digita]
  F --> G[Resultados: melhor resultado, músicas, álbuns, artistas, playlists]
  E --> G
  G --> H[Toca numa música]
  H --> I[Música tocando; resultados viram a fila]
  G --> J{Nada encontrado?}
  J -- sim --> K[Vazio: confira a grafia; busca só no aparelho]
```

- O filtro por tipo (Tudo, Músicas, Álbuns, Artistas, Playlists) aparece só quando há resultados.
- Erro de rede: mantém o texto, mostra "Não deu para buscar agora" e "Tentar de novo". Sem conexão: busca só no aparelho e diz isso.
- Aceite: AC-02 (3 toques fora a digitação).

## F3. Do mini-player ao player e de volta

```mermaid
stateDiagram-v2
  [*] --> SemFaixa
  SemFaixa --> Mini: tocar algo
  Mini --> Player: toque ou arrastar para cima
  Player --> Mini: botão fechar, arrastar para baixo ou Voltar
  Player --> Fila: botão Fila
  Fila --> Player: fechar, arrastar ou Voltar
  Player --> Menu: botão Mais
  Menu --> Player: fechar ou Voltar
  Mini --> SemFaixa: fila acabou e continuar com parecidas desligado
```

- O mini-player mostra capa, título, artista, tocar/pausar e próxima. A barra de progresso é a borda inferior dele.
- No player: buscar posição arrastando ou tocando na barra; alternativa por teclado e leitor de tela com passos de 10 s.
- Aceite: AC-08 (Voltar fecha sempre a camada de cima).

## F4. Fila

```mermaid
flowchart LR
  A[Player] --> B[Toca em Fila]
  B --> C[Folha Fila: Tocando agora + A seguir]
  C --> D[Toca numa linha: pula para ela]
  C --> E[Remover: sai da fila + aviso Desfazer]
  C --> F[Mover para cima]
  C --> G[Limpar: pede confirmação no próprio botão]
  C --> H[Chegou ao fim: nota Continua com parecidas]
```

- Reordenar arrastando depende de `PlaybackCommand::Move` (lacuna no `kazzu-audio`); até lá, "Mover para cima" é a alternativa e continua existindo depois, para quem não arrasta.
- Aceite: AC-09 (estados), AC-13 (ações destrutivas têm Desfazer ou confirmação).

## F5. Curtir

```mermaid
flowchart LR
  A[Player, menu da faixa ou cabeçalho do álbum] --> B[Toca no coração]
  B --> C[Coração preenche na hora]
  C --> D[Aviso: Adicionada aos Curtidos, com Desfazer]
  D --> E[Biblioteca: Curtidos mostra a faixa no topo]
```

- Atualização otimista: a UI muda na hora; se gravar falhar, volta e avisa.
- Curtidas ficam no aparelho. Não há conta.

## F6. Criar playlist e adicionar faixa

```mermaid
flowchart LR
  A[Biblioteca: Criar playlist] --> B[Folha Nova playlist, campo em foco]
  B --> C{Nome válido?}
  C -- vazio --> D[Botão Criar desativado com a razão ao lado]
  C -- sim --> E[Cria e abre a playlist vazia]
  F[Menu da faixa: Adicionar à playlist] --> G[Folha com as playlists + Nova playlist]
  G --> H[Toca numa playlist]
  H --> I[Aviso: Adicionada a Nome, com Desfazer]
  G --> B
```

- Playlist vazia explica o próximo passo: "Toque em Mais numa música e escolha Adicionar à playlist".

## F7. Música do aparelho

```mermaid
flowchart LR
  A[Biblioteca sem pastas] --> B[Cartão Neste aparelho: Adicionar pasta]
  B --> C[Seletor do sistema]
  C --> D[Varredura com progresso e contagem]
  D --> E[Álbuns e músicas aparecem]
  C -- cancelou --> A
  D -- permissão negada --> F[Explica e oferece Abrir ajustes do sistema]
```

- A permissão é pedida só aqui, quando a pessoa pede o recurso, com uma frase antes do diálogo do sistema.

## F8. Sem conexão e volta da conexão

```mermaid
flowchart LR
  A[Falha de rede real] --> B[Barra Sem conexão no topo do conteúdo]
  B --> C[Início mostra as músicas do aparelho]
  B --> D[Busca procura só no aparelho]
  B --> E[Faixa remota na fila: aviso com Tentar de novo]
  F[Rede voltou] --> G[Barra some + aviso Conexão de volta]
  G --> H[Início recarrega]
```

- Não há sondagem de rede em segundo plano: o app descobre que voltou quando a pessoa tenta algo ou quando a faixa seguinte resolve.

## F9. Erro do provedor

```mermaid
flowchart LR
  A[YouTube Music não respondeu] --> B{Há conteúdo em cache?}
  B -- sim --> C[Mantém a tela + faixa de aviso com Tentar de novo]
  B -- não --> D[Estado de erro: frase calma + Tentar de novo + código pequeno]
  E[Faixa indisponível] --> F[Pula para a próxima + aviso calmo]
```

## F10. Ajustar transparência e movimento

```mermaid
flowchart LR
  A[Engrenagem] --> B[Ajustes: Aparência]
  B --> C[Liga Reduzir transparência]
  C --> D[Vidro vira superfície sólida na hora, nesta mesma tela]
  B --> E[Liga Reduzir movimento]
  E --> F[Transições viram troca seca ou esmaecer curto]
  G[App mede quadros lentos] --> H[Oferece uma vez: Deixar mais leve?]
  H --> C
```

## F11. Reconhecer pelo som (futuro, desenhado para não mudar a navegação depois)

```mermaid
flowchart LR
  A[Buscar] --> B[Reconhecer pelo som]
  B --> C{Recurso disponível?}
  C -- não, hoje --> D[Linha esmaecida com Em breve]
  C -- sim, futuro --> E[Pede o microfone, com explicação]
  E --> F[Ouvindo: onda animada e Cancelar]
  F --> G[Achou: cartão da música com Tocar e Curtir]
  F --> H[Não achou: Tentar de novo ou digitar]
```
