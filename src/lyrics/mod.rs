//! Letras: LRCLIB (sincronizadas) con respaldo a letra plana. Port de ByTune src/lib.

pub mod lrc;
pub mod query;
pub mod timing;

use std::time::Duration;

use lrc::{add_gaps, parse_lrc, Line};
use query::{lookup_urls, pick_search_result, Record, Track};

#[derive(Debug, Clone)]
pub enum Lyrics {
    Synced(Vec<Line>),
    Plain(Vec<String>),
    None,
}

/// Segundos de diferencia aceptados en un resultado de search.
const TOLERANCE: f64 = 3.0;
/// Toda la fase LRCLIB tiene tope de tiempo, como en ByTune.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Resultado y si se puede guardar en caché (false si algo falló a medias).
pub async fn fetch(http: &reqwest::Client, track: &Track) -> (Lyrics, bool) {
    match tokio::time::timeout(TIMEOUT, query_lrclib(http, track)).await {
        Ok(r) => r,
        Err(_) => (Lyrics::None, false),
    }
}

async fn query_lrclib(http: &reqwest::Client, track: &Track) -> (Lyrics, bool) {
    let known = track.duration.is_finite() && track.duration > 0.0;
    let mut complete = true;
    let mut plain: Option<Vec<String>> = None;
    for url in lookup_urls(track) {
        let res = match http.get(&url).header("User-Agent", concat!("ByTune-rs/", env!("CARGO_PKG_VERSION"))).send().await {
            Ok(r) => r,
            Err(_) => {
                complete = false;
                continue;
            }
        };
        match res.status().as_u16() {
            404 => continue,
            200 => {}
            _ => {
                complete = false;
                continue;
            }
        }
        let is_get = url.contains("/api/get?");
        let record: Option<Record> = if is_get {
            res.json::<Record>().await.ok()
        } else {
            res.json::<Vec<Record>>().await.ok().and_then(|list| pick_search_result(&list, track.duration).cloned())
        };
        let Some(record) = record else { continue };
        // En search la sincronizada solo vale si la duración encaja.
        let trusted = is_get || !known || (record.duration - track.duration).abs() <= TOLERANCE;
        if let (true, Some(s)) = (trusted, record.synced_lyrics.as_deref()) {
            let lines = add_gaps(parse_lrc(s), 6.0);
            if !lines.is_empty() {
                return (Lyrics::Synced(lines), complete);
            }
        }
        if plain.is_none() {
            if let Some(p) = record.plain_lyrics.filter(|p| !p.is_empty()) {
                plain = Some(p.lines().map(str::to_owned).collect());
            }
        }
    }
    (plain.map_or(Lyrics::None, Lyrics::Plain), complete)
}
