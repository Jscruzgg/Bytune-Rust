//! El audio sale del reproductor oficial de music.youtube.com, en un WebView2 hijo de la
//! ventana de egui y oculto. La interfaz sigue siendo nativa: desde aquí solo se le dan
//! órdenes (bridge.js) y se recibe su estado.
//!
//! El mismo WebView sirve para iniciar sesión: se muestra encima de la ventana con la página
//! de Google y se oculta al volver a music.youtube.com con la sesión iniciada.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use wry::WebViewBuilderExtWindows;
use wry::dpi::{LogicalPosition, LogicalSize};

use crate::login::{self, Session};

const HOME: &str = "https://music.youtube.com/";
const BRIDGE: &str = include_str!("bridge.js");

#[derive(Debug, Clone, Default)]
pub struct State {
    /// El reproductor de la página ya está listo para recibir órdenes.
    pub ready: bool,
    pub id: Option<String>,
    pub playing: bool,
    pub buffering: bool,
    pub position: Duration,
    pub duration: Option<Duration>,
    /// La canción pedida terminó (la UI pasa a la siguiente). Se consume con `take_finished`.
    pub finished: Option<String>,
    /// La canción tiene vídeo (el conmutador canción/vídeo de la web está activo).
    pub has_video: bool,
    /// Lo que está puesto en el conmutador: true = vídeo.
    pub video: bool,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
enum FromPage {
    State {
        ready: bool,
        id: Option<String>,
        pos: f64,
        dur: f64,
        playing: bool,
        buffering: bool,
        #[serde(default, rename = "hasVideo")]
        has_video: bool,
        #[serde(default)]
        av: Option<String>,
    },
    Ended { id: String },
}

pub struct WebPlayer {
    view: wry::WebView,
    state: Arc<Mutex<State>>,
    login_open: bool,
    /// Última orden pendiente hasta que el reproductor esté listo.
    pending_load: Option<String>,
    volume: f32,
    /// Dónde está el WebView ahora (None = 1×1, escondido).
    placed: Option<egui::Rect>,
    video_mode: bool,
}

impl WebPlayer {
    pub fn new(window: &impl raw_window_handle::HasWindowHandle, repaint: impl Fn() + Send + Sync + 'static) -> Result<Self> {
        let state = Arc::new(Mutex::new(State::default()));
        let st = state.clone();
        let mut context = wry::WebContext::new(Some(login::webview_dir()));
        let view = wry::WebViewBuilder::new_with_web_context(&mut context)
            .with_url(HOME)
            .with_initialization_script(BRIDGE)
            .with_autoplay(true)
            // Chromium no empieza a cargar audio en una página oculta (lo aplaza hasta que se
            // vea), así que el WebView queda "visible" pero de 1×1 px. Sin el cálculo de
            // oclusión tampoco se considera oculto con la ventana minimizada o tapada.
            .with_additional_browser_args(format!(
                "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,CalculateNativeWinOcclusion \
                 --autoplay-policy=no-user-gesture-required{}",
                // Solo desarrollo: BYTUNE_DEBUG_PORT=9222 abre DevTools por CDP (como YTM_DEBUG_PORT en ByTune).
                std::env::var("BYTUNE_DEBUG_PORT").map(|p| format!(" --remote-debugging-port={p}")).unwrap_or_default()
            ))
            .with_visible(true)
            .with_devtools(cfg!(debug_assertions))
            .with_bounds(wry::Rect { position: LogicalPosition::new(0, 0).into(), size: LogicalSize::new(1, 1).into() })
            .with_ipc_handler(move |req| {
                log::debug!("página: {}", req.body());
                let Ok(msg) = serde_json::from_str::<FromPage>(req.body()) else { return };
                let mut s = st.lock().unwrap();
                match msg {
                    FromPage::State { ready, id, pos, dur, playing, buffering, has_video, av } => {
                        s.has_video = has_video;
                        s.video = av.as_deref() == Some("video");
                        if ready != s.ready {
                            log::info!("reproductor web {}", if ready { "listo" } else { "no disponible" });
                        }
                        s.ready = ready;
                        s.id = id;
                        s.position = Duration::from_secs_f64(pos.max(0.0));
                        s.duration = (dur > 0.0).then(|| Duration::from_secs_f64(dur));
                        s.playing = playing;
                        s.buffering = buffering;
                    }
                    FromPage::Ended { id } => s.finished = Some(id),
                }
                drop(s);
                repaint();
            })
            .build_as_child(window)
            .map_err(|e| anyhow!("WebView2: {e}"))?;
        // El contexto solo hace falta al crear el WebView, pero WebView2 lo usa mientras viva.
        std::mem::forget(context);
        Ok(Self { view, state, login_open: false, pending_load: None, volume: 1.0, placed: None, video_mode: false })
    }

