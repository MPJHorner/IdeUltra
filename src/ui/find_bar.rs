use std::ops::Range;

use egui::{Color32, Stroke, Ui};

use crate::find::{find_matches, FindError, FindOptions};

#[derive(Default)]
pub struct FindState {
    pub open: bool,
    pub query: String,
    pub replacement: String,
    pub options: FindOptions,
    pub show_replace: bool,
    pub matches: Vec<Range<usize>>,
    pub current: usize,
    pub last_error: Option<String>,
    /// Cleared after the editor scrolls to the current match.
    pub scroll_pending: bool,
}

impl FindState {
    pub fn open_find(&mut self) {
        self.open = true;
        self.show_replace = false;
    }

    pub fn open_replace(&mut self) {
        self.open = true;
        self.show_replace = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Re-run the search against `text`. Cheap to call every frame —
    /// callers can skip when query/options/buffer haven't changed.
    pub fn refresh(&mut self, text: &str) {
        match find_matches(text, &self.query, self.options) {
            Ok(m) => {
                if self.current >= m.len() {
                    self.current = 0;
                }
                self.matches = m;
                self.last_error = None;
            }
            Err(FindError::InvalidRegex(e)) => {
                self.matches.clear();
                self.last_error = Some(e);
            }
        }
    }

    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current = (self.current + 1) % self.matches.len();
        self.scroll_pending = true;
    }

    pub fn prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current = if self.current == 0 {
            self.matches.len() - 1
        } else {
            self.current - 1
        };
        self.scroll_pending = true;
    }
}

pub enum FindAction {
    None,
    Next,
    Prev,
    ReplaceCurrent,
    ReplaceAll,
    Close,
}

/// Render the find/replace bar above the editor. Mutates `state`'s
/// input fields directly; returns an action describing button intent.
pub fn show(ui: &mut Ui, state: &mut FindState) -> FindAction {
    let mut action = FindAction::None;
    let frame = egui::Frame::group(ui.style()).inner_margin(egui::Margin::symmetric(6.0, 4.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let invalid = state.last_error.is_some();
            let mut text_edit = egui::TextEdit::singleline(&mut state.query)
                .hint_text("Find")
                .desired_width(220.0);
            if invalid {
                text_edit = text_edit
                    .text_color(Color32::from_rgb(220, 80, 80));
            }
            let resp = ui.add(text_edit);
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                action = if ui.input(|i| i.modifiers.shift) {
                    FindAction::Prev
                } else {
                    FindAction::Next
                };
            }

            if ui.small_button("◀").on_hover_text("Previous (Shift+Enter)").clicked() {
                action = FindAction::Prev;
            }
            if ui.small_button("▶").on_hover_text("Next (Enter)").clicked() {
                action = FindAction::Next;
            }

            ui.label(match_label(state));
            ui.separator();

            ui.toggle_value(&mut state.options.case_sensitive, "Aa")
                .on_hover_text("Match case");
            ui.toggle_value(&mut state.options.whole_word, "W")
                .on_hover_text("Whole word");
            ui.toggle_value(&mut state.options.regex, ".*")
                .on_hover_text("Regex");

            ui.separator();
            if ui
                .toggle_value(&mut state.show_replace, "Replace")
                .clicked()
            {
                // No-op — state already flipped.
            }
            if ui.small_button("✕").on_hover_text("Close (Esc)").clicked() {
                action = FindAction::Close;
            }
        });

        if state.show_replace {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.replacement)
                        .hint_text("Replace with")
                        .desired_width(220.0),
                );
                if ui.button("Replace").clicked() {
                    action = FindAction::ReplaceCurrent;
                }
                if ui.button("Replace All").clicked() {
                    action = FindAction::ReplaceAll;
                }
            });
        }

        if let Some(err) = &state.last_error {
            ui.label(
                egui::RichText::new(format!("Invalid regex: {err}"))
                    .small()
                    .color(Color32::from_rgb(220, 80, 80)),
            );
        }
        // Visible focus hint at the bottom edge.
        let _ = Stroke::NONE;
    });
    action
}

fn match_label(state: &FindState) -> String {
    if state.query.is_empty() {
        String::new()
    } else if state.matches.is_empty() {
        "0 matches".to_string()
    } else {
        format!("{}/{}", state.current + 1, state.matches.len())
    }
}

