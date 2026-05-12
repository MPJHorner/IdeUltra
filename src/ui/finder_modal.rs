use std::path::{Path, PathBuf};

use egui::{Align, Key, Layout, RichText, ScrollArea};

use crate::editor::language::ColorTheme;
use crate::finder::{search_with_recent, FileIndex, Match};
use crate::style::{space, tokens, ts};
use crate::ui::components::{
    empty_state, hint_row, list_row, modal_frame, search_input, RowState,
};

#[derive(Default)]
pub struct FinderState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub results: Vec<Match>,
    /// Set the frame the modal is being opened so we can grab focus once.
    pub just_opened: bool,
}

impl FinderState {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.results.clear();
        self.just_opened = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn refresh(
        &mut self,
        index: &FileIndex,
        recent: &[PathBuf],
        ws_root: Option<&Path>,
    ) {
        self.results = search_with_recent(index, &self.query, recent, 60);
        if let Some(root) = ws_root {
            let query = self.query.trim();
            if !query.is_empty() && !query.split('/').any(|c| c == "..") {
                let last = query.rsplit('/').next().unwrap_or(query);
                if crate::fs_ops::validate_name(last).is_ok() {
                    let target = root.join(query);
                    let already_real = self
                        .results
                        .iter()
                        .any(|m| !m.is_create && m.path == target);
                    if !target.exists() && !already_real {
                        self.results.push(crate::finder::Match::create(
                            target,
                            query.to_string(),
                        ));
                    }
                }
            }
        }
        if self.selected >= self.results.len() {
            self.selected = 0;
        }
    }
}

pub enum FinderAction {
    None,
    Open(PathBuf),
    Create(PathBuf),
    Close,
}

pub fn show(
    ctx: &egui::Context,
    state: &mut FinderState,
    index: &FileIndex,
    theme: ColorTheme,
) -> FinderAction {
    let mut action = FinderAction::None;
    let mut want_close = false;
    let mut want_open: Option<Match> = None;

    ctx.input(|i| {
        if i.key_pressed(Key::Escape) {
            want_close = true;
        }
        if i.key_pressed(Key::ArrowDown) && !state.results.is_empty() {
            state.selected = (state.selected + 1).min(state.results.len() - 1);
        }
        if i.key_pressed(Key::ArrowUp) && state.selected > 0 {
            state.selected -= 1;
        }
        if i.key_pressed(Key::Enter) {
            if let Some(m) = state.results.get(state.selected) {
                want_open = Some(m.clone());
            }
        }
    });

    egui::Window::new("Go to File")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 84.0])
        .default_width(640.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = space::S2;

            let input = search_input(ui, theme, "⌕", "Type to filter files", &mut state.query);
            if state.just_opened {
                input.request_focus();
                state.just_opened = false;
            }

            ui.add_space(space::S1);
            ui.separator();

            ScrollArea::vertical()
                .max_height(420.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    if state.results.is_empty() {
                        empty_state(
                            ui,
                            theme,
                            if state.query.is_empty() { "⌕" } else { "∅" },
                            if state.query.is_empty() {
                                "Start typing"
                            } else {
                                "No matches"
                            },
                            if state.query.is_empty() {
                                "Search the workspace by filename or path."
                            } else {
                                "Try a different query or check spelling."
                            },
                        );
                    } else {
                        for (idx, m) in state.results.iter().enumerate() {
                            let row_state = if idx == state.selected {
                                RowState::SelectedFocused
                            } else {
                                RowState::Default
                            };
                            let row_resp = list_row(ui, theme, row_state, |ui| {
                                render_row_content(ui, theme, m, idx == state.selected);
                            });
                            if row_resp.clicked() {
                                want_open = Some(m.clone());
                            }
                        }
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                hint_row(ui, theme, &["↑↓ navigate", "⏎ open", "esc close"]);
                let count_text = if index.truncated {
                    format!("{}+ files", index.len())
                } else {
                    format!("{} files", index.len())
                };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(count_text)
                            .color(tokens(theme).text_muted)
                            .size(ts::CAPTION),
                    );
                });
            });
        });

    if let Some(m) = want_open {
        action = if m.is_create {
            FinderAction::Create(m.path)
        } else {
            FinderAction::Open(m.path)
        };
    } else if want_close {
        action = FinderAction::Close;
    }
    action
}

fn render_row_content(ui: &mut egui::Ui, theme: ColorTheme, m: &Match, selected: bool) {
    let t = tokens(theme);
    let name_color = if selected { t.accent } else { t.text_primary };
    let path_color = if selected {
        t.accent.linear_multiply(0.75)
    } else {
        t.text_muted
    };
    ui.horizontal(|ui| {
        if m.is_create {
            ui.label(
                RichText::new("+")
                    .color(t.accent)
                    .size(ts::BODY)
                    .strong(),
            );
            ui.add_space(space::S1);
            ui.label(
                RichText::new(&m.display)
                    .color(t.accent)
                    .size(ts::BODY)
                    .strong(),
            );
            ui.add_space(space::S2);
            ui.label(
                RichText::new("create new file")
                    .color(t.text_muted)
                    .size(ts::LABEL_SM)
                    .italics(),
            );
        } else {
            let (filename, dir) = split_display(&m.display);
            let icon = file_glyph(filename);
            ui.label(RichText::new(icon).color(path_color).size(ts::BODY));
            ui.add_space(space::S1);
            ui.label(
                RichText::new(filename)
                    .color(name_color)
                    .size(ts::BODY)
                    .strong(),
            );
            if !dir.is_empty() {
                ui.add_space(space::S2);
                ui.label(
                    RichText::new(dir)
                        .color(path_color)
                        .size(ts::LABEL_SM),
                );
            }
        }
    });
}

fn split_display(display: &str) -> (&str, &str) {
    match display.rfind('/') {
        Some(i) => (&display[i + 1..], &display[..i]),
        None => (display, ""),
    }
}

fn file_glyph(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => "🦀",
        "toml" | "yaml" | "yml" | "json" | "ini" | "cfg" | "conf" => "⚙",
        "md" | "markdown" | "mdx" | "rst" | "txt" => "📝",
        "sh" | "bash" | "zsh" | "fish" => "⚡",
        "py" => "🐍",
        "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" => "𝙅",
        "html" | "htm" => "◉",
        "css" | "scss" | "sass" | "less" => "✦",
        "go" => "ɢ",
        "rb" => "♦",
        "php" => "𝙿",
        "lock" => "🔒",
        "" => "·",
        _ => "•",
    }
}
