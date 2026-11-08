//! `bytune --probe "<búsqueda>"`: prueba la parte nativa sin ventana y mide cada paso
//! (búsqueda, radio, letra). El audio no entra: lo da el WebView (webplayer.rs).

use std::time::Instant;

use crate::innertube::Client;
use crate::lyrics::{self, query::Track, Lyrics};

pub async fn run(query: &str) -> i32 {
    match go(query).await {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            1
        }
    }
}

async fn go(query: &str) -> anyhow::Result<()> {
    let http = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
        .gzip(true)
        .build()?;
    let yt = Client::new(http.clone(), "es", None);

    let t = Instant::now();
    let results = yt.search_songs(query).await?;
    println!("buscar \"{query}\": {} resultados en {} ms", results.len(), t.elapsed().as_millis());
    for s in results.iter().take(3) {
        println!("  {} | {} | {} | {}s | {}", s.video_id, s.title, s.artist, s.duration, s.album);
    }
    let song = results.first().ok_or_else(|| anyhow::anyhow!("sin resultados"))?.clone();

    let t = Instant::now();
    let radio = yt.radio(&song.video_id).await?;
    println!("radio: {} canciones en {} ms", radio.len(), t.elapsed().as_millis());

    let t = Instant::now();
    let track = Track { video_id: song.video_id.clone(), title: song.title, artist: song.artist, album: song.album, duration: song.duration as f64 };
    let (ly, _) = lyrics::fetch(&http, &track).await;
    let desc = match &ly {
        Lyrics::Synced(l) => format!("sincronizada, {} líneas", l.len()),
        Lyrics::Plain(l) => format!("plana, {} líneas", l.len()),
        Lyrics::None => "ninguna en LRCLIB".into(),
    };
    println!("letra: {desc} en {} ms", t.elapsed().as_millis());
    if matches!(ly, Lyrics::None) {
        let t = Instant::now();
        let page = yt.page_lyrics(&track.video_id).await.map(|l| l.len());
        println!("letra de YouTube Music: {page:?} en {} ms", t.elapsed().as_millis());
    }
    Ok(())
}
