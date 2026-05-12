//! Button styles per STYLE_GUIDE.md §3.3.
//!
//!   * `primary_button` — gradient-tone fill (no actual gradient in
//!     egui, but the solid accent looks close), white-on-accent text
//!   * `ghost_button` — transparent until hovered, then `bg_subtle`
//!   * `icon_button` — square 32×32 ghost with one glyph

use egui::{Button, Response, RichText, Sense, Stroke, Ui, Vec2};

use crate::editor::language::ColorTheme;
use crate::style::{radii, ts, tokens};

/// Filled primary action button.
pub fn primary_button(ui: &mut Ui, theme: ColorTheme, label: &str) -> Response {
    let t = tokens(theme);
    let btn = Button::new(
        RichText::new(label)
            .color(t.text_on_accent)
            .size(ts::LABEL)
            .strong(),
    )
    .min_size(Vec2::new(0.0, 28.0))
    .fill(t.accent)
    .stroke(Stroke::NONE)
    .rounding(egui::Rounding::same(radii::SM));
    ui.add(btn)
}

/// Outlined / transparent secondary action button.
pub fn ghost_button(ui: &mut Ui, theme: ColorTheme, label: &str) -> Response {
    let t = tokens(theme);
    let btn = Button::new(
        RichText::new(label)
            .color(t.text_primary)
            .size(ts::LABEL),
    )
    .min_size(Vec2::new(0.0, 28.0))
    .fill(egui::Color32::TRANSPARENT)
    .stroke(Stroke::new(1.0, t.border_default))
    .rounding(egui::Rounding::same(radii::SM));
    ui.add(btn)
}

/// 32×32 transparent square button for a single glyph in the top bar.
pub fn icon_button(ui: &mut Ui, theme: ColorTheme, glyph: &str) -> Response {
    let _ = theme;
    let btn = Button::new(RichText::new(glyph).size(ts::BODY))
        .min_size(Vec2::splat(32.0))
        .fill(egui::Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .rounding(egui::Rounding::same(radii::SM))
        .sense(Sense::click());
    ui.add(btn)
}
