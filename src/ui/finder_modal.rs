use std::path::PathBuf;

use egui::{Align, Key, Layout, RichText, ScrollArea, Sense};

use crate::finder::{search_with_recent, FileIndex, Match};

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

    pub fn refresh(&mut self, index: &FileIndex, recent: &[PathBuf]) {
        self.results = search_with_recent(index, &self.query, recent, 60);
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
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 84.0])
        .default_width(620.0)
        .frame(modal_frame(ctx))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            // Search input with magnifying-glass affordance.
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌕").size(16.0).weak());
                let input = ui.add(
                    egui::TextEdit::singleline(&mut state.query)
                        .hint_text("Type to filter files")
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(15.0))
                        .frame(false),
                );
                if state.just_opened {
                    input.request_focus();
                    state.just_opened = false;
                }
            });
            ui.separator();

            ScrollArea::vertical()
                .max_height(400.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    if state.results.is_empty() {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(if state.query.is_empty() {
                                    "Start typing to filter"
                                } else {
                                    "No matches"
                                })
                                .weak()
                                .small(),
                            );
                        });
                        ui.add_space(8.0);
                    } else {
                        for (idx, m) in state.results.iter().enumerate() {
                            if render_row(ui, m, idx == state.selected) {
                                want_open = Some(m.path.clone());
                            }
                            if ui
                                .interact(
                                    ui.min_rect(),
                                    egui::Id::new(("finder_row_hover", idx)),
                                    Sense::hover(),
                                )
                                .hovered()
                            {
                                // Hover-to-select is too jittery on a dense
                                // list; left out intentionally. Click selects.
                            }
                        }
                    }
                });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("↑↓ navigate   ⏎ open   esc close")
                        .small()
                        .weak(),
                );
                let count_text = if index.truncated {
                    format!("{}+ files", index.len())
                } else {
                    format!("{} files", index.len())
                };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(count_text).small().weak());
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

/// Render one result row. Returns `true` if the user clicked it.
fn render_row(ui: &mut egui::Ui, m: &Match, selected: bool) -> bool {
    let visuals = ui.visuals();
    let row_fill = if selected {
        visuals.selection.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };
    let name_color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    let path_color = if selected {
        // Pull the dim path color toward the accent so it stays legible
        // on the selection background.
        visuals.selection.stroke.color.linear_multiply(0.7)
    } else {
        visuals.weak_text_color()
    };

    let (filename, dir) = split_display(&m.display);
    let icon = file_glyph(filename);

    let frame = egui::Frame::default()
        .fill(row_fill)
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin {
            left: 10.0,
            right: 10.0,
            top: 6.0,
            bottom: 6.0,
        });

    let response = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(14.0).color(path_color));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(filename)
                        .size(14.0)
                        .strong()
                        .color(name_color),
                );
                if !dir.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(dir)
                            .size(12.0)
                            .color(path_color),
                    );
                }
            });
        })
        .response
        .interact(Sense::click());
    response.clicked()
}

fn modal_frame(ctx: &egui::Context) -> egui::Frame {
    let v = ctx.style().visuals.clone();
    egui::Frame::window(&ctx.style())
        .fill(v.panel_fill)
        .stroke(egui::Stroke::new(1.0, v.widgets.noninteractive.bg_stroke.color))
        .rounding(egui::Rounding::same(10.0))
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(80),
        })
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
}

/// Split `src/foo/bar.rs` into (`bar.rs`, `src/foo`).
fn split_display(display: &str) -> (&str, &str) {
    match display.rfind('/') {
        Some(i) => (&display[i + 1..], &display[..i]),
        None => (display, ""),
    }
}

/// Cheap file-glyph by extension. Not exhaustive — picks the kind of icon
/// that helps the eye scan the list.
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
