# Interface do HTML integrada ao Kazzu

A interface ativa está em crates/kazzu-app. O executável isolado foi arquivado em design/prototypes/archived/kazzu-html-player, fora dos membros do workspace.

## Integrações

- Os seis esquemas de cor do HTML entram pelo gerador de tokens. Solar é a seleção padrão, com acento #ff6b00 e base escura #120a05. Modos claro, escuro e OLED continuam operados pelo estado real. A seleção de paleta é salva na configuração Rust.
- Glass ganhou o gradiente branco em três paradas, fio de luz superior e as bordas existentes. FemtoVG pode aplicar o desfoque já suportado no Kazzu. Software usa a mesma camada de brilho sobre a superfície sólida. Reduzir transparência limita o material.
- A grade de escolha rápida usa até oito faixas efetivas da descoberta ou do acervo. A nona célula chama o atalho aleatório real já implementado. Nenhuma faixa do catálogo fixo do HTML foi importada para a produção.
- Explorar exibe categorias e conteúdo publicados pelo provedor. Quantidade, rótulos, capas e ações vêm dos modelos reais. A grade segue duas colunas no telefone e quatro em telas largas.
- Biblioteca usa cartões de coleções compactos e conserva edição, grade/lista, pesquisa, ordenação, pastas e playlists reais.
- O console do player recebe o material Glass e mantém os callbacks reais de transporte e fila.

## Mini-player e navegação

A composição inferior usa um único layout vertical com duas linhas limitadas a 58 px, intervalo de 12 px e margem inferior de 10 px acrescida do inset do sistema. O mini-player é um componente independente. A reserva de conteúdo usa as mesmas medidas. A linha inferior segue o HTML mais recente com Início, Explorar e Biblioteca, além da bolha de Pesquisa de 58 px.

A posição da linha inferior é altura da janela menos inset, margem e 58 px. O mini-player termina 12 px acima dessa linha. Essa regra vale com a área segura de 34 px usada no teste de renderização e sem inset.

## Verificação

Os logs desta integração usam prefixo target/html-integration-. A suíte Slint cobre a nova navegação, troca de paleta, reserva inferior e toques no mini-player e na pesquisa. O teste de configuração cobre a persistência de uma paleta diferente do padrão.

As capturas do protótipo separado são históricas. Os resultados de build e teste da integração são registrados no fechamento da validação desta entrega.

Os empacotamentos desta rodada usam a pasta `target/android/html-integrated/`. O APK de `frontend-reconstruction/` corresponde à rodada anterior. O arquivo de fontes em `dist/source/` é histórico e foi identificado como tal.
