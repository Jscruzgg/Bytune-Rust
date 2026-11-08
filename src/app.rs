//! Estado de la app, navegación entre secciones, cola y reproducción, e integración con
//! Windows. Las vistas están en ui/ y devuelven `Act`s que se aplican aquí.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use egui::{Align, Color32, CornerRadius, Layout, RichText, Sense, Vec2, ViewportCommand};

use crate::desktop::{self, Action, Desktop};
use crate::i18n::{t, L, S};
use crate::innertube::{Client, Song};
use crate::lyrics::{self, query::Track, Lyrics};
use crate::media::{Media, MediaKey};
use crate::pages::{Item, Kind, Page, Target};
use crate::settings::{Reach, Recent, Settings, ShiftEnter};
use crate::theme;
use crate::ui::{self, quickmenu::QmOut, quickmenu::QuickMenu, Act};
use crate::webplayer::WebPlayer;

/// Secciones de la app. Cada una es una página distinta con su propio contenido.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Route {
    Home,
    Explore,
    Library(LibTab),
    Search { query: String, params: Option<String> },
    /// Álbum, artista, playlist, novedades, listas, mood... (cualquier página de YTM).
    Browse { id: String, params: Option<String> },
    NowPlaying,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibTab {
    Playlists,
    Songs,
    Albums,
    Artists,
    Subscriptions,
    History,
}

impl LibTab {
    const ALL: [LibTab; 6] = [LibTab::Playlists, LibTab::Songs, LibTab::Albums, LibTab::Artists, LibTab::Subscriptions, LibTab::History];

    fn browse_id(self) -> &'static str {
        match self {
            LibTab::Playlists => "FEmusic_liked_playlists",
            LibTab::Songs => "VLLM",
            LibTab::Albums => "FEmusic_liked_albums",
            LibTab::Artists => "FEmusic_library_corpus_track_artists",
            LibTab::Subscriptions => "FEmusic_library_corpus_artists",
            LibTab::History => "FEmusic_history",
        }
    }

    fn label(self) -> S {
        match self {
            LibTab::Playlists => S::lib_playlists,
            LibTab::Songs => S::lib_songs,
            LibTab::Albums => S::lib_albums,
            LibTab::Artists => S::lib_artists,
            LibTab::Subscriptions => S::lib_subscriptions,
            LibTab::History => S::lib_history,
        }
    }
}

enum PageState {
    Loading,
    Ready(Page),
    Failed(String),
}

/// Resultados de las tareas de red, de vuelta al hilo de la UI.
enum Msg {
    Page(Route, anyhow::Result<Page>),
    More(Route, usize, anyhow::Result<(Vec<Item>, Option<String>)>),
    /// Cola nueva (playlist, álbum, mix): empezar por `start` si está.
    Queue { songs: anyhow::Result<Vec<Song>>, start: Option<String>, shuffle: bool },
    /// Radio de una canción: se añade detrás de ella (o al final si `append`).
    Radio { seed: String, songs: Vec<Song>, append: bool },
    Lyrics { id: String, lyrics: Lyrics },
    Liked(String, bool),
    Playlists(Vec<Item>),
    QmSongs(String, Vec<Song>),
    Status(String),
}

pub struct App {
    rt: tokio::runtime::Handle,
    http: reqwest::Client,
    yt: Client,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    player: WebPlayer,
    media: Option<Media>,
    media_rx: Receiver<MediaKey>,
    desktop: Desktop,

    settings: Settings,
    settings_dirty: bool,
    sys_lang: String,
    l: L,

    history: Vec<Route>,
    pos: usize,
    pages: HashMap<Route, PageState>,
    loading_more: Option<(Route, usize)>,
    query: String,
    focus_search: bool,
    playlists: Vec<Item>,

    queue: Vec<Song>,
    current: Option<usize>,
    pending: Option<(String, Instant)>,
    loading: bool,
    status: Option<(String, Instant)>,
    lyrics: HashMap<String, Lyrics>,
    lyrics_loading: Option<String>,
    last_line: Option<usize>,
    liked: HashMap<String, bool>,
    notified: Option<String>,
    radio_pending: bool,

