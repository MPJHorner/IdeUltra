use egui::{Align, Color32, Key, Layout, ScrollArea};

use crate::command_palette::{rank, CommandId, RankedCommand};

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
        self.results = rank(&self.query, 50);
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

pub fn show(ctx: &egui::Context, state: &mut CommandPaletteState) -> CommandPaletteAction {
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
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
        .default_width(560.0)
        .show(ctx, |ui| {
            let input = ui.add(
                egui::TextEdit::singleline(&mut state.query)
                    .hint_text("Type a command…")
                    .desired_width(540.0),
            );
            if state.just_opened {
                input.request_focus();
                state.just_opened = false;
            }
            if input.changed() {
                state.refresh();
            }

            ui.separator();
            ScrollArea::vertical()
                .max_height(360.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (idx, r) in state.results.iter().enumerate() {
                        let selected = idx == state.selected;
                        let bg = if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            Color32::TRANSPARENT
                        };
                        let frame = egui::Frame::default()
                            .fill(bg)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0));
                        let resp = frame
                            .show(ui, |ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label(egui::RichText::new(r.entry.label).monospace());
                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            if !r.entry.keys.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(r.entry.keys)
                                                        .monospace()
                                                        .weak()
                                                        .small(),
                                                );
                                            }
                                        },
                                    );
                                });
                            })
                            .response;
                        if resp.clicked() {
                            want_run = Some(r.entry.id);
                        }
                        if resp.hovered() {
                            state.selected = idx;
                        }
                    }
                    if state.results.is_empty() {
                        ui.label(egui::RichText::new("No matches").weak().small());
                    }
                });
        });

    if let Some(id) = want_run {
        action = CommandPaletteAction::Run(id);
    } else if want_close {
        action = CommandPaletteAction::Close;
    }
    action
}
