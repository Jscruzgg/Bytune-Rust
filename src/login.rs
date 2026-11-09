//! Sesión de Google.
//!
//! El inicio de sesión se hace en el WebView del reproductor (webplayer.rs), que guarda las
//! cookies en su perfil (`webview_dir`), como un navegador. Aquí solo se leen las cookies de
//! sesión de ese perfil para firmar las peticiones a InnerTube igual que la web.

use std::time::{SystemTime, UNIX_EPOCH};

use sha1::{Digest, Sha1};

pub const START: &str = "https://accounts.google.com/ServiceLogin?service=youtube&continue=https%3A%2F%2Fmusic.youtube.com%2F";
const ORIGIN: &str = "https://music.youtube.com";

/// Solo las cookies que identifican la sesión.
const KEEP: &[&str] = &[
    "SID", "HSID", "SSID", "APISID", "SAPISID", "LOGIN_INFO",
    "__Secure-1PSID", "__Secure-3PSID", "__Secure-1PAPISID", "__Secure-3PAPISID",
    "__Secure-1PSIDTS", "__Secure-3PSIDTS", "__Secure-1PSIDCC", "__Secure-3PSIDCC", "SIDCC",
];

#[derive(Debug, Clone)]
pub struct Session {
    /// Cabecera Cookie lista para enviar.
    pub cookie: String,
    sapisid: String,
}

impl Session {
    fn from_cookie(cookie: String) -> Option<Self> {
        let sapisid = cookie
            .split("; ")
            .filter_map(|kv| kv.split_once('='))
            .find(|(k, _)| *k == "SAPISID" || *k == "__Secure-3PAPISID")
            .map(|(_, v)| v.to_string())?;
        Some(Self { cookie, sapisid })
    }

    /// Cabecera Authorization que la web de YouTube Music manda con la sesión iniciada.
    pub fn authorization(&self) -> String {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let hash = Sha1::digest(format!("{ts} {} {ORIGIN}", self.sapisid));
        format!("SAPISIDHASH {ts}_{}", hex::encode(hash))
    }
}

/// Perfil del WebView (cookies, caché) en %LOCALAPPDATA%\ByTune-rs\webview.
pub fn webview_dir() -> std::path::PathBuf {
    directories::ProjectDirs::from("", "", "ByTune-rs")
        .map(|d| d.data_local_dir().join("webview"))
        .unwrap_or_else(|| std::env::temp_dir().join("ByTune-rs-webview"))
}

pub fn session_from_webview(view: &wry::WebView) -> Option<Session> {
    let cookies = view.cookies_for_url(&format!("{ORIGIN}/")).ok()?;
    let cookie = cookies
        .iter()
        .filter(|c| KEEP.contains(&c.name()))
        .map(|c| format!("{}={}", c.name(), c.value()))
        .collect::<Vec<_>>()
        .join("; ");
    Session::from_cookie(cookie)
}
