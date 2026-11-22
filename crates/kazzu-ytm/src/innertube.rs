//! Innertube clients: request contexts, client identities and the session
//! bootstrap (`sw.js_data`).
//!
//! All the values that YouTube may invalidate (client versions, user agents)
//! live in this file. See `docs/research/ytm-notes.md` for how to update them.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::json::strings;
use crate::locale::Locale;

pub(crate) const MUSIC_API: &str = "https://music.youtube.com/youtubei/v1";
pub(crate) const PLAYER_API: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
pub(crate) const SW_JS_DATA: &str = "https://music.youtube.com/sw.js_data";

/// WEB_REMIX (music.youtube.com) — metadata only. Refreshed at runtime from
/// `sw.js_data` when YouTube rejects this one.
pub(crate) const WEB_REMIX_VERSION: &str = "1.20261117.06.00";
pub(crate) const WEB_REMIX_ID: u32 = 67;
pub(crate) const WEB_UA: &str = crate::transport::DEFAULT_UA;

/// Body of a WEB_REMIX request. No visitorData, cookies or user ids: the
/// metadata endpoints answer anonymous requests as-is.
pub(crate) fn web_remix_body(version: &str, locale: &Locale, mut body: Value) -> Value {
    let mut client = json!({
        "clientName": "WEB_REMIX",
        "clientVersion": version,
    });
    if let Some(hl) = &locale.hl {
        client["hl"] = json!(hl);
    }
    if let Some(gl) = &locale.gl {
        client["gl"] = json!(gl);
    }
    body["context"] = json!({ "client": client, "user": {} });
    body
}

/// Innertube clients that can return directly downloadable audio URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerClient {
    /// Apple Vision Pro app. Direct Opus+AAC URLs, no PO token, no cipher.
    VisionOs,
    /// YouTube Studio-like iOS producer app. AAC only, no PO token.
    IosProducer,
    /// Oculus app. Returns URLs but googlevideo cuts them after ~1 MB without
    /// a PO token (2026-11); kept for experiments only.
    AndroidVr,
    /// iOS app. Same ~1 MB limit as AndroidVr (2026-11); experiments only.
    Ios,
}

/// Default fallback chain, best first.
pub const DEFAULT_PLAYER_CLIENTS: [PlayerClient; 2] = [PlayerClient::VisionOs, PlayerClient::IosProducer];

pub(crate) struct ClientSpec {
    pub name: &'static str,
    pub version: &'static str,
    pub id: u32,
    pub user_agent: &'static str,
    pub fields: &'static [(&'static str, &'static str)],
    pub sdk: Option<u32>,
}