    /// Canción o vídeo (el conmutador de la web).
    pub fn set_av(&self, video: bool) {
        self.js(&format!("window.__bt.av('{}')", if video { "video" } else { "song" }));
    }

    /// Muestra el vídeo de la web ocupando `rect`, o lo esconde (None). Se llama en cada
    /// fotograma con la zona de la portada en Now Playing.
    pub fn show_video(&mut self, rect: Option<egui::Rect>) {
        if self.login_open {
            return;
        }
        let on = rect.is_some();
        if on != self.video_mode {
            self.video_mode = on;
            self.js(&format!("window.__bt.videoMode({on})"));
        }
        let key = |r: egui::Rect| [r.min.x, r.min.y, r.max.x, r.max.y].map(|v| v.round() as i32);
        if rect.map(key) != self.placed.map(key) {
            match rect {
                Some(r) => self.place(r),
                None => self.hide(),
            }
            self.placed = rect;
        }
    }

    fn hide(&self) {
        let _ = self.view.set_bounds(wry::Rect { position: LogicalPosition::new(0, 0).into(), size: LogicalSize::new(1, 1).into() });
    }

    pub fn snapshot(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    pub fn take_finished(&self) -> Option<String> {
        self.state.lock().unwrap().finished.take()
    }

    fn js(&self, code: &str) {
        if let Err(e) = self.view.evaluate_script(&format!("window.__bt && {code}")) {
            log::warn!("evaluate_script: {e}");
        }
    }

    pub fn load(&mut self, id: &str) {
        if self.snapshot().ready {
            self.js(&format!("window.__bt.load({})", serde_json::to_string(id).unwrap()));
            self.pending_load = None;
        } else {
            self.pending_load = Some(id.to_string());
        }
    }

    /// Llamar en cada fotograma: manda la orden que esperaba a que la página estuviera lista.
    pub fn poll(&mut self) {
        if self.pending_load.is_some() && self.snapshot().ready {
            let id = self.pending_load.take().unwrap();
            self.load(&id);
            self.set_volume(self.volume);
        }
    }

    pub fn toggle(&self) {
        self.js("window.__bt.toggle()");
    }
    pub fn play(&self) {
        self.js("window.__bt.play()");
    }
    pub fn pause(&self) {
        self.js("window.__bt.pause()");
    }
    pub fn seek(&self, to: Duration) {
        self.js(&format!("window.__bt.seek({})", to.as_secs_f64()));
    }
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v;
        self.js(&format!("window.__bt.volume({v})"));
    }

    // ---------- inicio de sesión ----------

    pub fn login_open(&self) -> bool {
        self.login_open
    }

    /// Muestra la página de Google ocupando `rect` (en puntos lógicos de la ventana).
    pub fn open_login(&mut self, rect: egui::Rect) {
        if self.video_mode {
            self.video_mode = false;
            self.js("window.__bt.videoMode(false)");
        }
        self.login_open = true;
        let _ = self.view.load_url(login::START);
        self.place(rect);
        let _ = self.view.set_visible(true);
    }

    pub fn place(&self, rect: egui::Rect) {
        let _ = self.view.set_bounds(wry::Rect {
            position: LogicalPosition::new(rect.min.x as f64, rect.min.y as f64).into(),
            size: LogicalSize::new(rect.width() as f64, rect.height() as f64).into(),
        });
    }

    pub fn close_login(&mut self) {
        self.login_open = false;
        self.hide();
        self.placed = None;
        let _ = self.view.focus_parent();
        if !self.view.url().is_ok_and(|u| u.starts_with(HOME)) {
            let _ = self.view.load_url(HOME);
        }
    }

    /// La sesión de Google del WebView, si la hay (para las peticiones de InnerTube).
    pub fn session(&self) -> Option<Session> {
        login::session_from_webview(&self.view)
    }

    /// Borra cookies y caché del WebView y vuelve a cargar YouTube Music sin sesión.
    pub fn logout(&mut self) {
        let _ = self.view.clear_all_browsing_data();
        self.state.lock().unwrap().ready = false;
        let _ = self.view.load_url(HOME);
    }

    /// Mientras el login está abierto: ¿ya volvió a YouTube Music con la sesión iniciada?
    pub fn login_done(&self) -> bool {
        self.view.url().is_ok_and(|u| u.starts_with(HOME)) && self.session().is_some()
    }
}
