use std::path::PathBuf;

use egui::{Ui, RichText};

use crate::editor::language::ColorTheme;
use crate::find::FindOptions;
use crate::project_search::{LineMatch, SearchOutcome};
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, primary_button, status_pill, StatusKind};

#[derive(Default)]
pub struct ProjectSearchState {
    pub open: bool,
    pub query: String,
    pub replacement: String,
    pub options: FindOptions,
    pub outcome: Option<SearchOutcome>,
    /// `true` when the query/options changed but the search hasn't yet run.
    pub dirty: bool,
    /// Set on the frame we want focus.
    pub just_opened: bool,
    /// Whether the replace input + button are visible.
    pub show_replace: bool,
}

impl ProjectSearchState {
    pub fn open(&mut self) {
        self.open = true;
        self.just_opened = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub enum ProjectSearchAction {
    None,
    Run,
    OpenAt {
        path: PathBuf,
        byte_range: std::ops::Range<usize>,
    },
    ReplaceAll,
}

pub fn show(ui: &mut Ui, state: &mut ProjectSearchState, theme: ColorTheme) -> ProjectSearchAction {
    let mut action = ProjectSearchAction::None;
    let t = tokens(theme);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Project Search")
                .color(t.text_primary)
                .size(ts::LABEL)
                .strong(),
        );
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if ui.small_button("✕").on_hover_text("Close").clicked() {
                    state.open = false;
                }
            },
        );
    });
    ui.separator();

    let resp = ui.add(
        egui::TextEdit::singleline(&mut state.query)
            .hint_text("Search across all files…")
            .desired_width(f32::INFINITY),
    );
    if state.just_opened {
        resp.request_focus();
        state.just_opened = false;
    }
    if resp.changed() {
        state.mark_dirty();
    }
    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

    ui.horizontal(|ui| {
        if ui
            .toggle_value(&mut state.options.case_sensitive, "Aa")
            .on_hover_text("Match case")
            .changed()
        {
            state.mark_dirty();
        }
        if ui
            .toggle_value(&mut state.options.whole_word, "W")
            .on_hover_text("Whole word")
            .changed()
        {
            state.mark_dirty();
        }
        if ui
            .toggle_value(&mut state.options.regex, ".*")
            .on_hover_text("Regex")
            .changed()
        {
            state.mark_dirty();
        }
        ui.toggle_value(&mut state.show_replace, "Replace")
            .on_hover_text("Toggle the replacement row");
        if ui.button("Search").clicked() || enter {
            action = ProjectSearchAction::Run;
        }
    });

    if state.show_replace {
        ui.add_space(space::S1);
        ui.add(
            egui::TextEdit::singleline(&mut state.replacement)
                .hint_text("Replace with")
                .desired_width(f32::INFINITY),
        );
        ui.horizontal(|ui| {
            let has_matches = state
                .outcome
                .as_ref()
                .map(|o| o.total_matches > 0 && o.error.is_none())
                .unwrap_or(false);
            ui.add_enabled_ui(has_matches, |ui| {
                if primary_button(ui, theme, "Replace All in files").clicked() {
                    action = ProjectSearchAction::ReplaceAll;
                }
            });
            ui.label(
                RichText::new("Writes through · no undo")
                    .color(t.warning)
                    .size(ts::CAPTION),
            );
        });
    }

    ui.separator();

    if let Some(outcome) = &state.outcome {
        if let Some(err) = &outcome.error {
            ui.label(
                RichText::new(format!("Invalid regex: {err}"))
                    .color(t.error)
                    .size(ts::LABEL_SM),
            );
            return action;
        }

        let summary = if outcome.truncated {
            format!(
                "{}+ matches in {} files (cap reached)",
                outcome.total_matches,
                outcome.hits.len()
            )
        } else {
            format!(
                "{} matches in {} files · {} files scanned · {} skipped",
                outcome.total_matches,
                outcome.hits.len(),
                outcome.files_scanned,
                outcome.files_skipped
            )
        };
        ui.label(RichText::new(summary).color(t.text_muted).size(ts::CAPTION));

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for file in &outcome.hits {
                    let name = file
                        .path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("?");
                    let parent = file
                        .path
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or("");
                    ui.add_space(space::S1);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(name)
                                .color(t.text_primary)
                                .monospace()
                                .size(ts::LABEL)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(parent)
                                .color(t.text_muted)
                                .monospace()
                                .size(ts::CAPTION),
                        );
                        // Match-count pill on the right.
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                status_pill(
                                    ui,
                                    theme,
                                    StatusKind::Accent,
                                    &file.matches.len().to_string(),
                                );
                            },
                        );
                    });
                    for m in &file.matches {
                        if render_match_row(ui, theme, m) {
                            action = ProjectSearchAction::OpenAt {
                                path: file.path.clone(),
                                byte_range: m.byte_range.clone(),
                            };
                        }
                    }
                }
                if outcome.hits.is_empty() && !state.dirty {
                    ui.label(
                        RichText::new("No matches")
                            .color(t.text_muted)
                            .size(ts::LABEL_SM),
                    );
                }
            });
    } else if state.dirty {
        ui.label(
            RichText::new("Press Enter or click Search")
                .color(t.text_muted)
                .size(ts::LABEL_SM),
        );
    } else {
        // No search yet — empty hint.
        ui.add_space(space::S2);
        ui.label(
            RichText::new("Type a query and press Enter")
                .color(t.text_muted)
                .size(ts::LABEL_SM),
        );
    }

    // Close button at the very top closed via state.open; bubble up.
    if !state.open {
        // We toggled it above; nothing else to do — the parent loop reads state.
        let _ = ghost_button; // satisfy unused-fn warning when only used above
    }

    action
}

fn render_match_row(ui: &mut Ui, theme: ColorTheme, m: &LineMatch) -> bool {
    let t = tokens(theme);
    let preview = truncate_middle(m.line_text.trim_start(), 140);
    let label = format!("  {:>5}:  {preview}", m.line);
    let resp = ui.add(
        egui::SelectableLabel::new(
            false,
            RichText::new(label)
                .color(t.text_secondary)
                .monospace()
                .size(ts::MONO_UI),
        ),
    );
    resp.clicked()
}

fn truncate_middle(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max / 2).collect();
    let tail: String = s.chars().rev().take(max / 2).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{head}…{tail}")
}
