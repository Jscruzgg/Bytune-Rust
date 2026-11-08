//! Estado de la app y vistas: búsqueda, cola, Now Playing con letras y barra del reproductor.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use egui::{Align, Color32, CornerRadius, Layout, RichText, Sense, Vec2};

use crate::innertube::{Client, Song};
use crate::lyrics::{self, lrc::Line, query::Track, timing, Lyrics};
use crate::media::{Media, MediaKey};
use crate::theme;
use crate::webplayer::WebPlayer;

/// Resultados de las tareas de red, de vuelta al hilo de la UI.
enum Msg {
    Search { seq: u64, result: anyhow::Result<Vec<Song>> },
    Radio { seed: String, songs: Vec<Song> },
    Lyrics { id: String, lyrics: Lyrics },
}

#[derive(PartialEq, Clone, Copy)]
enum View {
    Search,
    NowPlaying,
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

    view: View,
    query: String,
    search_seq: u64,
    searching: bool,
    results: Vec<Song>,

    queue: Vec<Song>,
    current: Option<usize>,
    /// La canción pedida y cuándo, para medir el tiempo hasta el primer sonido.
    pending: Option<(String, Instant)>,
    loading: bool,
    status: Option<String>,

    lyrics: HashMap<String, Lyrics>,
    lyrics_loading: Option<String>,
    last_line: Option<usize>,

    seek_drag: Option<f32>,
    volume: f32,
    was_playing: bool,
    /// Reproducir el primer resultado de la próxima búsqueda (BYTUNE_AUTOPLAY).
    autoplay: bool,
}

