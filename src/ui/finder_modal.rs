use std::path::PathBuf;

use egui::{Align, Color32, Key, Layout, ScrollArea};

use crate::finder::{search, FileIndex, Match};

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

    pub fn refresh(&mut self, index: &FileIndex) {
        self.results = search(index, &self.query, 50);
        if self.selected >= self.results.len() {
            self.selected = 0;
        }
    }
}

pub enum FinderAction {
    None,
    Open(PathBuf),
    Close,
}

pub fn show(
    ctx: &egui::Context,
    state: &mut FinderState,
    index: &FileIndex,
) -> FinderAction {
    let mut action = FinderAction::None;
    let mut want_close = false;
    let mut want_open: Option<PathBuf> = None;

    // Keyboard handling that should fire whether the input has focus or not.
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
                want_open = Some(m.path.clone());
            }
        }
    });

    egui::Window::new("Go to File")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
        .default_width(560.0)
        .show(ctx, |ui| {
            let input = ui.add(
                egui::TextEdit::singleline(&mut state.query)
                    .hint_text("Type to filter files…")
                    .desired_width(540.0),
            );
            if state.just_opened {
                input.request_focus();
                state.just_opened = false;
            }

            ui.separator();
            ScrollArea::vertical()
                .max_height(360.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (idx, m) in state.results.iter().enumerate() {
                        let selected = idx == state.selected;
                        let resp = ui.with_layout(
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let bg = if selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    Color32::TRANSPARENT
                                };
                                let frame = egui::Frame::default()
                                    .fill(bg)
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0));
                                frame
                                    .show(ui, |ui| {
                                        let (name, dir) = split_display(&m.display);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(name)
                                                    .monospace()
                                                    .strong(),
                                            );
                                            if !dir.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "  {dir}"
                                                    ))
                                                    .monospace()
                                                    .weak()
                                                    .small(),
                                                );
                                            }
                                        });
                                    })
                                    .response
                            },
                        );
                        if resp.inner.clicked() {
                            want_open = Some(m.path.clone());
                        }
                        if resp.inner.hovered() {
                            state.selected = idx;
                        }
                    }
                    if state.results.is_empty() {
                        ui.label(
                            egui::RichText::new("No matches")
                                .weak()
                                .small(),
                        );
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(
                        "↑/↓ navigate  ·  Enter to open  ·  Esc to close",
                    )
                    .small()
                    .weak(),
                );
                let count_text = if index.truncated {
                    format!("{}+ files indexed", index.len())
                } else {
                    format!("{} files indexed", index.len())
                };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(egui::RichText::new(count_text).small().weak());
                });
            });
        });

    if let Some(path) = want_open {
        action = FinderAction::Open(path);
    } else if want_close {
        action = FinderAction::Close;
    }
    action
}

/// Split "src/foo/bar.rs" into ("bar.rs", "src/foo"). Mirrors how Cmd+P
/// shows the filename in bold with the path dimmed beside it.
fn split_display(display: &str) -> (&str, &str) {
    match display.rfind('/') {
        Some(i) => (&display[i + 1..], &display[..i]),
        None => (display, ""),
    }
}
