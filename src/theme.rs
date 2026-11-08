//! Colores y estilo, sacados de ByTune src/chrome.css.

use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const BG: Color32 = Color32::BLACK;
pub const TEXT: Color32 = Color32::WHITE;
pub const TEXT_2: Color32 = Color32::from_rgba_premultiplied(179, 179, 179, 179); // blanco 70 %
pub const TEXT_3: Color32 = Color32::from_rgba_premultiplied(143, 143, 143, 143); // blanco 56 %
pub const TEXT_OFF: Color32 = Color32::from_rgba_premultiplied(97, 97, 97, 97); // blanco 38 %
pub const GLASS: Color32 = Color32::from_rgb(12, 12, 12);
pub const BAR: Color32 = Color32::from_rgb(14, 14, 14);
pub const RAISED: Color32 = Color32::from_rgb(30, 30, 30);
pub const FIELD: Color32 = Color32::from_rgb(20, 20, 20); // blanco 8 % sobre negro
pub const HOVER: Color32 = Color32::from_rgb(20, 20, 20);
pub const SELECTED: Color32 = Color32::from_rgb(36, 36, 36); // blanco 14 %
pub const LINE: Color32 = Color32::from_rgb(20, 20, 20);
pub const ACCENT: Color32 = Color32::from_rgb(223, 28, 67); // rojo del icono
pub const DANGER: Color32 = Color32::from_rgb(255, 139, 139);

pub const RADIUS: u8 = 10;

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = RAISED;
    v.extreme_bg_color = FIELD;
    v.faint_bg_color = GLASS;
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = TEXT;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0, TEXT);
    v.window_corner_radius = CornerRadius::same(RADIUS);
    v.menu_corner_radius = CornerRadius::same(RADIUS);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(RADIUS);
        w.bg_stroke = Stroke::NONE;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_fill = SELECTED;
    v.widgets.hovered.weak_bg_fill = HOVER;
    v.widgets.hovered.bg_fill = HOVER;
    v.widgets.active.weak_bg_fill = SELECTED;
    v.widgets.active.bg_fill = SELECTED;
    v.widgets.inactive.fg_stroke = Stroke::new(1.5, TEXT_2);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT);
    v.widgets.active.fg_stroke = Stroke::new(1.5, TEXT);
    // Seleccionado (pestaña activa, botón "Letra"): gris claro con texto blanco, como ByTune.
    v.selection.bg_fill = Color32::from_rgb(48, 48, 48);
    // ByTune es siempre oscuro: no se sigue el tema claro/oscuro de Windows.
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, v);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.slider_rail_height = 4.0;
    });
    fonts(ctx);
}

/// Iconos de Segoe Fluent Icons (Windows 11) / Segoe MDL2 Assets (Windows 10).
pub mod icon {
    pub const PREV: &str = "\u{E892}";
    pub const NEXT: &str = "\u{E893}";
    pub const PLAY: &str = "\u{E768}";
    pub const PAUSE: &str = "\u{E769}";
    pub const VOLUME: &str = "\u{E767}";
}

/// Texto en Segoe UI y los iconos de Windows, leídos de C:\Windows\Fonts (sin copiarlos al
/// ejecutable). Si faltan, se queda con las fuentes de egui.
fn fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;

    let dir = std::path::Path::new(r"C:\Windows\Fonts");
    let read = |names: &[&str]| names.iter().find_map(|n| std::fs::read(dir.join(n)).ok());
    let mut defs = FontDefinitions::default();
    if let Some(text) = read(&["segoeui.ttf"]) {
        defs.font_data.insert("segoe".into(), Arc::new(FontData::from_owned(text)));
        defs.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
    }
    if let Some(icons) = read(&["SegoeIcons.ttf", "segmdl2.ttf"]) {
        // Los iconos están en la zona de uso privado de Unicode: van de respaldo sin pisar nada.
        defs.font_data.insert("icons".into(), Arc::new(FontData::from_owned(icons)));
        defs.families.entry(FontFamily::Proportional).or_default().push("icons".into());
    }
    ctx.set_fonts(defs);
}

/// Icono de la app: círculo rojo con tres barras blancas en forma de "play". Mismo dibujo que
/// ByTune scripts/make-icon.mjs, rasterizado al arrancar (sin archivos).
pub fn app_icon() -> egui::IconData {
    const SIZE: u32 = 64;
    const FIGMA: f32 = 536.0;
    const BARS: [(f32, f32, f32); 3] = [(182.0, 143.0, 250.0), (262.0, 183.0, 170.0), (342.0, 223.0, 90.0)];
    const BAR_W: f32 = 56.0;
    const R: f32 = 28.0;
    let inside_bar = |x: f32, y: f32| {
        BARS.iter().any(|&(x0, y0, h)| {
            if x < x0 || x > x0 + BAR_W || y < y0 || y > y0 + h {
                return false;
            }
            // Esquinas redondeadas.
            let cx = x.clamp(x0 + R, x0 + BAR_W - R);
            let cy = y.clamp(y0 + R, y0 + h - R);
            (x - cx).hypot(y - cy) <= R
        })
    };
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    const S: u32 = 4; // 4×4 muestras por píxel
    for py in 0..SIZE {
        for px in 0..SIZE {
            let (mut r, mut g, mut b, mut a) = (0u32, 0u32, 0u32, 0u32);
            for sy in 0..S {
                for sx in 0..S {
                    let u = (px as f32 + (sx as f32 + 0.5) / S as f32) / SIZE as f32;
                    let v = (py as f32 + (sy as f32 + 0.5) / S as f32) / SIZE as f32;
                    if (u - 0.5).hypot(v - 0.5) > 0.5 {
                        continue;
                    }
                    let c = if inside_bar(u * FIGMA, v * FIGMA) { [255, 255, 255] } else { [223, 28, 67] };
                    r += c[0];
                    g += c[1];
                    b += c[2];
                    a += 255;
                }
            }
            let n = S * S;
            let cover = a / 255;
            let div = cover.max(1);
            rgba.extend_from_slice(&[(r / div) as u8, (g / div) as u8, (b / div) as u8, (a / n) as u8]);
        }
    }
    egui::IconData { rgba, width: SIZE, height: SIZE }
}