impl PlayerClient {
    pub const ALL: [PlayerClient; 4] = [
        PlayerClient::VisionOs,
        PlayerClient::IosProducer,
        PlayerClient::AndroidVr,
        PlayerClient::Ios,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PlayerClient::VisionOs => "visionos",
            PlayerClient::IosProducer => "ios_producer",
            PlayerClient::AndroidVr => "android_vr",
            PlayerClient::Ios => "ios",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name.trim())
    }

    pub(crate) fn spec(self) -> ClientSpec {
        match self {
            PlayerClient::VisionOs => ClientSpec {
                name: "VISIONOS",
                version: "1.02",
                id: 101,
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15",
                fields: &[
                    ("deviceMake", "Apple"),
                    ("deviceModel", "RealityDevice17,1"),
                    ("osName", "visionOS"),
                    ("osVersion", "26.5.23O471"),
                ],
                sdk: None,
            },
            PlayerClient::IosProducer => ClientSpec {
                name: "IOS_PRODUCER",
                version: "0.147.945",
                id: 92,
                user_agent: "com.google.ios.youtubeproducer/0.147.945 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)",
                fields: &[
                    ("deviceMake", "Apple"),
                    ("deviceModel", "iPhone16,2"),
                    ("osName", "iPhone"),
                    ("osVersion", "18.3.2.22D82"),
                ],
                sdk: None,
            },
            PlayerClient::AndroidVr => ClientSpec {
                name: "ANDROID_VR",
                version: "1.65.10",
                id: 28,
                user_agent: "com.google.android.apps.youtube.vr.oculus/1.65.10 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip",
                fields: &[
                    ("deviceMake", "Oculus"),
                    ("deviceModel", "Quest 3"),
                    ("osName", "Android"),
                    ("osVersion", "12L"),
                ],
                sdk: Some(32),
            },
            PlayerClient::Ios => ClientSpec {
                name: "IOS",
                version: "21.26.4",
                id: 5,
                user_agent: "com.google.ios.youtube/21.26.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)",
                fields: &[
                    ("deviceMake", "Apple"),
                    ("deviceModel", "iPhone16,2"),
                    ("osName", "iPhone"),
                    ("osVersion", "18.3.2.22D82"),
                ],
                sdk: None,
            },
        }
    }

    /// Body of a `player` request. `hl` is always `en` so that playability
    /// reasons can be classified; `gl` follows the user's locale.
    pub(crate) fn player_body(self, video_id: &str, visitor_data: &str, locale: &Locale) -> Value {
        let spec = self.spec();
        let mut client = json!({
            "clientName": spec.name,
            "clientVersion": spec.version,
            "hl": "en",
            "visitorData": visitor_data,
            "userAgent": spec.user_agent,
        });
        for (k, v) in spec.fields {
            client[*k] = json!(v);
        }
        if let Some(sdk) = spec.sdk {
            client["androidSdkVersion"] = json!(sdk);
        }
        if let Some(gl) = &locale.gl {
            client["gl"] = json!(gl);
        }
        json!({
            "context": { "client": client, "user": {} },
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true,
        })
    }
}

/// What `sw.js_data` gives us: an anonymous visitor token and the current
/// WEB_REMIX version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Bootstrap {
    pub visitor_data: Option<String>,
    pub web_remix_version: Option<String>,
}

/// Parses `sw.js_data` (`)]}'` + JSON array). Tolerant: scans all strings
/// instead of relying on array positions.
pub(crate) fn parse_bootstrap(body: &str) -> Bootstrap {
    let json = body.trim_start_matches(")]}'").trim();
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return Bootstrap::default();
    };
    let all = strings(&value);
    let visitor_data = all
        .iter()
        .find(|s| looks_like_visitor_data(s))
        .map(|s| (*s).to_owned());
    let web_remix_version = all
        .iter()
        .find(|s| looks_like_web_remix_version(s))
        .map(|s| (*s).to_owned());
    Bootstrap {
        visitor_data,
        web_remix_version,
    }
}

/// visitorData is a base64url protobuf that starts with field 1 (`Cg`).
pub(crate) fn looks_like_visitor_data(s: &str) -> bool {
    s.len() >= 24
        && s.starts_with("Cg")
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '%' | '='))
}

fn looks_like_web_remix_version(s: &str) -> bool {
    // 1.YYYYMMDD.NN.NN
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4
        && parts[0] == "1"
        && parts[1].len() == 8
        && parts[1].starts_with("20")
        && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_parsing() {
        let body = r#")]}'
[["yt.sw.adr",null,[[["pt","BR",null,"2001:db8::1",null,null,null,null,null,null,null,"","","CgtGSVhUVVJFVklTSVQ%3D%3D","Mozilla/5.0","1.20261117.06.00"]]]]]"#;
        let b = parse_bootstrap(body);
        assert_eq!(b.visitor_data.as_deref(), Some("CgtGSVhUVVJFVklTSVQ%3D%3D"));
        assert_eq!(b.web_remix_version.as_deref(), Some("1.20261117.06.00"));
        assert_eq!(parse_bootstrap("garbage"), Bootstrap::default());
    }

    #[test]
    fn player_body_has_no_user_identifiers() {
        let body = PlayerClient::VisionOs.player_body("abc", "CgVISITOR", &Locale::new("pt", "BR"));
        let s = body.to_string();
        assert!(s.contains("\"hl\":\"en\""));
        assert!(s.contains("\"gl\":\"BR\""));
        assert!(!s.contains("cookie"));
        assert_eq!(PlayerClient::parse("ios_producer"), Some(PlayerClient::IosProducer));
    }
}
