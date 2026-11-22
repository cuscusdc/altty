# Sincronização, controle remoto e "ouvir junto" — pesquisa técnica

Data: 2026-11-21. **[verificar]** = conhecimento prévio ou fonte única.

## Resumo executivo

- **Transporte recomendado: `iroh` 1.x** (QUIC, "disque chaves, não IPs", hole punching, relay **open source e auto-hospedável**, MIT/Apache, 1.0 em 2026-06-15). Serve LAN e internet com o mesmo código. Descoberta em LAN: `mdns-sd` (ou o discovery mDNS do próprio iroh **[verificar]**).
- **Estado entre dispositivos do mesmo usuário:** começar com **SQLite + LWW por campo + relógio lógico híbrido (HLC)**; usar **`yrs`** (ou `automerge`) só onde a ordem importa (playlists/fila). CRDT para tudo é exagero.
- **Ouvir junto = sincronização de *linha do tempo*, não de áudio:** cada par resolve a faixa com a **própria conta/arquivo**; o app troca só `(TrackRef, posição, relógio)`. Sincronia de relógio estilo NTP (4 timestamps) sobre a conexão QUIC + agendamento no futuro + correção de deriva.
- **Licença:** nunca retransmitir áudio de provedor (Spotify/YT/Tidal/Qobuz). Para arquivos locais, só entre dispositivos do próprio usuário.

## 1. Transporte e descoberta — candidatas

