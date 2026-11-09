//! Piezas de la interfaz. Las vistas no cambian el estado de la app: devuelven `Act`s que
//! app.rs aplica después, así cada vista solo pinta.

pub mod nowplaying;
pub mod quickmenu;
pub mod settings;

use egui::{Align, Color32, CornerRadius, Layout, RichText, Sense, Vec2};

use crate::i18n::{t, L, S};
use crate::innertube::Song;
use crate::pages::{Item, Kind, Layout as SecLayout, Page, Target};
use crate::theme;

/// Algo que la vista pide a la app.
#[derive(Debug, Clone)]
pub enum Act {
    /// Abrir un destino (página, búsqueda) o reproducirlo (Watch).
    Open(Target),
    /// Reproducir un destino (botón de play de una tarjeta o de la cabecera).
    Play(Target),
    Shuffle(Target),
    /// Pulsar una canción de la página: (sección, elemento).
    PlayInPage(usize, usize),
    PlayNext(Song),
    AddToQueue(Song),
    Radio(String),
    Like(String, bool),
    LoadMore(usize),
    Retry,
}

pub fn fmt_time(secs: f32) -> String {
    let s = secs.max(0.0) as u32;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

pub fn icon_button(icon: &str, size: f32, filled: bool) -> egui::Button<'_> {
    let text = RichText::new(icon).size(size * 0.42).color(if filled { Color32::BLACK } else { theme::TEXT });
    egui::Button::new(text)
        .min_size(Vec2::splat(size))
        .corner_radius(CornerRadius::same((size / 2.0) as u8))
        .fill(if filled { theme::TEXT } else { Color32::TRANSPARENT })
}

/// Botón con icono y texto, redondeado (Reproducir, Aleatorio, Radio...).
pub fn pill(ui: &mut egui::Ui, icon: &str, text: &str, filled: bool) -> egui::Response {
    let label = RichText::new(format!("{icon}  {text}")).size(14.0).color(if filled { Color32::BLACK } else { theme::TEXT });
    ui.add(egui::Button::new(label).min_size(Vec2::new(0.0, 36.0)).corner_radius(CornerRadius::same(18)).fill(if filled { theme::TEXT } else { theme::SELECTED }))
}

fn cover(ui: &mut egui::Ui, url: &str, size: f32, round: bool) -> egui::Response {
    let radius = if round { (size / 2.0) as u8 } else { 6 };
    if url.is_empty() {
        let (rect, r) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::same(radius), theme::SELECTED);
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, theme::icon::MUSIC, egui::FontId::proportional(size * 0.35), theme::TEXT_3);
        return r;
    }
    ui.add(egui::Image::new(url).fit_to_exact_size(Vec2::splat(size)).corner_radius(CornerRadius::same(radius)))
}

/// Menú contextual de una canción (clic derecho o "⋯").
pub fn song_menu(ui: &mut egui::Ui, l: L, song: &Song, liked: Option<bool>, acts: &mut Vec<Act>) {
    if ui.button(t(l, S::play_next)).clicked() {
        acts.push(Act::PlayNext(song.clone()));
        ui.close();
    }
    if ui.button(t(l, S::add_to_queue)).clicked() {
        acts.push(Act::AddToQueue(song.clone()));
        ui.close();
    }
    if ui.button(t(l, S::start_radio)).clicked() {
        acts.push(Act::Radio(song.video_id.clone()));
        ui.close();
    }
    if let Some(liked) = liked {
        if ui.button(t(l, if liked { S::unlike } else { S::like })).clicked() {
            acts.push(Act::Like(song.video_id.clone(), !liked));
            ui.close();
        }
    }
    ui.separator();
    if let Some(id) = &song.artist_id {
        if ui.button(t(l, S::go_to_artist)).clicked() {
            acts.push(Act::Open(Target::Browse { id: id.clone(), params: None }));
            ui.close();
        }
    }
    if let Some(id) = &song.album_id {
        if ui.button(t(l, S::go_to_album)).clicked() {
            acts.push(Act::Open(Target::Browse { id: id.clone(), params: None }));
            ui.close();
        }
    }
}

