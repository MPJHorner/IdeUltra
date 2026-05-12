use std::path::PathBuf;

use egui::{Color32, Ui};

use crate::find::FindOptions;
use crate::project_search::{LineMatch, SearchOutcome};

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
    /// Run a search now with the current query.
    Run,
    /// Open the given file and scroll to the byte range.
    OpenAt {
        path: PathBuf,
        byte_range: std::ops::Range<usize>,
    },
    /// Replace every match across every file from the current outcome.
    ReplaceAll,
}

pub fn show(ui: &mut Ui, state: &mut ProjectSearchState) -> ProjectSearchAction {
    let mut action = ProjectSearchAction::None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Project Search").strong());
        if ui.small_button("✕").on_hover_text("Close").clicked() {
            state.open = false;
        }
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
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.replacement)
                    .hint_text("Replace with")
                    .desired_width(f32::INFINITY),
            );
        });
        ui.horizontal(|ui| {
            let has_matches = state
                .outcome
                .as_ref()
                .map(|o| o.total_matches > 0 && o.error.is_none())
                .unwrap_or(false);
            let btn = egui::Button::new("Replace All in files");
            if ui.add_enabled(has_matches, btn).clicked() {
                action = ProjectSearchAction::ReplaceAll;
            }
            ui.label(
                egui::RichText::new("Writes through to disk · no undo")
                    .small()
                    .weak(),
            );
        });
    }

    ui.separator();

    if let Some(outcome) = &state.outcome {
        if let Some(err) = &outcome.error {
            ui.label(
                egui::RichText::new(format!("Invalid regex: {err}"))
                    .color(Color32::from_rgb(220, 80, 80))
                    .small(),
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
                "{} matches in {} files  ·  {} files scanned, {} skipped",
                outcome.total_matches,
                outcome.hits.len(),
                outcome.files_scanned,
                outcome.files_skipped
            )
        };
        ui.label(egui::RichText::new(summary).small().weak());

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
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(name)
                                .monospace()
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(parent)
                                .monospace()
                                .weak()
                                .small(),
                        );
                    });
                    for m in &file.matches {
                        if render_match_row(ui, m) {
                            action = ProjectSearchAction::OpenAt {
                                path: file.path.clone(),
                                byte_range: m.byte_range.clone(),
                            };
                        }
                    }
                    ui.add_space(4.0);
                }
                if outcome.hits.is_empty() && !state.dirty {
                    ui.label(
                        egui::RichText::new("No matches")
                            .weak()
                            .small(),
                    );
                }
            });
    } else if state.dirty {
        ui.label(
            egui::RichText::new("Press Enter or click Search")
                .small()
                .weak(),
        );
    }

    action
}

fn render_match_row(ui: &mut Ui, m: &LineMatch) -> bool {
    // Show "  42: line text" — clickable. Truncate very long lines.
    let preview = truncate_middle(m.line_text.trim_start(), 140);
    let label = format!("  {:>5}:  {preview}", m.line);
    let resp = ui.add(egui::SelectableLabel::new(false, label));
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
