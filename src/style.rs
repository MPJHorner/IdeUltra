//! Centralised egui style configuration. Replaces the default `Visuals`
//! with something more deliberately styled — better contrast, softer
//! corner rounding, less aggressive selection colors, consistent spacing.
//!
//! Called once per frame after the theme is applied so theme changes
//! flow through immediately.

use egui::{Color32, Context, Rounding, Stroke, Visuals};

use crate::editor::language::ColorTheme;

pub fn apply(ctx: &Context, theme: ColorTheme) {
    let mut visuals = match theme {
        ColorTheme::Dark => dark_visuals(),
        ColorTheme::Light => light_visuals(),
    };

    // Universal tweaks regardless of theme.
    visuals.window_rounding = Rounding::same(10.0);
    visuals.menu_rounding = Rounding::same(8.0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 6.0),
        blur: 22.0,
        spread: 0.0,
        color: Color32::from_black_alpha(60),
    };
    visuals.popup_shadow = visuals.window_shadow;

    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(5.0);
    visuals.widgets.hovered.rounding = Rounding::same(5.0);
    visuals.widgets.active.rounding = Rounding::same(5.0);
    visuals.widgets.open.rounding = Rounding::same(5.0);

    ctx.set_visuals(visuals);

    // Spacing + sizing.
    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 4.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.menu_margin = egui::Margin::symmetric(8.0, 4.0);
        s.spacing.window_margin = egui::Margin::symmetric(0.0, 0.0);
        s.spacing.indent = 18.0;
        s.spacing.scroll.bar_width = 8.0;
        s.spacing.scroll.handle_min_length = 24.0;
    });
}

fn dark_visuals() -> Visuals {
    // Anchored at a near-black indigo, accents in electric blue.
    let bg = Color32::from_rgb(0x10, 0x12, 0x18);
    let bg_elev = Color32::from_rgb(0x16, 0x19, 0x22);
    let bg_high = Color32::from_rgb(0x1d, 0x21, 0x2c);
    let stroke = Color32::from_rgb(0x2a, 0x2f, 0x3c);
    let stroke_strong = Color32::from_rgb(0x3a, 0x40, 0x52);
    let text = Color32::from_rgb(0xe6, 0xe8, 0xee);
    let text_dim = Color32::from_rgb(0x8b, 0x91, 0xa0);
    let accent = Color32::from_rgb(0x4d, 0xa6, 0xff); // electric blue
    let accent_dim = Color32::from_rgb(0x2b, 0x55, 0x88);

    let mut v = Visuals::dark();
    v.dark_mode = true;
    v.panel_fill = bg;
    v.window_fill = bg_elev;
    v.window_stroke = Stroke::new(1.0, stroke);
    v.extreme_bg_color = Color32::from_rgb(0x0a, 0x0c, 0x12);
    v.faint_bg_color = bg_elev;
    v.code_bg_color = Color32::from_rgb(0x0e, 0x11, 0x18);

    v.override_text_color = Some(text);
    v.hyperlink_color = accent;

    v.widgets.noninteractive.bg_fill = bg_elev;
    v.widgets.noninteractive.weak_bg_fill = bg_elev;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, stroke);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_dim);

    v.widgets.inactive.bg_fill = bg_high;
    v.widgets.inactive.weak_bg_fill = bg_elev;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, stroke);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, text);

    v.widgets.hovered.bg_fill = Color32::from_rgb(0x24, 0x29, 0x36);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x1f, 0x24, 0x30);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, stroke_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, text);

    v.widgets.active.bg_fill = accent_dim;
    v.widgets.active.weak_bg_fill = accent_dim;
    v.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    v.widgets.active.fg_stroke = Stroke::new(1.5, text);

    v.widgets.open.bg_fill = bg_high;
    v.widgets.open.bg_stroke = Stroke::new(1.0, stroke_strong);
    v.widgets.open.fg_stroke = Stroke::new(1.0, text);

    v.selection.bg_fill = Color32::from_rgb(0x2b, 0x4a, 0x80);
    v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xe6, 0xe8, 0xee));

    v.weak_text_color();
    v
}

fn light_visuals() -> Visuals {
    let bg = Color32::from_rgb(0xfb, 0xfb, 0xfc);
    let bg_elev = Color32::from_rgb(0xff, 0xff, 0xff);
    let bg_high = Color32::from_rgb(0xf3, 0xf4, 0xf6);
    let stroke = Color32::from_rgb(0xe0, 0xe3, 0xe8);
    let stroke_strong = Color32::from_rgb(0xc8, 0xcc, 0xd4);
    let text = Color32::from_rgb(0x1c, 0x1f, 0x26);
    let text_dim = Color32::from_rgb(0x6a, 0x70, 0x7c);
    let accent = Color32::from_rgb(0x09, 0x69, 0xda);
    let accent_dim = Color32::from_rgb(0xcf, 0xe5, 0xff);

    let mut v = Visuals::light();
    v.dark_mode = false;
    v.panel_fill = bg;
    v.window_fill = bg_elev;
    v.window_stroke = Stroke::new(1.0, stroke);
    v.extreme_bg_color = Color32::from_rgb(0xf4, 0xf6, 0xfa);
    v.faint_bg_color = bg_high;
    v.code_bg_color = Color32::from_rgb(0xf4, 0xf6, 0xfa);

    v.override_text_color = Some(text);
    v.hyperlink_color = accent;

    v.widgets.noninteractive.bg_fill = bg_elev;
    v.widgets.noninteractive.weak_bg_fill = bg_elev;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, stroke);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_dim);

    v.widgets.inactive.bg_fill = bg_elev;
    v.widgets.inactive.weak_bg_fill = bg_high;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, stroke);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, text);

    v.widgets.hovered.bg_fill = bg_high;
    v.widgets.hovered.weak_bg_fill = bg_high;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, stroke_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, text);

    v.widgets.active.bg_fill = accent_dim;
    v.widgets.active.weak_bg_fill = accent_dim;
    v.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    v.widgets.active.fg_stroke = Stroke::new(1.5, text);

    v.widgets.open.bg_fill = bg_high;
    v.widgets.open.bg_stroke = Stroke::new(1.0, stroke_strong);
    v.widgets.open.fg_stroke = Stroke::new(1.0, text);

    v.selection.bg_fill = accent_dim;
    v.selection.stroke = Stroke::new(1.0, accent);

    v
}