    seek_drag: Option<f32>,
    was_playing: bool,
    fullscreen: bool,
    want_video: bool,
    hidden: bool,
    quitting: bool,
    qm: QuickMenu,
    qm_was_focused: bool,
    recording: bool,
    shortcut_error: Option<String>,
    autoplay_query: Option<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, rt: tokio::runtime::Handle, background: bool) -> Self {
        theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut settings = Settings::load(cc.storage);
        // El registro manda: si se quitó a mano de "Inicio" de Windows, el ajuste lo refleja.
        settings.start_with_windows = desktop::starts_with_windows();
        let sys_lang = sys_lang();
        let l = L::resolve(settings.lang, &sys_lang);

        let http = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("cliente HTTP");
        let (tx, rx) = crossbeam_channel::unbounded();

        let ctx = cc.egui_ctx.clone();
        let mut player = WebPlayer::new(cc, move || ctx.request_repaint()).expect("no se pudo crear el WebView2");
        player.set_volume(settings.volume * settings.volume);
        // Las cookies de la sesión ya están en el perfil del WebView si se inició antes.
        let yt = Client::new(http.clone(), l.hl(), player.session());

        let (media_tx, media_rx) = crossbeam_channel::unbounded();
        let ctx = cc.egui_ctx.clone();
        let media = Media::new(window_handle(cc), media_tx, move || ctx.request_repaint());
        let ctx = cc.egui_ctx.clone();
        let mut desktop = Desktop::new(l, move || ctx.request_repaint());
        if settings.quick_menu_reach == Reach::Anywhere && !desktop.set_quick_menu_global(Some(&settings.quick_menu_shortcut)) {
            log::warn!("no se pudo registrar el atajo global del menú rápido");
        }

        let mut app = Self {
            rt,
            http,
            yt,
            tx,
            rx,
            player,
            media,
            media_rx,
            desktop,
            settings,
            settings_dirty: false,
            sys_lang,
            l,
            history: vec![Route::Home],
            pos: 0,
            pages: HashMap::new(),
            loading_more: None,
            query: String::new(),
            focus_search: false,
            playlists: Vec::new(),
            queue: Vec::new(),
            current: None,
            pending: None,
            loading: false,
            status: None,
            lyrics: HashMap::new(),
            lyrics_loading: None,
            last_line: None,
            liked: HashMap::new(),
            notified: None,
            radio_pending: false,
            seek_drag: None,
            was_playing: false,
            fullscreen: false,
            want_video: false,
            hidden: background,
            quitting: false,
            qm: QuickMenu::default(),
            qm_was_focused: false,
            recording: false,
            shortcut_error: None,
            autoplay_query: std::env::var("BYTUNE_AUTOPLAY").ok(),
        };
        app.load(&Route::Home, &cc.egui_ctx);
        app.load_playlists(&cc.egui_ctx);
        // BYTUNE_AUTOPLAY="búsqueda": reproduce el primer resultado al arrancar (pruebas y
        // comparativa de rendimiento).
        if let Some(q) = app.autoplay_query.clone() {
            let (yt, tx, ctx) = (app.yt.clone(), app.tx.clone(), cc.egui_ctx.clone());
            app.rt.spawn(async move {
                let songs = yt.search_songs(&q).await.map(|s| s.into_iter().take(1).collect());
                let _ = tx.send(Msg::Queue { songs, start: None, shuffle: false });
                ctx.request_repaint();
            });
        }
        app
    }

    fn route(&self) -> &Route {
        &self.history[self.pos]
    }

    // ---------- navegación ----------

    fn navigate(&mut self, route: Route, ctx: &egui::Context) {
        if *self.route() == route {
            return;
        }
        self.history.truncate(self.pos + 1);
        self.history.push(route.clone());
        self.pos = self.history.len() - 1;
        if !self.pages.contains_key(&route) {
            self.load(&route, ctx);
        }
    }

    fn back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    fn forward(&mut self) {
        if self.pos + 1 < self.history.len() {
            self.pos += 1;
        }
    }

    fn load(&mut self, route: &Route, ctx: &egui::Context) {
        let (yt, tx, ctx, r) = (self.yt.clone(), self.tx.clone(), ctx.clone(), route.clone());
        let fut = async move {
            let res = match &r {
                Route::Home => yt.browse("FEmusic_home", None).await,
                Route::Explore => yt.browse("FEmusic_explore", None).await,
                Route::Library(tab) => yt.browse(tab.browse_id(), None).await,
                Route::Search { query, params } => yt.search(query, params.as_deref()).await,
                Route::Browse { id, params } => yt.browse(id, params.as_deref()).await,
                Route::NowPlaying | Route::Settings => return,
            };
            let _ = tx.send(Msg::Page(r, res));
            ctx.request_repaint();
        };
        if matches!(route, Route::NowPlaying | Route::Settings) {
            return;
        }
        if matches!(route, Route::Library(_)) && !self.yt.logged_in() {
            return;
        }
        self.pages.insert(route.clone(), PageState::Loading);
        self.rt.spawn(fut);
    }

    fn load_playlists(&mut self, ctx: &egui::Context) {
        if !self.yt.logged_in() {
            self.playlists.clear();
            return;
        }
        let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
        self.rt.spawn(async move {
            if let Ok(p) = yt.browse("FEmusic_liked_playlists", None).await {
                let items = p.sections.into_iter().flat_map(|s| s.items).filter(|i| i.kind == Kind::Playlist).collect();
                let _ = tx.send(Msg::Playlists(items));
                ctx.request_repaint();
            }
        });
    }

    fn open(&mut self, target: Target, ctx: &egui::Context) {
        match target {
            Target::Browse { id, params } => {
                let route = match id.as_str() {
                    "FEmusic_home" => Route::Home,
                    "FEmusic_explore" => Route::Explore,
                    _ => LibTab::ALL.iter().find(|t| t.browse_id() == id).map(|t| Route::Library(*t)).unwrap_or(Route::Browse { id, params }),
                };
                self.navigate(route, ctx);
            }
            Target::Search { query, params } => {
                self.query = query.clone();
                self.navigate(Route::Search { query, params }, ctx);
            }
            w @ Target::Watch { .. } => self.play_target(w, false, ctx),
        }
    }

    // ---------- reproducción ----------

    fn play_target(&mut self, target: Target, shuffle: bool, ctx: &egui::Context) {
        match target {
            Target::Watch { video_id, playlist_id: Some(pl) } => {
                self.loading = true;
                let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
                self.rt.spawn(async move {
                    let songs = yt.queue(video_id.as_deref(), &pl).await;
                    let _ = tx.send(Msg::Queue { songs, start: video_id, shuffle });
                    ctx.request_repaint();
                });
            }
            Target::Watch { video_id: Some(v), playlist_id: None } => {
                let song = Song { video_id: v, ..Default::default() };
                self.play_with_radio(song, ctx);
            }
            other => self.open(other, ctx),
        }
    }

    /// Una canción sola y, detrás, su radio (como al pulsar una canción en YouTube Music).
    fn play_with_radio(&mut self, song: Song, ctx: &egui::Context) {
        self.queue = vec![song.clone()];
        self.play_index(0, ctx);
        self.fetch_radio(&song.video_id, false, ctx);
    }

    fn fetch_radio(&mut self, seed: &str, append: bool, ctx: &egui::Context) {
        let (yt, tx, ctx, seed) = (self.yt.clone(), self.tx.clone(), ctx.clone(), seed.to_string());
        self.radio_pending = true;
        self.rt.spawn(async move {
            match yt.radio(&seed).await {
                Ok(songs) => {
                    let _ = tx.send(Msg::Radio { seed, songs, append });
                }
                Err(e) => {
                    let _ = tx.send(Msg::Status(format!("Radio: {e:#}")));
                }
            }
            ctx.request_repaint();
        });
    }

    /// Clic en una canción de una página.
    fn play_in_page(&mut self, si: usize, ii: usize, ctx: &egui::Context) {
        let route = self.route().clone();
        let Some(PageState::Ready(page)) = self.pages.get(&route) else { return };
        let Some(section) = page.sections.get(si) else { return };
        let Some(song) = section.items.get(ii).and_then(Item::song) else { return };
        // Álbum o playlist: la cola es la de la playlist, empezando por esa canción.
        let header_pl = page.header.as_ref().and_then(|h| match &h.play {
            Some(Target::Watch { playlist_id: Some(pl), .. }) => Some(pl.clone()),
            _ => None,
        });
        if let Some(pl) = header_pl {
            self.play_target(Target::Watch { video_id: Some(song.video_id), playlist_id: Some(pl) }, false, ctx);
        } else if matches!(route, Route::Library(_)) {
            // Biblioteca (canciones, historial): la propia lista es la cola.
            let songs: Vec<Song> = section.items.iter().filter_map(Item::song).collect();
            let start = songs.iter().position(|s| s.video_id == song.video_id).unwrap_or(0);
            self.queue = songs;
            self.play_index(start, ctx);
        } else {
            self.play_with_radio(song, ctx);
        }
    }

    fn play_index(&mut self, i: usize, ctx: &egui::Context) {
        let Some(song) = self.queue.get(i).cloned() else { return };
        self.current = Some(i);
        self.loading = true;
        self.last_line = None;
        self.pending = Some((song.video_id.clone(), Instant::now()));
        self.player.load(&song.video_id);
        self.update_now_playing(&song);
        self.request_lyrics(&song, ctx);
        if self.yt.logged_in() && !self.liked.contains_key(&song.video_id) {
            let (yt, tx, ctx, id) = (self.yt.clone(), self.tx.clone(), ctx.clone(), song.video_id.clone());
            self.rt.spawn(async move {
                if let Ok(l) = yt.liked(&id).await {
                    let _ = tx.send(Msg::Liked(id, l));
                    ctx.request_repaint();
                }
            });
        }
    }

    fn update_now_playing(&mut self, song: &Song) {
        if let Some(m) = &mut self.media {
            m.set_track(&song.title, &song.artist, &song.album, &song.cover(544), song.duration);
        }
        let text = if song.artist.is_empty() { song.title.clone() } else { format!("{} — {}", song.title, song.artist) };
        self.desktop.set_now_playing(&text);
    }

    fn request_lyrics(&mut self, song: &Song, ctx: &egui::Context) {
        if song.title.is_empty() || self.lyrics.contains_key(&song.video_id) || self.lyrics_loading.as_deref() == Some(&song.video_id) {
            return;
        }
        self.lyrics_loading = Some(song.video_id.clone());
        let track = Track {
            video_id: song.video_id.clone(),
            title: song.title.clone(),
            artist: song.artist.clone(),
            album: song.album.clone(),
            duration: song.duration as f64,
        };
        let (yt, http, tx, ctx) = (self.yt.clone(), self.http.clone(), self.tx.clone(), ctx.clone());
        self.rt.spawn(async move {
            let (mut found, _) = lyrics::fetch(&http, &track).await;
            if matches!(found, Lyrics::None) {
                if let Ok(lines) = yt.page_lyrics(&track.video_id).await {
                    if !lines.is_empty() {
                        found = Lyrics::Plain(lines);
                    }
                }
            }
            let _ = tx.send(Msg::Lyrics { id: track.video_id, lyrics: found });
            ctx.request_repaint();
        });
    }

    fn next(&mut self, ctx: &egui::Context) {
        let Some(i) = self.current else { return };
        if i + 1 < self.queue.len() {
            self.play_index(i + 1, ctx);
        } else if self.settings.autoplay && !self.radio_pending {
            // Fin de la cola: seguir con canciones parecidas a la última.
            let seed = self.queue[i].video_id.clone();
            self.fetch_radio(&seed, true, ctx);
        }
    }

    fn previous(&mut self, ctx: &egui::Context) {
        if self.player.snapshot().position > Duration::from_secs(3) {
            self.player.seek(Duration::ZERO);
        } else if let Some(i) = self.current {
            self.play_index(i.saturating_sub(1), ctx);
        }
    }

    fn current_song(&self) -> Option<&Song> {
        self.current.and_then(|i| self.queue.get(i))
    }

    fn play_next(&mut self, song: Song) {
        let at = self.current.map_or(0, |i| i + 1);
        self.queue.insert(at.min(self.queue.len()), song);
    }

    fn remove_from_queue(&mut self, i: usize) {
        if i >= self.queue.len() || Some(i) == self.current {
            return;
        }
        self.queue.remove(i);
        if let Some(c) = self.current {
            if i < c {
                self.current = Some(c - 1);
            }
        }
    }

    fn move_in_queue(&mut self, from: usize, to: usize) {
        if from >= self.queue.len() || to > self.queue.len() || from == to {
            return;
        }
        let song = self.queue.remove(from);
        let to = if to > from { to - 1 } else { to };
        self.queue.insert(to, song);
        if let Some(c) = self.current {
            self.current = Some(if c == from {
                to
            } else if from < c && to >= c {
                c - 1
            } else if from > c && to <= c {
                c + 1
            } else {
                c
            });
        }
    }

    fn like(&mut self, id: String, like: bool, ctx: &egui::Context) {
        self.liked.insert(id.clone(), like);
        let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
        self.rt.spawn(async move {
            if let Err(e) = yt.like(&id, like).await {
                let _ = tx.send(Msg::Liked(id, !like));
                let _ = tx.send(Msg::Status(format!("Me gusta: {e:#}")));
            }
            ctx.request_repaint();
        });
    }

    fn apply(&mut self, acts: Vec<Act>, ctx: &egui::Context) {
        for a in acts {
            match a {
                Act::Open(t) => self.open(t, ctx),
                Act::Play(t) => self.play_target(t, false, ctx),
                Act::Shuffle(t) => self.play_target(t, true, ctx),
                Act::PlayInPage(s, i) => self.play_in_page(s, i, ctx),
                Act::PlayNext(s) => self.play_next(s),
                Act::AddToQueue(s) => self.queue.push(s),
                Act::Radio(id) => self.play_with_radio(Song { video_id: id, ..Default::default() }, ctx),
                Act::Like(id, l) => self.like(id, l, ctx),
                Act::LoadMore(si) => self.load_more(si, ctx),
                Act::Retry => {
                    let r = self.route().clone();
                    self.load(&r, ctx);
                }
            }
        }
    }

    fn load_more(&mut self, si: usize, ctx: &egui::Context) {
        let route = self.route().clone();
        let Some(PageState::Ready(page)) = self.pages.get(&route) else { return };
        let Some(token) = page.sections.get(si).and_then(|s| s.continuation.clone()) else { return };
        self.loading_more = Some((route.clone(), si));
        let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
        self.rt.spawn(async move {
            let r = yt.continuation(&token).await;
            let _ = tx.send(Msg::More(route, si, r));
            ctx.request_repaint();
        });
    }

    // ---------- mensajes y eventos ----------

    fn drain(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Page(route, res) => {
                    let state = match res {
                        Ok(p) => PageState::Ready(p),
                        Err(e) => PageState::Failed(format!("{e:#}")),
                    };
                    self.pages.insert(route, state);
                }
                Msg::More(route, si, res) => {
                    self.loading_more = None;
                    if let (Some(PageState::Ready(p)), Ok((items, next))) = (self.pages.get_mut(&route), res) {
                        if let Some(s) = p.sections.get_mut(si) {
                            s.items.extend(items);
                            s.continuation = next;
                        }
                    }
                }
                Msg::Queue { songs, start, shuffle } => match songs {
                    Ok(mut songs) if !songs.is_empty() => {
                        if shuffle {
                            shuffle_vec(&mut songs);
                        }
                        let i = start.and_then(|s| songs.iter().position(|x| x.video_id == s)).filter(|_| !shuffle).unwrap_or(0);
                        self.queue = songs;
                        self.play_index(i, ctx);
                    }
                    Ok(_) => {
                        self.loading = false;
                        self.set_status("—".into());
                    }
                    Err(e) => {
                        self.loading = false;
                        self.set_status(format!("{e:#}"));
                    }
                },
                Msg::Radio { seed, songs, append } => {
                    self.radio_pending = false;
                    let fresh: Vec<Song> = songs.into_iter().filter(|s| !self.queue.iter().any(|q| q.video_id == s.video_id)).collect();
                    if append {
                        let was_at_end = self.current.is_some_and(|c| c + 1 >= self.queue.len());
                        self.queue.extend(fresh);
                        if was_at_end && self.player.snapshot().finished.is_none() && !self.was_playing {
                            self.next(ctx);
                        }
                    } else if let Some(pos) = self.queue.iter().position(|s| s.video_id == seed) {
                        // La primera canción llega sin datos (solo el id): la radio los trae.
                        if let Some(first) = fresh.iter().find(|s| s.video_id == seed).cloned() {
                            self.queue[pos] = first;
                        }
                        let rest: Vec<Song> = fresh.into_iter().filter(|s| s.video_id != seed).collect();
                        self.queue.splice(pos + 1..pos + 1, rest);
                    }
                    // Datos de la canción actual si llegó solo con el id.
                    if let Some(song) = self.current_song().cloned() {
                        if !song.title.is_empty() {
                            self.update_now_playing(&song);
                            self.request_lyrics(&song, ctx);
                        }
                    }
                }
                Msg::Lyrics { id, lyrics } => {
                    if self.lyrics_loading.as_deref() == Some(&id) {
                        self.lyrics_loading = None;
                    }
                    self.lyrics.insert(id, lyrics);
                }
                Msg::Liked(id, l) => {
                    self.liked.insert(id, l);
                }
                Msg::Playlists(p) => self.playlists = p,
                Msg::QmSongs(q, songs) => {
                    if q == self.qm.asked {
                        self.qm.songs = songs;
                        self.qm.searching = false;
                    }
                }
                Msg::Status(s) => self.set_status(s),
            }
        }
        while let Ok(key) = self.media_rx.try_recv() {
            match key {
                MediaKey::Toggle => self.player.toggle(),
                MediaKey::Play => self.player.play(),
                MediaKey::Pause => self.player.pause(),
                MediaKey::Next => self.next(ctx),
                MediaKey::Previous => self.previous(ctx),
            }
        }
        for a in self.desktop.poll() {
            match a {
                Action::Show => self.show_window(ctx),
                Action::ToggleWindow => {
                    if self.hidden {
                        self.show_window(ctx)
                    } else {
                        self.hide_window(ctx)
                    }
                }
                Action::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                Action::PlayPause => self.player.toggle(),
                Action::Next => self.next(ctx),
                Action::Previous => self.previous(ctx),
                Action::QuickMenu => self.toggle_quick_menu(),
            }
        }
    }

    fn set_status(&mut self, s: String) {
        log::warn!("{s}");
        self.status = Some((s, Instant::now()));
    }

    fn show_window(&mut self, ctx: &egui::Context) {
        self.hidden = false;
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        self.hidden = true;
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    fn toggle_quick_menu(&mut self) {
        if self.qm.open {
            self.qm.open = false;
        } else {
            self.qm.show();
            self.qm_was_focused = false;
        }
    }

    fn sign_in(&mut self, ctx: &egui::Context) {
        self.player.pause();
        self.navigate(Route::Home, ctx);
        let rect = scaled(ctx, ctx.content_rect());
        self.player.open_login(rect);
    }

    fn sign_out(&mut self, ctx: &egui::Context) {
        self.player.logout();
        self.yt.set_session(None);
        self.playlists.clear();
        self.liked.clear();
        self.pages.clear();
        self.loading = false;
        self.load(&Route::Home, ctx);
    }

    /// Lo que no depende de pintar: llega aunque la ventana esté oculta en la bandeja.
    fn tick(&mut self, ctx: &egui::Context) {
        self.drain(ctx);
        self.player.poll();

        // Inicio de sesión abierto: se comprueba cada medio segundo si ya volvió a YouTube Music.
        if self.player.login_open() {
            if self.player.login_done() {
                self.player.close_login();
                self.yt.set_session(self.player.session());
                self.pages.clear();
                self.load(&Route::Home, ctx);
                self.load_playlists(ctx);
            } else {
                ctx.request_repaint_after(Duration::from_millis(500));
            }
        }

        // Fin de canción, cambios de play/pausa, primer sonido y notificación.
        let state = self.player.snapshot();
        let current_id = self.current_song().map(|s| s.video_id.clone());
        if let Some(done) = self.player.take_finished() {
            if Some(&done) == current_id.as_ref() {
                self.next(ctx);
            }
        }
        if self.loading && state.playing && state.id == current_id {
            self.loading = false;
        }
        if state.playing != self.was_playing {
            self.was_playing = state.playing;
            if let Some(m) = &mut self.media {
                m.set_playing(state.playing);
            }
            if state.playing {
                if let Some((id, t)) = self.pending.take() {
                    log::info!("primer sonido de {id}: {} ms desde el clic", t.elapsed().as_millis());
                }
            }
        }
        if state.playing && state.id.is_some() && state.id == current_id && self.notified != current_id {
            self.notified = current_id.clone();
            let focused = ctx.input(|i| i.viewport().focused.unwrap_or(false));
            if self.settings.notifications && (!focused || self.hidden) {
                if let Some(s) = self.current_song() {
                    desktop::notify(&s.title, &s.artist);
                }
            }
        }

        // Menú rápido: busca cuando se deja de escribir 250 ms.
        if self.qm.open {
            if let Some(at) = self.qm.typed_at {
                let q = self.qm.query.trim().to_string();
                if at.elapsed() >= Duration::from_millis(250) {
                    self.qm.typed_at = None;
                    if q != self.qm.asked {
                        self.qm.asked = q.clone();
                        if q.is_empty() {
                            self.qm.songs.clear();
                        } else {
                            self.qm.searching = true;
                            let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
                            self.rt.spawn(async move {
                                let songs = yt.search_songs(&q).await.unwrap_or_default();
                                let _ = tx.send(Msg::QmSongs(q, songs));
                                ctx.request_repaint();
                            });
                        }
                    }
                } else {
                    ctx.request_repaint_after(Duration::from_millis(260));
                }
            }
            if self.hidden {
                ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
            }
        }

        // Cerrar la ventana: a la bandeja (si está activado).
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            if self.settings.close_to_tray {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.hide_window(ctx);
            } else {
                self.quitting = true;
            }
        }
    }

    fn keyboard(&mut self, ctx: &egui::Context) {
        if self.recording {
            return;
        }
        let typing = ctx.egui_wants_keyboard_input();
        let (space, alt_l, alt_r, f5, f11, esc, ctrl_f, zoom_in, zoom_out, zoom_0, back_btn, fwd_btn, qm) = ctx.input(|i| {
            let alt = i.modifiers.alt;
            let ctrl = i.modifiers.command;
            (
                !typing && i.key_pressed(egui::Key::Space),
                alt && i.key_pressed(egui::Key::ArrowLeft),
                alt && i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::F5),
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
                ctrl && i.key_pressed(egui::Key::F),
                ctrl && (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)),
                ctrl && i.key_pressed(egui::Key::Minus),
                ctrl && i.key_pressed(egui::Key::Num0),
                i.pointer.button_pressed(egui::PointerButton::Extra1),
                i.pointer.button_pressed(egui::PointerButton::Extra2),
                i.events.iter().any(|e| matches!(e, egui::Event::Key { key, pressed: true, modifiers, repeat: false, .. }
                    if desktop::matches(&self.settings.quick_menu_shortcut, *modifiers, *key))),
            )
        });
        if space {
            self.player.toggle();
        }
        if alt_l || back_btn {
            self.back();
        }
        if alt_r || fwd_btn {
            self.forward();
        }
        if f5 {
            let r = self.route().clone();
            self.load(&r, ctx);
        }
        if f11 {
            self.set_fullscreen(!self.fullscreen, ctx);
        }
        if esc && self.fullscreen {
            self.set_fullscreen(false, ctx);
        }
        if ctrl_f {
            self.focus_search = true;
        }
        let z = ctx.zoom_factor();
        if zoom_in {
            ctx.set_zoom_factor((z + 0.1).min(2.0));
        }
        if zoom_out {
            ctx.set_zoom_factor((z - 0.1).max(0.6));
        }
        if zoom_0 {
            ctx.set_zoom_factor(1.0);
        }
        // El atajo del menú rápido con la ventana enfocada (si no es global ya lo recibe Windows).
        if qm && self.settings.quick_menu_reach == Reach::Focused {
            self.toggle_quick_menu();
        }
    }

    fn set_fullscreen(&mut self, on: bool, ctx: &egui::Context) {
        self.fullscreen = on;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(on));
        if on {
            self.navigate(Route::NowPlaying, ctx);
        }
    }

    // ---------- vistas ----------

    fn sidebar(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Act>) {
        let ctx = ui.ctx().clone();
        let l = self.l;
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(egui::include_image!("../assets/icon.png")).fit_to_exact_size(Vec2::splat(28.0)));
            ui.label(RichText::new("ByTune").size(20.0).strong());
        });
        ui.add_space(16.0);
        let current = self.route().clone();
        let nav = [
            (Route::Home, theme::icon::HOME, S::home),
            (Route::Explore, theme::icon::EXPLORE, S::explore),
            (Route::Library(LibTab::Playlists), theme::icon::LIBRARY, S::library),
            (Route::NowPlaying, theme::icon::NOW_PLAYING, S::now_playing),
        ];
        for (route, icon, label) in nav {
            let selected = match (&route, &current) {
                (Route::Library(_), Route::Library(_)) => true,
                (a, b) => a == b,
            };
            if ui::nav_item(ui, selected, icon, t(l, label)).clicked() {
                self.navigate(route, &ctx);
            }
        }
        ui.add_space(10.0);
        ui.separator();

        // Cuenta y ajustes abajo; tus playlists en medio (con portada, como en ByTune).
        let bottom_h = 120.0;
        let list_h = (ui.available_height() - bottom_h).max(40.0);
        egui::ScrollArea::vertical().id_salt("side-playlists").max_height(list_h).auto_shrink([false, true]).show(ui, |ui| {
            for p in &self.playlists {
                let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), Sense::click());
                if r.hovered() {
                    ui.painter().rect_filled(rect, CornerRadius::same(6), theme::HOVER);
                }
                let mut c = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(6.0, 3.0))).layout(Layout::left_to_right(Align::Center)));
                c.add(egui::Image::new(p.cover(96)).fit_to_exact_size(Vec2::splat(36.0)).corner_radius(CornerRadius::same(5)));
                c.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new(&p.title).size(13.0)).truncate().selectable(false));
                    ui.add(egui::Label::new(RichText::new(&p.subtitle).size(11.0).color(theme::TEXT_3)).truncate().selectable(false));
                });
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    if let Some(o) = p.open.clone() {
                        acts.push(Act::Open(o));
                    }
                }
            }
        });
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(12.0);
            if self.player.login_open() {
                if ui.add_sized([ui.available_width(), 32.0], egui::Button::new(t(l, S::cancel))).clicked() {
                    self.player.close_login();
                }
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(t(l, S::waiting_google)).color(theme::TEXT_2));
                });
            } else if !self.yt.logged_in() {
                let b = egui::Button::new(RichText::new(t(l, S::sign_in)).color(Color32::BLACK)).fill(theme::TEXT);
                if ui.add_sized([ui.available_width(), 34.0], b).clicked() {
                    self.sign_in(&ctx);
                }
            }
            if ui::nav_item(ui, current == Route::Settings, theme::icon::SETTINGS, t(l, S::settings)).clicked() {
                self.navigate(Route::Settings, &ctx);
            }
        });
    }

    /// Barra superior: atrás / adelante y buscador.
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(self.pos > 0, ui::icon_button(theme::icon::BACK, 34.0, false)).clicked() {
                self.back();
            }
            if ui.add_enabled(self.pos + 1 < self.history.len(), ui::icon_button(theme::icon::FORWARD, 34.0, false)).clicked() {
                self.forward();
            }
            ui.add_space(8.0);
            let w = (ui.available_width() - 40.0).clamp(160.0, 520.0);
            let edit = egui::TextEdit::singleline(&mut self.query)
                .hint_text(format!("{}  {}", theme::icon::SEARCH, t(self.l, S::search_hint)))
                .desired_width(w)
                .margin(egui::Margin::symmetric(14, 9));
            let r = ui.add(edit);
            if self.focus_search {
                r.request_focus();
                self.focus_search = false;
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.query.trim().is_empty() {
                let route = Route::Search { query: self.query.trim().to_string(), params: None };
                self.pages.remove(&route); // buscar otra vez lo mismo vuelve a pedirlo
                self.navigate(route.clone(), &ctx);
                if !self.pages.contains_key(&route) {
                    self.load(&route, &ctx);
                }
            }
        });
        ui.add_space(6.0);
    }

    fn page_view(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Act>) {
        let route = self.route().clone();
        let l = self.l;
        let playing = self.current_song().map(|s| s.video_id.clone());
        let loading_more = self.loading_more.as_ref().filter(|(r, _)| *r == route).map(|(_, s)| *s);

        egui::ScrollArea::vertical().id_salt(("page", format!("{route:?}"))).auto_shrink(false).show(ui, |ui| {
            ui.set_width(ui.available_width() - 8.0);
            match &route {
                Route::Home => {
                    ui.add_space(8.0);
                    ui.label(RichText::new(t(l, S::home)).size(30.0).strong());
                }
                Route::Explore => {
                    ui.add_space(8.0);
                    ui.label(RichText::new(t(l, S::explore)).size(30.0).strong());
                }
                Route::Library(tab) => {
                    ui.add_space(8.0);
                    ui.label(RichText::new(t(l, S::library)).size(30.0).strong());
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for tb in LibTab::ALL {
                            if ui.add(egui::Button::selectable(*tab == tb, t(l, tb.label())).corner_radius(CornerRadius::same(16))).clicked() {
                                acts.push(Act::Open(Target::Browse { id: tb.browse_id().into(), params: None }));
                            }
                        }
                    });
                    ui.add_space(10.0);
                    if !self.yt.logged_in() {
                        ui::notice(ui, t(l, S::need_login));
                        return;
                    }
                }
                Route::Search { query, .. } => {
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{} «{query}»", t(l, S::search))).size(26.0).strong());
                    ui.add_space(6.0);
                }
                _ => {}
            }
            match self.pages.get(&route) {
                None | Some(PageState::Loading) => {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| ui.spinner());
                }
                Some(PageState::Failed(e)) => {
                    ui::notice(ui, &format!("{}: {e}", t(l, S::error)));
                    ui.vertical_centered(|ui| {
                        if ui.button(t(l, S::retry)).clicked() {
                            acts.push(Act::Retry);
                        }
                    });
                }
                Some(PageState::Ready(p)) => {
                    if p.sections.is_empty() && p.header.is_none() {
                        ui::notice(ui, "—");
                    }
                    ui::page(ui, l, p, playing.as_deref(), loading_more, acts);
                }
            }
        });
    }

    fn queue_view(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Act>) {
        let ctx = ui.ctx().clone();
        let l = self.l;
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(t(l, S::queue)).size(18.0).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !self.queue.is_empty() && ui.small_button(t(l, S::clear_queue)).clicked() {
                    // Se queda solo la que suena.
                    if let Some(c) = self.current {
                        let s = self.queue[c].clone();
                        self.queue = vec![s];
                        self.current = Some(0);
                    } else {
                        self.queue.clear();
                    }
                }
            });
        });
        ui.add_space(6.0);
        let mut play = None;
        let mut moved = None;
        let mut remove = None;
        egui::ScrollArea::vertical().id_salt("queue").auto_shrink(false).show(ui, |ui| {
            for i in 0..self.queue.len() {
                let song = &self.queue[i];
                let item = Item {
                    kind: Kind::Song,
                    title: if song.title.is_empty() { "…".into() } else { song.title.clone() },
                    subtitle: song.artist.clone(),
                    thumbnail: song.thumbnail.clone(),
                    duration: song.duration,
                    video_id: Some(song.video_id.clone()),
                    ..Default::default()
                };
                let active = Some(i) == self.current;
                // Arrastrar para reordenar (como queue-reorder.js en ByTune).
                let id = egui::Id::unique(("queue-row", i, &song.video_id));
                let r = ui.dnd_drag_source(id, i, |ui| ui::row(ui, &item, active, None)).inner;
                if let Some(from) = r.dnd_release_payload::<usize>() {
                    moved = Some((*from, i));
                }
                if r.dnd_hover_payload::<usize>().is_some() {
                    let y = r.rect.top();
                    ui.painter().hline(r.rect.x_range(), y, egui::Stroke::new(2.0, theme::ACCENT));
                }
                if r.clicked() {
                    play = Some(i);
                }
                let s = song.clone();
                r.context_menu(|ui| {
                    if !active && ui.button(t(l, S::remove)).clicked() {
                        remove = Some(i);
                        ui.close();
                    }
                    ui::song_menu(ui, l, &s, None, acts);
                });
            }
            if self.radio_pending {
                ui.spinner();
            }
        });
        if let Some((from, to)) = moved {
            self.move_in_queue(from, to);
        }
        if let Some(i) = remove {
            self.remove_from_queue(i);
        }
        if let Some(i) = play {
            self.play_index(i, &ctx);
        }
    }

    fn player_bar(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Act>) {
        let ctx = ui.ctx().clone();
        let l = self.l;
        let mut state = self.player.snapshot();
        let song = self.current_song().cloned();
        // Lo que la web tenga por su cuenta (al abrir restaura la última canción) no cuenta.
        if song.as_ref().map(|s| &s.video_id) != state.id.as_ref() {
            state = Default::default();
        }
        let total = state.duration.map(|d| d.as_secs_f32()).or(song.as_ref().map(|s| s.duration as f32)).unwrap_or(0.0);

        ui.add_space(8.0);
        ui.columns(3, |cols| {
            cols[0].horizontal(|ui| {
                if let Some(s) = &song {
                    let img = ui.add(egui::Image::new(s.cover(120)).fit_to_exact_size(Vec2::splat(56.0)).corner_radius(CornerRadius::same(6)).sense(Sense::click()));
                    if img.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.navigate(Route::NowPlaying, &ctx);
                    }
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        ui.set_max_width((ui.available_width() - 48.0).max(60.0));
                        ui.add(egui::Label::new(RichText::new(&s.title).strong()).truncate());
                        let r = ui.add(egui::Label::new(RichText::new(&s.artist).color(theme::TEXT_2)).truncate().sense(Sense::click()));
                        if let Some(id) = &s.artist_id {
                            if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                acts.push(Act::Open(Target::Browse { id: id.clone(), params: None }));
                            }
                        }
                    });
                    if let Some(liked) = self.liked.get(&s.video_id).copied() {
                        let icon = if liked { theme::icon::HEART_FILL } else { theme::icon::HEART };
                        if ui.add(ui::icon_button(icon, 34.0, false)).clicked() {
                            acts.push(Act::Like(s.video_id.clone(), !liked));
                        }
                    }
                }
            });

            cols[1].vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    let w = 44.0 + 36.0 * 2.0 + 16.0;
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    if ui.add(ui::icon_button(theme::icon::PREV, 36.0, false)).clicked() {
                        self.previous(&ctx);
                    }
                    let icon = if state.playing { theme::icon::PAUSE } else { theme::icon::PLAY };
                    if self.loading {
                        ui.add_sized([44.0, 44.0], egui::Spinner::new().size(28.0).color(theme::TEXT));
                    } else if ui.add(ui::icon_button(icon, 44.0, true)).clicked() && song.is_some() {
                        self.player.toggle();
                    }
                    if ui.add(ui::icon_button(theme::icon::NEXT, 36.0, false)).clicked() {
                        self.next(&ctx);
                    }
                });
                ui.horizontal(|ui| {
                    let pos = self.seek_drag.unwrap_or(state.position.as_secs_f32());
                    ui.label(RichText::new(ui::fmt_time(pos)).size(12.0).color(theme::TEXT_3));
                    let mut v = pos;
                    ui.spacing_mut().slider_width = (ui.available_width() - 48.0).max(60.0);
                    let r = ui.add(egui::Slider::new(&mut v, 0.0..=total.max(1.0)).show_value(false).trailing_fill(true));
                    if r.dragged() || r.changed() {
                        self.seek_drag = Some(v);
                    }
                    if r.drag_stopped() || (r.changed() && !r.dragged()) {
                        self.player.seek(Duration::from_secs_f32(v));
                        self.seek_drag = None;
                    }
                    ui.label(RichText::new(ui::fmt_time(total)).size(12.0).color(theme::TEXT_3));
                });
            });

            cols[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
                let np = *self.route() == Route::NowPlaying;
                if ui.add(egui::Button::selectable(np, RichText::new(theme::icon::LYRICS).size(16.0)).min_size(Vec2::splat(34.0))).on_hover_text(t(l, S::lyrics)).clicked() {
                    if np {
                        self.back();
                    } else {
                        self.navigate(Route::NowPlaying, &ctx);
                    }
                }
                ui.spacing_mut().slider_width = 110.0;
                let mut vol = self.settings.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                    self.settings.volume = vol;
                    self.settings_dirty = true;
                    // Curva cuadrática: el oído percibe el volumen de forma logarítmica.
                    self.player.set_volume(vol * vol);
                }
                let icon = if vol <= 0.0 { theme::icon::MUTE } else { theme::icon::VOLUME };
                ui.label(RichText::new(icon).size(16.0).color(theme::TEXT_2));
            });
        });
    }

    fn quick_menu(&mut self, ctx: &egui::Context) {
        if !self.qm.open {
            return;
        }
        let builder = egui::ViewportBuilder::default()
            .with_title("ByTune")
            .with_inner_size([640.0, 460.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_resizable(false)
            .with_taskbar(false);
        let l = self.l;
        let st = self.player.snapshot();
        let playing = st.playing && st.id.is_some() && st.id.as_deref() == self.current_song().map(|s| s.video_id.as_str());
        let playlists = self.playlists.clone();
        let recent = self.settings.recent.clone();
        let mut outs = Vec::new();
        let mut focused_now = false;
        let qm = &mut self.qm;
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("quick-menu"), builder, |ui, _class| {
            focused_now = ui.input(|i| i.viewport().focused.unwrap_or(false));
            if ui.input(|i| i.viewport().close_requested()) {
                outs.push(QmOut::Close);
            }
            egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BG)).show(ui, |ui| {
                outs.extend(qm.ui(ui, l, &playlists, &recent, playing));
            });
        });
        // Se cierra al perder el foco (como en ByTune).
        if focused_now {
            self.qm_was_focused = true;
        } else if self.qm_was_focused {
            outs.push(QmOut::Close);
        }
        for o in outs {
            match o {
                QmOut::Close => self.qm.open = false,
                QmOut::PlayPause => self.player.toggle(),
                QmOut::Next => self.next(ctx),
                QmOut::Previous => self.previous(ctx),
                QmOut::Queue(s) => match self.settings.shift_enter {
                    ShiftEnter::Queue => self.queue.push(s),
                    ShiftEnter::Next => self.play_next(s),
                },
                QmOut::Play(e) => {
                    self.qm.open = false;
                    match e {
                        ui::quickmenu::Entry::Song(s) => {
                            self.settings.push_recent(Recent {
                                title: s.title.clone(),
                                subtitle: s.artist.clone(),
                                thumbnail: s.thumbnail.clone(),
                                video_id: Some(s.video_id.clone()),
                                playlist_id: None,
                                browse_id: None,
                            });
                            self.play_with_radio(s, ctx);
                        }
                        ui::quickmenu::Entry::Open { title, subtitle, thumbnail, play } => {
                            if let Target::Watch { video_id, playlist_id } = &play {
                                self.settings.push_recent(Recent {
                                    title,
                                    subtitle,
                                    thumbnail,
                                    video_id: video_id.clone(),
                                    playlist_id: playlist_id.clone(),
                                    browse_id: None,
                                });
                            }
                            self.play_target(play, false, ctx);
                        }
                    }
                    self.settings_dirty = true;
                }
            }
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        self.keyboard(ctx);
        if std::mem::take(&mut self.settings_dirty) {
            if let Some(s) = frame.storage_mut() {
                self.settings.save(s);
            }
        }
        if let Some((_, at)) = &self.status {
            if at.elapsed() > Duration::from_secs(6) {
                self.status = None;
            }
        }

        let mut acts = Vec::new();
        let full = self.fullscreen;
        egui::Panel::bottom("player")
            .exact_size(96.0)
            .frame(egui::Frame::new().fill(theme::BAR).inner_margin(egui::Margin::symmetric(16, 8)))
            .show(ui, |ui| self.player_bar(ui, &mut acts));
        if !full {
            egui::Panel::left("nav")
                .resizable(true)
                .default_size(240.0)
                .size_range(200.0..=360.0)
                .frame(egui::Frame::new().fill(theme::GLASS).inner_margin(egui::Margin::symmetric(12, 0)))
                .show(ui, |ui| self.sidebar(ui, &mut acts));
            if *self.route() != Route::Settings {
                egui::Panel::right("queue")
                    .resizable(true)
                    .default_size(340.0)
                    .size_range(260.0..=480.0)
                    .frame(egui::Frame::new().fill(theme::GLASS).inner_margin(egui::Margin::symmetric(12, 0)))
                    .show(ui, |ui| self.queue_view(ui, &mut acts));
            }
        }
        let mut video_rect = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(egui::Margin::symmetric(24, 0)))
            .show(ui, |ui| {
                // El login de Google ocupa la zona central (el WebView va encima).
                if self.player.login_open() {
                    self.player.place(scaled(ctx, ui.max_rect()));
                    return;
                }
                if let Some((s, _)) = &self.status {
                    ui.add_space(6.0);
                    ui.colored_label(theme::DANGER, s);
                }
                match self.route().clone() {
                    Route::NowPlaying => {
                        ui.add_space(8.0);
                        match self.current_song().cloned() {
                            None => ui::notice(ui, t(self.l, S::nothing_playing)),
                            Some(song) => {
                                let st = self.player.snapshot();
                                let mine = st.id.as_deref() == Some(song.video_id.as_str());
                                let np = ui::nowplaying::Np {
                                    l: self.l,
                                    song: &song,
                                    lyrics: self.lyrics.get(&song.video_id),
                                    pos: if mine { st.position.as_secs_f64() } else { 0.0 },
                                    playing: mine && st.playing,
                                    lyrics_open: self.settings.lyrics_open,
                                    video: self.want_video,
                                    has_video: mine && st.has_video,
                                    liked: self.liked.get(&song.video_id).copied(),
                                    fullscreen: self.fullscreen,
                                };
                                let out = ui::nowplaying::show(ui, np, &mut self.last_line, &mut acts);
                                video_rect = out.video_rect;
                                if let Some(t) = out.seek {
                                    self.player.seek(Duration::from_secs_f64(t));
                                }
                                if out.toggle_lyrics {
                                    self.settings.lyrics_open = !self.settings.lyrics_open;
                                    self.settings_dirty = true;
                                }
                                if let Some(v) = out.set_video {
                                    self.want_video = v;
                                    self.player.set_av(v);
                                }
                                if out.toggle_fullscreen {
                                    self.set_fullscreen(!self.fullscreen, ctx);
                                }
                            }
                        }
                    }
                    Route::Settings => {
                        let out = ui::settings::show(
                            ui,
                            ui::settings::View {
                                l: self.l,
                                s: &mut self.settings,
                                recording: &mut self.recording,
                                shortcut_error: &mut self.shortcut_error,
                                logged_in: self.yt.logged_in(),
                            },
                        );
                        self.apply_settings(out, ctx);
                    }
                    _ => {
                        self.top_bar(ui);
                        self.page_view(ui, &mut acts);
                    }
                }
            });
        self.player.show_video(video_rect.map(|r| scaled(ctx, r)));
        self.apply(acts, ctx);
        self.quick_menu(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.settings.save(storage);
    }
}

