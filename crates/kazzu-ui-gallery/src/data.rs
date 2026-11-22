//! Dados falsos e realistas em pt-BR para exercitar a UI inteira.

use std::collections::HashMap;
use std::rc::Rc;

use slint::{Color, Image, ModelRc, SharedString, VecModel};

use crate::art::{self, Kind};
use crate::{Album, Artist, Collection, Highlight, OutputDevice, Playlist, ProviderRow, Track};

pub struct TrackDef {
    pub title: &'static str,
    pub artist: &'static str,
    pub album: &'static str,
    pub kind: Kind,
    pub dur: &'static str,
    pub codec: &'static str,
    pub detail: &'static str,
    pub detail_long: &'static str,
    pub hires: bool,
}

#[allow(clippy::too_many_arguments)]
const fn t(title: &'static str, artist: &'static str, album: &'static str, kind: Kind, dur: &'static str, codec: &'static str, detail: &'static str, detail_long: &'static str, hires: bool) -> TrackDef {
    TrackDef { title, artist, album, kind, dur, codec, detail, detail_long, hires }
}

pub const TRACKS: [TrackDef; 24] = [
    t("Quase Verão", "Nina Sabiá", "Calor de Março", Kind::Sunset, "3:21", "OPUS", "160", "160 kbps", false),
    t("Respiração de Vidro", "Kaito e Vê", "Vidro Líquido", Kind::Waves, "4:02", "OPUS", "160", "160 kbps", false),
    t("Hora do Café", "Doma", "Papel de Seda", Kind::Pale, "2:48", "FLAC", "24/96", "24-bit/96 kHz", true),
    t("Canal Sete", "Os Faróis", "Frequência", Kind::Radar, "5:17", "OPUS", "160", "160 kbps", false),
    t("Madrugada Azul", "Aurora Vento", "Cidade Submersa", Kind::Teal, "3:55", "FLAC", "16/44,1", "16-bit/44,1 kHz", false),
    t("Eletricidade", "Bruma", "Sal e Cinza", Kind::Gray, "3:09", "MP3", "320", "320 kbps", false),
    t("Praia de Inverno", "Ilha Branca", "Papel de Seda", Kind::Pale, "4:31", "OPUS", "160", "160 kbps", false),
    t("Meridiano", "Os Faróis", "Frequência", Kind::Radar, "3:44", "OPUS", "160", "160 kbps", false),
    t("Luz de Neon", "Kaito e Vê", "Vidro Líquido", Kind::Waves, "2:58", "OPUS", "160", "160 kbps", false),
    t("Dobra do Tempo", "Doma", "Energia", Kind::Stripes, "3:12", "FLAC", "24/88,2", "24-bit/88,2 kHz", true),
    t("Cais", "Nina Sabiá", "Calor de Março", Kind::Sunset, "4:20", "OPUS", "160", "160 kbps", false),
    t("Pausa Longa", "Lia Monteiro", "Estrada Amarela", Kind::Yellow, "6:03", "OPUS", "160", "160 kbps", false),
    t("Vento Norte", "Aurora Vento", "Cidade Submersa", Kind::Teal, "3:36", "OPUS", "160", "160 kbps", false),
    t("Rosa dos Ventos", "Maré Rosa", "Rosa Quartzo", Kind::Rose, "3:27", "OPUS", "160", "160 kbps", false),
    t("Sala Escura", "Bruma", "Sal e Cinza", Kind::Gray, "4:44", "MP3", "320", "320 kbps", false),
    t("Noite Inteira", "Lia Monteiro", "Céu de Anil", Kind::Night, "5:01", "OPUS", "160", "160 kbps", false),
    t("Fio de Prumo", "Os Faróis", "Frequência", Kind::Radar, "3:19", "OPUS", "160", "160 kbps", false),
    t("Lado B do Verão", "Nina Sabiá", "Calor de Março", Kind::Sunset, "3:50", "OPUS", "160", "160 kbps", false),
    t("Areia Fina", "Ilha Branca", "Papel de Seda", Kind::Pale, "2:41", "OPUS", "160", "160 kbps", false),
    t("Onda Curta", "Kaito e Vê", "Vidro Líquido", Kind::Waves, "3:33", "OPUS", "160", "160 kbps", false),
    t("Luz Baixa", "Maré Rosa", "Rosa Quartzo", Kind::Rose, "4:08", "OPUS", "160", "160 kbps", false),
    t("Mapa Antigo", "Lia Monteiro", "Estrada Amarela", Kind::Yellow, "3:15", "OPUS", "160", "160 kbps", false),
    t("Lua de Anil", "Lia Monteiro", "Céu de Anil", Kind::Night, "4:57", "OPUS", "160", "160 kbps", false),
    t("Ruído Branco", "Bruma", "Sal e Cinza", Kind::Gray, "2:22", "MP3", "320", "320 kbps", false),
];

