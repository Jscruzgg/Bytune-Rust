//! Consultas a LRCLIB y elección del mejor resultado (sin red).
//! Port de ByTune src/lib/lyrics-query.js.

use serde::Deserialize;

const BASE: &str = "https://lrclib.net/api/";

#[derive(Debug, Clone, Default)]
pub struct Track {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Segundos; 0 si no se sabe.
    pub duration: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(default)]
    pub duration: f64,
    pub synced_lyrics: Option<String>,
    pub plain_lyrics: Option<String>,
}

/// ¿Es `s` (en minúsculas, sin paréntesis) un sufijo que no forma parte del nombre?
fn is_noise(inner: &str) -> bool {
    let s = inner.trim().to_lowercase();
    let kind = s.strip_prefix("official ").map(str::trim_start).unwrap_or(&s);
    if ["music video", "lyric video", "video", "audio", "visualizer", "mv"].contains(&kind) {
        return true;
    }
    let s = s.as_str();
    // feat / ft (con o sin punto) seguido de un límite de palabra, o "with <algo>".
    for p in ["feat", "ft"] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
                return true;
            }
        }
    }
    s.strip_prefix("with").is_some_and(|r| r.starts_with(char::is_whitespace))
}

/// Quita sufijos finales entre () o [] como "(Official Video)" o "[feat. X]".
pub fn clean_title(title: &str) -> String {
    let mut s = title.trim().to_string();
    loop {
        // Como el regex de ByTune: abre con ( o [ y cierra con ) o ], sin anidar.
        if !s.ends_with([')', ']']) {
            break;
        }
        let body = &s[..s.len() - 1];
        let Some(start) = body.rfind(['(', '[']) else { break };
        if body[start + 1..].contains([')', ']']) || !is_noise(&body[start + 1..]) {
            break;
        }
        s = body[..start].trim_end().to_string();
    }
    s
}

pub fn primary_artist(artist: &str) -> String {
    let lower = artist.to_lowercase();
    let mut cut = artist.len();
    for sep in [" y ", " & ", " and ", ",", " x ", " feat. "] {
        if let Some(i) = lower.find(sep) {
            cut = cut.min(i);
        }
    }
    artist[..cut].trim().to_string()
}

fn build_url(endpoint: &str, params: &[(&str, String)]) -> String {
    let mut url = url::Url::parse(&format!("{BASE}{endpoint}")).expect("URL base válida");
    {
        let mut q = url.query_pairs_mut();
        for (k, v) in params {
            if !v.is_empty() {
                q.append_pair(k, v);
            }
        }
    }
    url.into()
}

pub fn lookup_urls(t: &Track) -> Vec<String> {
    let has_duration = t.duration.is_finite() && t.duration > 0.0;
    let secs = if has_duration { (t.duration.round() as i64).to_string() } else { String::new() };
    let clean = Some(clean_title(&t.title)).filter(|c| !c.is_empty()).unwrap_or_else(|| t.title.clone());
    let main_a = primary_artist(&t.artist);
    let mut urls = Vec::new();
    if has_duration {
        urls.push(build_url("get", &[("track_name", t.title.clone()), ("artist_name", t.artist.clone()), ("album_name", t.album.clone()), ("duration", secs.clone())]));
        if clean != t.title || main_a != t.artist {
            urls.push(build_url("get", &[("track_name", clean.clone()), ("artist_name", main_a.clone()), ("album_name", t.album.clone()), ("duration", secs)]));
        }
    }
    urls.push(build_url("search", &[("track_name", clean), ("artist_name", main_a)]));
    urls.dedup();
    urls
}

pub fn pick_search_result(results: &[Record], duration: f64) -> Option<&Record> {
    let known = duration.is_finite() && duration > 0.0;
    results
        .iter()
        .find(|r| r.synced_lyrics.as_deref().is_some_and(|s| !s.is_empty()) && (!known || (r.duration - duration).abs() <= 3.0))
        .or_else(|| results.iter().find(|r| r.plain_lyrics.as_deref().is_some_and(|s| !s.is_empty())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_suffixes() {
        assert_eq!(clean_title("Song (Official Music Video)"), "Song");
        assert_eq!(clean_title("Song [feat. X] (Audio)"), "Song");
        assert_eq!(clean_title("Song (with Y)"), "Song");
        assert_eq!(clean_title("Song (Remix)"), "Song (Remix)");
        assert_eq!(clean_title("Song (Featuring)"), "Song (Featuring)");
        assert_eq!(clean_title("(Official Video)"), "");
    }

    #[test]
    fn primary() {
        assert_eq!(primary_artist("A & B"), "A");
        assert_eq!(primary_artist("A, B"), "A");
        assert_eq!(primary_artist("A x B"), "A");
        assert_eq!(primary_artist("Solo"), "Solo");
    }

    #[test]
    fn urls() {
        let t = Track { title: "S (Official Video)".into(), artist: "A & B".into(), duration: 200.4, ..Default::default() };
        let u = lookup_urls(&t);
        assert_eq!(u.len(), 3);
        assert!(u[0].contains("/get?") && u[0].contains("duration=200"));
        assert!(u[2].contains("/search?track_name=S&artist_name=A"));
        let no_dur = lookup_urls(&Track { title: "S".into(), artist: "A".into(), ..Default::default() });
        assert_eq!(no_dur.len(), 1);
    }

    #[test]
    fn picks() {
        let r = vec![
            Record { duration: 100.0, synced_lyrics: Some("[00:01]x".into()), plain_lyrics: None },
            Record { duration: 200.0, synced_lyrics: Some("[00:01]y".into()), plain_lyrics: None },
        ];
        assert_eq!(pick_search_result(&r, 201.0).unwrap().duration, 200.0);
        assert!(pick_search_result(&r, 500.0).is_none());
    }
}
