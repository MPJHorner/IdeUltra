//! EmptyState: deliberate "this panel is empty" rendering.
//!
//! Centred icon + title + description + optional CTA. Per
//! STYLE_GUIDE.md §3.6 — the space is the design statement.

use egui::{RichText, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};

pub fn empty_state(
    ui: &mut Ui,
    theme: ColorTheme,
    icon: &str,
    title: &str,
    description: &str,
) {
    let t = tokens(theme);
    ui.add_space(space::S10);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(icon)
                .color(t.text_muted)
                .size(48.0),
        );
        ui.add_space(space::S2);
        ui.label(
            RichText::new(title)
                .color(t.text_primary)
                .size(ts::BODY)
                .strong(),
        );
        ui.add_space(space::S1);
        ui.label(
            RichText::new(description)
                .color(t.text_muted)
                .size(ts::LABEL),
        );
    });
    ui.add_space(space::S10);
}