const WHY: [&str; 6] = ["Em alta hoje", "Novidade para você", "Para a manhã", "Parecido com Os Faróis", "Você ouviu ontem", "Do seu mix Foco"];

pub fn ss(s: &str) -> SharedString {
    s.into()
}

pub fn model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

pub fn color(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb_u8(r, g, b)
}

pub struct World {
    pub thumbs: HashMap<Kind, Image>,
}

impl World {
    pub fn new() -> Self {
        Self { thumbs: art::ALL_KINDS.iter().map(|k| (*k, art::image(*k, 320))).collect() }
    }

    pub fn thumb(&self, k: Kind) -> Image {
        self.thumbs[&k].clone()
    }

    pub fn track(&self, i: usize) -> Track {
        let d = &TRACKS[i % TRACKS.len()];
        Track {
            id: format!("t{i}").into(),
            title: ss(d.title),
            artist: ss(d.artist),
            album: ss(d.album),
            cover: self.thumb(d.kind),
            duration_label: ss(d.dur),
            codec: ss(d.codec),
            detail: ss(d.detail),
            detail_long: ss(d.detail_long),
            hires: d.hires,
            source: ss("YouTube Music"),
            liked: i.is_multiple_of(5),
            playing: false,
            why: SharedString::default(),
        }
    }

    pub fn tracks(&self, idx: &[usize]) -> ModelRc<Track> {
        model(idx.iter().map(|i| self.track(*i)).collect())
    }

    pub fn quick_picks(&self) -> ModelRc<Track> {
        model((1..7).map(|i| Track { why: ss(WHY[i - 1]), ..self.track(i + 1) }).collect())
    }

    pub fn trending(&self) -> ModelRc<Track> {
        model((4..11).map(|i| self.track(i)).collect())
    }

    pub fn album(&self, title: &str, artist: &str, k: Kind, id: usize) -> Album {
        Album { id: format!("a{id}").into(), title: ss(title), artist: ss(artist), cover: self.thumb(k) }
    }

    pub fn albums(&self) -> ModelRc<Album> {
        model(vec![
            self.album("Calor de Março", "Nina Sabiá", Kind::Sunset, 0),
            self.album("Vidro Líquido", "Kaito e Vê", Kind::Waves, 1),
            self.album("Papel de Seda", "Ilha Branca", Kind::Pale, 2),
            self.album("Frequência", "Os Faróis", Kind::Radar, 3),
            self.album("Cidade Submersa", "Aurora Vento", Kind::Teal, 4),
            self.album("Sal e Cinza", "Bruma", Kind::Gray, 5),
            self.album("Estrada Amarela", "Lia Monteiro", Kind::Yellow, 6),
            self.album("Rosa Quartzo", "Maré Rosa", Kind::Rose, 7),
        ])
    }

    pub fn new_releases(&self) -> ModelRc<Album> {
        model(vec![
            self.album("Vidro Líquido", "Kaito e Vê", Kind::Waves, 1),
            self.album("Cidade Submersa", "Aurora Vento", Kind::Teal, 4),
            self.album("Papel de Seda", "Ilha Branca", Kind::Pale, 2),
            self.album("Céu de Anil", "Lia Monteiro", Kind::Night, 8),
            self.album("Rosa Quartzo", "Maré Rosa", Kind::Rose, 7),
        ])
    }

    pub fn hl(&self, id: &str, kind: &str, caption: &str, title: &str, sub: &str, k: Kind) -> Highlight {
        Highlight { id: ss(id), kind: ss(kind), caption: ss(caption), title: ss(title), subtitle: ss(sub), cover: self.thumb(k), tint: Color::from_argb_u8(0, 0, 0, 0) }
    }

    pub fn highlights(&self) -> ModelRc<Highlight> {
        model(vec![
            self.hl("a0", "album", "Novo no YouTube Music", "Calor de Março", "Nina Sabiá", Kind::Sunset),
            self.hl("m1", "playlist", "Mix para você", "Mix Foco", "Os Faróis, Doma e mais", Kind::Radar),
            self.hl("a1", "album", "Lançamento", "Vidro Líquido", "Kaito e Vê", Kind::Waves),
            self.hl("shuffle", "shuffle", "Para ouvir agora", "Tudo em ordem aleatória", "Do seu acervo", Kind::Night),
        ])
    }

    pub fn mixes(&self) -> ModelRc<Highlight> {
        model(vec![
            self.hl("m1", "playlist", "", "Mix Foco", "Os Faróis, Doma", Kind::Radar),
            self.hl("m2", "playlist", "", "Mix Madrugada", "Aurora Vento, Bruma", Kind::Teal),
            self.hl("m3", "playlist", "", "Mix Verão", "Nina Sabiá, Maré Rosa", Kind::Sunset),
            self.hl("m4", "playlist", "", "Mix Calmaria", "Ilha Branca, Doma", Kind::Pale),
            self.hl("m5", "playlist", "", "Mix Estrada", "Lia Monteiro", Kind::Yellow),
        ])
    }

