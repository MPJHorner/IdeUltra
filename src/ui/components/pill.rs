//! Pill / Badge: small chromatic tag. Two flavours:
//!
//!   * `neutral_pill(label)` — `bg_subtle`, caption type
//!   * `status_pill(kind, label)` — 3-variant status bundle

use egui::{Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{radii, space, tokens, ts};

pub fn neutral_pill(ui: &mut Ui, theme: ColorTheme, label: &str) -> Response {
    let t = tokens(theme);
    Frame::default()
        .fill(t.bg_subtle)
        .stroke(Stroke::new(1.0, t.border_subtle))
        .rounding(Rounding::same(radii::XS))
        .inner_margin(Margin::symmetric(space::S2, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .color(t.text_secondary)
                    .size(ts::CAPTION),
            );
        })
        .response
        .interact(Sense::click())
}

#[derive(Clone, Copy)]
pub enum StatusKind {
    Error,
    Warning,
    Success,
    Info,
    Accent,
}

pub fn status_pill(
    ui: &mut Ui,
    theme: ColorTheme,
    kind: StatusKind,
    label: &str,
) -> Response {
    let t = tokens(theme);
    let (fg, bg, border) = match kind {
        StatusKind::Error => (t.error, t.error_bg, t.error_border),
        StatusKind::Warning => (t.warning, t.warning_bg, t.warning_border),
        StatusKind::Success => (t.success, t.success_bg, t.success_border),
        StatusKind::Info => (t.info, t.info_bg, t.info_border),
        StatusKind::Accent => (t.accent, t.accent_bg, t.accent_border),
    };
    Frame::default()
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .rounding(Rounding::same(radii::XS))
        .inner_margin(Margin::symmetric(space::S2, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .color(fg)
                    .size(ts::CAPTION)
                    .strong(),
            );
        })
        .response
        .interact(Sense::click())
}
