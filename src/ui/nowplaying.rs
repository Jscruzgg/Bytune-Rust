//! Now Playing: portada (o vídeo), datos de la canción y letra sincronizada al lado.

use std::time::Duration;

use egui::{Align, CornerRadius, Layout, RichText, Sense, Vec2};

use super::Act;
use crate::i18n::{t, L, S};
use crate::innertube::Song;
use crate::lyrics::{lrc::Line, timing, Lyrics};
use crate::pages::Target;
use crate::theme;

pub struct Np<'a> {
    pub l: L,
    pub song: &'a Song,
    pub lyrics: Option<&'a Lyrics>,
    pub pos: f64,
    pub playing: bool,
    pub lyrics_open: bool,
    pub video: bool,
    pub has_video: bool,
    pub liked: Option<bool>,
    pub fullscreen: bool,
}

/// Lo que devuelve la vista además de los Act.
#[derive(Default)]
pub struct NpOut {
    /// Dónde poner el vídeo (si está en modo vídeo).
    pub video_rect: Option<egui::Rect>,
    pub seek: Option<f64>,
    pub toggle_lyrics: bool,
    pub set_video: Option<bool>,
    pub toggle_fullscreen: bool,
}

pub fn show(ui: &mut egui::Ui, np: Np, last_line: &mut Option<usize>, acts: &mut Vec<Act>) -> NpOut {
    let mut out = NpOut::default();
    let avail = ui.available_size();
    let lyrics_w = if np.lyrics_open { (avail.x * 0.48).max(320.0) } else { 0.0 };
    let left_w = (avail.x - lyrics_w - 24.0).max(240.0);
    let art = (avail.y - 200.0).min(left_w - 32.0).clamp(160.0, 720.0);

    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(Vec2::new(left_w, avail.y), Layout::top_down(Align::Center), |ui| {
            ui.add_space(((avail.y - art - 170.0) / 2.0).max(16.0));
            // Portada, o el hueco donde se coloca el WebView con el vídeo.
            if np.video && np.has_video {
                let w = art * 16.0 / 9.0;
                let w = w.min(left_w - 16.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(w, w * 9.0 / 16.0), Sense::hover());
                ui.painter().rect_filled(rect, CornerRadius::same(12), egui::Color32::BLACK);
                out.video_rect = Some(rect);
            } else {
                ui.add(egui::Image::new(np.song.cover(720)).fit_to_exact_size(Vec2::splat(art)).corner_radius(CornerRadius::same(12)));
            }
            ui.add_space(16.0);
            ui.add(egui::Label::new(RichText::new(&np.song.title).size(24.0).strong()).truncate());
            // Artista y álbum, con enlace a sus páginas.
            ui.horizontal(|ui| {
                let w = ui.available_width();
                ui.allocate_ui_with_layout(Vec2::new(w, 22.0), Layout::left_to_right(Align::Center).with_main_align(Align::Center), |ui| {
                    link(ui, &np.song.artist, np.song.artist_id.as_deref(), acts);
                    if !np.song.album.is_empty() {
                        ui.label(RichText::new("·").color(theme::TEXT_3));
                        link(ui, &np.song.album, np.song.album_id.as_deref(), acts);
                    }
                });
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let n = 3.0 + np.has_video as i32 as f32;
                ui.add_space(((ui.available_width() - n * 52.0) / 2.0).max(0.0));
                if let Some(liked) = np.liked {
                    let icon = if liked { theme::icon::HEART_FILL } else { theme::icon::HEART };
                    if ui.add(super::icon_button(icon, 44.0, false)).on_hover_text(t(np.l, if liked { S::unlike } else { S::like })).clicked() {
                        acts.push(Act::Like(np.song.video_id.clone(), !liked));
                    }
                }
                if np.has_video {
                    let (icon, tip) = if np.video { (theme::icon::HEADPHONES, S::song) } else { (theme::icon::VIDEO, S::video) };
                    if ui.add(super::icon_button(icon, 44.0, false)).on_hover_text(t(np.l, tip)).clicked() {
                        out.set_video = Some(!np.video);
                    }
                }
                let lyr = ui.add(egui::Button::selectable(np.lyrics_open, RichText::new(theme::icon::LYRICS).size(18.0)).min_size(Vec2::splat(44.0)).corner_radius(CornerRadius::same(22)));
                if lyr.on_hover_text(t(np.l, S::lyrics)).clicked() {
                    out.toggle_lyrics = true;
                }
                let (icon, tip) = if np.fullscreen { (theme::icon::EXIT_FULLSCREEN, S::exit_fullscreen) } else { (theme::icon::FULLSCREEN, S::fullscreen) };
                if ui.add(super::icon_button(icon, 44.0, false)).on_hover_text(t(np.l, tip)).clicked() {
                    out.toggle_fullscreen = true;
                }
            });
        });
        if np.lyrics_open {
            ui.add_space(24.0);
            ui.allocate_ui_with_layout(Vec2::new(lyrics_w, avail.y), Layout::top_down(Align::Min), |ui| {
                out.seek = lyrics(ui, np.l, np.lyrics, np.pos, np.playing, last_line);
            });
        }
    });
    out
}

