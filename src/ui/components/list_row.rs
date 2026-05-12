//! ListRow: selectable row for fuzzy finder, palette, project search,
//! recovery list, keymap picker, MRU lists.
//!
//! Per STYLE_GUIDE.md §3.4: 4-state lifecycle, `radius.sm`,
//! inset so the selection pill floats inside the parent panel.

use egui::{Frame, Margin, Response, Rounding, Sense, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{radii, space, tokens};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    Default,
    Hovered,
    Selected,
    SelectedFocused,
}

/// Render a row with the given visual state and the caller's content
/// closure. Returns the row's click response so the caller can react
/// to selection / activation.
pub fn list_row<R>(
    ui: &mut Ui,
    theme: ColorTheme,
    state: RowState,
    content: impl FnOnce(&mut Ui) -> R,
) -> Response {
    let t = tokens(theme);
    let (fill, stroke) = match state {
        RowState::Default => (egui::Color32::TRANSPARENT, egui::Stroke::NONE),
        RowState::Hovered => (t.bg_hover, egui::Stroke::NONE),
        RowState::Selected => (
            t.accent_bg,
            egui::Stroke::new(1.0, t.accent_border),
        ),
        RowState::SelectedFocused => (
            t.accent_bg,
            egui::Stroke::new(1.5, t.accent),
        ),
    };
    let frame = Frame::default()
        .fill(fill)
        .stroke(stroke)
        .rounding(Rounding::same(radii::SM))
        .inner_margin(Margin {
            left: space::S3,
            right: space::S3,
            top: space::S1 + 2.0,
            bottom: space::S1 + 2.0,
        });
    let inner = frame.show(ui, |ui| {
        content(ui);
    });
    inner.response.interact(Sense::click())
}
