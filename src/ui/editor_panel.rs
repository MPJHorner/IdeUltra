use egui::{FontFamily, FontId, TextStyle, Ui};

use crate::editor::EditorTab;

/// Render the active editor tab. Returns true if the buffer changed this frame.
pub fn show(ui: &mut Ui, tab: &mut EditorTab) -> bool {
    let text_style = TextStyle::Monospace;
    let font_id = FontId::new(13.5, FontFamily::Monospace);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let resp = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut tab.buffer.text)
                    .font(font_id)
                    .code_editor()
                    .desired_rows(40)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY),
            );
            // Suppress unused: text_style helps reviewers see we mean monospace.
            let _ = text_style;
            resp.changed()
        })
        .inner
}
