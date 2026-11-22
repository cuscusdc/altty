//! Interface language (`hl`) and content country (`gl`) sent to YouTube.
//!
//! Derived from the system locale; never a fixed `en`/`US`. When the system
//! gives no country, `gl` is left out and YouTube picks the country from the
//! connection (same as an anonymous browser).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Locale {
    /// YouTube interface language, e.g. `pt`, `pt-PT`, `en-GB`, `es-419`.
    pub hl: Option<String>,
    /// ISO 3166-1 alpha-2 country, e.g. `BR`. Drives charts and the home feed.
    pub gl: Option<String>,
}

impl Locale {
    pub fn new(hl: impl Into<String>, gl: impl Into<String>) -> Self {
        Self {
            hl: Some(hl.into()),
            gl: Some(gl.into()),
        }
    }

    /// Locale of the operating system (Android, iOS, macOS, Windows, Linux),
    /// falling back to `LC_ALL`/`LC_MESSAGES`/`LANG`, then to "let YouTube
    /// decide".
    pub fn system() -> Self {
        sys_locale::get_locale()
            .and_then(|tag| Self::from_tag(&tag))
            .or_else(|| {
                ["LC_ALL", "LC_MESSAGES", "LANG"]
                    .iter()
                    .filter_map(|var| std::env::var(var).ok())
                    .find_map(|tag| Self::from_tag(&tag))
            })
            .unwrap_or_default()
    }

    /// Parses a BCP 47 tag (`pt-BR`, `zh-Hant-TW`, `es-419`) or a POSIX locale
    /// (`pt_BR.UTF-8`, `sr_RS@latin`). Returns `None` for `C`/`POSIX`/empty.
    pub fn from_tag(tag: &str) -> Option<Self> {
        let tag = tag.split(['.', '@']).next().unwrap_or("").trim();
        if tag.is_empty() || tag.eq_ignore_ascii_case("c") || tag.eq_ignore_ascii_case("posix") {
            return None;
        }
        let mut parts = tag.split(['-', '_']);
        let lang = parts.next()?.to_ascii_lowercase();
        if !(2..=3).contains(&lang.len()) || !lang.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        let mut script: Option<String> = None;
        let mut region: Option<String> = None;
        for part in parts {
            if part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) && script.is_none() {
                script = Some(part.to_ascii_lowercase());
            } else if region.is_none()
                && ((part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()))
                    || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit())))
            {
                region = Some(part.to_ascii_uppercase());
            }
        }
        let country = region
            .as_deref()
            .filter(|r| r.len() == 2)
            .map(str::to_owned);
        Some(Self {
            hl: Some(youtube_hl(&lang, script.as_deref(), region.as_deref())),
            gl: country,
        })
    }

    /// `true` when neither language nor country is known.
    pub fn is_unknown(&self) -> bool {
        self.hl.is_none() && self.gl.is_none()
    }

    /// Short form for cache keys, e.g. `pt.BR`.
    pub(crate) fn key(&self) -> String {
        format!(
            "{}.{}",
            self.hl.as_deref().unwrap_or("-"),
            self.gl.as_deref().unwrap_or("-")
        )
    }
}

/// Maps a language/script/region to the `hl` values YouTube understands.
fn youtube_hl(lang: &str, script: Option<&str>, region: Option<&str>) -> String {
    let region = region.unwrap_or("");
    match lang {
        "pt" => match region {
            "" | "BR" => "pt".into(),
            _ => "pt-PT".into(),
        },
        "en" => match region {
            "GB" | "IE" | "AU" | "NZ" | "ZA" | "SG" | "MT" => "en-GB".into(),
            "IN" => "en-IN".into(),
            _ => "en".into(),
        },
        "es" => match region {
            "" | "ES" => "es".into(),
            "US" => "es-US".into(),
            _ => "es-419".into(),
        },
        "fr" if region == "CA" => "fr-CA".into(),
        "zh" => {
            if script == Some("hant") || region == "TW" || region == "MO" {
                "zh-TW".into()
            } else if region == "HK" {
                "zh-HK".into()
            } else {
                "zh-CN".into()
            }
        }
        "sr" if script == Some("latn") => "sr-Latn".into(),
        "he" => "iw".into(),
        "nb" | "nn" => "no".into(),
        "tl" => "fil".into(),
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(tag: &str) -> (Option<String>, Option<String>) {
        let loc = Locale::from_tag(tag).unwrap_or_default();
        (loc.hl, loc.gl)
    }

    #[test]
    fn parses_bcp47_and_posix() {
        assert_eq!(l("pt-BR"), (Some("pt".into()), Some("BR".into())));
        assert_eq!(l("pt_BR.UTF-8"), (Some("pt".into()), Some("BR".into())));
        assert_eq!(l("pt-PT"), (Some("pt-PT".into()), Some("PT".into())));
        assert_eq!(l("en-GB"), (Some("en-GB".into()), Some("GB".into())));
        assert_eq!(l("es-MX"), (Some("es-419".into()), Some("MX".into())));
        assert_eq!(l("es-419"), (Some("es-419".into()), None));
        assert_eq!(l("zh-Hant-TW"), (Some("zh-TW".into()), Some("TW".into())));
        assert_eq!(l("sr_RS@latin"), (Some("sr".into()), Some("RS".into())));
        assert_eq!(l("de"), (Some("de".into()), None));
        assert_eq!(l("he-IL"), (Some("iw".into()), Some("IL".into())));
    }

    #[test]
    fn rejects_c_locale() {
        assert!(Locale::from_tag("C").is_none());
        assert!(Locale::from_tag("POSIX").is_none());
        assert!(Locale::from_tag("C.UTF-8").is_none());
        assert!(Locale::from_tag("").is_none());
        assert!(Locale::default().is_unknown());
    }
}
