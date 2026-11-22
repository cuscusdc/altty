# Biblioteca local (`kazzu-library`)

Decisões (etapa 1):

- **SQLite via `rusqlite` com `bundled`**: o mesmo SQLite (com FTS5) em todos os SOs, sem depender da lib do sistema. Custo: compilar C no build.
- **Migrações** por `PRAGMA user_version`; a lista fica em `src/db.rs`. Nunca edite uma migração publicada — acrescente outra. Banco mais novo que o app recusa abrir.
- **Scan incremental**: arquivo é relido apenas se `mtime` ou tamanho mudaram; arquivos sumidos são removidos (cascata em playlists/curtidos/histórico). Pasta raiz inacessível (volume desmontado) **não** apaga suas faixas; `remove_folder` apaga.
- **Capas** em `<cache>/covers/` (chave = FNV-1a 64 dos bytes originais, deduplica): `<k>.jpg` (≤640 px), `<k>_thumb.jpg` (≤160 px) e `<k>_blur.jpg` (48×48 já desfocado, para o fundo glass — a UI deve esticá-la). Fonte: imagem embutida, senão `cover/folder/front/album(art).jpg|png|webp` da pasta. Cor dominante (accent) por histograma ponderado por saturação.
- **Álbuns**: chave `(título, artista do álbum)` sem diferenciar maiúsculas. Sem tag de artista do álbum, agrupa por (título, pasta) e usa o artista comum ou `Various Artists`.
- **Histórico** (`play_history`) é só local, nunca enviado à rede; serve à retrospectiva (`history_range`).
- **Ids**: `i64` do SQLite; `to_core()`/`from_core()` convertem para os ids string de `kazzu-core` (`"42"`).
- Busca: FTS5 (prefixo, sem acentos/caixa) nas faixas; `LIKE` em álbuns/artistas.
- Tabela `downloads` é só placeholder para o modo offline (etapa 4).

## Banco v2 e downgrade

- O esquema v2 (playlists com faixas remotas por referência em `playlist_items`) **não abre em app antigo** (`Error::SchemaTooNew`): downgrade impossível. **Faça backup de `library.sqlite` antes de atualizar.**
- Índice "unificado" de playlist: `playlist_entries`, `remove_from_playlist` e `reorder_playlist` usam a mesma lista de itens *visíveis*. Itens com `remote_json` inválido ou local órfão ficam fora dos índices (com `log::warn!`), são preservados no banco e não deslocam a ordem relativa. Remover/reordenar é uma única transação `BEGIN IMMEDIATE`.