    pub fn moods(&self) -> ModelRc<Highlight> {
        let mk = |id: &str, title: &str, k: Kind, tint: Option<Color>| {
            let mut h = self.hl(id, "mood", "", title, "", k);
            if let Some(c) = tint {
                h.tint = c;
                h.cover = Image::default();
            }
            h
        };
        model(vec![
            mk("foco", "Foco", Kind::Radar, None),
            mk("madrugada", "Madrugada", Kind::Teal, None),
            mk("energia", "Energia", Kind::Stripes, None),
            mk("verao", "Verão", Kind::Sunset, None),
            mk("calmaria", "Calmaria", Kind::Pale, Some(color(0xd9, 0xa7, 0x9a))),
            mk("estrada", "Estrada", Kind::Yellow, Some(color(0xc9, 0x9a, 0x1b))),
            mk("chuva", "Chuva", Kind::Gray, Some(color(0x55, 0x60, 0x70))),
            mk("festa", "Festa", Kind::Rose, Some(color(0xb8, 0x45, 0x78))),
        ])
    }

    pub fn playlist(&self, id: &str, name: &str, n: usize, kinds: &[Kind]) -> Playlist {
        Playlist { id: ss(id), name: ss(name), count_label: format!("{n} músicas").into(), covers: model(kinds.iter().map(|k| self.thumb(*k)).collect()) }
    }

    pub fn playlists(&self) -> ModelRc<Playlist> {
        model(vec![
            self.playlist("p1", "Para correr", 24, &[Kind::Sunset, Kind::Stripes, Kind::Yellow, Kind::Rose]),
            self.playlist("p2", "Estudo profundo", 41, &[Kind::Radar, Kind::Teal, Kind::Night, Kind::Gray]),
            self.playlist("p3", "Domingo de manhã", 17, &[Kind::Pale, Kind::Pale, Kind::Sunset, Kind::Waves]),
        ])
    }

    pub fn collections(&self, n_albums: usize) -> ModelRc<Collection> {
        model(vec![
            Collection { id: ss("downloads"), title: ss("Baixados"), count_label: ss("0 músicas"), covers: model(vec![]), soon: true },
            Collection { id: ss("liked"), title: ss("Curtidas"), count_label: ss("12 músicas"), covers: model(vec![]), soon: false },
            Collection { id: ss("albums"), title: ss("Álbuns"), count_label: format!("{n_albums} álbuns").into(), covers: model(vec![self.thumb(Kind::Teal), self.thumb(Kind::Stripes), self.thumb(Kind::Radar), self.thumb(Kind::Pale)]), soon: false },
            Collection { id: ss("playlists"), title: ss("Playlists"), count_label: ss("3 playlists"), covers: model(vec![self.thumb(Kind::Sunset), self.thumb(Kind::Waves), self.thumb(Kind::Yellow), self.thumb(Kind::Rose)]), soon: false },
        ])
    }

    pub fn artists(&self) -> ModelRc<Artist> {
        model(vec![
            Artist { id: ss("ar0"), name: ss("Aurora Vento"), subtitle: ss("Artista, 2,1 mi de ouvintes"), cover: self.thumb(Kind::Teal) },
            Artist { id: ss("ar1"), name: ss("Nina Sabiá"), subtitle: ss("Artista, 980 mil de ouvintes"), cover: self.thumb(Kind::Sunset) },
            Artist { id: ss("ar2"), name: ss("Os Faróis"), subtitle: ss("Banda, 1,4 mi de ouvintes"), cover: self.thumb(Kind::Radar) },
        ])
    }

    pub fn search_playlists(&self) -> ModelRc<Highlight> {
        model(vec![
            self.hl("sp1", "playlist", "", "Aurora e companhia", "Playlist, 32 músicas", Kind::Teal),
            self.hl("sp2", "playlist", "", "Vento a favor", "Playlist, 18 músicas", Kind::Sunset),
        ])
    }

    pub fn outputs(&self) -> ModelRc<OutputDevice> {
        model(vec![OutputDevice { id: ss("default"), name: ss("Este celular"), active: true }])
    }

    pub fn providers(&self) -> ModelRc<ProviderRow> {
        model(vec![ProviderRow { id: ss("ytm"), name: ss("YouTube Music"), status_label: ss("Ativo e padrão. Sem login."), connected: true, available: true }])
    }
}

pub fn strings(v: &[&str]) -> ModelRc<SharedString> {
    model(v.iter().map(|s| ss(s)).collect())
}
