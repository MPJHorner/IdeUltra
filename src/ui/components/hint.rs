//! Hint row: the "↑↓ navigate · ⏎ open · esc close" footer found in
//! every modal.

use egui::{RichText, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{tokens, ts};

/// Render a single hint line composed of pieces joined by `·`.
pub fn hint_row(ui: &mut Ui, theme: ColorTheme, pieces: &[&str]) {
    let t = tokens(theme);
    let joined = pieces.join("   ·   ");
    ui.label(
        RichText::new(joined)
            .color(t.text_muted)
            .size(ts::CAPTION),
    );
}
