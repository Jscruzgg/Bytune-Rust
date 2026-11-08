//! Cliente mínimo de InnerTube (la API interna de YouTube Music).
//!
//! Las respuestas son árboles JSON enormes cuya forma cambia a menudo. En vez de rutas fijas,
//! se buscan los "renderers" que interesan en cualquier punto del árbol (`find_all`), y de cada
//! uno se leen los pocos campos que hacen falta.

use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use crate::login::Session;

const YTM: &str = "https://music.youtube.com/youtubei/v1";
const WEB_REMIX_VERSION: &str = "1.20260101.01.00";
/// Filtro "Canciones" de la búsqueda de YouTube Music.
const SONGS_FILTER: &str = "EgWKAQIIAWoMEA4QChADEAQQCRAF";

#[derive(Debug, Clone, Default)]
pub struct Song {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Segundos (0 si no se sabe).
    pub duration: u32,
    pub thumbnail: String,
}

impl Song {
    /// Portada a un tamaño concreto (las URLs de googleusercontent aceptan =wN-hN).
    pub fn cover(&self, px: u32) -> String {
        match self.thumbnail.rfind('=') {
            Some(i) if self.thumbnail.contains("googleusercontent") => format!("{}=w{px}-h{px}-l90-rj", &self.thumbnail[..i]),
            _ => self.thumbnail.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    hl: String,
    /// Sesión de Google (login.rs); sin ella las peticiones van como anónimo.
    session: Arc<RwLock<Option<Session>>>,
}

impl Client {
    pub fn new(http: reqwest::Client, hl: &str, session: Option<Session>) -> Self {
        Self { http, hl: hl.to_string(), session: Arc::new(RwLock::new(session)) }
    }

    fn context(&self) -> Value {
        json!({ "client": { "clientName": "WEB_REMIX", "clientVersion": WEB_REMIX_VERSION, "hl": self.hl, "gl": "US" } })
    }

    async fn post(&self, endpoint: &str, mut body: Value) -> Result<Value> {
        body["context"] = self.context();
        let mut req = self
            .http
            .post(format!("{YTM}/{endpoint}?prettyPrint=false"))
            .header("Origin", "https://music.youtube.com")
            .header("X-Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/");
        if let Some(s) = self.session.read().unwrap().as_ref() {
            req = req
                .header("Cookie", &s.cookie)
                .header("Authorization", s.authorization())
                .header("X-Goog-AuthUser", "0");
        }
        let res = req.json(&body).send().await?;
        let status = res.status();
        if !status.is_success() {
            bail!("InnerTube {endpoint}: HTTP {status}");
        }
        Ok(res.json().await?)
    }

    pub async fn search_songs(&self, query: &str) -> Result<Vec<Song>> {
        let v = self.post("search", json!({ "query": query, "params": SONGS_FILTER })).await?;
        Ok(find_all(&v, "musicResponsiveListItemRenderer").into_iter().filter_map(song_from_list_item).collect())
    }

    /// Cola de reproducción a partir de una canción (radio de esa canción).
    pub async fn radio(&self, video_id: &str) -> Result<Vec<Song>> {
        let v = self
            .post("next", json!({ "videoId": video_id, "playlistId": format!("RDAMVM{video_id}"), "isAudioOnly": true }))
            .await?;
        Ok(find_all(&v, "playlistPanelVideoRenderer").into_iter().filter_map(song_from_panel).collect())
    }

    /// Letra plana de la página de YouTube Music (respaldo cuando LRCLIB no tiene nada).
    pub async fn page_lyrics(&self, video_id: &str) -> Result<Vec<String>> {
        let next = self.post("next", json!({ "videoId": video_id })).await?;
        let browse_id = find_all(&next, "browseEndpoint")
            .into_iter()
            .filter_map(|b| b.get("browseId").and_then(Value::as_str))
            .find(|id| id.starts_with("MPLY"))
            .ok_or_else(|| anyhow!("sin letra"))?
            .to_string();
        let page = self.post("browse", json!({ "browseId": browse_id })).await?;
        let text = find_all(&page, "musicDescriptionShelfRenderer")
            .first()
            .and_then(|r| r.get("description"))
            .map(runs_text)
            .unwrap_or_default();
        Ok(text.lines().map(str::to_owned).collect())
    }

    pub fn logged_in(&self) -> bool {
        self.session.read().unwrap().is_some()
    }

    pub fn set_session(&self, s: Option<Session>) {
        *self.session.write().unwrap() = s;
    }
}

// ---------- lectura del JSON ----------

/// Todos los objetos guardados bajo la clave `key`, en cualquier nivel del árbol.
pub fn find_all<'a>(v: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut out = Vec::new();
    let mut stack = vec![v];
    while let Some(v) = stack.pop() {
        match v {
            Value::Object(map) => {
                for (k, child) in map {
                    if k == key {
                        out.push(child);
                    } else {
                        stack.push(child);
                    }
                }
            }
            Value::Array(items) => stack.extend(items.iter().rev()),
            _ => {}
        }
    }
    // La pila invierte el orden de los hermanos de un objeto; para listas se respeta el orden.
    out
}

fn runs(v: &Value) -> Vec<&str> {
    v.get("runs")
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(|x| x.get("text").and_then(Value::as_str)).collect())
        .unwrap_or_default()
}