impl App {
    fn apply_settings(&mut self, out: ui::settings::SettingsOut, ctx: &egui::Context) {
        if out.changed {
            self.settings_dirty = true;
        }
        if let Some(on) = out.start_with_windows {
            match desktop::set_start_with_windows(on) {
                Ok(()) => self.settings.start_with_windows = on,
                Err(e) => self.set_status(format!("Inicio con Windows: {e}")),
            }
        }
        if out.lang {
            self.l = L::resolve(self.settings.lang, &self.sys_lang);
            self.desktop.set_lang(self.l);
            self.yt.set_lang(self.l.hl());
            self.pages.clear();
            self.load(&Route::Home, ctx);
        }
        if out.quick_menu {
            let global = (self.settings.quick_menu_reach == Reach::Anywhere).then_some(self.settings.quick_menu_shortcut.as_str());
            if !self.desktop.set_quick_menu_global(global) {
                self.shortcut_error = Some(t(self.l, S::set_qm_invalid).into());
            }
        }
        if out.sign_in {
            self.sign_in(ctx);
        }
        if out.sign_out {
            self.sign_out(ctx);
        }
    }
}

/// Puntos de egui a píxeles lógicos de la ventana (cambian con el zoom de la app).
fn scaled(ctx: &egui::Context, r: egui::Rect) -> egui::Rect {
    let z = ctx.zoom_factor();
    egui::Rect::from_min_max((r.min.to_vec2() * z).to_pos2(), (r.max.to_vec2() * z).to_pos2())
}

/// Mezcla sin dependencias (xorshift con la hora como semilla).
fn shuffle_vec<T>(v: &mut [T]) {
    let mut x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    for i in (1..v.len()).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        v.swap(i, (x % (i as u64 + 1)) as usize);
    }
}

/// Idioma del sistema ("es", "en"...).
fn sys_lang() -> String {
    sys_locale::get_locale()
        .and_then(|l| l.split(['-', '_']).next().map(str::to_lowercase))
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| "en".into())
}

#[cfg(windows)]
fn window_handle(cc: &eframe::CreationContext<'_>) -> Option<*mut std::ffi::c_void> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match cc.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as *mut std::ffi::c_void),
        _ => None,
    }
}

#[cfg(not(windows))]
fn window_handle(_: &eframe::CreationContext<'_>) -> Option<*mut std::ffi::c_void> {
    None
}
