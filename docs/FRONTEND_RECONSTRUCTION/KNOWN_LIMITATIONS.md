# Limitações e fronteiras

- **YouTube Music conectado.** O backend disponível oferece descoberta/reprodução visitante e importação de playlists via token da API YouTube. Login interativo, renovação e revogação de sessão exigem serviço autenticado. A interface condiciona entrada/saída às capacidades; usar um token de importação não torna a sessão de reprodução autenticada.
- **Spotify.** Importa metadados e coleções, sujeito a autorização do aplicativo e às restrições do serviço. Reprodução de referência segue o link do Spotify. Validação com conta real não foi feita.
- **Letras.** Componente e contrato implementados. A exibição sincronizada depende de um fornecedor que entregue texto e linha ativa para a faixa correta. A interface de produção oculta a ação por padrão.
- **DSP e áudio.** Equalizador, sleep timer, crossfade, seletor de dispositivos e controle de gapless aguardam operações do motor. Dados técnicos são os relatados pelo provedor. Não há aferição física da saída Hi-Res.
- **Downloads e arquivos externos.** Downloads offline aguardam gerenciador real. O Android lista diretórios de filesystem acessíveis com permissão; importação de URI de nuvem/SAF não está contemplada no contrato atual.
- **Histórico e ordenação.** O histórico mostra até 200 faixas locais recentemente ouvidas. A ordem alfabética de biblioteca é de sessão. A pilha de detalhes restaura rolagem externa; posição de cada trilho horizontal, sublista virtualizada e elemento assistivo exato não são serializados.
- **Fila.** A ação de mover para cima, combinada com a seleção da ocorrência, permite reordenar. Não há arraste livre entre quaisquer posições. A fila do motor é a única autoridade.
- **Plataformas.** Não foi realizada uma sessão autenticada em dispositivo físico, ensaio com TalkBack, comparação pixel a pixel com Apple Music ou medição prolongada de memória/GPU. O ambiente instalado não inclui target Windows. O relatório de testes registra os alvos realmente compilados.