fn runs_text(v: &Value) -> String {
    if let Some(s) = v.get("simpleText").and_then(Value::as_str) {
        return s.to_string();
    }
    runs(v).concat()
}

fn last_thumbnail(v: &Value) -> String {
    find_all(v, "thumbnails")
        .first()
        .and_then(|t| t.as_array())
        .and_then(|a| a.last())
        .and_then(|t| t.get("url"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// "3:45" o "1:02:03" a segundos.
pub fn parse_duration(s: &str) -> Option<u32> {
    let mut total = 0u32;
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    for p in parts {
        total = total * 60 + p.parse::<u32>().ok()?;
    }
    Some(total)
}

/// Separa "Artista • Álbum • 3:45" (o con " · ") en sus partes.
fn split_byline(text: &str) -> Vec<String> {
    text.split(['•', '·']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn song_from_list_item(r: &Value) -> Option<Song> {
    let video_id = r
        .pointer("/playlistItemData/videoId")
        .or_else(|| find_all(r, "watchEndpoint").first().and_then(|w| w.get("videoId")))
        .and_then(Value::as_str)?
        .to_string();
    let cols = r.get("flexColumns")?.as_array()?;
    let col_text = |i: usize| cols.get(i).and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text")).map(runs_text).unwrap_or_default();
    let title = col_text(0);
    let parts = split_byline(&col_text(1));
    // En canciones: [artista(s), álbum, duración]; a veces con el tipo ("Canción") delante.
    let duration = parts.iter().rev().find_map(|p| parse_duration(p)).unwrap_or(0);
    let meta: Vec<&String> = parts.iter().filter(|p| parse_duration(p).is_none()).collect();
    let (artist, album) = match meta.as_slice() {
        [_, b, c, ..] => ((*b).clone(), (*c).clone()), // tipo, artista, álbum
        [a, b] => ((*a).clone(), (*b).clone()),
        [a] => ((*a).clone(), String::new()),
        _ => (String::new(), String::new()),
    };
    Some(Song { video_id, title, artist, album, duration, thumbnail: last_thumbnail(r) })
}

fn song_from_panel(r: &Value) -> Option<Song> {
    let video_id = r.get("videoId")?.as_str()?.to_string();
    let title = r.get("title").map(runs_text).unwrap_or_default();
    let parts = r.get("longBylineText").map(|b| split_byline(&runs_text(b))).unwrap_or_default();
    let duration = r.get("lengthText").map(runs_text).and_then(|s| parse_duration(&s)).unwrap_or(0);
    Some(Song {
        video_id,
        title,
        artist: parts.first().cloned().unwrap_or_default(),
        album: parts.get(1).filter(|a| a.parse::<u32>().is_err()).cloned().unwrap_or_default(),
        duration,
        thumbnail: last_thumbnail(r),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(parse_duration("3:45"), Some(225));
        assert_eq!(parse_duration("1:02:03"), Some(3723));
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("2026"), None);
    }

    #[test]
    fn panel_item() {
        let v = json!({
            "videoId": "abc",
            "title": { "runs": [{ "text": "Song" }] },
            "longBylineText": { "runs": [{ "text": "Artist" }, { "text": " • " }, { "text": "Album" }, { "text": " • " }, { "text": "2020" }] },
            "lengthText": { "runs": [{ "text": "4:01" }] },
            "thumbnail": { "thumbnails": [{ "url": "a" }, { "url": "https://lh3.googleusercontent.com/x=w120-h120" }] }
        });
        let s = song_from_panel(&v).unwrap();
        assert_eq!((s.title.as_str(), s.artist.as_str(), s.album.as_str(), s.duration), ("Song", "Artist", "Album", 241));
        assert_eq!(s.cover(544), "https://lh3.googleusercontent.com/x=w544-h544-l90-rj");
    }

    #[test]
    fn list_item() {
        let v = json!({
            "playlistItemData": { "videoId": "v1" },
            "flexColumns": [
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "T" }] } } },
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "A" }, { "text": " • " }, { "text": "Al" }, { "text": " • " }, { "text": "3:00" }] } } }
            ]
        });
        let s = song_from_list_item(&v).unwrap();
        assert_eq!((s.artist.as_str(), s.album.as_str(), s.duration), ("A", "Al", 180));
    }
}
