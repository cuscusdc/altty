# Kazzu — brief do produto e regras de desenvolvimento

Fonte: mapa mental do autor (`docs/mindmap/Kazzu.canvas`, Obsidian). Este documento é a fonte única de verdade para todos os agentes.

## O que é

Kazzu é um player/cliente de música multiplataforma, **Rust + Slint**, com visual Apple-like e glassmorphism, áudio em qualidade original (hi-fi) e sem telemetria.

## Requisitos do mapa mental

**Produção**
- Fazer **em etapas**; nunca o app inteiro de uma vez.

**Tecnologia**
- Slint + Rust. O que der, em Rust (performance).
- Provedores de música: Spotify, YT Music, Tidal, Qobuz.
- Algoritmo de recomendações: de preferência usar algo pronto.
- Backend: preferir bibliotecas prontas e open source a reinventar a roda.

**Propostas (diferenciais)**
- Sem telemetria (só recebe dados).
- Áudio em qualidade original: `.flac`, `.wav`, `.opus`*.
- Equalizador sofisticado.
- Procurar música pelo som ambiente.
- Integração com o Discord.
- Player offline/local.
- Retrospectiva estilo Spotify.
- Sincronização entre dispositivos.
  - Ouvir junto com amigo.
    - Um dispositivo pode controlar a música do outro, sem a limitação do Spotify de tocar 1 música por dispositivo.
  - Funcionar offline.

**Design**
- Apple-like, glassmorphism.
- Referências visuais: Intertune, Bitchord, Umihi Music (imagens em `.refs/images/`, não versionadas).
  - **Bitchord**: menu de início; barra de navegação (Início / Explorar / Biblioteca) e um botão de busca com ícone próprio dentro da barra principal. Logo no canto superior esquerdo. **Não** fazer a search bar no topo.
  - **Bitchord, player**: capa grande com fundo derivado da capa, título/artista, barra de progresso, controles grandes, volume, letra/AirPlay/fila.
  - **Umihi Music, biblioteca**: grade de cartões com Baixados, Curtidos, Álbuns, Playlists; botão "Criar playlist".
  - **Intertune**: referência de ícone/identidade.
- Biblioteca: álbuns, playlists, baixados, curtidos.

## Regras inegociáveis

1. **Não copiar código nem assets de nenhuma referência.** Metrolist (GPL-3.0) e os apps citados servem só de inspiração conceitual (funcionalidades, arquitetura geral). Não cole trechos, não traduza arquivo por arquivo.
2. **Sem telemetria.** Nenhuma chamada de rede que não seja a necessária ao recurso que o usuário acionou.
3. **Preferir crates prontas e open source** (licenças compatíveis com MIT/Apache; registre cada dependência e licença em `docs/DEPENDENCIES.md`).
4. **Rust sempre que possível.** Dependências nativas só quando não houver equivalente maduro.
5. **Qualidade original**: sem resample/normalização/EQ forçados no caminho padrão. Resample só se o dispositivo de saída exigir (e documentado). EQ é opt-in e bypass = bit-perfect.
6. **Etapas pequenas e verificáveis.** Cada etapa termina com `cargo build`, `cargo clippy -- -D warnings`, `cargo test` verdes.
7. **Design**: aplicar as skills `taste-skill` e `hallmark` (cópias locais em `.refs/taste-skill` e `.refs/hallmark`, só leitura; upstream: github.com/Leonxlnx/taste-skill e github.com/Nutlope/hallmark). São skills de web, então traduza os princípios (direção de arte, anti-"cara de IA", contraste, foco, movimento reduzido, hierarquia) para Slint, não copie CSS/HTML cegamente.
8. Documente decisões em `docs/` em português do Brasil. Código e identificadores em inglês.
9. Não faça `git commit`/`push`: o orquestrador integra. Edite apenas os diretórios designados para você.

## Direção v2 (decidida pelo autor em 2026-11-22)

1. **Mobile-first.** O alvo principal é o celular (Android primeiro, depois desktop e iOS). Projetar e construir a partir de 360–412 px de largura, com o polegar como referência; tablet/desktop são expansões do mesmo design, não o contrário.
2. **Glassmorphism de verdade.** Painéis translúcidos com desfoque real sobre fundos vivos e coloridos (malha de gradiente e capa desfocada), com borda de luz, brilho especular e sombra suave. Se o painel parecer um cartão escuro opaco, está errado. Sempre com alternativa acessível (reduzir transparência).
3. **UX planejada com convenções e boas práticas** (heurísticas de Nielsen, WCAG 2.2 AA, Material 3/HIG para gestos e alvos, lei de Fitts/zona do polegar, divulgação progressiva, estados vazios/carregando/erro/offline).
4. **YouTube Music é o provedor padrão, já ativo, sem login nem configuração** (início, busca, reprodução funcionam no primeiro uso). Outros provedores entram depois como opt-in. O autor aceitou o risco de termos de uso do YT Music.
5. **Licença do app: GPL-3.0** (hipótese de trabalho, já que permite as crates GPL necessárias e é gratuita; reversível enquanto o repositório for privado).

## Regra de custo: 100% grátis

O Kazzu, tudo o que ele usa e tudo o que é preciso para publicá-lo devem ser gratuitos para o autor e para quem usa.

- Só dependências open source, sem plano pago, sem chave de API paga, sem servidor pago. Serviços externos só se tiverem camada gratuita estável e sem cartão (ex.: MusicBrainz/ListenBrainz); sempre opt-in.
- Slint: usar a licença GPL-3.0 (gratuita). Não usar a licença comercial. O app seguirá GPL-3.0 (hipótese de trabalho, a confirmar pelo autor; isso também libera crates GPL).
- Sem infraestrutura hospedada: sync e "ouvir junto" por LAN/P2P (mDNS, QUIC); relay só se for gratuito e auto-hospedável pelo usuário.
- Distribuição só por canais gratuitos (GitHub Releases, AppImage, Flatpak/Flathub, Android APK/F-Droid). Nada de contas de desenvolvedor pagas (Apple Developer, Google Play) por ora.
- Provedores: o app é gratuito; o acesso a Spotify (Premium), Tidal e Qobuz depende de uma assinatura que o USUÁRIO já tem. Nunca o app cobra, nem embute conta do autor. Prioridade: fontes gratuitas (arquivos locais, YT Music sem Premium, Internet Archive/Jamendo etc., a avaliar na Etapa 2). Reconhecimento por som: só soluções gratuitas e abertas; serviços pagos descartados.

## Etapas

| Etapa | Escopo |
|---|---|
| 1 — Fundação | Design system; workspace; motor de áudio (FLAC/WAV/Opus/MP3/etc.); biblioteca local (scan + SQLite); UI Slint: Início, Player, Biblioteca; busca em botão próprio |
| 2 — Provedores | Trait `Provider`; YT Music e Spotify primeiro (ver `docs/research/`); Tidal/Qobuz depois; recomendações |
| 3 — Som | Equalizador avançado; integração com Discord; MPRIS/controles de mídia |
| 4 — Social/sync | Sincronização, controle remoto entre dispositivos, ouvir junto, offline |
| 5 — Extras | Retrospectiva; busca por som ambiente |

## Layout do repositório

```
crates/kazzu-core      tipos de domínio, trait Provider, eventos, erros
crates/kazzu-audio     motor de áudio (decode, saída, fila, gapless, seek)
crates/kazzu-library   scan de arquivos, tags, SQLite, playlists, curtidos
crates/kazzu-app       binário Slint (UI)
design/                design system, tokens, protótipos
docs/                  brief, arquitetura, pesquisa, mapa mental
```
