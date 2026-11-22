//! `player` responses: playability classification and audio format choice.

use std::time::{Duration, SystemTime};

use kazzu_core::{AudioFormat, Codec, StreamQuality};
use serde_json::Value;

use crate::error::{Result, YtmError};
use crate::json::{arr_at, at, num, str_at, text_at};
use crate::model::ResolvedStream;

/// One adaptive audio format of a `player` response.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AudioFmt {
    pub itag: u32,
    pub mime: String,
    pub codec: Codec,
    /// Average bitrate when known, else peak, in bit/s.
    pub bitrate: u64,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub content_length: Option<u64>,
    pub url: Option<String>,
    /// `signatureCipher` instead of `url` (needs the JS player: unsupported).
    pub ciphered: bool,
    /// Dynamic-range-compressed variant (never chosen: not original quality).
    pub drc: bool,
    /// `Some(false)` for a dubbed / secondary audio track.
    pub default_track: Option<bool>,
    pub loudness_db: Option<f32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlayerData {
    pub formats: Vec<AudioFmt>,
    pub expires_in: Option<u64>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub length_secs: Option<u64>,
}

fn codec_of(mime: &str) -> Codec {
    let m = mime.to_ascii_lowercase();
    if m.contains("opus") {
        Codec::Opus
    } else if m.contains("mp4a") {
        Codec::Aac
    } else if m.contains("vorbis") {
        Codec::Vorbis
    } else if m.contains("ec-3") || m.contains("ac-3") {
        Codec::Other("Dolby".into())
    } else {
        Codec::Other(m)
    }
}

/// Maps `playabilityStatus` to `Ok` or a typed error. Reasons are English
/// because `player` requests use `hl=en`.
pub(crate) fn check_playability(resp: &Value) -> Result<()> {
    let ps = resp
        .get("playabilityStatus")
        .ok_or_else(|| YtmError::shape("playabilityStatus"))?;
    let status = ps.get("status").and_then(Value::as_str).unwrap_or("");
    let reason = ps
        .get("reason")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| text_at(ps, &["errorScreen", "playerErrorMessageRenderer", "reason"]))
        .unwrap_or_default();
    let sub = text_at(ps, &["errorScreen", "playerErrorMessageRenderer", "subreason"]).unwrap_or_default();
    let all = format!("{reason} {sub}").to_lowercase();
    let msg = if sub.is_empty() { reason.clone() } else { format!("{reason} ({sub})") };
    let client_problem = all.contains("no longer supported")
        || all.contains("not supported")
        || all.contains("needs to be reloaded")
        || all.contains("update");
    match status {
        "OK" => Ok(()),
        "LOGIN_REQUIRED" if all.contains("age") => Err(YtmError::AgeRestricted(msg)),
        "LOGIN_REQUIRED" if all.contains("private") => Err(YtmError::Unavailable(msg)),
        "LOGIN_REQUIRED" => Err(YtmError::Blocked(msg)),
        "UNPLAYABLE" | "ERROR" if client_problem => Err(YtmError::UpstreamChanged(msg)),
        "UNPLAYABLE" | "ERROR" if all.contains("country") || all.contains("region") => {
            Err(YtmError::RegionBlocked(msg))
        }
        "UNPLAYABLE" if all.contains("age") => Err(YtmError::AgeRestricted(msg)),
        "UNPLAYABLE" | "ERROR" | "LIVE_STREAM_OFFLINE" => Err(YtmError::Unavailable(msg)),
        "CONTENT_CHECK_REQUIRED" | "AGE_CHECK_REQUIRED" | "AGE_VERIFICATION_REQUIRED" => {
            Err(YtmError::AgeRestricted(msg))
        }
        other => Err(YtmError::UpstreamChanged(format!("playability status {other:?}: {msg}"))),
    }
}

pub(crate) fn parse_player(resp: &Value) -> Result<PlayerData> {
    check_playability(resp)?;
    let sd = resp
        .get("streamingData")
        .ok_or_else(|| YtmError::NoStream("no streamingData".into()))?;
    let formats = arr_at(sd, &["adaptiveFormats"])
        .iter()
        .filter_map(|f| {
            let mime = f.get("mimeType")?.as_str()?;
            if !mime.starts_with("audio/") {
                return None;
            }
            Some(AudioFmt {
                itag: num(f.get("itag"))? as u32,
                mime: mime.to_owned(),
                codec: codec_of(mime),
                bitrate: num(f.get("averageBitrate")).or_else(|| num(f.get("bitrate"))).unwrap_or(0),
                sample_rate: num(f.get("audioSampleRate")).map(|r| r as u32),
                channels: num(f.get("audioChannels")).map(|c| c as u16),
                content_length: num(f.get("contentLength")),
                url: f.get("url").and_then(Value::as_str).map(str::to_owned),
                ciphered: f.get("signatureCipher").is_some() || f.get("cipher").is_some(),
                drc: f.get("isDrc").and_then(Value::as_bool).unwrap_or(false)
                    || f.get("xtags").and_then(Value::as_str).is_some_and(|x| x.contains("drc")),
                default_track: at(f, &["audioTrack", "audioIsDefault"]).and_then(Value::as_bool),
                loudness_db: f.get("loudnessDb").and_then(Value::as_f64).map(|x| x as f32),
                duration_ms: num(f.get("approxDurationMs")),
            })
        })
        .collect();
    let vd = resp.get("videoDetails");
    Ok(PlayerData {
        formats,
        expires_in: num(sd.get("expiresInSeconds")),
        title: vd.and_then(|v| str_at(v, &["title"])).map(str::to_owned),
        author: vd.and_then(|v| str_at(v, &["author"])).map(str::to_owned),
        length_secs: vd.and_then(|v| num(v.get("lengthSeconds"))),
    })
}

