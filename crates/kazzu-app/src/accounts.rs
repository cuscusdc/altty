//! Read-only account imports. Bearer tokens are used only for the fixed API host
//! and live only for the duration of the import.
use std::time::Duration;

use kazzu_core::{AlbumRef, ArtistRef, CoverRef, ProviderKind, Track, TrackSource};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Import {
    pub key: String,
    pub name: String,
    pub tracks: Vec<Track>,
}

struct Client<'a> {
    http: ureq::Agent,
    token: &'a str,
    base: &'static str,
}

impl Client<'_> {
    fn get(&self, path: &str) -> Result<Value, String> {
        let mut response = self.http.get(format!("{}{path}", self.base))
            .header("Authorization", format!("Bearer {}", self.token))
            .call().map_err(|_| "Não foi possível alcançar a API.".to_string())?;
        match response.status().as_u16() {
            200 => {}
            401 => return Err("Token inválido ou expirado.".into()),
            403 => return Err("A conta ou o aplicativo não tem permissão para esta operação.".into()),
            429 => return Err("Limite de requisições atingido. Tente novamente mais tarde.".into()),
            status => return Err(format!("A API respondeu com HTTP {status}.")),
        }
        let bytes = response.body_mut().with_config().limit(8 * 1024 * 1024).read_to_vec()
            .map_err(|_| "Não foi possível ler a resposta da API.".to_string())?;
        serde_json::from_slice(&bytes).map_err(|_| "Resposta da API inválida.".into())
    }

    fn pages(&self, path: &str, google: bool) -> Result<Vec<Value>, String> {
        let mut result = Vec::new();
        let mut page = path.to_owned();
        for _ in 0..200 {
            let response = self.get(&page)?;
            let items = response["items"].as_array().ok_or("A API não retornou a lista de itens.")?;
            result.extend(items.iter().cloned());
            if google {
                let Some(token) = response["nextPageToken"].as_str() else { return Ok(result); };
                let escaped = token.bytes().map(|b| format!("%{b:02X}")).collect::<String>();
                page = format!("{path}&pageToken={escaped}");
            } else {
                let Some(next) = response["next"].as_str() else { return Ok(result); };
                page = next.strip_prefix(self.base).filter(|rest| rest.starts_with('/'))
                    .ok_or("URL de paginação inesperada.")?.to_owned();
            }
        }
        Err("A coleção ultrapassa o limite desta importação.".into())
    }
}

pub fn fetch(provider: &str, token: &str) -> Result<(Vec<Import>, usize), String> {
    let google = match provider {
        "spotify" => false,
        "youtube" => true,
        _ => return Err("Provedor desconhecido.".into()),
    };
    let client = Client {
        http: ureq::Agent::new_with_config(ureq::Agent::config_builder()
            .http_status_as_error(false).max_redirects(0).https_only(true)
            .timeout_global(Some(Duration::from_secs(25))).build()),
        token,
        base: if google { "https://www.googleapis.com/youtube/v3" } else { "https://api.spotify.com/v1" },
    };
    let mut imports = Vec::new();
    let mut skipped = 0;
    if !google {
        let account = client.get("/me")?;
        let account_id = id(&account["id"]).ok_or("A API não identificou a conta.")?;
        let saved = client.pages("/me/tracks?limit=50", false)?;
        imports.push(Import {
            key: format!("spotify:saved:{account_id}"),
            name: "Spotify · Músicas salvas".into(),
            tracks: saved.iter().filter_map(spotify_track).collect(),
        });
    }
    let playlists = client.pages(if google { "/playlists?part=snippet&mine=true&maxResults=50" } else { "/me/playlists?limit=50" }, google)?;
    for playlist in playlists {
        let Some(playlist_id) = id(&playlist["id"]) else { continue; };
        let path = if google {
            format!("/playlistItems?part=snippet,contentDetails&playlistId={playlist_id}&maxResults=50")
        } else {
            format!("/playlists/{playlist_id}/items?limit=50")
        };
        let items = match client.pages(&path, google) {
            Ok(items) => items,
            Err(error) if !google && error.contains("não tem permissão") => { skipped += 1; continue; }
            Err(error) => return Err(error),
        };
        let name = if google { playlist["snippet"]["title"].as_str() } else { playlist["name"].as_str() }.unwrap_or("Playlist");
        imports.push(Import {
            key: format!("{provider}:playlist:{playlist_id}"),
            name: format!("{} · {name}", if google { "YouTube" } else { "Spotify" }),
            tracks: items.iter().filter_map(if google { youtube_track } else { spotify_track }).collect(),
        });
    }
    Ok((imports, skipped))
}

fn id(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
}

fn spotify_track(row: &Value) -> Option<Track> {
    let item = row.get("item").or_else(|| row.get("track"))?;
    if item["type"].as_str() != Some("track") || item["is_local"].as_bool() == Some(true) { return None; }
    let id = id(&item["id"])?;
    let mut track = Track::new(id, item["name"].as_str()?, TrackSource::Remote { provider: ProviderKind::Spotify, id: id.into() });
    track.artists = item["artists"].as_array().into_iter().flatten()
        .filter_map(|a| a["name"].as_str().map(ArtistRef::named)).collect();
    track.album = item["album"]["name"].as_str().map(|title| AlbumRef { id: None, title: title.into() });
    track.duration = item["duration_ms"].as_u64().map(Duration::from_millis);
    track.cover = item["album"]["images"][0]["url"].as_str().map(|url| CoverRef::Url { url: url.into() });
    Some(track)
}

fn youtube_track(row: &Value) -> Option<Track> {
    let id = id(&row["contentDetails"]["videoId"])?;
    let snippet = &row["snippet"];
    let title = snippet["title"].as_str()?;
    let artist = snippet["videoOwnerChannelTitle"].as_str()?;
    let mut track = Track::new(id, title, TrackSource::Remote { provider: ProviderKind::YtMusic, id: id.into() });
    track.artists.push(ArtistRef::named(artist));
    track.cover = snippet["thumbnails"]["medium"]["url"].as_str().map(|url| CoverRef::Url { url: url.into() });
    Some(track)
}
