# Kazzu v4: direção de arte

Data: 2026-11-22. Estado: **proposta, aguardando o autor**. Depende do plano de UX em `ux/`.

## 1. O que a direção precisa resolver

1. O autor pediu Apple-like e recebeu pílulas escuras sobre uma malha colorida (v2), depois pílulas escuras sobre preto (v3).
2. O vidro nunca pareceu vidro, porque nunca houve nada com detalhe passando por trás dele.
3. O player, a tela mais olhada, ficou com um degradê marrom e uma cápsula de controles.

Não partimos da v2 nem da v3. O que elas ensinam está em `ux/01-diagnostico-e-benchmark.md`.

## 2. Moodboard escrito

Sem imagens de terceiros e sem copiar: são qualidades, não telas.

| Qualidade | De onde vem a ideia | Como aparece no Kazzu |
|---|---|---|
| Vitrine de loja de discos à noite | O vidro só existe por causa do que está atrás dele | A interface é escura e neutra; as capas são a luz |
| Mesa de vidro com um objeto em cima | O reflexo borrado do objeto na mesa | No player, a capa e o reflexo dela desfocado; o console é o vidro da mesa |
| Painel de carro bom | Poucos controles, grandes, onde a mão cai | Transporte sem caixas, um botão de tocar branco |
| Encarte de disco | Tipografia grande e quieta, muito ar | Títulos de 34, corpo de 17, três pesos |
| Aplicativos do sistema da Apple | Listas agrupadas, barra de abas flutuante, título que recolhe | Convenções de uso, não o visual copiado |
| Brasa | Um ponto de cor quente num lugar escuro | O acento: só estado (tocando, curtido, ligado) e links |

O que fica de fora, de propósito: malha de degradê, manchas de cor, brilho neon, cartão dentro de cartão, pílulas para tudo, ilustrações vetoriais como capa, texto em caixa alta espaçada, selos decorativos.

## 3. Três direções comparadas

| | A. Vitrine (escolhida) | B. Estúdio | C. Aquário |
|---|---|---|---|
| Ideia | Interface neutra; toda a cor vem das capas, vistas através do vidro | Painéis foscos de grafite, fios finos, zero flutuantes | O app inteiro sobre a capa atual desfocada |
| Fundo | Quase preto ou quase branco, liso | Cinza grafite em três níveis | Sempre colorido, muda a cada música |
| Vidro | Só no que flutua: doca, cabeçalho recolhido, folhas, botões sobre capa, console do player | Só no player | Em todos os cartões |
| Navegação | Cápsula de vidro + busca redonda | Barra sólida presa embaixo | Cápsula de vidro |
| Player | Capa inteira de ponta a ponta, o reflexo dela, console de vidro sobre o reflexo | Capa centrada em cartão, controles em painel fosco | Capa centrada, tudo sobre desfoque |
| Ponto forte | Vidro evidente e legível; capas mandam; realizável no Slint | Sóbrio, barato de renderizar | Vidro máximo |
| Risco | Depende de capas boas; 4 desfoques ao vivo por tela | Repete a queixa "sem glassmorphism" | Volta ao "degradê feio"; contraste imprevisível; cansa |
| Atende o autor? | Sim, nos quatro pedidos | Não no vidro | Não no degradê |

**Escolha: A, Vitrine.** É a única que cumpre os quatro pedidos de uma vez (celular primeiro, Apple-like, vidro evidente, sem degradê) e a única em que o vidro tem razão de existir: ele mostra a música. De B veio a disciplina de superfícies sólidas para tudo o que não flutua (listas agrupadas, barra lateral do desktop). De C veio o player, que é o único lugar onde a capa pode tomar a tela inteira sem brigar com nada.

## 4. A ideia em uma frase

> A interface é neutra e de vidro; toda a cor vem das capas, e as capas passam por trás de tudo o que flutua.

Consequências, de cima a baixo:

