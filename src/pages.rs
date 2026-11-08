//! Páginas de YouTube Music en un modelo común.
//!
//! Inicio, Explorar, Biblioteca, búsqueda, álbum, artista, playlist, novedades, listas y
//! moods se construyen con los mismos bloques: una cabecera, chips de filtro y secciones
//! (listas de canciones, carruseles y rejillas de tarjetas). Aquí se leen todos a `Page`.

use serde_json::Value;

use crate::innertube::{find_all, last_thumbnail, parse_duration, runs_text, split_byline, Song};

/// A dónde lleva algo al pulsarlo.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    /// Abrir una página (álbum, artista, playlist, mood...).
    Browse { id: String, params: Option<String> },
    /// Reproducir: una canción, una playlist/álbum/mix, o una canción dentro de una playlist.
    Watch { video_id: Option<String>, playlist_id: Option<String> },
    /// Otra búsqueda (chips de filtro).
    Search { query: String, params: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Other,
    Song,
    Video,
    Album,
    Playlist,
    Artist,
    Podcast,
    Episode,
    Mood,
}

impl Kind {
    fn from_page_type(t: &str) -> Kind {
        match t {
            "MUSIC_PAGE_TYPE_ALBUM" | "MUSIC_PAGE_TYPE_AUDIOBOOK" => Kind::Album,
            "MUSIC_PAGE_TYPE_PLAYLIST" => Kind::Playlist,
            "MUSIC_PAGE_TYPE_ARTIST" | "MUSIC_PAGE_TYPE_USER_CHANNEL" | "MUSIC_PAGE_TYPE_LIBRARY_ARTIST" => Kind::Artist,
            "MUSIC_PAGE_TYPE_PODCAST_SHOW_DETAIL_PAGE" => Kind::Podcast,
            "MUSIC_PAGE_TYPE_NON_MUSIC_AUDIO_TRACK_PAGE" => Kind::Episode,
            _ => Kind::Other,
        }
    }

    pub fn round(&self) -> bool {
        matches!(self, Kind::Artist)
    }
}

/// Un enlace con nombre dentro de una fila (artista o álbum de una canción).
#[derive(Debug, Clone, Default)]
pub struct Link {
    pub text: String,
    pub target: Option<Target>,
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub kind: Kind,
    pub title: String,
    pub subtitle: String,
    pub thumbnail: String,
    /// Segundos (0 si no se sabe).
    pub duration: u32,
    pub video_id: Option<String>,
    /// Lo que pasa al pulsar la fila o la tarjeta.
    pub open: Option<Target>,
    /// El botón de play (si es distinto de `open`).
    pub play: Option<Target>,
    pub artists: Vec<Link>,
    pub album: Option<Link>,
    /// Para mood/géneros: color de la tarjeta (0xAARRGGBB).
    pub color: Option<u32>,
}

impl Item {
    pub fn cover(&self, px: u32) -> String {
        cover_url(&self.thumbnail, px)
    }

    /// Como canción, si se puede reproducir sola.
    pub fn song(&self) -> Option<Song> {
        Some(Song {
            video_id: self.video_id.clone()?,
            title: self.title.clone(),
            artist: if self.artists.is_empty() {
                self.subtitle.split(['•', '·']).next().unwrap_or_default().trim().to_string()
            } else {
                self.artists.iter().map(|a| a.text.as_str()).collect::<Vec<_>>().join(", ")
            },
            album: self.album.as_ref().map(|a| a.text.clone()).unwrap_or_default(),
            duration: self.duration,
            thumbnail: self.thumbnail.clone(),
            artist_id: self.artists.iter().find_map(|a| browse_id(a.target.as_ref())),
            album_id: self.album.as_ref().and_then(|a| browse_id(a.target.as_ref())),
        })
    }

    /// Lo que se reproduce al pulsar play.
    pub fn play_target(&self) -> Option<Target> {
        if let Some(p) = &self.play {
            return Some(p.clone());
        }
        match &self.open {
            Some(t @ Target::Watch { .. }) => Some(t.clone()),
            _ => self.video_id.clone().map(|v| Target::Watch { video_id: Some(v), playlist_id: None }),
        }
    }
}

fn browse_id(t: Option<&Target>) -> Option<String> {
    match t {
        Some(Target::Browse { id, .. }) => Some(id.clone()),
        _ => None,
    }
}

