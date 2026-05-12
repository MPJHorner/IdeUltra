use std::ops::Range;

use egui::text::{CCursor, CCursorRange};
use egui::widgets::text_edit::TextEditState;
use egui::{FontFamily, FontId, Id, Ui};

use crate::editor::language::ColorTheme;
use crate::editor::EditorTab;

const FONT_SIZE: f32 = 13.5;

pub enum Jump {
    /// Byte range — used by find/replace to select the match.
    ByteRange(Range<usize>),
    /// Character index — used by go-to-line to put the caret at start of line.
    CharIndex(usize),
}

/// Caret position (1-based line, 1-based column). `None` if the editor
/// has no caret yet (e.g. first frame).
pub struct ShowResult {
    pub caret_char_index: Option<usize>,
}

pub fn show(
    ui: &mut Ui,
    tab: &mut EditorTab,
    theme: ColorTheme,
    jump_to: Option<Jump>,
    soft_wrap: bool,
) -> ShowResult {
    let editor_id = Id::new(("ide_editor", tab.path.as_path()));

    // Last frame's caret position drives this frame's bracket-match
    // highlight. One frame of latency is invisible at 60fps.
    let prev_caret_char = TextEditState::load(ui.ctx(), editor_id)
        .and_then(|s| s.cursor.char_range().map(|r| r.primary.index));
    let bracket_match = prev_caret_char.and_then(|c| {
        let byte = char_index_to_byte(&tab.buffer.text, c);
        crate::brackets::find_matching(&tab.buffer.text, byte)
    });

    let syntax = tab.syntax();
    let cache = &mut tab.highlight;
    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let job = cache.layout(
            text,
            syntax,
            theme,
            wrap_width,
            FONT_SIZE,
            bracket_match,
        );
        let job = (*job).clone();
        ui.fonts(|f| f.layout_job(job))
    };

    let caret_char_index = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .id_source(editor_id)
        .show(ui, |ui| {
            let mut edit = egui::TextEdit::multiline(&mut tab.buffer.text)
                .id(editor_id)
                .font(FontId::new(FONT_SIZE, FontFamily::Monospace))
                .code_editor()
                .desired_rows(40)
                .lock_focus(true)
                .layouter(&mut layouter);
            if !soft_wrap {
                edit = edit.desired_width(f32::INFINITY);
            }
            let resp = ui.add_sized(ui.available_size(), edit);

            if let Some(jump) = jump_to {
                if let Some(mut state) = TextEditState::load(ui.ctx(), editor_id) {
                    let (start, end) = match jump {
                        Jump::ByteRange(r) => (
                            byte_to_char_index(&tab.buffer.text, r.start),
                            byte_to_char_index(&tab.buffer.text, r.end),
                        ),
                        Jump::CharIndex(c) => (c, c),
                    };
                    state.cursor.set_char_range(Some(CCursorRange::two(
                        CCursor::new(start),
                        CCursor::new(end),
                    )));
                    state.store(ui.ctx(), editor_id);
                    resp.scroll_to_me(Some(egui::Align::Center));
                    ui.ctx().request_repaint();
                }
            }

            TextEditState::load(ui.ctx(), editor_id)
                .and_then(|s| s.cursor.char_range().map(|r| r.primary.index))
        })
        .inner;

    ShowResult { caret_char_index }
}

fn byte_to_char_index(text: &str, byte: usize) -> usize {
    if byte >= text.len() {
        return text.chars().count();
    }
    text[..byte].chars().count()
}

fn char_index_to_byte(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    let mut count = 0usize;
    for (i, _) in text.char_indices() {
        if count == char_index {
            return i;
        }
        count += 1;
    }
    text.len()
}