/// Fila de lista: portada, título, subtítulo, duración. Devuelve la respuesta de la fila.
pub fn row(ui: &mut egui::Ui, it: &Item, active: bool, index: Option<usize>) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::click());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{} — {}", it.title, it.subtitle)));
    if !ui.is_rect_visible(rect) {
        return r;
    }
    let fill = if active { theme::SELECTED } else if r.hovered() { theme::HOVER } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(8.0, 4.0))).layout(Layout::left_to_right(Align::Center)));
    if let Some(n) = index {
        child.add_sized([24.0, 48.0], egui::Label::new(RichText::new((n + 1).to_string()).color(theme::TEXT_3)).selectable(false));
    }
    cover(&mut child, &it.cover(120), 48.0, it.kind.round());
    let right_w = if it.duration > 0 { 56.0 } else { 0.0 };
    let text_w = (child.available_width() - right_w).max(40.0);
    child.allocate_ui_with_layout(Vec2::new(text_w, 48.0), Layout::top_down(Align::Min), |ui| {
        ui.add_space(4.0);
        let color = if active { theme::ACCENT } else { theme::TEXT };
        ui.add(egui::Label::new(RichText::new(&it.title).strong().color(color)).truncate().selectable(false));
        ui.add(egui::Label::new(RichText::new(&it.subtitle).color(theme::TEXT_2)).truncate().selectable(false));
    });
    if it.duration > 0 {
        child.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(fmt_time(it.duration as f32)).color(theme::TEXT_3));
        });
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Tarjeta cuadrada (álbum, playlist, artista, mix). Devuelve (clic en la tarjeta, clic en play).
pub fn tile(ui: &mut egui::Ui, it: &Item, size: f32, salt: (usize, usize)) -> (bool, bool) {
    if it.kind == Kind::Mood {
        let (rect, r) = ui.allocate_exact_size(Vec2::new(size, 48.0), Sense::click());
        r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &it.title));
        let c = it.color.map(|c| Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)).unwrap_or(theme::ACCENT);
        ui.painter().rect_filled(rect, CornerRadius::same(8), if r.hovered() { theme::SELECTED } else { theme::RAISED });
        ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, Vec2::new(6.0, rect.height())), CornerRadius { nw: 8, sw: 8, ne: 0, se: 0 }, c);
        ui.painter().text(rect.left_center() + Vec2::new(18.0, 0.0), egui::Align2::LEFT_CENTER, &it.title, egui::FontId::proportional(15.0), theme::TEXT);
        return (r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked(), false);
    }
    let mut play = false;
    let resp = ui
        .allocate_ui_with_layout(Vec2::new(size, size + 52.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(size);
            let img = cover(ui, &it.cover(320), size, it.kind.round());
            // Botón de play al pasar el ratón por la portada.
            if img.hovered() || ui.rect_contains_pointer(img.rect) {
                if it.play_target().is_some() {
                    let c = img.rect.right_bottom() - Vec2::new(26.0, 26.0);
                    let btn = egui::Rect::from_center_size(c, Vec2::splat(36.0));
                    let r = ui.interact(btn, ui.scope_id().with(("play", salt)), Sense::click());
                    ui.painter().circle_filled(c, 18.0, if r.hovered() { Color32::WHITE } else { Color32::from_white_alpha(230) });
                    ui.painter().text(c, egui::Align2::CENTER_CENTER, theme::icon::PLAY, egui::FontId::proportional(15.0), Color32::BLACK);
                    play = r.clicked();
                }
            }
            ui.add(egui::Label::new(RichText::new(&it.title).strong()).truncate().selectable(false));
            ui.add(egui::Label::new(RichText::new(&it.subtitle).color(theme::TEXT_2).size(13.0)).truncate().selectable(false));
        })
        .response;
    let r = ui.interact(resp.rect, ui.scope_id().with(("tile", salt)), Sense::click());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{} — {}", it.title, it.subtitle)));
    (r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !play, play)
}

