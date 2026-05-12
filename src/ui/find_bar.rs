use std::ops::Range;

use egui::{Ui, RichText};

use crate::editor::language::ColorTheme;
use crate::find::{find_matches, FindError, FindOptions};
use crate::style::{radii, space, tokens, ts};
use crate::ui::components::{status_pill, StatusKind};

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
    pub scroll_pending: bool,
    pub scope: Option<(usize, usize)>,
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
    pub fn open_in_selection(&mut self, scope: (usize, usize)) {
        self.open = true;
        self.show_replace = false;
        self.scope = Some(scope);
    }
    pub fn close(&mut self) {
        self.open = false;
        self.scope = None;
    }
    pub fn refresh(&mut self, text: &str) {
        let (slice, offset) = match self.scope {
            Some((a, b)) if b <= text.len() => (&text[a..b], a),
            _ => (text, 0),
        };
        match find_matches(slice, &self.query, self.options) {
            Ok(m) => {
                let shifted: Vec<Range<usize>> = m
                    .into_iter()
                    .map(|r| (r.start + offset)..(r.end + offset))
                    .collect();
                if self.current >= shifted.len() {
                    self.current = 0;
                }
                self.matches = shifted;
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

/// Render the find/replace bar above the editor.
pub fn show(ui: &mut Ui, state: &mut FindState, theme: ColorTheme) -> FindAction {
    let mut action = FindAction::None;
    let t = tokens(theme);

    let frame = egui::Frame::default()
        .fill(t.bg_surface)
        .stroke(egui::Stroke::new(1.0, t.border_subtle))
        .rounding(egui::Rounding::same(radii::SM))
        .inner_margin(egui::Margin::symmetric(space::S3, space::S2));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let invalid = state.last_error.is_some();
            let mut text_edit = egui::TextEdit::singleline(&mut state.query)
                .hint_text("Find")
                .desired_width(240.0);
            if invalid {
                text_edit = text_edit.text_color(t.error);
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

            ui.label(
                RichText::new(match_label(state))
                    .color(t.text_muted)
                    .size(ts::CAPTION),
            );
            ui.separator();

            ui.toggle_value(&mut state.options.case_sensitive, "Aa")
                .on_hover_text("Match case");
            ui.toggle_value(&mut state.options.whole_word, "W")
                .on_hover_text("Whole word");
            ui.toggle_value(&mut state.options.regex, ".*")
                .on_hover_text("Regex");
            if state.scope.is_some() {
                if status_pill(ui, theme, StatusKind::Accent, "In selection")
                    .on_hover_text("Restricted to the original selection. Click to clear.")
                    .clicked()
                {
                    state.scope = None;
                }
            }

            ui.separator();
            if ui
                .toggle_value(&mut state.show_replace, "Replace")
                .clicked()
            {}
            if ui.small_button("✕").on_hover_text("Close (Esc)").clicked() {
                action = FindAction::Close;
            }
        });

        if state.show_replace {
            ui.add_space(space::S1);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.replacement)
                        .hint_text("Replace with")
                        .desired_width(240.0),
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
                RichText::new(format!("Invalid regex: {err}"))
                    .color(t.error)
                    .size(ts::LABEL_SM),
            );
        }
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
