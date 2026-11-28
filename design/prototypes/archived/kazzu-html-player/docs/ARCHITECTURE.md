# Estrutura do protótipo

A crate kazzu-html-player é descoberta pelo workspace existente através de crates/*. Seu executável tem título explícito de protótipo simulado. Os dados embutidos vêm do HTML fornecido e ficam em JSON revisável.

State é o contrato Slint de apresentação. Theme centraliza acento, acento secundário, base, superfície e cores de texto. O controlador Rust publica faixas, tempos, paletas e configurações. Slint mantém navegação, visibilidade dos painéis, filtros visuais e marcações de like/repeat/shuffle.

Audio define o contrato do transporte. SimulatedAudio controla reprodução, pausa, volume e avanço pelo tempo transcorrido. Um Timer Slint de 100 ms agenda a atualização. O controlador mantém as posições individuais de todas as faixas. Próxima e seleção de card zeram a posição. A seleção na fila preserva a posição. Anterior reinicia a faixa quando a posição ultrapassa quatro segundos e volta uma faixa nos demais casos.

Compartilhar usa arboard e conserva a instância de clipboard até o encerramento, para atender a ownership de clipboard em X11. Uma falha de acesso exibe mensagem explícita.

Escaneamento utiliza prazos no Timer, sem bloquear o loop gráfico. Os números publicados ao concluir são os números da simulação do HTML. Limpar cache e histórico exigem confirmação dentro da janela. Toasts expiram após 2,2 segundos.

A detecção de movimento reduzido é conservadora. Esta implementação não agenda animações de transição, atendendo também ambientes que pedem redução de movimento. A preferência visual de animações pode ser alternada, mas não introduz movimento neste port.

A janela se adapta entre 300 px e larguras de desktop. O dispositivo é centralizado, com máximo de 420 × 900 px. Todas as páginas usam uma área rolável. O player tem rolagem própria para manter os controles acessíveis em janelas pequenas. A ilha inferior fica fora da rolagem das páginas.

O programa oferece geração de capturas por software, além da execução normal. Esta via usa as mesmas classes Slint e callbacks da interface. Ela verifica transporte, anterior, shuffle, seek, volume e tema, antes de gerar as capturas.