/// Una página entera: cabecera, chips y secciones.
pub fn page(ui: &mut egui::Ui, l: L, p: &Page, playing: Option<&str>, loading_more: Option<usize>, acts: &mut Vec<Act>) {
    if let Some(h) = &p.header {
        header(ui, l, h, acts);
    }
    if !p.chips.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for c in &p.chips {
                if ui.add(egui::Button::selectable(c.selected, &c.text).corner_radius(CornerRadius::same(16))).clicked() {
                    acts.push(Act::Open(c.target.clone()));
                }
            }
        });
        ui.add_space(12.0);
    }
    // Una sola sección de tarjetas (biblioteca, moods): rejilla; si hay varias, carruseles.
    let wrap_grids = p.sections.iter().filter(|s| s.layout == SecLayout::Grid).count() <= 2;
    for (si, s) in p.sections.iter().enumerate() {
        ui.add_space(8.0);
        if !s.title.is_empty() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&s.title).size(20.0).strong());
                if let Some(m) = &s.more {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(t(l, S::more)).clicked() {
                            acts.push(Act::Open(m.clone()));
                        }
                    });
                }
            });
            ui.add_space(4.0);
        }
        match s.layout {
            SecLayout::List => {
                // Numeradas en álbumes (la cabecera es de álbum y las pistas no traen portada propia).
                let numbered = p.header.is_some() && s.items.iter().all(|i| i.thumbnail == p.header.as_ref().unwrap().thumbnail);
                for (ii, it) in s.items.iter().enumerate() {
                    let active = it.video_id.as_deref().is_some_and(|v| Some(v) == playing);
                    let r = row(ui, it, active, numbered.then_some(ii));
                    if r.clicked() {
                        if it.video_id.is_some() {
                            acts.push(Act::PlayInPage(si, ii));
                        } else if let Some(o) = &it.open {
                            acts.push(Act::Open(o.clone()));
                        }
                    }
                    if let Some(song) = it.song() {
                        r.context_menu(|ui| song_menu(ui, l, &song, None, acts));
                    } else if let Some(o) = &it.open {
                        let o = o.clone();
                        let play = it.play_target();
                        r.context_menu(|ui| {
                            if let Some(p) = &play {
                                if ui.button(t(l, S::play)).clicked() {
                                    acts.push(Act::Play(p.clone()));
                                    ui.close();
                                }
                                if ui.button(t(l, S::shuffle)).clicked() {
                                    acts.push(Act::Shuffle(p.clone()));
                                    ui.close();
                                }
                            }
                            if ui.button(t(l, S::more)).clicked() {
                                acts.push(Act::Open(o.clone()));
                                ui.close();
                            }
                        });
                    }
                }
            }
            SecLayout::Grid => {
                let size = 168.0;
                let mut show = |ui: &mut egui::Ui| {
                    for (ii, it) in s.items.iter().enumerate() {
                        let (open, play) = tile(ui, it, if it.kind == Kind::Mood { 200.0 } else { size }, (si, ii));
                        if play {
                            if let Some(p) = it.play_target() {
                                acts.push(Act::Play(p));
                            }
                        } else if open {
                            if let Some(o) = it.open.clone().or_else(|| it.play_target()) {
                                acts.push(Act::Open(o));
                            }
                        }
                    }
                };
                if wrap_grids {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(16.0, 16.0);
                        show(ui)
                    });
                } else {
                    egui::ScrollArea::horizontal().id_salt(("carousel", si)).auto_shrink([false, true]).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            show(ui)
                        });
                    });
                }
            }
        }
        if s.continuation.is_some() {
            ui.add_space(4.0);
            if loading_more == Some(si) {
                ui.spinner();
            } else if ui.button(t(l, S::load_more)).clicked() {
                acts.push(Act::LoadMore(si));
            }
        }
        ui.add_space(12.0);
    }
}

fn header(ui: &mut egui::Ui, l: L, h: &crate::pages::Header, acts: &mut Vec<Act>) {
    ui.add_space(12.0);
    ui.horizontal_top(|ui| {
        if !h.thumbnail.is_empty() {
            cover(ui, &crate::pages::cover_url(&h.thumbnail, 400), 200.0, false);
            ui.add_space(16.0);
        }
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(&h.title).size(30.0).strong());
            if !h.subtitle.is_empty() {
                ui.label(RichText::new(&h.subtitle).color(theme::TEXT_2));
            }
            if !h.description.is_empty() {
                let short: String = h.description.chars().take(260).collect();
                let more = h.description.chars().count() > 260;
                let r = ui.add(egui::Label::new(RichText::new(if more { format!("{short}…") } else { short }).color(theme::TEXT_3)).wrap());
                if more {
                    r.on_hover_text(&h.description);
                }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if let Some(p) = &h.play {
                    if pill(ui, theme::icon::PLAY, t(l, S::play), true).clicked() {
                        acts.push(Act::Play(p.clone()));
                    }
                }
                if let Some(s) = h.shuffle.clone().or_else(|| h.play.clone()) {
                    if pill(ui, theme::icon::SHUFFLE, t(l, S::shuffle), false).clicked() {
                        acts.push(Act::Shuffle(s));
                    }
                }
                if let Some(r) = &h.radio {
                    if pill(ui, theme::icon::RADIO, t(l, S::radio), false).clicked() {
                        acts.push(Act::Play(r.clone()));
                    }
                }
            });
        });
    });
    ui.add_space(16.0);
}

/// Entrada del menú lateral: icono y texto alineados a la izquierda, fondo si está activa.
pub fn nav_item(ui: &mut egui::Ui, selected: bool, icon: &str, text: &str) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::click());
    let fill = if selected { theme::SELECTED } else if r.hovered() { theme::HOVER } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
    let color = if selected { theme::TEXT } else { theme::TEXT_2 };
    let y = rect.center().y;
    ui.painter().text(egui::pos2(rect.left() + 14.0, y), egui::Align2::LEFT_CENTER, icon, egui::FontId::proportional(17.0), color);
    ui.painter().text(egui::pos2(rect.left() + 46.0, y), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(15.0), color);
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, text));
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Mensaje centrado (vacío, error...).
pub fn notice(ui: &mut egui::Ui, text: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).color(theme::TEXT_3).size(16.0)));
}
