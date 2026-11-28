> Arquivo histórico fora do workspace. A interface ativa foi integrada em `crates/kazzu-app`. As instruções abaixo documentam o protótipo isolado original.

# kazzu — port nativo do protótipo HTML

Aplicativo independente no workspace existente. O transporte é simulado por solicitação do protótipo. Não emite áudio e não utiliza o backend de produção do Kazzu.

## Executar

Na raiz do repositório:

```sh
cargo build -p kazzu-html-player
cargo run -p kazzu-html-player
```

Na pasta desta crate, também funciona `cargo build` e `cargo run`.

Para capturas por software sem janela do sistema:

```sh
cargo run -p kazzu-html-player -- --snapshot-dir=target/html-player-snapshots
```

## Inventário

[INVENTORY.md](docs/INVENTORY.md) lista telas, componentes, textos e todos os handlers encontrados. [source-inventory.json](docs/source-inventory.json) preserva o catálogo, paletas, ajustes e eventos extraídos.

## Vidro

Abordagem A. O componente Glass combina preenchimento translúcido em gradiente, borda clara, linha superior e sombra. Não realiza blur do conteúdo subjacente. As capas radiais do HTML são aproximadas por gradientes lineares com as cores originais. O fundo do player usa as três cores por faixa.

## Limites conhecidos

- Protótipo desktop com composição móvel centralizada de até 420 px. Empacotamento Android desta crate ainda não configurado.
- Estatísticas, perfil, coleções, texto de qualidade e crossfade são demonstrações extraídas do HTML. Normalização, timer, seleção de dispositivos e opções adicionais mantêm o feedback do protótipo e não processam áudio.
- Nova playlist não tem fluxo no HTML. O botão informa sua indisponibilidade.
- Pesquisa implementada apenas no catálogo embutido de oito faixas, incluindo letras. A seleção de fonte tem estado visual e ambas as fontes usam esse catálogo demonstrativo.
- Chips têm seleção visual. O HTML não define a filtragem de conteúdo por categoria.
- Interface sem animações por padrão, inclusive quando o sistema solicita redução de movimento. O toggle de animações conserva apenas o estado de apresentação do protótipo.
- A letra é estática. O realce das linhas 3 e 4 reproduz a demonstração, sem sincronização por timestamps.
- Compartilhamento copia título e artista. Não há URL pública do protótipo desktop para compartilhar.
- A identificação v1.4.2 pertence ao material fornecido. A crate usa versão própria 0.1.0.
- O retorno por Escape fecha confirmação, fila, player e configurações nessa ordem. Espaço, J/L, S/R e M executam os atalhos listados pelo HTML.
- A tipografia usa as fontes disponíveis no sistema. Ícones dos controles foram extraídos dos SVGs do HTML fornecido.

Os controles de reprodução e as operações simuladas são mantidos em Rust. Os modelos de áudio e de apresentação usam contratos separados.
