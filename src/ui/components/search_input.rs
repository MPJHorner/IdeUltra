//! Frameless search input with a leading glyph. Used at the top of the
//! finder and command palette.

use egui::{RichText, TextEdit, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{tokens, ts};

/// Render the glyph + text input. Returns the TextEdit response so the
/// caller can request focus / detect changes.
pub fn search_input(
    ui: &mut Ui,
    theme: ColorTheme,
    glyph: &str,
    hint: &str,
    buffer: &mut String,
) -> egui::Response {
    let t = tokens(theme);
    let mut resp = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(glyph)
                .color(t.text_muted)
                .size(ts::BODY_LG),
        );
        ui.add_space(2.0);
        let r = ui.add(
            TextEdit::singleline(buffer)
                .hint_text(hint)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(15.0))
                .frame(false),
        );
        resp = Some(r);
    });
    resp.unwrap()
}