1. **O fundo nunca tem cor própria.** Nada de malha, mancha ou degradê decorativo. Os dois únicos degradês da UI são brancos ou neutros e servem ao vidro: o brilho no topo de um painel e o esvaecimento leve sob a doca.
2. **Vidro só onde há conteúdo atrás.** A barra lateral do desktop e as listas agrupadas são sólidas, porque nada rola por trás delas.
3. **O acento é sinal, não tinta.** Brasa aparece em menos de 5% de qualquer tela.
4. **No vidro, todo texto é a tinta principal.** Cinza e acento dependem do que passa atrás; hierarquia ali é tamanho e peso.
5. **O player é a assinatura.** A capa inteira ocupa a largura toda, o reflexo dela continua para baixo, e um único console de vidro fica sobre o reflexo. O mini-player é esse console em miniatura, tingido pela mesma capa.
6. **No escuro, vidro neutro é fosco.** A cápsula e as folhas usam um cinza mais claro que o fundo, com fio de luz forte: sobre preto ainda são uma lâmina, não um buraco.

## 5. Aparência em resumo

| Decisão | Valor | De onde vem |
|---|---|---|
| Fundo | escuro `#07080a`, claro `#f3f4f6` (neutros com um traço de azul, croma 0,003 a 0,005) | Neutro como os apps do sistema; não é preto nem branco puros |
| Tinta | escuro `#f2f3f6`, claro `#0f1217` | Contraste de 17:1 sobre o fundo |
| Acento "Brasa" | escuro `#ff7345`, claro `#cc2c05` | Continuidade com o K laranja da marca; quente, distinto de Apple Music, Spotify e YouTube Music |
| Tipografia | Inter (400, 600, 700) e Inter Display (700) nos títulos | Pedido do autor: "SF-like livre como Inter". OFL, arquivos estáticos (o Slint não tem fontes variáveis) |
| Forma | raios 8, 12, 14, 16, 22, 32 e cápsula | Um raio por nível, concêntricos |
| Ícones | traço uniforme de 1,75 em grade de 24, pontas redondas; versão preenchida para o item ativo | Conjunto próprio do projeto (57 ícones) |
| Movimento | 120, 220 e 420 ms; curva de chegada que imita mola sem ressalto | Convenção da Apple, sem pulo em estado de UI |
| Capas de demonstração | 22 fotografias CC0 ou de domínio público | Sem capa com detalhe o vidro não se prova |

Detalhes e medidas: `DESIGN.md`. Contrato para o implementador: `SLINT_SPEC.md`.

## 6. Marca

Mantive o K da marca atual (haste clara e chevron na cor de acento) e a palavra "kazzu" desenhada com o mesmo traço, redesenhados para a paleta da v4. Não criei uma marca nova: o autor não reclamou dela, e trocar marca e interface ao mesmo tempo esconderia qual das duas mudanças agradou. Arquivos: `assets/kazzu-icon.svg`, `kazzu-mark-*.svg`, `kazzu-wordmark-*.svg`. Se o autor quiser uma marca nova, é uma decisão separada (A7 no plano de UX).

## 7. Como a direção foi verificada

| Pergunta | Resposta | Evidência |
|---|---|---|
| Dá para fazer no Slint 1.19? | Sim | `slint/*.slint` compila; 44 renders em `shots/slint/` |
| O vidro é vidro? | Sim, em 11 casos medidos | `shots/glass-proof.md` |
| Passa em AA? | 0 textos abaixo do limite em 188 capturas | `contrast-report.md`; pares dos tokens em `contrast-tokens.md` |
| Passa nas skills? | hallmark: 0 FAIL, 0 REFLEX. taste-skill: copy-lint limpo; o portão de sites fica aberto, com os itens de site listados | `SKILLS_REPORT.md` |
| Um leitor sem contexto concorda? | Disse "fix", com 8 achados; corrigidos e pontuados | `SKILLS_REPORT.md`, seção da revisão |
