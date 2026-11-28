# Sistema visual nativo

Atualização da interface do HTML registrada em [HTML_INTEGRATION.md](HTML_INTEGRATION.md). Ela acrescenta as paletas e a rota própria de Explorar ao Kazzu existente.

| Papel | Implementação | Comportamento |
|---|---|---|
| Papel, superfície, texto, seleção e foco | Theme, gerado por `design/v4/tools/gen_tokens.py` | Automático, claro, escuro e preto OLED. O papel OLED usa preto puro. A mídia usa uma escala própria para texto sobre capa. |
| Material de navegação | Glass e MediaGlass | Uma superfície translúcida para navegação ou ferramenta; linha de luz, sombra e desfoque restritos. Software ou redução de transparência usa superfície sólida. |
| Custo gráfico | Theme.quality | Econômico desliga vidro desfocado; Equilibrado usa 12/16 px (cromo/folha); Alto usa 24/28; Máximo utiliza os raios integrais do tema. Preferência salva na configuração visual. |
| Texto | Inter e Inter Display já distribuídos | Título e artista com hierarquia por peso e escala; elipse em rótulos curtos, quebra nas explicações e erros. |
| Arte | Cover, Mosaic, FeatureArtwork | Arte real do modelo. Disco neutro quando ausente. O ambiente do player usa a imagem já preparada pelo adaptador. |
| Reprodução | PlayerConsole, Scrubber | Trilho de 10 px, alvo de 48 px, ajuste por teclado e ações assistivas. Seek depende da duração informada pelo motor. |
| Conteúdo | Tile, TrackRow, SectionHeader, Rail, Rows | Capas lideram a descoberta; listas para títulos e operações densas. O menu da faixa é acessível pela elipse e pelo clique contextual no desktop. |
| Formulários e estados | SearchField, Segmented, Button, Switch, StateCard, SkeletonRows, Spinner, Toast | Estado local do campo e feedback de operação separados. Importações mantêm o resultado no painel. |
| Folhas | SheetBase, SheetHeader, QueueSheet | Painel lateral da fila em desktop amplo. Folha inferior nas dimensões compactas. Véu, fechar e Voltar com papéis distintos. |
| Layout móvel | MobileDock, MobileMiniPlayer, MobilePlayer | Ilha de três destinos, mini-player independente e topo/reprodução fora da rolagem. Insets laterais protegem a composição móvel. |

Animações herdam as durações centrais, respeitando redução de movimento. Ícones de ação têm rótulos acessíveis e tooltip. Modelos de teste entram pela View nas suítes de renderização, sem alimentar produção. O Android solicita `adjustResize` para o teclado. Teclado de terceiros, TalkBack, leitores de tela de desktop e GPU requerem validação em dispositivo.

SettingsIndex e SettingsSections concentram os campos das doze seções de Ajustes. Desktop escolhe a categoria pela lista lateral ou pela faixa compacta; Android conserva a faixa horizontal tocável. As categorias Áudio Hi-Res, Qualidade gráfica, Armazenamento, Downloads e Acessibilidade têm destinos pesquisáveis próprios. Downloads informa a disponibilidade do serviço. Armazenamento permite operar nas pastas reais.
