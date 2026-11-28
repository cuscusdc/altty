use serde::Deserialize;
#[derive(Clone, Deserialize)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
    #[serde(rename = "pos")]
    pub position: f32,
    #[serde(rename = "artGradient")]
    pub art_gradient: String,
    #[serde(rename = "bgGradient")]
    pub background: String,
    pub lyrics: Vec<String>,
}
#[derive(Default)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<usize>,
}
pub struct Settings {
    pub quality: usize,
    pub crossfade: usize,
    pub toggles: [bool; 4],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            quality: 0,
            crossfade: 2,
            toggles: [true; 4],
        }
    }
}