pub fn cover_url(thumb: &str, px: u32) -> String {
    match thumb.rfind('=') {
        Some(i) if thumb.contains("googleusercontent") || thumb.contains("ggpht") => format!("{}=w{px}-h{px}-p-l90-rj", &thumb[..i]),
        _ => thumb.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Filas (canciones, resultados de búsqueda, pistas de un álbum).
    List,
    /// Tarjetas en carrusel o rejilla (álbumes, playlists, artistas, moods).
    Grid,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub layout: Layout,
    pub items: Vec<Item>,
    /// "Más" / "Ver todo": otra página.
    pub more: Option<Target>,
    /// Token para cargar más elementos de esta sección.
    pub continuation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Chip {
    pub text: String,
    pub target: Target,
    pub selected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Header {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub thumbnail: String,
    /// Botón "Reproducir" / "Aleatorio" / "Radio" de la cabecera.
    pub play: Option<Target>,
    pub shuffle: Option<Target>,
    pub radio: Option<Target>,
}

#[derive(Debug, Clone, Default)]
pub struct Page {
    pub header: Option<Header>,
    pub chips: Vec<Chip>,
    pub sections: Vec<Section>,
}

impl Page {
    /// Todas las canciones reproducibles de la página, en orden (para "Reproducir todo").
    pub fn songs(&self) -> Vec<Song> {
        self.sections.iter().filter(|s| s.layout == Layout::List).flat_map(|s| s.items.iter().filter_map(Item::song)).collect()
    }
}

// ---------- lectura ----------

pub fn parse(v: &Value) -> Page {
    let mut page = Page::default();

    // Cabecera: la primera que aparezca de cualquiera de los tipos conocidos.
    for key in [
        "musicImmersiveHeaderRenderer",
        "musicResponsiveHeaderRenderer",
        "musicDetailHeaderRenderer",
        "musicVisualHeaderRenderer",
        "musicEditablePlaylistDetailHeaderRenderer",
        "musicHeaderRenderer",
    ] {
        if let Some(h) = find_all(v, key).into_iter().next() {
            page.header = Some(header(h));
            break;
        }
    }

    // Chips de filtro (búsqueda, biblioteca, Inicio con sesión).
    for c in find_all(v, "chipCloudChipRenderer") {
        let text = c.get("text").map(runs_text).unwrap_or_default();
        let selected = c.get("isSelected").and_then(Value::as_bool).unwrap_or(false);
        if let Some(target) = c.get("navigationEndpoint").and_then(target_of) {
            page.chips.push(Chip { text, target, selected });
        }
    }

    for list in find_all(v, "sectionListRenderer").into_iter().chain(find_all(v, "sectionListContinuation")) {
        if let Some(contents) = list.get("contents").and_then(Value::as_array) {
            for c in contents {
                section(c, &mut page);
            }
        }
    }
    // Algunas páginas (Biblioteca) traen la rejilla sin sectionListRenderer.
    if page.sections.is_empty() {
        for g in find_all(v, "gridRenderer") {
            grid(g, &mut page);
        }
    }

    // Las pistas de un álbum no traen portada: se usa la de la cabecera.
    if let Some(h) = &page.header {
        for s in &mut page.sections {
            for it in &mut s.items {
                if it.thumbnail.is_empty() {
                    it.thumbnail = h.thumbnail.clone();
                }
                if it.album.is_none() && it.kind == Kind::Song && !h.title.is_empty() {
                    it.album = Some(Link { text: h.title.clone(), target: None });
                }
            }
        }
    }
    page
}

/// Lee una respuesta de continuación: los elementos nuevos y el siguiente token.
pub fn parse_continuation(v: &Value) -> (Vec<Item>, Option<String>) {
    let mut items = Vec::new();
    let mut next = None;
    let mut push_contents = |contents: &Vec<Value>, next: &mut Option<String>| {
        for c in contents {
            if let Some(it) = item(c) {
                items.push(it);
            } else if let Some(t) = continuation_item_token(c) {
                *next = Some(t);
            }
        }
    };
    for key in ["musicShelfContinuation", "musicPlaylistShelfContinuation", "gridContinuation", "sectionListContinuation"] {
        for c in find_all(v, key) {
            if let Some(contents) = c.get("contents").or_else(|| c.get("items")).and_then(Value::as_array) {
                push_contents(contents, &mut next);
            }
            if next.is_none() {
                next = old_continuation(c);
            }
        }
    }
    for a in find_all(v, "appendContinuationItemsAction") {
        if let Some(contents) = a.get("continuationItems").and_then(Value::as_array) {
            push_contents(contents, &mut next);
        }
    }
    (items, next)
}

fn header(h: &Value) -> Header {
    let subtitle = [h.get("subtitle"), h.get("straplineTextOne"), h.get("secondSubtitle")]
        .into_iter()
        .flatten()
        .map(runs_text)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let description = h
        .get("description")
        .map(|d| d.get("musicDescriptionShelfRenderer").and_then(|m| m.get("description")).map(runs_text).unwrap_or_else(|| runs_text(d)))
        .unwrap_or_default();
    let mut hd = Header {
        title: h.get("title").map(runs_text).unwrap_or_default(),
        subtitle,
        description,
        thumbnail: last_thumbnail(h),
        ..Default::default()
    };
    // Botones: el primero que reproduce es "Reproducir"; los de radio y aleatorio se distinguen
    // por el parámetro/playlist que llevan.
    for w in find_all(h, "watchEndpoint").into_iter().chain(find_all(h, "watchPlaylistEndpoint")) {
        let t = watch_target(w);
        let pl = w.get("playlistId").and_then(Value::as_str).unwrap_or("");
        let params = w.get("params").and_then(Value::as_str).unwrap_or("");
        if pl.starts_with("RDAO") || pl.starts_with("RDEM") || pl.starts_with("RDAMPL") {
            hd.radio.get_or_insert(t);
        } else if params.starts_with("wAEB8gECKAE") {
            hd.shuffle.get_or_insert(t);
        } else {
            hd.play.get_or_insert(t);
        }
    }
    hd
}

fn section(c: &Value, page: &mut Page) {
    if let Some(s) = c.get("musicCarouselShelfRenderer") {
        let hdr = s.pointer("/header/musicCarouselShelfBasicHeaderRenderer");
        let title = hdr.and_then(|h| h.get("title")).map(runs_text).unwrap_or_default();
        let more = hdr
            .and_then(|h| h.pointer("/moreContentButton/buttonRenderer/navigationEndpoint"))
            .or_else(|| hdr.and_then(|h| h.pointer("/title/runs/0/navigationEndpoint")))
            .and_then(target_of);
        let items: Vec<Item> = s.get("contents").and_then(Value::as_array).into_iter().flatten().filter_map(item).collect();
        // Carruseles de canciones (Selección rápida) se ven mejor como lista.
        let layout = if items.iter().all(|i| i.kind == Kind::Song) && s.get("numItemsPerColumn").is_some() { Layout::List } else { Layout::Grid };
        push(page, title, layout, items, more, None);
    } else if let Some(s) = c.get("musicShelfRenderer").or_else(|| c.get("musicPlaylistShelfRenderer")) {
        let title = s.get("title").map(runs_text).unwrap_or_default();
        let more = s
            .pointer("/bottomEndpoint")
            .or_else(|| s.pointer("/title/runs/0/navigationEndpoint"))
            .and_then(target_of);
        let mut items = Vec::new();
        let mut cont = old_continuation(s);
        for it in s.get("contents").and_then(Value::as_array).into_iter().flatten() {
            if let Some(i) = item(it) {
                items.push(i);
            } else if let Some(t) = continuation_item_token(it) {
                cont = Some(t);
            }
        }
        push(page, title, Layout::List, items, more, cont);
    } else if let Some(s) = c.get("musicCardShelfRenderer") {
        // "Resultado principal" de la búsqueda: la tarjeta grande y sus filas.
        let mut top = Vec::new();
        let mut card = Item {
            title: s.get("title").map(runs_text).unwrap_or_default(),
            subtitle: s.get("subtitle").map(runs_text).unwrap_or_default(),
            thumbnail: last_thumbnail(s.get("thumbnail").unwrap_or(&Value::Null)),
            open: s.pointer("/title/runs/0/navigationEndpoint").and_then(target_of),
            ..Default::default()
        };
        card.kind = s.pointer("/title/runs/0/navigationEndpoint").map(kind_of).unwrap_or_default();
        if let Some(Target::Watch { video_id, .. }) = &card.open {
            card.video_id = video_id.clone();
            card.kind = Kind::Song;
        }
        card.artists = links(s.get("subtitle")).0;
        top.push(card);
        top.extend(s.get("contents").and_then(Value::as_array).into_iter().flatten().filter_map(item));
        let title = s.pointer("/header/musicCardShelfHeaderBasicRenderer/title").map(runs_text).unwrap_or_default();
        push(page, title, Layout::List, top, None, None);
    } else if let Some(g) = c.get("gridRenderer") {
        grid(g, page);
    } else if let Some(s) = c.get("itemSectionRenderer") {
        // La búsqueda nueva manda cada resultado en su propio itemSectionRenderer: los
        // elementos sueltos se juntan en una lista; lo demás se lee como sección.
        let contents: Vec<&Value> = s.get("contents").and_then(Value::as_array).into_iter().flatten().collect();
        let items: Vec<Item> = contents.iter().filter_map(|c| item(c)).collect();
        if items.is_empty() {
            for inner in contents {
                section(inner, page);
            }
        } else {
            push(page, String::new(), Layout::List, items, None, None);
        }
    } else if let Some(d) = c.get("musicDescriptionShelfRenderer") {
        // Descripción (artista, letras): a la cabecera.
        let text = d.get("description").map(runs_text).unwrap_or_default();
        let h = page.header.get_or_insert_with(Default::default);
        if h.description.is_empty() {
            h.description = text;
        }
    }
}

fn grid(g: &Value, page: &mut Page) {
    let title = g.pointer("/header/gridHeaderRenderer/title").map(runs_text).unwrap_or_default();
    let mut items = Vec::new();
    let mut cont = old_continuation(g);
    for it in g.get("items").and_then(Value::as_array).into_iter().flatten() {
        if let Some(i) = item(it) {
            items.push(i);
        } else if let Some(t) = continuation_item_token(it) {
            cont = Some(t);
        }
    }
    push(page, title, Layout::Grid, items, None, cont);
}

fn push(page: &mut Page, title: String, layout: Layout, items: Vec<Item>, more: Option<Target>, continuation: Option<String>) {
    if items.is_empty() {
        return;
    }
    // Listas sin título seguidas (resultados de búsqueda) van juntas.
    if title.is_empty() && layout == Layout::List && more.is_none() {
        if let Some(last) = page.sections.last_mut() {
            if last.title.is_empty() && last.layout == Layout::List && last.more.is_none() {
                last.items.extend(items);
                last.continuation = continuation.or(last.continuation.take());
                return;
            }
        }
    }
    page.sections.push(Section { title, layout, items, more, continuation });
}

/// Cualquier elemento de una sección.
fn item(c: &Value) -> Option<Item> {
    if let Some(r) = c.get("musicResponsiveListItemRenderer") {
        list_item(r)
    } else if let Some(r) = c.get("musicTwoRowItemRenderer") {
        two_row(r)
    } else if let Some(r) = c.get("musicNavigationButtonRenderer") {
        nav_button(r)
    } else if let Some(r) = c.get("playlistPanelVideoRenderer") {
        let s = crate::innertube::song_from_panel(r)?;
        Some(Item {
            kind: Kind::Song,
            title: s.title.clone(),
            subtitle: s.artist.clone(),
            thumbnail: s.thumbnail.clone(),
            duration: s.duration,
            video_id: Some(s.video_id.clone()),
            ..Default::default()
        })
    } else {
        None
    }
}

fn list_item(r: &Value) -> Option<Item> {
    let cols = r.get("flexColumns")?.as_array()?;
    let col = |i: usize| cols.get(i).and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"));
    let title = col(0).map(runs_text).unwrap_or_default();
    if title.is_empty() {
        return None;
    }
    // Resto de columnas: artista(s), álbum, reproducciones...
    let rest: Vec<&Value> = (1..cols.len()).filter_map(col).collect();
    let subtitle_parts: Vec<String> = rest.iter().map(|v| runs_text(v)).filter(|s| !s.is_empty()).collect();
    let mut subtitle = subtitle_parts.join(" • ");

    let mut it = Item { title, thumbnail: last_thumbnail(r.get("thumbnail").unwrap_or(&Value::Null)), ..Default::default() };
    let (mut artists, mut album) = (Vec::new(), None);
    for v in &rest {
        let (a, al) = links(Some(v));
        artists.extend(a);
        album = album.or(al);
    }
    it.artists = artists;
    it.album = album;

    // Duración: columna fija, o el último trozo "m:ss" del subtítulo.
    let fixed = r
        .pointer("/fixedColumns/0/musicResponsiveListItemFixedColumnRenderer/text")
        .map(runs_text)
        .and_then(|s| parse_duration(&s));
    let parts = split_byline(&subtitle);
    it.duration = fixed.or_else(|| parts.iter().rev().find_map(|p| parse_duration(p))).unwrap_or(0);
    if fixed.is_none() && it.duration > 0 {
        // Se quita del subtítulo para no repetirla.
        subtitle = parts.into_iter().filter(|p| parse_duration(p).is_none()).collect::<Vec<_>>().join(" • ");
    }
    it.subtitle = subtitle;

    it.video_id = r
        .pointer("/playlistItemData/videoId")
        .or_else(|| find_all(r.get("overlay").unwrap_or(&Value::Null), "watchEndpoint").first().and_then(|w| w.get("videoId")))
        .and_then(Value::as_str)
        .map(str::to_owned);
    it.open = r.get("navigationEndpoint").and_then(target_of);
    it.play = find_all(r.get("overlay").unwrap_or(&Value::Null), "playNavigationEndpoint").first().and_then(|e| target_of(e));
    it.kind = match (&it.video_id, r.get("navigationEndpoint")) {
        (_, Some(n)) if n.get("browseEndpoint").is_some() => kind_of(n),
        (Some(_), _) => {
            let mvt = find_all(r, "musicVideoType").first().and_then(|v| v.as_str()).unwrap_or("");
            if mvt == "MUSIC_VIDEO_TYPE_ATV" || mvt.is_empty() { Kind::Song } else { Kind::Video }
        }
        _ => Kind::Other,
    };
    if matches!(it.kind, Kind::Video) {
        it.kind = Kind::Song; // se reproduce igual; el tipo solo cambia el icono en la web
    }
    Some(it)
}

fn two_row(r: &Value) -> Option<Item> {
    let nav = r.get("navigationEndpoint");
    let mut it = Item {
        title: r.get("title").map(runs_text).unwrap_or_default(),
        subtitle: r.get("subtitle").map(runs_text).unwrap_or_default(),
        thumbnail: last_thumbnail(r.get("thumbnailRenderer").unwrap_or(r)),
        open: nav.and_then(target_of),
        play: find_all(r.get("thumbnailOverlay").unwrap_or(&Value::Null), "playNavigationEndpoint").first().and_then(|e| target_of(e)),
        ..Default::default()
    };
    it.kind = nav.map(kind_of).unwrap_or_default();
    if let Some(Target::Watch { video_id: Some(v), .. }) = &it.open {
        it.video_id = Some(v.clone());
        it.kind = Kind::Song;
    }
    it.artists = links(r.get("subtitle")).0;
    (!it.title.is_empty()).then_some(it)
}

fn nav_button(r: &Value) -> Option<Item> {
    let title = r.get("buttonText").map(runs_text)?;
    let open = r.get("clickCommand").and_then(target_of);
    let color = r.pointer("/solid/leftStripeColor").and_then(Value::as_u64).map(|c| c as u32);
    Some(Item { kind: Kind::Mood, title, open, color, ..Default::default() })
}

/// Artistas y álbum enlazados dentro de un texto con runs.
fn links(v: Option<&Value>) -> (Vec<Link>, Option<Link>) {
    let mut artists = Vec::new();
    let mut album = None;
    for run in v.and_then(|v| v.get("runs")).and_then(Value::as_array).into_iter().flatten() {
        let text = run.get("text").and_then(Value::as_str).unwrap_or_default().to_string();
        let Some(nav) = run.get("navigationEndpoint") else { continue };
        let target = target_of(nav);
        match kind_of(nav) {
            Kind::Artist => artists.push(Link { text, target }),
            Kind::Album => album = Some(Link { text, target }),
            _ => {}
        }
    }
    (artists, album)
}

fn kind_of(nav: &Value) -> Kind {
    if nav.get("watchEndpoint").is_some() {
        return Kind::Song;
    }
    nav.pointer("/browseEndpoint/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType")
        .and_then(Value::as_str)
        .map(Kind::from_page_type)
        .unwrap_or_default()
}

fn watch_target(w: &Value) -> Target {
    Target::Watch {
        video_id: w.get("videoId").and_then(Value::as_str).map(str::to_owned),
        playlist_id: w.get("playlistId").and_then(Value::as_str).map(str::to_owned),
    }
}

/// Destino de un navigationEndpoint (o clickCommand).
pub fn target_of(nav: &Value) -> Option<Target> {
    if let Some(b) = nav.get("browseEndpoint") {
        return Some(Target::Browse {
            id: b.get("browseId")?.as_str()?.to_string(),
            params: b.get("params").and_then(Value::as_str).map(str::to_owned),
        });
    }
    if let Some(w) = nav.get("watchEndpoint").or_else(|| nav.get("watchPlaylistEndpoint")) {
        return Some(watch_target(w));
    }
    if let Some(s) = nav.get("searchEndpoint") {
        return Some(Target::Search {
            query: s.get("query").and_then(Value::as_str).unwrap_or_default().to_string(),
            params: s.get("params").and_then(Value::as_str).map(str::to_owned),
        });
    }
    None
}

fn continuation_item_token(c: &Value) -> Option<String> {
    c.pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn old_continuation(s: &Value) -> Option<String> {
    s.pointer("/continuations/0/nextContinuationData/continuation")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(text: &str, browse: Option<(&str, &str)>) -> Value {
        match browse {
            Some((id, pt)) => json!({ "text": text, "navigationEndpoint": { "browseEndpoint": {
                "browseId": id,
                "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": pt } } } } }),
            None => json!({ "text": text }),
        }
    }

    #[test]
    fn song_row_with_links() {
        let v = json!({ "contents": [{ "musicShelfRenderer": {
            "title": { "runs": [{ "text": "Canciones" }] },
            "contents": [{ "musicResponsiveListItemRenderer": {
                "playlistItemData": { "videoId": "v1" },
                "flexColumns": [
                    { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "Tema" }] } } },
                    { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                        run("Artista", Some(("UC1", "MUSIC_PAGE_TYPE_ARTIST"))), run(" • ", None),
                        run("Disco", Some(("MPREb_1", "MUSIC_PAGE_TYPE_ALBUM"))), run(" • ", None), run("3:10", None)
                    ] } } }
                ]
            } }]
        } }] });
        let page = parse(&json!({ "sectionListRenderer": v }));
        let s = &page.sections[0];
        assert_eq!((s.title.as_str(), s.layout), ("Canciones", Layout::List));
        let it = &s.items[0];
        assert_eq!(it.kind, Kind::Song);
        assert_eq!(it.duration, 190);
        assert_eq!(it.subtitle, "Artista • Disco");
        let song = it.song().unwrap();
        assert_eq!((song.artist.as_str(), song.album.as_str()), ("Artista", "Disco"));
        assert_eq!(song.artist_id.as_deref(), Some("UC1"));
        assert_eq!(song.album_id.as_deref(), Some("MPREb_1"));
    }

    #[test]
    fn carousel_cards_and_moods() {
        let v = json!({ "sectionListRenderer": { "contents": [
            { "musicCarouselShelfRenderer": {
                "header": { "musicCarouselShelfBasicHeaderRenderer": { "title": { "runs": [{ "text": "Álbumes" }] } } },
                "contents": [{ "musicTwoRowItemRenderer": {
                    "title": { "runs": [{ "text": "Disco" }] },
                    "subtitle": { "runs": [{ "text": "Álbum • 2020" }] },
                    "navigationEndpoint": { "browseEndpoint": { "browseId": "MPREb_1",
                        "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": "MUSIC_PAGE_TYPE_ALBUM" } } } },
                    "thumbnailOverlay": { "musicItemThumbnailOverlayRenderer": { "content": { "musicPlayButtonRenderer": {
                        "playNavigationEndpoint": { "watchPlaylistEndpoint": { "playlistId": "OLAK5" } } } } } }
                } }]
            } },
            { "gridRenderer": { "items": [{ "musicNavigationButtonRenderer": {
                "buttonText": { "runs": [{ "text": "Relax" }] },
                "clickCommand": { "browseEndpoint": { "browseId": "FEmusic_moods_and_genres_category", "params": "p1" } }
            } }] } }
        ] } });
        let page = parse(&v);
        assert_eq!(page.sections.len(), 2);
        let album = &page.sections[0].items[0];
        assert_eq!(album.kind, Kind::Album);
        assert_eq!(album.open, Some(Target::Browse { id: "MPREb_1".into(), params: None }));
        assert_eq!(album.play_target(), Some(Target::Watch { video_id: None, playlist_id: Some("OLAK5".into()) }));
        let mood = &page.sections[1].items[0];
        assert_eq!(mood.kind, Kind::Mood);
        assert_eq!(mood.open, Some(Target::Browse { id: "FEmusic_moods_and_genres_category".into(), params: Some("p1".into()) }));
    }
}
