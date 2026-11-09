//! Menú rápido (Ctrl+Shift+Space): una ventana flotante para buscar canciones, saltar a tus
//! playlists y controlar la reproducción sin abrir la ventana principal. Como el de ByTune.

use egui::{Align, CornerRadius, Key, Layout, RichText, Sense, Vec2};

use crate::i18n::{t, L, S};
use crate::innertube::Song;
use crate::pages::{Item, Target};
use crate::settings::Recent;
use crate::theme;

#[derive(Default)]
pub struct QuickMenu {
    pub open: bool,
    pub query: String,
    /// La búsqueda que ya se pidió (para no repetirla).
    pub asked: String,
    pub songs: Vec<Song>,
    pub searching: bool,
    pub sel: usize,
    pub focus: bool,
    /// Cuándo cambió el texto por última vez (la búsqueda espera a que pare de escribir).
    pub typed_at: Option<std::time::Instant>,
}

/// Una entrada de la lista: canción, playlist o reciente.
#[derive(Clone)]
pub enum Entry {
    Song(Song),
    Open { title: String, subtitle: String, thumbnail: String, play: Target },
}

pub enum QmOut {
    Play(Entry),
    /// Shift+Enter: a la cola.
    Queue(Song),
    PlayPause,
    Next,
    Previous,
    Close,
}

impl QuickMenu {
    pub fn show(&mut self) {
        self.open = true;
        self.focus = true;
        self.sel = 0;
    }