fn usable(f: &AudioFmt) -> bool {
    f.url.is_some() && !f.drc && f.default_track != Some(false)
}

/// Best audio format for `quality`. Prefers Opus (WebM, itag 251 ≈ 130–160
/// kbps) over AAC (M4A, itag 140 ≈ 128 kbps) unless the AAC one is much
/// richer (e.g. a 256 kbps AAC next to a 128 kbps Opus).
pub(crate) fn select_format(formats: &[AudioFmt], quality: StreamQuality) -> Option<&AudioFmt> {
    let usable: Vec<&AudioFmt> = formats.iter().filter(|f| usable(f)).collect();
    let best_of = |codec: &Codec| {
        usable
            .iter()
            .filter(|f| &f.codec == codec)
            .max_by_key(|f| f.bitrate)
            .copied()
    };
    match quality {
        StreamQuality::Low => {
            let opus_low = usable
                .iter()
                .filter(|f| f.codec == Codec::Opus)
                .min_by_key(|f| f.bitrate)
                .copied();
            opus_low.or_else(|| usable.iter().min_by_key(|f| f.bitrate).copied())
        }
        _ => {
            let opus = best_of(&Codec::Opus);
            let aac = best_of(&Codec::Aac);
            match (opus, aac) {
                (Some(o), Some(a)) if a.bitrate > o.bitrate * 3 / 2 => Some(a),
                (Some(o), _) => Some(o),
                (None, Some(a)) => Some(a),
                (None, None) => usable.iter().max_by_key(|f| f.bitrate).copied(),
            }
        }
    }
}

/// `expire=<unix>` query parameter of a googlevideo URL.
pub(crate) fn url_expiry(url: &str) -> Option<SystemTime> {
    let query = url.split_once('?')?.1;
    let secs: u64 = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("expire="))?
        .parse()
        .ok()?;
    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
}

pub(crate) fn to_resolved(
    video_id: &str,
    data: &PlayerData,
    fmt: &AudioFmt,
    client: &str,
    user_agent: &str,
    now: SystemTime,
) -> Option<ResolvedStream> {
    let url = fmt.url.clone()?;
    let expires_at = url_expiry(&url)
        .or_else(|| data.expires_in.map(|s| now + Duration::from_secs(s)))
        .unwrap_or(now + Duration::from_secs(5 * 3600));
    Some(ResolvedStream {
        video_id: video_id.to_owned(),
        headers: vec![("User-Agent".into(), user_agent.to_owned())],
        format: AudioFormat {
            codec: fmt.codec.clone(),
            sample_rate: fmt.sample_rate,
            bit_depth: None,
            channels: fmt.channels,
            bitrate_kbps: Some(((fmt.bitrate + 500) / 1000) as u32).filter(|b| *b > 0),
        },
        mime: fmt.mime.clone(),
        itag: fmt.itag,
        content_length: fmt.content_length,
        duration: fmt
            .duration_ms
            .map(Duration::from_millis)
            .or_else(|| data.length_secs.map(Duration::from_secs)),
        expires_at,
        client: client.to_owned(),
        loudness_db: fmt.loudness_db,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(itag: u32, mime: &str, bitrate: u64) -> AudioFmt {
        AudioFmt {
            itag,
            mime: mime.into(),
            codec: codec_of(mime),
            bitrate,
            sample_rate: Some(48_000),
            channels: Some(2),
            content_length: None,
            url: Some(format!("https://x/videoplayback?expire=2000000000&itag={itag}")),
            ciphered: false,
            drc: false,
            default_track: None,
            loudness_db: None,
            duration_ms: None,
        }
    }

    const OPUS: &str = "audio/webm; codecs=\"opus\"";
    const AAC: &str = "audio/mp4; codecs=\"mp4a.40.2\"";

    #[test]
    fn prefers_opus_and_skips_drc() {
        let mut drc = fmt(251, OPUS, 140_000);
        drc.drc = true;
        let formats = vec![fmt(140, AAC, 130_000), fmt(249, OPUS, 50_000), fmt(251, OPUS, 120_000), drc];
        assert_eq!(select_format(&formats, StreamQuality::Original).unwrap().itag, 251);
        assert_eq!(select_format(&formats, StreamQuality::High).unwrap().itag, 251);
        assert_eq!(select_format(&formats, StreamQuality::Low).unwrap().itag, 249);
    }

    #[test]
    fn falls_back_to_aac_and_respects_much_better_aac() {
        assert_eq!(select_format(&[fmt(140, AAC, 130_000)], StreamQuality::Original).unwrap().itag, 140);
        let rich = vec![fmt(141, AAC, 256_000), fmt(251, OPUS, 120_000)];
        assert_eq!(select_format(&rich, StreamQuality::Original).unwrap().itag, 141);
        let mut ciphered = fmt(251, OPUS, 120_000);
        ciphered.url = None;
        assert!(select_format(&[ciphered], StreamQuality::Original).is_none());
    }

    #[test]
    fn dubbed_tracks_are_ignored() {
        let mut dub = fmt(251, OPUS, 160_000);
        dub.default_track = Some(false);
        let mut orig = fmt(250, OPUS, 70_000);
        orig.default_track = Some(true);
        assert_eq!(select_format(&[dub, orig], StreamQuality::Original).unwrap().itag, 250);
    }

    #[test]
    fn expiry_from_url() {
        let t = url_expiry("https://x/videoplayback?a=1&expire=1795346788&b=2").unwrap();
        assert_eq!(t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), 1_795_346_788);
        assert!(url_expiry("https://x/").is_none());
    }
}