fn link(ui: &mut egui::Ui, text: &str, id: Option<&str>, acts: &mut Vec<Act>) {
    let r = ui.add(egui::Label::new(RichText::new(text).size(16.0).color(theme::TEXT_2)).sense(Sense::click()).truncate());
    if let Some(id) = id {
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            acts.push(Act::Open(Target::Browse { id: id.to_string(), params: None }));
        }
    }
}

/// Letra: sincronizada (línea activa resaltada y centrada; clic para saltar) o plana.
fn lyrics(ui: &mut egui::Ui, l: L, lyrics: Option<&Lyrics>, pos: f64, playing: bool, last_line: &mut Option<usize>) -> Option<f64> {
    match lyrics {
        None => {
            ui.add_space(40.0);
            ui.spinner();
            None
        }
        Some(Lyrics::None) => {
            ui.add_space(40.0);
            ui.label(RichText::new(t(l, S::no_lyrics)).color(theme::TEXT_3).size(18.0));
            None
        }
        Some(Lyrics::Plain(lines)) => {
            egui::ScrollArea::vertical().id_salt("lyrics").auto_shrink(false).show(ui, |ui| {
                ui.add_space(24.0);
                for line in lines {
                    ui.label(RichText::new(line).size(24.0).strong().color(theme::TEXT_2));
                }
                ui.add_space(40.0);
            });
            None
        }
        Some(Lyrics::Synced(lines)) => {
            let lines: &Vec<Line> = lines;
            let times: Vec<f64> = lines.iter().map(|x| x.t).collect();
            let active = timing::active_index(&times, pos);
            let changed = active != *last_line;
            let mut seek = None;
            egui::ScrollArea::vertical().id_salt("lyrics").auto_shrink(false).show(ui, |ui| {
                ui.add_space(ui.available_height() * 0.35);
                for (i, line) in lines.iter().enumerate() {
                    let is_active = Some(i) == active;
                    let color = if is_active {
                        theme::TEXT
                    } else if active.is_some_and(|a| i < a) {
                        theme::TEXT_OFF
                    } else {
                        theme::TEXT_3
                    };
                    let size = if line.gap { 22.0 } else { 32.0 };
                    let r = ui.add(egui::Label::new(RichText::new(&line.text).size(size).strong().color(color)).sense(Sense::click()).wrap());
                    if is_active && changed {
                        r.scroll_to_me(Some(Align::Center));
                    }
                    if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        seek = Some(line.t);
                    }
                    ui.add_space(14.0);
                }
                ui.add_space(ui.available_height() * 0.6);
            });
            *last_line = active;
            // Mientras suena, se repinta justo cuando toca la siguiente línea.
            if playing {
                if let Some(ms) = timing::ms_until_next(&times, pos, 1.0) {
                    ui.ctx().request_repaint_after(Duration::from_millis(ms.max(16)));
                }
            }
            seek
        }
    }
}