const VOLUME_KEY: &str = "volume";

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, rt: tokio::runtime::Handle) -> Self {
        theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let volume = cc.storage.and_then(|s| s.get_string(VOLUME_KEY)).and_then(|v| v.parse().ok()).unwrap_or(0.8f32);
        let http = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("cliente HTTP");
        let hl = sys_lang();
        let (tx, rx) = crossbeam_channel::unbounded();

        let ctx = cc.egui_ctx.clone();
        let mut player = WebPlayer::new(cc, move || ctx.request_repaint()).expect("no se pudo crear el WebView2");
        player.set_volume(volume * volume);
        // Las cookies de la sesión ya están en el perfil del WebView si se inició antes.
        let yt = Client::new(http.clone(), &hl, player.session());

        let (media_tx, media_rx) = crossbeam_channel::unbounded();
        let ctx = cc.egui_ctx.clone();
        let media = Media::new(window_handle(cc), media_tx, move || ctx.request_repaint());

        let mut app = Self {
            rt,
            http,
            yt,
            tx,
            rx,
            player,
            media,
            media_rx,
            view: View::Search,
            query: String::new(),
            search_seq: 0,
            searching: false,
            results: Vec::new(),
            queue: Vec::new(),
            current: None,
            pending: None,
            loading: false,
            status: None,
            lyrics: HashMap::new(),
            lyrics_loading: None,
            last_line: None,
            seek_drag: None,
            volume,
            was_playing: false,
            autoplay: false,
        };
        // BYTUNE_AUTOPLAY="búsqueda": busca y reproduce el primer resultado al arrancar.
        // Escenario repetible para pruebas y para la comparativa de rendimiento.
        if let Ok(q) = std::env::var("BYTUNE_AUTOPLAY") {
            app.query = q;
            app.autoplay = true;
            app.search(&cc.egui_ctx);
        }
        app
    }

    fn login(&mut self, ctx: &egui::Context) {
        self.player.pause();
        // Al principio, la ventana entera; desde el siguiente fotograma se ajusta a la zona
        // central (CentralPanel), para que siga viéndose el botón de cancelar.
        let rect = ctx.content_rect();
        self.player.open_login(rect);
    }

    fn logout(&mut self) {
        self.player.logout();
        self.yt.set_session(None);
        self.loading = false;
    }

    // ---------- acciones ----------

    fn search(&mut self, ctx: &egui::Context) {
        let q = self.query.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.search_seq += 1;
        self.searching = true;
        self.view = View::Search;
        let (yt, tx, seq, ctx) = (self.yt.clone(), self.tx.clone(), self.search_seq, ctx.clone());
        self.rt.spawn(async move {
            let result = yt.search_songs(&q).await;
            let _ = tx.send(Msg::Search { seq, result });
            ctx.request_repaint();
        });
    }

    /// Reproduce una canción de los resultados: la cola pasa a ser esa canción + su radio.
    fn play_from_results(&mut self, i: usize, ctx: &egui::Context) {
        let song = self.results[i].clone();
        self.queue = vec![song.clone()];
        self.play_index(0, ctx);
        let (yt, tx, ctx) = (self.yt.clone(), self.tx.clone(), ctx.clone());
        self.rt.spawn(async move {
            match yt.radio(&song.video_id).await {
                Ok(songs) => {
                    let _ = tx.send(Msg::Radio { seed: song.video_id, songs });
                    ctx.request_repaint();
                }
                Err(e) => log::warn!("radio: {e:#}"),
            }
        });
    }

    fn play_index(&mut self, i: usize, ctx: &egui::Context) {
        let Some(song) = self.queue.get(i).cloned() else { return };
        self.current = Some(i);
        self.loading = true;
        self.status = None;
        self.last_line = None;
        self.pending = Some((song.video_id.clone(), Instant::now()));
        self.player.load(&song.video_id);
        if let Some(m) = &mut self.media {
            m.set_track(&song.title, &song.artist, &song.album, &song.cover(544), song.duration);
        }
        self.request_lyrics(&song, ctx);
    }

    fn request_lyrics(&mut self, song: &Song, ctx: &egui::Context) {
        if self.lyrics.contains_key(&song.video_id) || self.lyrics_loading.as_deref() == Some(&song.video_id) {
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
        if let Some(i) = self.current {
            if i + 1 < self.queue.len() {
                self.play_index(i + 1, ctx);
            }
        }
    }

    fn previous(&mut self, ctx: &egui::Context) {
        // Como en cualquier reproductor: pasados 3 s vuelve al principio de la misma.
        if self.player.snapshot().position > Duration::from_secs(3) {
            self.player.seek(Duration::ZERO);
        } else if let Some(i) = self.current {
            self.play_index(i.saturating_sub(1), ctx);
        }
    }

    fn current_song(&self) -> Option<&Song> {
        self.current.and_then(|i| self.queue.get(i))
    }

    // ---------- mensajes ----------

    fn drain(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Search { seq, result } if seq == self.search_seq => {
                    self.searching = false;
                    match result {
                        Ok(r) => self.results = r,
                        Err(e) => self.status = Some(format!("Búsqueda: {e:#}")),
                    }
                    if std::mem::take(&mut self.autoplay) && !self.results.is_empty() {
                        self.play_from_results(0, ctx);
                        self.view = View::NowPlaying;
                    }
                }
                Msg::Search { .. } => {}
                Msg::Radio { seed, songs } => {
                    // Solo si la cola sigue siendo la de esa canción.
                    if self.queue.len() == 1 && self.queue[0].video_id == seed {
                        self.queue.extend(songs.into_iter().filter(|s| s.video_id != seed));
                    }
                }
                Msg::Lyrics { id, lyrics } => {
                    if self.lyrics_loading.as_deref() == Some(&id) {
                        self.lyrics_loading = None;
                    }
                    self.lyrics.insert(id, lyrics);
                }
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
    }

    // ---------- vistas ----------

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(egui::include_image!("../assets/icon.png")).fit_to_exact_size(Vec2::splat(28.0)));
            ui.label(RichText::new("ByTune").size(20.0).strong());
        });
        ui.add_space(20.0);
        for (view, label) in [(View::Search, "Buscar"), (View::NowPlaying, "Reproduciendo")] {
            let selected = self.view == view;
            let r = ui.add_sized([ui.available_width(), 36.0], egui::Button::selectable(selected, RichText::new(label).size(15.0)));
            if r.clicked() {
                self.view = view;
            }
        }

        // Cuenta, abajo del todo.
        let ctx = ui.ctx().clone();
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(16.0);
            if self.yt.logged_in() {
                if ui.add_sized([ui.available_width(), 32.0], egui::Button::new("Cerrar sesión")).clicked() {
                    self.logout();
                }
                ui.label(RichText::new("Sesión de Google iniciada").color(theme::TEXT_3).size(12.0));
            } else if self.player.login_open() {
                if ui.add_sized([ui.available_width(), 32.0], egui::Button::new("Cancelar")).clicked() {
                    self.player.close_login();
                }
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Esperando a Google…").color(theme::TEXT_2));
                });
            } else {
                let b = egui::Button::new(RichText::new("Iniciar sesión con Google").color(Color32::BLACK)).fill(theme::TEXT);
                if ui.add_sized([ui.available_width(), 36.0], b).clicked() {
                    self.login(&ctx);
                }
                ui.label(RichText::new("Para tu biblioteca y recomendaciones").color(theme::TEXT_3).size(12.0));
            }
        });
    }

    fn search_view(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            let edit = egui::TextEdit::singleline(&mut self.query)
                .hint_text("Buscar canciones")
                .desired_width(ui.available_width().min(480.0))
                .margin(egui::Margin::symmetric(14, 10))
                .font(egui::TextStyle::Body);
            let r = ui.add(edit);
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search(&ctx);
            }
            if self.searching {
                ui.spinner();
            }
        });
        if let Some(s) = &self.status {
            ui.colored_label(theme::DANGER, s);
        }
        ui.add_space(8.0);

        let mut clicked = None;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, 64.0, self.results.len(), |ui, range| {
            for i in range {
                let song = &self.results[i];
                if song_row(ui, song, false).double_clicked() {
                    clicked = Some(i);
                }
            }
        });
        if let Some(i) = clicked {
            self.play_from_results(i, &ctx);
        }
    }

    fn now_playing(&mut self, ui: &mut egui::Ui) {
        let Some(song) = self.current_song().cloned() else {
            ui.centered_and_justified(|ui| ui.label(RichText::new("Nada sonando").color(theme::TEXT_3).size(18.0)));
            return;
        };
        let state = self.player.snapshot();
        let pos = state.position.as_secs_f64();
        let avail = ui.available_size();
        let art = (avail.y - 120.0).min(avail.x * 0.42).clamp(160.0, 560.0);

        ui.horizontal_top(|ui| {
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.add_space(24.0);
                ui.add(egui::Image::new(song.cover(544)).fit_to_exact_size(Vec2::splat(art)).corner_radius(CornerRadius::same(12)));
                ui.add_space(12.0);
                ui.label(RichText::new(&song.title).size(22.0).strong());
                ui.label(RichText::new(&song.artist).size(16.0).color(theme::TEXT_2));
            });
            ui.add_space(32.0);
            ui.vertical(|ui| {
                ui.add_space(24.0);
                self.lyrics_view(ui, &song.video_id, pos);
            });
        });
    }

    fn lyrics_view(&mut self, ui: &mut egui::Ui, id: &str, pos: f64) {
        match self.lyrics.get(id) {
            None => {
                ui.spinner();
            }
            Some(Lyrics::None) => {
                ui.label(RichText::new("Sin letra para esta canción").color(theme::TEXT_3).size(18.0));
            }
            Some(Lyrics::Plain(lines)) => {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    for l in lines {
                        ui.label(RichText::new(l).size(22.0).color(theme::TEXT_2));
                    }
                });
            }
            Some(Lyrics::Synced(lines)) => {
                let lines: &Vec<Line> = lines;
                let times: Vec<f64> = lines.iter().map(|l| l.t).collect();
                let active = timing::active_index(&times, pos);
                let changed = active != self.last_line;
                let mut seek = None;
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.add_space(ui.available_height() * 0.3);
                    for (i, l) in lines.iter().enumerate() {
                        let is_active = Some(i) == active;
                        let color = if is_active { theme::TEXT } else if active.is_some_and(|a| i < a) { theme::TEXT_OFF } else { theme::TEXT_3 };
                        let size = if l.gap { 20.0 } else { 30.0 };
                        let r = ui.add(egui::Label::new(RichText::new(&l.text).size(size).strong().color(color)).sense(Sense::click()));
                        if r.clicked() {
                            seek = Some(l.t);
                        }
                        if is_active && changed {
                            r.scroll_to_me(Some(Align::Center));
                        }
                        ui.add_space(10.0);
                    }
                    ui.add_space(ui.available_height() * 0.5);
                });
                self.last_line = active;
                if let Some(t) = seek {
                    self.player.seek(Duration::from_secs_f64(t));
                }
                // Mientras suena, se repinta justo cuando toca la siguiente línea.
                if self.was_playing {
                    if let Some(ms) = timing::ms_until_next(&times, pos, 1.0) {
                        ui.ctx().request_repaint_after(Duration::from_millis(ms.max(16)));
                    }
                }
            }
        }
    }

    fn queue_view(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.add_space(16.0);
        ui.label(RichText::new("Cola").size(18.0).strong());
        ui.add_space(8.0);
        let mut clicked = None;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, 64.0, self.queue.len(), |ui, range| {
            for i in range {
                if song_row(ui, &self.queue[i], Some(i) == self.current).clicked() {
                    clicked = Some(i);
                }
            }
        });
        if let Some(i) = clicked {
            self.play_index(i, &ctx);
        }
    }

    fn player_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let state = self.player.snapshot();
        let song = self.current_song().cloned();
        let total = state.duration.map(|d| d.as_secs_f32()).or(song.as_ref().map(|s| s.duration as f32)).unwrap_or(0.0);

        ui.add_space(8.0);
        ui.columns(3, |cols| {
            // Izquierda: canción.
            cols[0].horizontal(|ui| {
                if let Some(s) = &song {
                    ui.add(egui::Image::new(s.cover(120)).fit_to_exact_size(Vec2::splat(52.0)).corner_radius(CornerRadius::same(6)));
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        ui.add(egui::Label::new(RichText::new(&s.title).strong()).truncate());
                        ui.add(egui::Label::new(RichText::new(&s.artist).color(theme::TEXT_2)).truncate());
                    });
                }
            });

            // Centro: controles y barra de progreso.
            cols[1].vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    let w = 44.0 * 3.0 + 16.0;
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    if ui.add(round_button("⏮", 36.0, false)).clicked() {
                        self.previous(&ctx);
                    }
                    let icon = if self.loading { "…" } else if state.playing { "⏸" } else { "▶" };
                    if ui.add(round_button(icon, 44.0, true)).clicked() && !self.loading {
                        self.player.toggle();
                    }
                    if ui.add(round_button("⏭", 36.0, false)).clicked() {
                        self.next(&ctx);
                    }
                });
                ui.horizontal(|ui| {
                    let pos = self.seek_drag.unwrap_or(state.position.as_secs_f32());
                    ui.label(RichText::new(fmt_time(pos)).size(12.0).color(theme::TEXT_3));
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
                    ui.label(RichText::new(fmt_time(total)).size(12.0).color(theme::TEXT_3));
                });
            });

            // Derecha: volumen y Now Playing.
            cols[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
                let np = self.view == View::NowPlaying;
                if ui.add(egui::Button::selectable(np, "Letra")).clicked() {
                    self.view = if np { View::Search } else { View::NowPlaying };
                }
                ui.spacing_mut().slider_width = 110.0;
                let mut vol = self.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                    self.volume = vol;
                    // Curva cuadrática: el oído percibe el volumen de forma logarítmica.
                    self.player.set_volume(vol * vol);
                }
                ui.label(RichText::new("🔊").color(theme::TEXT_2));
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        self.drain(ctx);

        self.player.poll();

        // Inicio de sesión abierto: se comprueba cada medio segundo si ya volvió a YouTube Music.
        if self.player.login_open() {
            if self.player.login_done() {
                self.player.close_login();
                self.yt.set_session(self.player.session());
                self.status = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(500));
            }
        }

        // Fin de canción, cambios de play/pausa y primer sonido.
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
        // Espacio: play/pausa (si no se está escribiendo).
        if !ctx.egui_wants_keyboard_input() && ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.player.toggle();
        }

        egui::Panel::bottom("player")
            .exact_size(96.0)
            .frame(egui::Frame::new().fill(theme::BAR).inner_margin(egui::Margin::symmetric(16, 8)))
            .show(ui, |ui| self.player_bar(ui));
        egui::Panel::left("nav")
            .resizable(true)
            .default_size(220.0)
            .size_range(180.0..=320.0)
            .frame(egui::Frame::new().fill(theme::GLASS).inner_margin(egui::Margin::symmetric(12, 0)))
            .show(ui, |ui| self.sidebar(ui));
        egui::Panel::right("queue")
            .resizable(true)
            .default_size(340.0)
            .size_range(260.0..=480.0)
            .frame(egui::Frame::new().fill(theme::GLASS).inner_margin(egui::Margin::symmetric(12, 0)))
            .show(ui, |ui| self.queue_view(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(egui::Margin::symmetric(20, 0)))
            .show(ui, |ui| {
                // El login de Google ocupa la zona central (el WebView va encima).
                if self.player.login_open() {
                    self.player.place(ui.max_rect());
                    return;
                }
                match self.view {
                    View::Search => self.search_view(ui),
                    View::NowPlaying => self.now_playing(ui),
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(VOLUME_KEY, self.volume.to_string());
    }
}

// ---------- piezas ----------

fn song_row(ui: &mut egui::Ui, song: &Song, active: bool) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::click());
    let fill = if active {
        theme::SELECTED
    } else if r.hovered() {
        theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(8.0, 4.0))).layout(Layout::left_to_right(Align::Center)));
    child.add(egui::Image::new(song.cover(120)).fit_to_exact_size(Vec2::splat(48.0)).corner_radius(CornerRadius::same(6)));
    child.vertical(|ui| {
        ui.add_space(4.0);
        let title = RichText::new(&song.title).strong().color(if active { theme::ACCENT } else { theme::TEXT });
        ui.add(egui::Label::new(title).truncate().selectable(false));
        let sub = if song.album.is_empty() { song.artist.clone() } else { format!("{} · {}", song.artist, song.album) };
        ui.add(egui::Label::new(RichText::new(sub).color(theme::TEXT_2)).truncate().selectable(false));
    });
    if song.duration > 0 {
        child.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(fmt_time(song.duration as f32)).color(theme::TEXT_3));
        });
    }
    ui.add_space(8.0);
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn round_button(icon: &str, size: f32, filled: bool) -> egui::Button<'_> {
    let text = RichText::new(icon).size(size * 0.45).color(if filled { Color32::BLACK } else { theme::TEXT });
    egui::Button::new(text)
        .min_size(Vec2::splat(size))
        .corner_radius(CornerRadius::same((size / 2.0) as u8))
        .fill(if filled { theme::TEXT } else { Color32::TRANSPARENT })
}

fn fmt_time(secs: f32) -> String {
    let s = secs.max(0.0) as u32;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Idioma del sistema ("es", "en"...), para InnerTube.
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
