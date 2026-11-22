//! Rótulos em pt-BR para a UI: durações, contagens, selos de formato.
//! Convenção única do app: sinal de menos real "−" (U+2212) nos tempos restantes.

use std::time::Duration;

/// "3:21", ou "1:02:03" a partir de uma hora.
pub fn duration_label(d: Duration) -> String {
    let secs = d.as_secs();
    let (h, m, s) = (secs / 3600, (secs / 60) % 60, secs % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

pub fn duration_label_ms(ms: u64) -> String {
    duration_label(Duration::from_millis(ms))
}

/// "−2:19": o que falta. Nunca negativo.
pub fn remaining_label(position: Duration, duration: Option<Duration>) -> String {
    match duration {
        Some(d) => format!("−{}", duration_label(d.saturating_sub(position))),
        None => "−0:00".to_owned(),
    }
}

/// Separador de milhar pt-BR: 1284 -> "1.284".
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// "1 música", "214 músicas".
pub fn count_label(n: u64, singular: &str, plural: &str) -> String {
    format!("{} {}", thousands(n), if n == 1 { singular } else { plural })
}

pub fn songs_label(n: u64) -> String {
    count_label(n, "música", "músicas")
}

pub fn albums_label(n: u64) -> String {
    count_label(n, "álbum", "álbuns")
}

pub fn playlists_label(n: u64) -> String {
    count_label(n, "playlist", "playlists")
}

/// "3 pastas, 1.284 músicas".
pub fn library_summary(folders: usize, tracks: u64) -> String {
    format!("{}, {}", count_label(folders as u64, "pasta", "pastas"), songs_label(tracks))
}

/// "A seguir, 6 músicas, 24 min".
pub fn queue_summary(count: usize, total: Duration) -> String {
    let mins = total.as_secs().div_ceil(60);
    format!("A seguir, {}, {} min", songs_label(count as u64), thousands(mins))
}

/// Taxa de amostragem em kHz com vírgula decimal: 44100 -> "44,1", 96000 -> "96".
pub fn khz(rate: u32) -> String {
    let whole = rate / 1000;
    let tenth = (rate % 1000) / 100;
    if rate.is_multiple_of(1000) {
        whole.to_string()
    } else if rate.is_multiple_of(100) {
        format!("{whole},{tenth}")
    } else {
        // taxas incomuns (ex.: 11025): duas casas, sem zeros à direita
        let s = format!("{:.2}", f64::from(rate) / 1000.0).replace('.', ",");
        s.trim_end_matches('0').trim_end_matches(',').to_owned()
    }
}

/// Dados do selo de formato.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Badge {
    /// "FLAC", "MP3", "OPUS"; vazio = sem selo.
    pub codec: String,
    /// Curto, para listas: "24/96", "16/44,1", "320".
    pub detail: String,
    /// Longo, para o player: "24-bit/96 kHz", "320 kbps".
    pub detail_long: String,
    /// Profundidade >= 24 bits (usa o acento).
    pub hires: bool,
}

/// Monta o selo. `lossless` decide entre "bits/kHz" e "kbps".
pub fn badge(codec: &str, lossless: bool, bit_depth: Option<u8>, sample_rate: Option<u32>, bitrate_kbps: Option<u32>) -> Badge {
    let codec = codec.trim();
    if codec.is_empty() {
        return Badge::default();
    }
    let hires = lossless && bit_depth.is_some_and(|b| b >= 24);
    let (detail, detail_long) = match (lossless, bit_depth, sample_rate, bitrate_kbps) {
        (true, Some(b), Some(r), _) => (format!("{b}/{}", khz(r)), format!("{b}-bit/{} kHz", khz(r))),
        (true, None, Some(r), _) => (khz(r), format!("{} kHz", khz(r))),
        (true, Some(b), None, _) => (b.to_string(), format!("{b}-bit")),
        (_, _, _, Some(k)) => (k.to_string(), format!("{k} kbps")),
        (false, _, Some(r), None) => (khz(r), format!("{} kHz", khz(r))),
        _ => (String::new(), String::new()),
    };
    Badge { codec: codec.to_uppercase(), detail, detail_long, hires }
}

/// Selo de qualidade em texto: "OPUS 160 kbps", "AAC 128 kbps", "FLAC 24/96".
pub fn quality_label(f: &kazzu_core::AudioFormat) -> String {
    let b = badge(&f.codec.to_string(), f.codec.is_lossless(), f.bit_depth, f.sample_rate, f.bitrate_kbps);
    if b.codec.is_empty() {
        String::new()
    } else if b.detail_long.is_empty() {
        b.codec
    } else if f.codec.is_lossless() {
        format!("{} {}", b.codec, b.detail)
    } else {
        format!("{} {}", b.codec, b.detail_long)
    }
}

/// O motor chama WAV/AIFF/CAF de "PCM"; ao usuário interessa o nome do contêiner.
pub fn pcm_label(path: Option<&std::path::Path>) -> &'static str {
    match path.and_then(|p| p.extension()).and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("wav" | "wave") => "WAV",
        Some("aif" | "aiff" | "aifc") => "AIFF",
        Some("caf") => "CAF",
        _ => "PCM",
    }
}