    /// Lo que se ve: con texto, canciones y playlists que coinciden; sin texto, recientes y
    /// playlists.
    pub fn entries(&self, playlists: &[Item], recent: &[Recent]) -> (Vec<(S, Vec<Entry>)>, usize) {
        let q = self.query.trim().to_lowercase();
        let pl: Vec<Entry> = playlists
            .iter()
            .filter(|p| q.is_empty() || p.title.to_lowercase().contains(&q))
            .filter_map(|p| {
                Some(Entry::Open { title: p.title.clone(), subtitle: p.subtitle.clone(), thumbnail: p.thumbnail.clone(), play: p.play_target()? })
            })
            .take(if q.is_empty() { 6 } else { 4 })
            .collect();
        let mut groups = Vec::new();
        if q.is_empty() {
            let rec: Vec<Entry> = recent
                .iter()
                .filter_map(|r| match (&r.video_id, &r.playlist_id) {
                    (Some(v), None) => Some(Entry::Song(Song { video_id: v.clone(), title: r.title.clone(), artist: r.subtitle.clone(), thumbnail: r.thumbnail.clone(), ..Default::default() })),
                    (v, Some(p)) => Some(Entry::Open {
                        title: r.title.clone(),
                        subtitle: r.subtitle.clone(),
                        thumbnail: r.thumbnail.clone(),
                        play: Target::Watch { video_id: v.clone(), playlist_id: Some(p.clone()) },
                    }),
                    _ => None,
                })
                .collect();
            if !rec.is_empty() {
                groups.push((S::qm_recent, rec));
            }
        } else if !self.songs.is_empty() {
            groups.push((S::qm_songs, self.songs.iter().take(6).cloned().map(Entry::Song).collect()));
        }
        if !pl.is_empty() {
            groups.push((S::qm_playlists, pl));
        }
        let n = groups.iter().map(|(_, e)| e.len()).sum();
        (groups, n)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, l: L, playlists: &[Item], recent: &[Recent], playing: bool) -> Vec<QmOut> {
        let mut out = Vec::new();
        let (groups, n) = self.entries(playlists, recent);
        if n > 0 {
            self.sel = self.sel.min(n - 1);
        }
        // Teclado: flechas, Enter, Shift+Enter, Esc, Alt+←↓→.
        let (up, down, enter, shift, esc, alt_left, alt_right, alt_down) = ui.input_mut(|i| {
            let alt = i.modifiers.alt;
            (
                !alt && i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                !alt && i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                i.key_pressed(Key::Enter),
                i.modifiers.shift,
                i.key_pressed(Key::Escape),
                i.consume_key(egui::Modifiers::ALT, Key::ArrowLeft),
                i.consume_key(egui::Modifiers::ALT, Key::ArrowRight),
                i.consume_key(egui::Modifiers::ALT, Key::ArrowDown),
            )
        });
        if up {
            self.sel = self.sel.saturating_sub(1);
        }
        if down && n > 0 {
            self.sel = (self.sel + 1).min(n - 1);
        }
        if esc {
            out.push(QmOut::Close);
        }
        if alt_left {
            out.push(QmOut::Previous);
        }
        if alt_right {
            out.push(QmOut::Next);
        }
        if alt_down {
            out.push(QmOut::PlayPause);
        }

        egui::Frame::new().fill(theme::RAISED).inner_margin(egui::Margin::same(14)).corner_radius(CornerRadius::same(14)).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.horizontal(|ui| {
                ui.label(RichText::new(theme::icon::SEARCH).size(16.0).color(theme::TEXT_2));
                let edit = egui::TextEdit::singleline(&mut self.query)
                    .hint_text(t(l, S::qm_hint))
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width() - 120.0)
                    .font(egui::FontId::proportional(18.0));
                let r = ui.add(edit);
                if self.focus {
                    r.request_focus();
                    self.focus = false;
                }
                if r.changed() {
                    self.typed_at = Some(std::time::Instant::now());
                    self.sel = 0;
                }
                if self.searching {
                    ui.spinner();
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(super::icon_button(theme::icon::NEXT, 30.0, false)).clicked() {
                        out.push(QmOut::Next);
                    }
                    let icon = if playing { theme::icon::PAUSE } else { theme::icon::PLAY };
                    if ui.add(super::icon_button(icon, 30.0, true)).clicked() {
                        out.push(QmOut::PlayPause);
                    }
                    if ui.add(super::icon_button(theme::icon::PREV, 30.0, false)).clicked() {
                        out.push(QmOut::Previous);
                    }
                });
            });
            ui.separator();
            let mut idx = 0;
            egui::ScrollArea::vertical().auto_shrink(false).max_height(ui.available_height() - 24.0).show(ui, |ui| {
                for (title, entries) in &groups {
                    ui.label(RichText::new(t(l, *title)).size(12.0).color(theme::TEXT_3));
                    for e in entries {
                        let selected = idx == self.sel;
                        let (title, subtitle, thumb) = match e {
                            Entry::Song(s) => (s.title.as_str(), s.artist.as_str(), s.cover(120)),
                            Entry::Open { title, subtitle, thumbnail, .. } => (title.as_str(), subtitle.as_str(), crate::pages::cover_url(thumbnail, 120)),
                        };
                        let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), Sense::click());
                        if selected || r.hovered() {
                            ui.painter().rect_filled(rect, CornerRadius::same(8), if selected { theme::SELECTED } else { theme::HOVER });
                        }
                        if selected && (up || down) {
                            r.scroll_to_me(None);
                        }
                        let mut c = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(8.0, 3.0))).layout(Layout::left_to_right(Align::Center)));
                        if !thumb.is_empty() {
                            c.add(egui::Image::new(thumb).fit_to_exact_size(Vec2::splat(38.0)).corner_radius(CornerRadius::same(5)));
                        }
                        c.vertical(|ui| {
                            ui.add(egui::Label::new(RichText::new(title).strong()).truncate().selectable(false));
                            ui.add(egui::Label::new(RichText::new(subtitle).color(theme::TEXT_2).size(12.0)).truncate().selectable(false));
                        });
                        if r.clicked() || (enter && selected) {
                            match (e, shift && enter) {
                                (Entry::Song(s), true) => out.push(QmOut::Queue(s.clone())),
                                _ => out.push(QmOut::Play(e.clone())),
                            }
                        }
                        idx += 1;
                    }
                    ui.add_space(6.0);
                }
                if n == 0 && !self.query.trim().is_empty() && !self.searching {
                    ui.label(RichText::new("—").color(theme::TEXT_3));
                }
            });
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(RichText::new(t(l, S::qm_help)).size(11.0).color(theme::TEXT_3));
            });
        });
        out
    }
}
