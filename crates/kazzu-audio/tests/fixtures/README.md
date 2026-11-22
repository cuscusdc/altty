# Fixtures de formato

Senos de 6 s gerados por nós com o ffmpeg (sem conteúdo de terceiros), para cobrir os contêineres que os provedores de streaming servem (YouTube Music: itag 251 = WebM/Opus, itag 140 = MP4/AAC fragmentado, estilo DASH).

```sh
ffmpeg -f lavfi -i "sine=frequency=440:sample_rate=48000:duration=6" -ac 2 -c:a libopus -b:a 24k -f webm sine_opus.webm
ffmpeg -f lavfi -i "sine=frequency=660:sample_rate=44100:duration=6" -ac 2 -c:a aac -b:a 32k \
  -movflags +frag_keyframe+empty_moov+default_base_moof+dash -frag_duration 1000000 sine_aac_frag.m4a
ffmpeg -f lavfi -i "sine=frequency=660:sample_rate=44100:duration=6" -ac 2 -c:a aac -b:a 32k -movflags +faststart sine_aac.m4a
```

Os testes de PCM/FLAC continuam gerando os arquivos em tempo de teste (`tests/engine.rs`, `tests/streaming.rs`); estes três ficam versionados porque não há codificador Opus/AAC em Rust puro para gerá-los no teste.
