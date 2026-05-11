use egui::{FontFamily, FontId, Ui};

use crate::editor::language::ColorTheme;
use crate::editor::EditorTab;

const FONT_SIZE: f32 = 13.5;

pub fn show(ui: &mut Ui, tab: &mut EditorTab, theme: ColorTheme) {
    let syntax = tab.syntax();

    // egui's TextEdit layouter is called every frame. We hand it a closure
    // that hits our per-tab highlight cache; the cache rebuilds the
    // LayoutJob only when text, theme, wrap width or font size change.
    let cache = &mut tab.highlight;
    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let job = cache.layout(text, syntax, theme, wrap_width, FONT_SIZE);
        // egui needs an owned LayoutJob inside an Arc<Galley>. We clone
        // the inner job (cheap — TextFormats are small) and let the font
        // system cache the galley by its own hash.
        let job = (*job).clone();
        ui.fonts(|f| f.layout_job(job))
    };

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut tab.buffer.text)
                    .font(FontId::new(FONT_SIZE, FontFamily::Monospace))
                    .code_editor()
                    .desired_rows(40)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
}