| Candidata | Link | Licença | Manutenção | Maturidade | Prós | Contras | Risco |
|---|---|---|---|---|---|---|---|
| **`iroh`** | [GitHub](https://github.com/n0-computer/iroh) | MIT/Apache | 1.5.0, 2026-11-19; 1.0 em 2026-06-15 (~200 mi conexões/mês segundo o anúncio) | alta | Dial por chave pública (NodeId), QUIC multipath, hole punching (QNT; ~9/10 direto segundo FAQ; heise cita ~95 % direto), relay stateless auto-hospedável (um Docker), E2E TLS 1.3 (relay não decifra), bindings Swift/Kotlin; `iroh-gossip` (HyParView/Plumtree) e `iroh-docs` (KV CRDT, pré-1.0) | Dependência grande; relays públicos da n0 têm limite de taxa | Baixo-médio |
| `mdns-sd` | [GitHub](https://github.com/keepsimple1/mdns-sd) | Apache-2.0/MIT | 0.22.0, 2026-11-17; ~8 mi downloads | alta | Rust puro, daemon próprio, responder+browser; sem internet | Só mesma sub-rede; Wi-Fi com *client isolation* bloqueia multicast; Android exige *MulticastLock*; iOS exige permissão "Rede local" + `NSBonjourServices` | Baixo |
| `quinn` | [GitHub](https://github.com/quinn-rs/quinn) | MIT/Apache | 0.11.12, 2026-09-14 | muito alta | QUIC puro e flexível | **Sem** NAT traversal/relay (teria que construir) | Médio (esforço) |
| `libp2p` | [GitHub](https://github.com/libp2p/rust-libp2p) | MIT | 0.57.1, 2026-10-15 | alta, ecossistema grande | DCUtR (hole punching), relay circuit v2, mDNS, gossipsub, Kademlia | Pesada/complexa; muitas peças; API ainda em 0.x | Médio |
| `webrtc` (webrtc-rs) | [GitHub](https://github.com/webrtc-rs/webrtc) | MIT/Apache | 0.21.1, 2026-11-07 | média | Interoperável com navegadores (útil só se houver cliente web) | Exige servidor de sinalização + STUN/TURN; API pesada | Médio |
| `matchbox_socket` | [GitHub](https://github.com/johanhelsing/matchbox) | MIT/Apache | 0.15.0, 2026-10-29 | média (focada em jogos) | WebRTC P2P simples | Voltada a jogos/WASM; exige sinalização | Médio |
| `sntpc` | [GitHub](https://github.com/vpetrigo/sntpc) | MIT/Apache | 0.11.0, 2026-07-06 | alta | Cliente SNTP — só útil contra servidor NTP real; para pares, é melhor medir RTT na própria conexão | — | Baixo |

### Recomendação (transporte)

- **`iroh` como camada única** (LAN + WAN); `mdns-sd` como descoberta de LAN *sem internet* (quando o discovery do iroh não for suficiente); modo "somente LAN" desliga relay e descoberta global (atende "funcionar offline").
- **Relay:** padrão = nenhum servidor do autor. Opções: relays públicos da n0 (terceiros, opt-in) ou **relay próprio** (`iroh-relay`) configurado pelo usuário/amigos. Relay só vê tráfego cifrado; nunca transporta áudio.
- **Pareamento:** QR/código de 6 dígitos que troca as chaves públicas (NodeId); lista de dispositivos confiáveis; papéis ("pode controlar", "só ouvir junto"). Revogação local.
- **Mobile:** iroh em Rust cross-compila para Android/iOS (existem bindings Kotlin/Swift); atenção a background no iOS (conexões suspensas).

## 2. Sincronizar biblioteca/playlists entre dispositivos próprios

| Candidata | Link | Licença | Manutenção | Notas |
|---|---|---|---|---|
| `yrs` (Yjs em Rust) | [GitHub](https://github.com/y-crdt/y-crdt) | MIT | 0.28.2, 2026-11-19; ~5 mi downloads | Rápida, listas/mapas/texto, protocolo de sync incremental; boa para playlists reordenáveis |
| `automerge` | [GitHub](https://github.com/automerge/automerge) | MIT | 0.12.0, 2026-09-16 | Modelo de documento JSON com histórico; mais pesada |
| `iroh-docs` | [iroh](https://github.com/n0-computer/iroh-docs) | MIT/Apache | pré-1.0 | KV replicado sobre iroh; ainda instável **[verificar]** |
| LWW + HLC em SQLite | — | próprio | — | Simples, fácil de depurar; basta para "curtidos", configurações, contadores |

**Recomendação:** modelo **"event log por dispositivo"** no SQLite (cada alteração = linha com HLC e `device_id`) + merge LWW por campo; playlists com ordem como **lista CRDT via `yrs`** (um doc por playlist). Contadores de reprodução somam por `device_id` (sem conflito). Conteúdo de provedores não é sincronizado — só **referências** (`provider+id`) e metadados mínimos. Tokens/cookies **nunca** sincronizam por padrão (cada dispositivo faz seu login).

## 3. Controle remoto (um dispositivo controla o outro)

Protocolo sobre um fluxo QUIC bidirecional (cada par com ALPN `kazzu/ctrl/1`):

- **Mensagens** (serde + `postcard` ou protobuf/`prost`, com campo de versão): `Hello{device, caps}`, `Snapshot{now_playing, queue, position_ms, at_clock, volume, state}`, `Cmd{Play|Pause|Seek|Next|Prev|SetQueue|Enqueue|Remove|Volume|Shuffle|Repeat|Transfer}`, `Event{…}` (patches incrementais), `Ack`.
- **Autoridade:** o dispositivo que *toca* é dono do estado; o controlador envia comandos com `cmd_id` e recebe `Snapshot` de volta (modelo "controlador burro"). Mais simples e sem conflito.
- **Transferir reprodução ("tocar aqui"):** envia `TrackRef + position + queue`; o alvo resolve com **a própria conta/arquivo** (nunca transmite áudio). Se o alvo não puder (sem acesso à faixa), erro explícito.
- **Sem a limitação do Spotify:** possível para fontes locais e para provedores sem trava de stream por conta; **Spotify mantém 1 stream ativo por conta (regra do serviço)**, e Tidal/Qobuz podem limitar sessões simultâneas **[verificar]**. A UI deve explicar, não esconder.
- **Offline:** LAN via mDNS/iroh sem internet funciona desde que a faixa esteja local/baixada.

## 4. Ouvir junto — sincronização de relógio e reprodução

Referência de projeto (conceito): **Snapcast** — o servidor carimba chunks com o tempo do servidor, cada cliente mantém sincronizado o relógio com o servidor, agenda a reprodução pelo carimbo e corrige deriva tocando um pouco mais rápido/lento (removendo/duplicando amostras); desvio típico < 0,2 ms em LAN, buffer padrão ~1 s. O Kazzu **não** transmite PCM, mas reusa a ideia da linha do tempo. O Metrolist usa um servidor com salas (host/convidados) — só como inspiração do fluxo.

**Modelo líder–seguidor:**
1. **Sincronia de relógio (NTP-like) sobre o canal de controle:** cada seguidor envia sondas com `t1` (envio), líder responde com `t2` (recebido) e `t3` (resposta); seguidor anota `t4`. `RTT = (t4−t1)−(t3−t2)`, `offset = ((t2−t1)+(t3−t4))/2`. Fazer ~8–16 sondas, **escolher as de menor RTT** (descartar *outliers*), manter estimativa de offset, incerteza e deriva (ppm). Usar **relógio monotônico**, nunca o de parede.
2. **Agendar no futuro:** líder define `start_at = t_líder + Δ` (Δ ≈ 300–800 ms, ajustável) para play/seek/troca; todos pré-carregam e iniciam quando o relógio local estimado atinge `start_at`. A latência de iniciar/seek do decodificador torna inviável "tocar agora" — por isso agendar.
3. **Posição esperada contínua:** `pos(t) = pos0 + (t_local + offset − start_at)`. Se o erro `|pos_real − pos_esperada|` > ~80 ms → **seek corretivo**; se pequeno → **reamostragem leve** (±0,05–0,2 %, `rubato`) para absorver deriva sem estalo.
4. **Latência de saída por dispositivo:** compensação manual/automática (Bluetooth tem 100–300 ms); calibração opcional por tap/áudio (o projeto `audiosync` usa microfone) — por usuário, nunca global.
5. **Quem tem a faixa:** casamento por `ISRC`/MBID/`provider+id`; se o convidado não tiver acesso (conta/região/arquivo), mostra "sem acesso" e opcionalmente cai para modo "só controle".
6. **Falhas:** perda do líder → pausar com segurança (nunca corromper a fila); reconexão envia `Snapshot`.
7. **Contagem:** só o host grava histórico/Retrospectiva (ver `sound-and-integrations.md`).

Precisão esperada: LAN: poucos ms; WAN: dezenas de ms (limitada por jitter de rota e relógio de áudio de cada placa). Suficiente para "ouvir junto"; não para multi-room fase-travado (Snapcast já faz isso).

## 5. Limites de licença por provedor (resumo)

| Origem | Retransmitir áudio? | O que pode circular |
|---|---|---|
| Arquivo local próprio | Entre **dispositivos do próprio usuário**: sim. Para amigos: **não recomendado** (direitos autorais) — padrão: só convidar quem já tem o arquivo | `TrackRef`/hash |
| YT Music | **Não** (ToS) | `video_id`, posição |
| Spotify | **Não**; Connect = 1 stream/conta; Soloist/librespot por conta | `spotify:track:` URI, posição |
| Tidal / Qobuz | **Não**; cada ouvinte com a própria assinatura | IDs |

## 6. Segurança

Chaves por dispositivo (iroh NodeId); pareamento explícito; nada de portas abertas expostas; autorização por papel; limites de taxa e tamanho de mensagem; sem metadados em relay além do necessário; logs locais. Desativado por padrão (a regra "sem telemetria" vale: nenhuma conexão sem ação do usuário).

## Decisões que cabem ao autor

1. Relay: oferecer relays públicos de terceiros (n0) como opt-in, exigir relay próprio, ou "só LAN"?
2. "Ouvir junto" com amigos apenas para quem **tem a faixa** (padrão sugerido) — aceitar essa restrição para não retransmitir áudio?
3. CRDT: começar com LWW+HLC (sugerido) ou investir em `yrs`/`automerge` desde o início?

## Fontes (acessadas em 2026-11-21)

- iroh 1.0: https://www.heise.de/en/news/Iroh-1-0-turns-the-internet-into-a-secure-localhost-11337046.html ; https://docs.iroh.computer/about/faq ; https://www.techtimes.com/articles/318490/20260616/peer-peer-library-iroh-10-ships-dial-devices-key-not-ip-address.htm ; https://pinggy.io/blog/iroh_1_0_dial_keys_not_ips/
- Snapcast: https://github.com/snapcast/snapcast ; `snapcast-rs`: https://github.com/metaneutrons/snapcast-rs ; `audiosync`: https://github.com/welchjrmxshtr/audiosync
- Desenho NTP-like para multi-celular: https://github.com/namidaco/namida/issues/1134 ; abordagem SNTP antiga: https://snarfed.org/synchronizing_mp3_playback
- crates.io API: iroh, mdns-sd, quinn, libp2p, webrtc, matchbox_socket, sntpc, yrs, automerge.
- Metrolist (só conceito): recurso "Listen Together" em `.refs/metrolist/README.md`.
