//! Ajustes: los mismos que el menú de ajustes de ByTune.

use egui::{RichText, Vec2};

use crate::i18n::{t, L, S};
use crate::settings::{Lang, Reach, Settings, ShiftEnter};
use crate::theme;

/// Cambios que la app tiene que aplicar fuera de los ajustes (registro, atajo global...).
#[derive(Default)]
pub struct SettingsOut {
    pub changed: bool,
    pub start_with_windows: Option<bool>,
    pub lang: bool,
    pub quick_menu: bool,
    pub sign_in: bool,
    pub sign_out: bool,
}

pub struct View<'a> {
    pub l: L,
    pub s: &'a mut Settings,
    /// Grabando un atajo nuevo para el menú rápido.
    pub recording: &'a mut bool,
    pub shortcut_error: &'a mut Option<String>,
    pub logged_in: bool,
}

fn title(ui: &mut egui::Ui, text: &str) {
    ui.add_space(20.0);
    ui.label(RichText::new(text).size(18.0).strong());
    ui.add_space(6.0);
}

pub fn show(ui: &mut egui::Ui, v: View) -> SettingsOut {
    let mut out = SettingsOut::default();
    let l = v.l;
    let s = v.s;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.set_max_width(640.0);
        ui.add_space(16.0);
        ui.label(RichText::new(t(l, S::settings)).size(30.0).strong());

        title(ui, t(l, S::set_general));
        out.changed |= ui.checkbox(&mut s.notifications, t(l, S::set_notify)).changed();
        out.changed |= ui.checkbox(&mut s.close_to_tray, t(l, S::set_close_to_tray)).changed();
        let mut sww = s.start_with_windows;
        if ui.checkbox(&mut sww, t(l, S::set_start_with_windows)).changed() {
            out.start_with_windows = Some(sww);
        }
        out.changed |= ui.checkbox(&mut s.autoplay, t(l, S::set_autoplay)).changed();
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(t(l, S::set_lang));
            for (lang, name) in [(Lang::Auto, t(l, S::set_lang_auto)), (Lang::Es, "Español"), (Lang::En, "English")] {
                if ui.selectable_label(s.lang == lang, name).clicked() && s.lang != lang {
                    s.lang = lang;
                    out.changed = true;
                    out.lang = true;
                }
            }
        });

        title(ui, t(l, S::set_quick_menu));
        ui.horizontal(|ui| {
            ui.label(t(l, S::set_qm_shortcut));
            if *v.recording {
                ui.label(RichText::new(t(l, S::set_qm_record)).color(theme::ACCENT));
                // La siguiente pulsación con modificador es el atajo nuevo.
                let pressed = ui.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        egui::Event::Key { key, pressed: true, modifiers, .. } => Some((*modifiers, *key)),
                        _ => None,
                    })
                });
                if let Some((mods, key)) = pressed {
                    if key == egui::Key::Escape {
                        *v.recording = false;
                    } else if let Some(acc) = crate::desktop::shortcut_from(mods, key) {
                        *v.recording = false;
                        if crate::desktop::valid_shortcut(&acc) {
                            s.quick_menu_shortcut = acc;
                            *v.shortcut_error = None;
                            out.changed = true;
                            out.quick_menu = true;
                        } else {
                            *v.shortcut_error = Some(t(l, S::set_qm_invalid).into());
                        }
                    }
                }
            } else {
                ui.add(egui::Label::new(RichText::new(&s.quick_menu_shortcut).monospace()));
                if ui.button(t(l, S::set_qm_change)).clicked() {
                    *v.recording = true;
                }
            }
        });
        if let Some(e) = v.shortcut_error.as_ref() {
            ui.colored_label(theme::DANGER, e);
        }
        ui.horizontal(|ui| {
            ui.label(t(l, S::set_qm_reach));
            for (r, name) in [(Reach::Focused, S::set_qm_focused), (Reach::Anywhere, S::set_qm_anywhere)] {
                if ui.selectable_label(s.quick_menu_reach == r, t(l, name)).clicked() && s.quick_menu_reach != r {
                    s.quick_menu_reach = r;
                    out.changed = true;
                    out.quick_menu = true;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(t(l, S::set_qm_shift_enter));
            for (r, name) in [(ShiftEnter::Queue, S::set_qm_queue), (ShiftEnter::Next, S::set_qm_next)] {
                if ui.selectable_label(s.shift_enter == r, t(l, name)).clicked() {
                    s.shift_enter = r;
                    out.changed = true;
                }
            }
        });

        title(ui, t(l, S::set_shortcuts));
        egui::Grid::new("shortcuts").num_columns(2).spacing(Vec2::new(24.0, 6.0)).show(ui, |ui| {
            let rows: &[(&str, &str, &str)] = &[
                (&s.quick_menu_shortcut, "Menú rápido", "Quick menu"),
                ("Ctrl+Alt+→ / ←", "Siguiente / anterior (global)", "Next / previous (global)"),
                ("Ctrl+Alt+M", "Mostrar / ocultar la ventana (global)", "Show / hide the window (global)"),
                (if l == L::Es { "Espacio" } else { "Space" }, "Reproducir / pausa", "Play / pause"),
                ("Alt+← / →", "Atrás / adelante", "Back / forward"),
                ("Ctrl+F", "Buscar", "Search"),
                ("F5", "Recargar la página", "Reload the page"),
                ("F11 / Esc", "Pantalla completa / salir", "Fullscreen / exit"),
                ("Ctrl + / - / 0", "Zoom", "Zoom"),
            ];
            for (keys, es, en) in rows {
                ui.label(RichText::new(*keys).monospace().color(theme::TEXT_2));
                ui.label(if l == L::Es { *es } else { *en });
                ui.end_row();
            }
        });

        title(ui, t(l, S::set_account));
        if v.logged_in {
            ui.label(RichText::new(t(l, S::signed_in)).color(theme::TEXT_2));
            if ui.button(t(l, S::sign_out)).clicked() {
                out.sign_out = true;
            }
        } else if ui.button(t(l, S::sign_in)).clicked() {
            out.sign_in = true;
        }

        title(ui, t(l, S::set_about));
        ui.label(format!("ByTune-rs {}", env!("CARGO_PKG_VERSION")));
        ui.hyperlink_to("github.com/Jscruzgg/Bytune-Rust", "https://github.com/Jscruzgg/Bytune-Rust");
        ui.add_space(40.0);
    });
    out
}
