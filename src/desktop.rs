//! Integración con Windows: icono de la bandeja, atajos globales, notificaciones e inicio con
//! Windows. Equivalente a lo que hace src/main.js en ByTune.

use std::collections::HashMap;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::i18n::{t, L, S};

/// Lo que piden la bandeja y los atajos globales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Show,
    ToggleWindow,
    Quit,
    PlayPause,
    Next,
    Previous,
    QuickMenu,
}

/// Atajos globales fijos (los de ByTune).
const PLAYER_SHORTCUTS: &[(&str, Action)] = &[
    ("Ctrl+Alt+ArrowRight", Action::Next),
    ("Ctrl+Alt+ArrowLeft", Action::Previous),
    ("Ctrl+Alt+M", Action::ToggleWindow),
];

pub struct Desktop {
    tray: Option<TrayIcon>,
    now_item: Option<MenuItem>,
    menu_ids: HashMap<String, Action>,
    hotkeys: Option<GlobalHotKeyManager>,
    hotkey_ids: HashMap<u32, Action>,
    quick_menu_key: Option<HotKey>,
}

impl Desktop {
    /// `wake` despierta a la app cuando llega algo (clic en la bandeja, atajo global).
    pub fn new(lang: L, wake: impl Fn() + Send + Sync + Clone + 'static) -> Self {
        let w = wake.clone();
        TrayIconEvent::set_event_handler(Some(move |e| {
            TRAY_EVENTS.lock().unwrap().push(e);
            w();
        }));
        let w = wake.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            MENU_EVENTS.lock().unwrap().push(e.id().0.clone());
            w();
        }));
        let w = wake;
        GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
            if e.state == HotKeyState::Pressed {
                HOTKEY_EVENTS.lock().unwrap().push(e.id);
                w();
            }
        }));

        let mut d = Self { tray: None, now_item: None, menu_ids: HashMap::new(), hotkeys: None, hotkey_ids: HashMap::new(), quick_menu_key: None };
        d.build_tray(lang);
        match GlobalHotKeyManager::new() {
            Ok(m) => {
                for (s, action) in PLAYER_SHORTCUTS {
                    if let Ok(hk) = s.parse::<HotKey>() {
                        if m.register(hk).is_ok() {
                            d.hotkey_ids.insert(hk.id(), *action);
                        } else {
                            log::warn!("atajo global ocupado: {s}");
                        }
                    }
                }
                d.hotkeys = Some(m);
            }
            Err(e) => log::warn!("atajos globales: {e}"),
        }
        d
    }

    fn build_tray(&mut self, lang: L) {
        let menu = Menu::new();
        let now = MenuItem::new("ByTune", false, None);
        let items = [
            (MenuItem::new(t(lang, S::tray_play_pause), true, None), Action::PlayPause),
            (MenuItem::new(t(lang, S::tray_next), true, None), Action::Next),
            (MenuItem::new(t(lang, S::tray_prev), true, None), Action::Previous),
            (MenuItem::new(t(lang, S::tray_show), true, None), Action::Show),
            (MenuItem::new(t(lang, S::tray_quit), true, None), Action::Quit),
        ];
        let _ = menu.append(&now);
        let _ = menu.append(&PredefinedMenuItem::separator());
        self.menu_ids.clear();
        for (i, (item, action)) in items.iter().enumerate() {
            if i == 3 {
                let _ = menu.append(&PredefinedMenuItem::separator());
            }
            let _ = menu.append(item);
            self.menu_ids.insert(item.id().0.clone(), *action);
        }
        let icon = crate::theme::app_icon();
        let icon = tray_icon::Icon::from_rgba(icon.rgba, icon.width, icon.height).ok();
        let mut b = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("ByTune").with_menu_on_left_click(false);
        if let Some(i) = icon {
            b = b.with_icon(i);
        }
        match b.build() {
            Ok(tray) => self.tray = Some(tray),
            Err(e) => log::warn!("bandeja: {e}"),
        }
        self.now_item = Some(now);
    }

    /// Rehace el menú (cambio de idioma).
    pub fn set_lang(&mut self, lang: L) {
        self.tray = None;
        self.build_tray(lang);
    }

    /// Canción actual en el menú y el tooltip.
    pub fn set_now_playing(&self, text: &str) {
        let label = if text.is_empty() { "ByTune".to_string() } else { truncate(text, 60) };
        if let Some(i) = &self.now_item {
            i.set_text(&label);
        }
        if let Some(t) = &self.tray {
            let _ = t.set_tooltip(Some(truncate(&format!("ByTune · {label}"), 120)));
        }
    }

    /// Registra el atajo del menú rápido como global (o lo quita). false si no se pudo.
    pub fn set_quick_menu_global(&mut self, accel: Option<&str>) -> bool {
        let Some(m) = &self.hotkeys else { return accel.is_none() };
        if let Some(old) = self.quick_menu_key.take() {
            let _ = m.unregister(old);
            self.hotkey_ids.remove(&old.id());
        }
        let Some(accel) = accel else { return true };
        match accel.parse::<HotKey>() {
            Ok(hk) if m.register(hk).is_ok() => {
                self.hotkey_ids.insert(hk.id(), Action::QuickMenu);
                self.quick_menu_key = Some(hk);
                true
            }
            _ => false,
        }
    }

    /// Lo que pasó desde la última llamada.
    pub fn poll(&self) -> Vec<Action> {
        let mut out = Vec::new();
        for e in std::mem::take(&mut *TRAY_EVENTS.lock().unwrap()) {
            match e {
                TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => out.push(Action::Show),
                TrayIconEvent::DoubleClick { .. } => out.push(Action::Show),
                _ => {}
            }
        }
        for id in std::mem::take(&mut *MENU_EVENTS.lock().unwrap()) {
            if let Some(a) = self.menu_ids.get(&id) {
                out.push(*a);
            }
        }
        for id in std::mem::take(&mut *HOTKEY_EVENTS.lock().unwrap()) {
            if let Some(a) = self.hotkey_ids.get(&id) {
                out.push(*a);
            }
        }
        out
    }
}

