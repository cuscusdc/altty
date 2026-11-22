# Recomendações — pesquisa técnica

Data: 2026-11-21. Marcadores **[verificar]** = conhecimento prévio/fonte única.

## Resumo executivo

- **Não existe uma crate Rust madura de recomendação "pronta"** (ALS/implicit). `implicit` no crates.io é de 2016 e GPL-2.0; `linfa` (0.8.1, MIT/Apache) cobre clustering/regressão, não filtragem colaborativa. Reinventar um ALS grande não compensa: **um único usuário local não tem "vizinhos"** para filtrar colaborativamente.
- O que é **pronto e aberto** são os **grafos de similaridade coletivos** do **ListenBrainz/MusicBrainz** (dados CC0) e o que os próprios provedores devolvem (radio/related). A arquitetura proposta: **sinais locais + similaridade externa opt-in + um ranqueador pequeno local**.
- **Sem telemetria:** o app só faz consultas por MBID de *coisas* (artistas/faixas), nunca envia perfil/histórico do usuário. Contas ListenBrainz/Last.fm só se o usuário ligar.

## Candidatas

| Fonte | Link | Licença dos dados / termos | Manutenção | Maturidade | Prós | Contras | Risco |
|---|---|---|---|---|---|---|---|
| **ListenBrainz Labs `similar-artists`** (por MBID) | [labs.api.listenbrainz.org](https://labs.api.listenbrainz.org/similar-artists) | dados MusicBrainz/LB: CC0 nos dumps; termos da API [verificar rate-limit/User-Agent na doc oficial] | MetaBrainz (fundação sem fins lucrativos), ativa | boa; algoritmo baseado em co-ocorrência de artistas em sessões de escuta | Sem chave, sem conta; privado (só MBID); 100% aberto | Endpoint "labs" (experimental); falha devolve `[]` (parece "sem similares"); exige MBID | Baixo |
| **ListenBrainz LB Radio / Troi** | [Troi docs](https://troi.readthedocs.io/en/latest/lb_radio.html), [repo](https://github.com/metabrainz/troi-recommendation-playground) | Troi GPL-2.0 (só como serviço/HTTP, não linkar) | `troi` 2026.3.10 | boa | Endpoints públicos `/1/lb-radio/artist/{mbid}` e `/1/lb-radio/tags`; "weekly jams/exploration" por usuário (filtragem colaborativa feita no servidor) | Recs personalizadas exigem conta+token do usuário e histórico no LB | Baixo |
| **ListenBrainz data dumps** | [listenbrainz.org dumps](https://listenbrainz.org) **[verificar URL]** | CC0 (dumps) | periódico | alta | Baixar **offline** uma tabela de similaridade e consultar localmente = zero chamadas por faixa | Tamanho; precisa de pipeline de build | Baixo |
| **MusicBrainz** (tags, relações) | [musicbrainz.org](https://musicbrainz.org) / crate `musicbrainz_rs` 0.14.0 (MIT, 2026-08-27) | núcleo CC0; algumas tabelas de contribuidores CC BY-NC-SA **[verificar]**; limite ~1 req/s e User-Agent obrigatório **[verificar]** | ativa | alta | Mapeamento texto→MBID, tags/gêneros para similaridade local | Rate limit baixo; mapeamento fuzzy imperfeito | Baixo |
| **Last.fm `artist.getSimilar`** | [ToS](https://www.last.fm/api/tos) | **Só uso não comercial** sem acordo; cache até 100 MB; atribuição obrigatória; chave própria; limites a critério deles (~5 req/s é folclore de agregadores) | ativa; relatos de chave passando a exigir plano pago [verificar] | alta (dados ricos) | Melhor cobertura de "similares" e tags populares | Termos restritivos; precisa de API key do autor ou do usuário; centralizado | Médio |
| **Recs do provedor** (YT Music radio/related, Tidal, Qobuz "suggestions") | via `Provider::related` | ToS do provedor | variável | variável | "De graça", personalizado pela conta do usuário | Cada um diferente; Spotify removeu `recommendations`/`related` para apps novos **[verificar]**; `ytmapi-rs` marca `GetHome`/`GetSongRelated` como "Not Planned", `rustypipe` tem radio | Médio |
| **`linfa`** | [crates.io](https://crates.io/crates/linfa) | MIT/Apache | 0.8.1, 2025-12 | média | k-means/cosseno em perfis de tags locais | não traz filtragem colaborativa | Baixo |
| **ALS próprio com `ndarray`** | [ndarray](https://crates.io/crates/ndarray) 0.17.2 | MIT/Apache | ativa | — | Possível se um dia houver dataset (dump LB) | Esforço alto, benefício só com dados de muitos usuários | Médio (esforço) |
| Projeto da comunidade `listenbrainz-recs` (ALS + FAISS) | [GitHub](https://github.com/mmastroianni/listenbrainz-recs) | **[verificar]** | experimental | baixa | Referência de como se constrói vetores de 64 dim. de artistas | Python/Spark; só inspiração | — |

## Recomendação

**Arquitetura em 4 camadas, tudo local por padrão, rede opt-in:**

1. **Sinais locais** (SQLite do `kazzu-library`): `play_events` (reprodução completa, skip, like, repetição, hora). Sem enviar nada. Mesmos dados alimentam a Retrospectiva (ver `sound-and-integrations.md`).
2. **Perfil de gosto local**: vetor esparso de artistas/tags com **decaimento temporal** (meia-vida ~60–90 dias), pesos: like > conclusão > escuta parcial > skip (negativo).
3. **Candidatos** (qualquer combinação, cada uma desligável):
   - (a) **Dump offline do ListenBrainz** (artista → similares), consultado localmente: zero rede por faixa.
   - (b) `similar-artists` do LB Labs por MBID, com cache longo (30 d).
   - (c) `Provider::related`/radio do provedor ativo.
   - (d) Tags/gêneros do MusicBrainz para "descoberta lateral" (cosseno em tags via `linfa`/`ndarray`).
   - (e) Opcional: Last.fm (se o usuário colar a própria API key; atribuição na UI; aviso de uso não comercial).
   - (f) Opcional: recs pessoais do ListenBrainz se o usuário conectar o próprio token.
4. **Ranqueador local pequeno**: pontuação = `α·similaridade_com_perfil + β·novidade (não ouvido recentemente) + γ·popularidade_suave`, depois **MMR** (maximal marginal relevance) para diversidade; explicação curta ("porque você ouviu X"). Tudo determinístico e testável em Rust puro, sem modelo pesado.

**Fase futura (só se valer a pena):** vetores de artistas treinados offline a partir do dump do LB e embutidos como arquivo de dados versionado (ALS rodando no CI do projeto, não no usuário).

**Privacidade (invariantes):**
- Nenhum identificador de usuário/dispositivo em nenhuma requisição; `User-Agent` genérico `Kazzu/versão` (necessário para MusicBrainz).
- Consulta por MBID revela interesse por um artista a quem opera o servidor (e o IP). Mitigar: dump offline (camada 3a), cache, opção de proxy/Tor, batching atrasado. Documentar na UI.
- Scrobble (enviar histórico) é **outra** opção, separada, off por padrão.

**Risco legal:** baixo para MusicBrainz/ListenBrainz (CC0/abertos; respeitar rate limit e User-Agent). Médio para Last.fm (não comercial; se o Kazzu um dia for comercial, precisa de acordo). Recs de provedores herdam o risco de ToS do provedor.

## Esboço de integração

```rust
#[async_trait::async_trait]
pub trait RecommendationSource: Send + Sync {
    fn id(&self) -> &'static str;               // "lb-similar", "lb-dump", "provider:ytm", "lastfm"
    fn requires_network(&self) -> bool;
    async fn similar_artists(&self, seed: &ArtistRef, limit: usize) -> Result<Vec<Scored<ArtistRef>>, RecError>;
    async fn similar_tracks(&self, seed: &TrackRef, limit: usize) -> Result<Vec<Scored<TrackRef>>, RecError> { Err(RecError::Unsupported) }
}
// Ranker é código puro (sem async, sem rede): recebe Vec<Candidate> + TasteProfile e devolve ordem + explicação.
```

Cada `RecommendationSource` é registrada num `RecommendationHub` que respeita `Settings.network_opt_in`. `Provider::related` do trait de provedores é adaptada como mais uma fonte.

## Decisões que cabem ao autor

1. Embutir/baixar o **dump do ListenBrainz** (arquivo de dados; privacidade máxima, custo de armazenamento) ou só consultar a API?
2. Oferecer Last.fm (qualidade melhor, mas termos não comerciais) ou ficar só com LB/MusicBrainz?
3. Mapear faixas de provedores a MBIDs via MusicBrainz/AcoustID (rede, rate limit) ou apenas por nome/artista?

## Fontes (acessadas em 2026-11-21)

- ListenBrainz Labs: https://labs.api.listenbrainz.org/similar-artists ; explicação do algoritmo (fórum): https://community.metabrainz.org/t/how-does-similar-artists-work/678642
- Troi / LB Radio: https://troi.readthedocs.io/en/latest/lb_radio.html ; https://github.com/metabrainz/troi-recommendation-playground
- Issues de uso sem Last.fm: https://github.com/wkusnierczyk/sleevesight/issues/1158 ; https://github.com/Parachord/parachord/issues/1539
- Last.fm ToS: https://www.last.fm/api/tos
- crates.io API: linfa, ndarray, musicbrainz_rs, listenbrainz, implicit.
- Metrolist (só conceito: seções "descobrir", importações e recursos): `.refs/metrolist/README.md`.
