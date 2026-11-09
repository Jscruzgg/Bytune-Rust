//! Cliente mínimo de InnerTube (la API interna de YouTube Music).
//!
//! Las respuestas son árboles JSON enormes cuya forma cambia a menudo. En vez de rutas fijas,
//! se buscan los "renderers" que interesan en cualquier punto del árbol (`find_all`), y de cada
//! uno se leen los pocos campos que hacen falta.

use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use crate::login::Session;
use crate::pages::{self, Item, Page};

const YTM: &str = "https://music.youtube.com/youtubei/v1";
const WEB_REMIX_VERSION: &str = "1.20260101.01.00";
/// Filtro "Canciones" de la búsqueda de YouTube Music.
const SONGS_FILTER: &str = "EgWKAQIIAWoMEA4QChADEAQQCRAF";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Song {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Segundos (0 si no se sabe).
    pub duration: u32,
    pub thumbnail: String,
    /// Para abrir la página del artista / álbum desde Now Playing.
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
}

impl Song {
    /// Portada a un tamaño concreto (las URLs de googleusercontent aceptan =wN-hN).
    pub fn cover(&self, px: u32) -> String {
        pages::cover_url(&self.thumbnail, px)
    }
}

fn home_continuation(v: &Value) -> Option<String> {
    find_all(v, "sectionListRenderer")
        .into_iter()
        .chain(find_all(v, "sectionListContinuation"))
        .find_map(|s| s.pointer("/continuations/0/nextContinuationData/continuation"))
        .and_then(Value::as_str)
        .map(str::to_owned)
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
            .post(format!("{YTM}/{endpoint}{}prettyPrint=false", if endpoint.contains('?') { '&' } else { '?' }))
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
        let v: Value = res.json().await?;
        // Depuración: BYTUNE_DUMP=<carpeta> guarda cada respuesta para ver su estructura.
        if let Ok(dir) = std::env::var("BYTUNE_DUMP") {
            let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
            let _ = std::fs::write(format!("{dir}/{}-{n}.json", endpoint.replace('/', "_")), serde_json::to_vec_pretty(&v).unwrap_or_default());
        }
        Ok(v)
    }

    /// Solo canciones (menú rápido y --probe).
    pub async fn search_songs(&self, query: &str) -> Result<Vec<Song>> {
        Ok(self.search(query, Some(SONGS_FILTER)).await?.songs())
    }

    /// Cola de reproducción a partir de una canción (radio de esa canción).
    pub async fn radio(&self, video_id: &str) -> Result<Vec<Song>> {
        let v = self
            .post("next", json!({ "videoId": video_id, "playlistId": format!("RDAMVM{video_id}"), "isAudioOnly": true }))
            .await?;
        Ok(panel_songs(&v))
    }

    /// Cola de una playlist, álbum o mix (y, si se da, empezando por esa canción).
    pub async fn queue(&self, video_id: Option<&str>, playlist_id: &str) -> Result<Vec<Song>> {
        let mut body = json!({ "playlistId": playlist_id, "isAudioOnly": true });
        if let Some(id) = video_id {
            body["videoId"] = json!(id);
        }
        let v = self.post("next", body).await?;
        Ok(panel_songs(&v))
    }

    /// Cualquier página: Inicio (FEmusic_home), Explorar, Biblioteca, álbum, artista,
    /// playlist, mood... Ver pages.rs.
    pub async fn browse(&self, id: &str, params: Option<&str>) -> Result<Page> {
        let mut body = json!({ "browseId": id });
        if let Some(p) = params {
            body["params"] = json!(p);
        }
        let v = self.post("browse", body).await?;
        let mut page = pages::parse(&v);
        // El Inicio con sesión llega en varias tandas: se piden un par más para que no se
        // quede en tres estanterías.
        if id == "FEmusic_home" {
            let mut token = home_continuation(&v);
            for _ in 0..3 {
                let Some(t) = token.take() else { break };
                let more = self.post_continuation(&t).await?;
                let extra = pages::parse(&more);
                page.sections.extend(extra.sections);
                token = home_continuation(&more);
            }
        }
        Ok(page)
    }

    /// Búsqueda; `params` es el filtro (de los chips de una búsqueda anterior).
    pub async fn search(&self, query: &str, params: Option<&str>) -> Result<Page> {
        let mut body = json!({ "query": query });
        if let Some(p) = params {
            body["params"] = json!(p);
        }
        let v = self.post("search", body).await?;
        Ok(pages::parse(&v))
    }

    /// Más elementos de una sección (playlists largas, biblioteca...).
    pub async fn continuation(&self, token: &str) -> Result<(Vec<Item>, Option<String>)> {
        let v = self.post_continuation(token).await?;
        Ok(pages::parse_continuation(&v))
    }

    /// Las continuaciones de browse llevan el token en la URL, no en el cuerpo.
    async fn post_continuation(&self, token: &str) -> Result<Value> {
        let t = url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>();
        self.post(&format!("browse?ctoken={t}&continuation={t}&type=next"), json!({})).await
    }

    /// Me gusta / quitar me gusta (necesita sesión).
    pub async fn like(&self, video_id: &str, like: bool) -> Result<()> {
        let endpoint = if like { "like/like" } else { "like/removelike" };
        self.post(endpoint, json!({ "target": { "videoId": video_id } })).await?;
        Ok(())
    }

    /// ¿Le diste me gusta? (lo dice /next en el menú de la canción).
    pub async fn liked(&self, video_id: &str) -> Result<bool> {
        let v = self.post("next", json!({ "videoId": video_id })).await?;
        Ok(find_all(&v, "likeStatus").first().and_then(|s| s.as_str()) == Some("LIKE"))
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

    pub fn set_lang(&mut self, hl: &str) {
        self.hl = hl.to_string();
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

pub fn runs_text(v: &Value) -> String {
    if let Some(s) = v.get("simpleText").and_then(Value::as_str) {
        return s.to_string();
    }
    runs(v).concat()
}

pub fn last_thumbnail(v: &Value) -> String {
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
pub fn split_byline(text: &str) -> Vec<String> {
    text.split(['•', '·']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

pub fn song_from_panel(r: &Value) -> Option<Song> {
    let video_id = r.get("videoId")?.as_str()?.to_string();
    let title = r.get("title").map(runs_text).unwrap_or_default();
    let byline = r.get("longBylineText");
    let parts = byline.map(|b| split_byline(&runs_text(b))).unwrap_or_default();
    let duration = r.get("lengthText").map(runs_text).and_then(|s| parse_duration(&s)).unwrap_or(0);
    // Ids del artista y del álbum, de los enlaces del subtítulo.
    let id_of = |page_type: &str| {
        byline
            .and_then(|b| b.get("runs"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|run| run.pointer("/navigationEndpoint/browseEndpoint"))
            .find(|b| {
                b.pointer("/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType").and_then(Value::as_str)
                    == Some(page_type)
            })
            .and_then(|b| b.get("browseId"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    Some(Song {
        video_id,
        title,
        artist: parts.first().cloned().unwrap_or_default(),
        album: parts.get(1).filter(|a| a.parse::<u32>().is_err()).cloned().unwrap_or_default(),
        duration,
        thumbnail: last_thumbnail(r),
        artist_id: id_of("MUSIC_PAGE_TYPE_ARTIST"),
        album_id: id_of("MUSIC_PAGE_TYPE_ALBUM"),
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
        assert_eq!(s.cover(544), "https://lh3.googleusercontent.com/x=w544-h544-p-l90-rj");
    }
}

/// Canciones de una cola de /next. Cada elemento es una canción o un "wrapper" con la
/// canción y su versión en vídeo (counterpart): solo cuenta la principal.
fn panel_songs(v: &Value) -> Vec<Song> {
    let mut out = Vec::new();
    for panel in find_all(v, "playlistPanelRenderer") {
        for c in panel.get("contents").and_then(Value::as_array).into_iter().flatten() {
            let r = c
                .get("playlistPanelVideoRenderer")
                .or_else(|| c.pointer("/playlistPanelVideoWrapperRenderer/primaryRenderer/playlistPanelVideoRenderer"));
            if let Some(s) = r.and_then(song_from_panel) {
                if !out.iter().any(|x: &Song| x.video_id == s.video_id) {
                    out.push(s);
                }
            }
        }
    }
    if out.is_empty() {
        // Estructura distinta: cualquier canción del árbol, sin repetir.
        for s in find_all(v, "playlistPanelVideoRenderer").into_iter().filter_map(song_from_panel) {
            if !out.iter().any(|x| x.video_id == s.video_id) {
                out.push(s);
            }
        }
    }
    out
}
