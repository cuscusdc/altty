# Contratos entre interface e serviços

| Domínio | Contrato atual | Resposta esperada |
|---|---|---|
| Transporte | View::set_playback, set_progress, set_seek_available, set_now_art; AppState.toggle-play, previous, next, seek, set-volume | PlaybackVm e NowArtVm; duração ausente desabilita seek. Erro, carregamento e repetição têm representações distintas. |
| Fila | QueueVm com faixa atual publicada em PlaybackVm e próximas indexadas por ID de item; queue-jump, queue-move-up, queue-remove, queue-clear | Publicar novamente as próximas, mensagem e origem. A publicação nunca fecha a folha. Limpar para o motor e pede confirmação. |
| Detalhe | DetailRoute::Library(DetailRef) ou Catalog(RemoteRef); JobResult::Detail/DetailFailed e equivalente remoto | Resultados vinculados à sequência da visita. Atualizar modelo antes de aplicar posição. |
| Coleção | LibraryVm, PlaylistVm, DetailVm.editable_playlist; PlaylistEdit::Rename/Delete/Remove/MoveUp | Persistir pelas APIs de biblioteca existentes. Índice e ID de faixa protegem contra remoção de outro item após mudança de ordem. |
| Importação | import-account(provedor, token); View::import_status; JobResult::AccountImported | Manter a operação pendente entre rede e persistência. Resultado sobrevive ao fechamento da folha. Faixas Spotify abrem externamente. |
| Busca | search(text), search-source-changed, search-kind-changed; SearchVm | Distinguir consulta pendente, erro, offline, vazio e resultados; consultas antigas não substituem a atual. Tipo permanece editável em todos os estados de consulta. |
| Arquivos | browse-folders, add-named-folder, add-folder, remove-folder, rescan; FolderListingVm e ScanVm | Listagem e scan em background. Progresso negativo representa descoberta sem contagem final. |
| Letras | OptionalServices.lyrics, request-lyrics(id), LyricsState, LyricsLine, lyrics-active-line, lyrics-error | Publicar somente o resultado da faixa atual. Tempo vazio admite letra estática; índice −1 indica ausência de linha ativa. Repetir pedido após falha. |
| Conta | OptionalServices.youtube-login/manage-session, begin-youtube-login, end-session, session-label | Adaptador de sessão deve conduzir autorização, retorno, expiração e saída. O modo visitante e a importação de coleções já são distintos. |
| Downloads e processamento | OptionalServices.downloads, audio-devices, equalizer e sleep-timer | Capacidade declarada separadamente. Falta fonte para progresso de transferência, dispositivos selecionáveis e parâmetros DSP. |

**Acordos propostos ao backend.** Downloads precisam de ID estável de tarefa, faixa, estado (pendente, ativo, pausado, concluído, falhou), progresso opcional, erro recuperável e regras de cancelamento. Para áudio, cada saída deve publicar recursos aceitos, saída selecionada e erro da troca. Para sessão, expor estado de autorização sem entregar credenciais persistentes ao Slint. Crossfade e gapless precisam de confirmação do motor para a rota de reprodução atual. Nenhuma dessas extensões deve habilitar um botão apenas por compilação de uma crate.

O painel técnico usa código do formato, qualidade e detalhes de profundidade/taxa presentes nos modelos existentes. O frontend não estima bitrate a partir da extensão do arquivo nem certifica a cadeia física como Hi-Res.