/// Codecs sem perda, pelo nome que a biblioteca grava.
pub fn is_lossless_name(codec: &str) -> bool {
    matches!(codec.to_ascii_uppercase().as_str(), "FLAC" | "ALAC" | "WAV" | "AIFF" | "PCM" | "CAF")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration_label_ms(201_000), "3:21");
        assert_eq!(duration_label_ms(3_723_000), "1:02:03");
        assert_eq!(duration_label_ms(0), "0:00");
        assert_eq!(remaining_label(Duration::from_secs(62), Some(Duration::from_secs(201))), "−2:19");
        assert_eq!(remaining_label(Duration::from_secs(300), Some(Duration::from_secs(201))), "−0:00");
        assert_eq!(remaining_label(Duration::ZERO, None), "−0:00");
    }

    #[test]
    fn counts_use_portuguese_plurals_and_dots() {
        assert_eq!(songs_label(1), "1 música");
        assert_eq!(songs_label(214), "214 músicas");
        assert_eq!(songs_label(1284), "1.284 músicas");
        assert_eq!(albums_label(0), "0 álbuns");
        assert_eq!(library_summary(3, 1284), "3 pastas, 1.284 músicas");
        assert_eq!(library_summary(1, 1), "1 pasta, 1 música");
        assert_eq!(queue_summary(6, Duration::from_secs(23 * 60 + 20)), "A seguir, 6 músicas, 24 min");
    }

    #[test]
    fn khz_formats() {
        assert_eq!(khz(44_100), "44,1");
        assert_eq!(khz(48_000), "48");
        assert_eq!(khz(96_000), "96");
        assert_eq!(khz(88_200), "88,2");
        assert_eq!(khz(192_000), "192");
        assert_eq!(khz(11_025), "11,03");
    }

    #[test]
    fn pcm_container_names() {
        use std::path::Path;
        assert_eq!(pcm_label(Some(Path::new("a/b.WAV"))), "WAV");
        assert_eq!(pcm_label(Some(Path::new("x.aiff"))), "AIFF");
        assert_eq!(pcm_label(Some(Path::new("x.bin"))), "PCM");
        assert_eq!(pcm_label(None), "PCM");
    }

    #[test]
    fn quality_labels() {
        use kazzu_core::{AudioFormat, Codec};
        let mut f = AudioFormat::new(Codec::Opus);
        f.bitrate_kbps = Some(160);
        assert_eq!(quality_label(&f), "OPUS 160 kbps");
        let mut f = AudioFormat::new(Codec::Flac);
        (f.bit_depth, f.sample_rate) = (Some(24), Some(96_000));
        assert_eq!(quality_label(&f), "FLAC 24/96");
        assert_eq!(quality_label(&AudioFormat::new(Codec::Aac)), "AAC");
    }

    #[test]
    fn badges() {
        let b = badge("flac", true, Some(24), Some(96_000), None);
        assert_eq!((b.codec.as_str(), b.detail.as_str(), b.detail_long.as_str(), b.hires), ("FLAC", "24/96", "24-bit/96 kHz", true));
        let b = badge("FLAC", true, Some(16), Some(44_100), None);
        assert_eq!((b.detail.as_str(), b.hires), ("16/44,1", false));
        let b = badge("MP3", false, None, Some(44_100), Some(320));
        assert_eq!((b.detail.as_str(), b.detail_long.as_str(), b.hires), ("320", "320 kbps", false));
        assert_eq!(badge("", true, None, None, None), Badge::default());
        assert!(is_lossless_name("Flac") && is_lossless_name("WAV") && !is_lossless_name("Opus"));
    }
}
