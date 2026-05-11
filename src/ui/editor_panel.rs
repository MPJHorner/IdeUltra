use std::ops::Range;

use egui::text::{CCursor, CCursorRange};
use egui::widgets::text_edit::TextEditState;
use egui::{FontFamily, FontId, Id, Ui};

use crate::editor::language::ColorTheme;
use crate::editor::EditorTab;

const FONT_SIZE: f32 = 13.5;

/// `jump_to`, when `Some`, sets the editor's cursor selection to the byte
/// range and triggers a scroll-into-view. Used by find/replace to move
/// to the next match.
pub fn show(
    ui: &mut Ui,
    tab: &mut EditorTab,
    theme: ColorTheme,
    jump_to: Option<Range<usize>>,
) {
    let syntax = tab.syntax();
    let cache = &mut tab.highlight;
    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let job = cache.layout(text, syntax, theme, wrap_width, FONT_SIZE);
        let job = (*job).clone();
        ui.fonts(|f| f.layout_job(job))
    };

    let editor_id = Id::new(("ide_editor", tab.path.as_path()));

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .id_source(editor_id)
        .show(ui, |ui| {
            let resp = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut tab.buffer.text)
                    .id(editor_id)
                    .font(FontId::new(FONT_SIZE, FontFamily::Monospace))
                    .code_editor()
                    .desired_rows(40)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );

            if let Some(range) = jump_to {
                if let Some(mut state) = TextEditState::load(ui.ctx(), editor_id) {
                    let start = byte_to_char_index(&tab.buffer.text, range.start);
                    let end = byte_to_char_index(&tab.buffer.text, range.end);
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::two(
                            CCursor::new(start),
                            CCursor::new(end),
                        )));
                    state.store(ui.ctx(), editor_id);
                    resp.scroll_to_me(Some(egui::Align::Center));
                    ui.ctx().request_repaint();
                }
            }
        });
}

fn byte_to_char_index(text: &str, byte: usize) -> usize {
    if byte >= text.len() {
        return text.chars().count();
    }
    text[..byte].chars().count()
}
