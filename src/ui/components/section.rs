//! Section + SectionHeader: vertical grouping inside settings panels
//! and large modals.

use egui::{RichText, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};

/// A small uppercase caption used to label a settings group.
pub fn section_header(ui: &mut Ui, theme: ColorTheme, label: &str) {
    let t = tokens(theme);
    ui.add_space(space::S2);
    ui.label(
        RichText::new(label.to_uppercase())
            .color(t.text_muted)
            .size(ts::CAPTION)
            .strong(),
    );
    ui.add_space(space::S1);
}

/// Header + grouped content in one call.
pub fn section(
    ui: &mut Ui,
    theme: ColorTheme,
    label: &str,
    content: impl FnOnce(&mut Ui),
) {
    section_header(ui, theme, label);
    ui.spacing_mut().item_spacing.y = space::S2;
    content(ui);
    ui.spacing_mut().item_spacing.y = space::S1;
}
