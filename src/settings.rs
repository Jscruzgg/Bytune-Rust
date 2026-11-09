//! Ajustes persistentes (los mismos que ByTune). Se guardan en el almacenamiento de eframe
//! (%APPDATA%\ByTune-rs) como JSON.

use serde::{Deserialize, Serialize};

pub const KEY: &str = "settings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Lang {
    #[default]
    Auto,
    Es,
    En,
}

/// Desde dónde abre el menú rápido su atajo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Reach {
    /// Solo con la ventana de ByTune enfocada.
    #[default]
    Focused,
    /// Desde cualquier app (atajo global).
    Anywhere,
}

/// Qué hace Shift+Enter en el menú rápido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShiftEnter {
    /// Añadir al final de la cola.
    #[default]
    Queue,
    /// Reproducir a continuación.
    Next,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub notifications: bool,
    pub close_to_tray: bool,
    pub start_with_windows: bool,
    pub lang: Lang,
    pub volume: f32,
    pub lyrics_open: bool,
    /// Al acabar la cola, seguir con canciones parecidas (radio).
    pub autoplay: bool,
    pub quick_menu_shortcut: String,
    pub quick_menu_reach: Reach,
    pub shift_enter: ShiftEnter,
    /// Lo último abierto desde el menú rápido (ids de canciones/playlists), para "Recientes".
    pub recent: Vec<Recent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recent {
    pub title: String,
    pub subtitle: String,
    pub thumbnail: String,
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub browse_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notifications: true,
            close_to_tray: true,
            start_with_windows: false,
            lang: Lang::Auto,
            volume: 0.8,
            lyrics_open: true,
            autoplay: true,
            quick_menu_shortcut: "Ctrl+Shift+Space".into(),
            quick_menu_reach: Reach::Focused,
            shift_enter: ShiftEnter::Queue,
            recent: Vec::new(),
        }
    }
}

impl Settings {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|s| s.get_string(KEY))
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(j) = serde_json::to_string(self) {
            storage.set_string(KEY, j);
        }
    }

    pub fn push_recent(&mut self, r: Recent) {
        self.recent.retain(|x| x != &r && !(x.title == r.title && x.subtitle == r.subtitle));
        self.recent.insert(0, r);
        self.recent.truncate(8);
    }
}