static TRAY_EVENTS: std::sync::Mutex<Vec<TrayIconEvent>> = std::sync::Mutex::new(Vec::new());
static MENU_EVENTS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
static HOTKEY_EVENTS: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max - 1).collect::<String>() + "…"
    }
}

/// ¿Es un atajo que se puede registrar? (al menos un modificador y una tecla).
pub fn valid_shortcut(accel: &str) -> bool {
    accel.contains('+') && accel.parse::<HotKey>().is_ok()
}

/// Atajo de egui ("Ctrl+Shift+Space") a partir de una pulsación.
pub fn shortcut_from(mods: egui::Modifiers, key: egui::Key) -> Option<String> {
    if !(mods.ctrl || mods.alt || mods.shift || mods.command) {
        return None;
    }
    let mut parts = Vec::new();
    if mods.ctrl || mods.command {
        parts.push("Ctrl".to_string());
    }
    if mods.alt {
        parts.push("Alt".into());
    }
    if mods.shift {
        parts.push("Shift".into());
    }
    let name = match key {
        egui::Key::ArrowLeft => "ArrowLeft".into(),
        egui::Key::ArrowRight => "ArrowRight".into(),
        egui::Key::ArrowUp => "ArrowUp".into(),
        egui::Key::ArrowDown => "ArrowDown".into(),
        k => k.name().to_string(),
    };
    parts.push(name);
    Some(parts.join("+"))
}

/// ¿Coincide la pulsación con el atajo guardado?
pub fn matches(accel: &str, mods: egui::Modifiers, key: egui::Key) -> bool {
    shortcut_from(mods, key).is_some_and(|s| s.eq_ignore_ascii_case(accel))
}

/// Notificación de Windows al cambiar de canción.
pub fn notify(title: &str, artist: &str) {
    let (title, artist) = (title.to_string(), artist.to_string());
    // En otro hilo: la API de notificaciones puede tardar y no debe frenar la interfaz.
    std::thread::spawn(move || {
        if let Err(e) = notify_rust::Notification::new().summary(&title).body(&artist).appname("ByTune").show() {
            log::warn!("notificación: {e}");
        }
    });
}

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_NAME: &str = "ByTune-rs";

/// Iniciar con Windows: entrada en HKCU\...\Run con la ruta del ejecutable.
pub fn set_start_with_windows(on: bool) -> std::io::Result<()> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(RUN_KEY)?;
    if on {
        let exe = std::env::current_exe()?;
        key.set_value(RUN_NAME, &format!("\"{}\" --background", exe.display()))
    } else {
        match key.delete_value(RUN_NAME) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

pub fn starts_with_windows() -> bool {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    hkcu.open_subkey(RUN_KEY).and_then(|k| k.get_value::<String, _>(RUN_NAME)).is_ok()
}
