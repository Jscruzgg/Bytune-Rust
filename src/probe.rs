//! `bytune --probe "<búsqueda>"`: prueba la parte nativa sin ventana y mide cada paso
//! (páginas, búsqueda, cola, letra). El audio no entra: lo da el WebView (webplayer.rs).

use std::time::Instant;

use crate::innertube::Client;
use crate::lyrics::{self, query::Track, Lyrics};
use crate::pages::{Kind, Page, Target};

pub async fn run(query: &str) -> i32 {
    match go(query).await {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ERROR: {e:#}");
            1
        }
    }
}

fn show(name: &str, page: &Page, ms: u128) {
    let h = page.header.as_ref().map(|h| format!(" «{}» {}", h.title, h.subtitle)).unwrap_or_default();
    println!("{name}:{h} — {} secciones, {} chips, {ms} ms", page.sections.len(), page.chips.len());
    for s in page.sections.iter().take(8) {
        let first = s.items.first().map(|i| format!("{:?} «{}» {}", i.kind, i.title, i.subtitle)).unwrap_or_default();
        let more = if s.more.is_some() { " +más" } else { "" };
        let cont = if s.continuation.is_some() { " +continuación" } else { "" };
        println!("  [{:?}] «{}» {} elementos{more}{cont} — {first}", s.layout, s.title, s.items.len());
    }
    if !page.chips.is_empty() {
        println!("  chips: {}", page.chips.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join(", "));
    }
}

fn first_of(page: &Page, kind: Kind) -> Option<String> {
    page.sections.iter().flat_map(|s| &s.items).find(|i| i.kind == kind).and_then(|i| match &i.open {
        Some(Target::Browse { id, .. }) => Some(id.clone()),
        _ => None,
    })
}

async fn go(query: &str) -> anyhow::Result<()> {
    let http = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
        .gzip(true)
        .build()?;
    let yt = Client::new(http.clone(), "es", None);

    for (name, id) in [("Inicio", "FEmusic_home"), ("Explorar", "FEmusic_explore"), ("Novedades", "FEmusic_new_releases"), ("Moods", "FEmusic_moods_and_genres"), ("Listas", "FEmusic_charts")] {
        let t = Instant::now();
        match yt.browse(id, None).await {
            Ok(p) => show(name, &p, t.elapsed().as_millis()),
            Err(e) => println!("{name}: ERROR {e:#}"),
        }
    }

    let t = Instant::now();
    let search = yt.search(query, None).await?;
    show(&format!("Buscar «{query}»"), &search, t.elapsed().as_millis());

    if let Some(album) = first_of(&search, Kind::Album) {
        let t = Instant::now();
        let p = yt.browse(&album, None).await?;
        show("Álbum", &p, t.elapsed().as_millis());
        println!("  reproducir: {:?}", p.header.as_ref().and_then(|h| h.play.clone()));
    }
    if let Some(artist) = first_of(&search, Kind::Artist) {
        let t = Instant::now();
        let p = yt.browse(&artist, None).await?;
        show("Artista", &p, t.elapsed().as_millis());
    }
    if let Some(pl) = first_of(&search, Kind::Playlist) {
        let t = Instant::now();
        let p = yt.browse(&pl, None).await?;
        show("Playlist", &p, t.elapsed().as_millis());
    }

    let songs = yt.search_songs(query).await?;
    let song = songs.first().ok_or_else(|| anyhow::anyhow!("sin canciones"))?.clone();
    println!("canción: {} — {} ({:?}, {:?})", song.title, song.artist, song.artist_id, song.album_id);

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
    Ok(())
}
