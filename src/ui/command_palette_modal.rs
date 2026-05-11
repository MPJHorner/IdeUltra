use egui::{Align, Color32, Key, Layout, RichText, ScrollArea, Sense};

use crate::command_palette::{rank, CommandId, RankedCommand};
use crate::keymap::Keymap;

#[derive(Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub results: Vec<RankedCommand>,
    pub just_opened: bool,
}

impl CommandPaletteState {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.just_opened = true;
        self.refresh();
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn refresh(&mut self) {
        self.results = rank(&self.query, 60);
        if self.selected >= self.results.len() {
            self.selected = 0;
        }
    }
}

pub enum CommandPaletteAction {
    None,
    Run(CommandId),
    Close,
}

pub fn show(
    ctx: &egui::Context,
    state: &mut CommandPaletteState,
    keymap: &Keymap,
) -> CommandPaletteAction {
    let mut action = CommandPaletteAction::None;
    let mut want_close = false;
    let mut want_run: Option<CommandId> = None;

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
            if let Some(r) = state.results.get(state.selected) {
                want_run = Some(r.entry.id);
            }
        }
    });

    egui::Window::new("Command Palette")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 84.0])
        .default_width(620.0)
        .frame(modal_frame(ctx))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌘").size(16.0).weak());
                let input = ui.add(
                    egui::TextEdit::singleline(&mut state.query)
                        .hint_text("Type a command")
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(15.0))
                        .frame(false),
                );
                if state.just_opened {
                    input.request_focus();
                    state.just_opened = false;
                }
                if input.changed() {
                    state.refresh();
                }
            });
            ui.separator();

            ScrollArea::vertical()
                .max_height(420.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (idx, r) in state.results.iter().enumerate() {
                        let selected = idx == state.selected;
                        if render_row(ui, r, keymap, selected) {
                            want_run = Some(r.entry.id);
                        }
                    }
                    if state.results.is_empty() {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(if state.query.is_empty() {
                                    "Start typing"
                                } else {
                                    "No matches"
                                })
                                .weak()
                                .small(),
                            );
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("↑↓ navigate   ⏎ run   esc close").small().weak(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(keymap.preset.label()).small().weak(),
                    );
                });
            });
        });

    if let Some(id) = want_run {
        action = CommandPaletteAction::Run(id);
    } else if want_close {
        action = CommandPaletteAction::Close;
    }
    action
}

fn render_row(
    ui: &mut egui::Ui,
    r: &RankedCommand,
    keymap: &Keymap,
    selected: bool,
) -> bool {
    let visuals = ui.visuals();
    let bg = if selected {
        visuals.selection.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let fg = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    let dim = if selected {
        visuals.selection.stroke.color.linear_multiply(0.7)
    } else {
        visuals.weak_text_color()
    };

    // Take the live shortcut from the keymap if there is one; otherwise
    // fall back to the entry's hint string (used for items that don't
    // have a global shortcut, like "Sort Lines").
    let live = keymap.label_for(r.entry.id);
    let keys = if live.is_empty() { r.entry.keys.to_string() } else { live };

    let frame = egui::Frame::default()
        .fill(bg)
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin {
            left: 10.0,
            right: 10.0,
            top: 6.0,
            bottom: 6.0,
        });
    let resp = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (group, action) = split_label(r.entry.label);
                ui.label(RichText::new(group).size(13.0).color(dim));
                ui.label(RichText::new(action).size(14.0).color(fg));
                if !keys.is_empty() {
                    ui.with_layout(
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.label(
                                RichText::new(keys)
                                    .monospace()
                                    .size(12.5)
                                    .color(dim),
                            );
                        },
                    );
                }
            });
        })
        .response
        .interact(Sense::click());
    resp.clicked()
}

/// Split "File: Open File…" into ("File", "Open File…"). Falls back to
/// ("", whole-label) if there's no `: ` separator.
fn split_label(label: &str) -> (&str, &str) {
    match label.find(": ") {
        Some(i) => (&label[..i], &label[i + 2..]),
        None => ("", label),
    }
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
