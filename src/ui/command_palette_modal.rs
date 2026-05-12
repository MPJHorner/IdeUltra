use egui::{Align, Key, Layout, RichText, ScrollArea};

use crate::command_palette::{rank, CommandId, RankedCommand};
use crate::editor::language::ColorTheme;
use crate::keymap::Keymap;
use crate::style::{space, tokens, ts};
use crate::ui::components::{
    empty_state, hint_row, list_row, modal_frame, search_input, RowState,
};

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
    theme: ColorTheme,
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
        .default_width(640.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = space::S2;

            let input =
                search_input(ui, theme, "⌘", "Type a command", &mut state.query);
            if state.just_opened {
                input.request_focus();
                state.just_opened = false;
            }
            if input.changed() {
                state.refresh();
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
                            "⌘",
                            if state.query.is_empty() {
                                "Start typing"
                            } else {
                                "No commands match"
                            },
                            "Search every menu action by name.",
                        );
                    } else {
                        for (idx, r) in state.results.iter().enumerate() {
                            let row_state = if idx == state.selected {
                                RowState::SelectedFocused
                            } else {
                                RowState::Default
                            };
                            let row_resp = list_row(ui, theme, row_state, |ui| {
                                render_row_content(ui, theme, r, keymap, idx == state.selected);
                            });
                            if row_resp.clicked() {
                                want_run = Some(r.entry.id);
                            }
                        }
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                hint_row(ui, theme, &["↑↓ navigate", "⏎ run", "esc close"]);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(keymap.preset.label())
                            .color(tokens(theme).text_muted)
                            .size(ts::CAPTION),
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

fn render_row_content(
    ui: &mut egui::Ui,
    theme: ColorTheme,
    r: &RankedCommand,
    keymap: &Keymap,
    selected: bool,
) {
    let t = tokens(theme);
    let primary = if selected { t.accent } else { t.text_primary };
    let dim = if selected {
        t.accent.linear_multiply(0.75)
    } else {
        t.text_muted
    };
    let live = keymap.label_for(r.entry.id);
    let keys = if live.is_empty() {
        r.entry.keys.to_string()
    } else {
        live
    };
    ui.horizontal(|ui| {
        let (group, label) = split_label(r.entry.label);
        if !group.is_empty() {
            ui.label(RichText::new(group).color(dim).size(ts::LABEL));
        }
        ui.label(RichText::new(label).color(primary).size(ts::BODY));
        if !keys.is_empty() {
            ui.with_layout(
                Layout::right_to_left(Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(keys)
                            .monospace()
                            .color(dim)
                            .size(ts::KEYCAP),
                    );
                },
            );
        }
    });
}

fn split_label(label: &str) -> (&str, &str) {
    match label.find(": ") {
        Some(i) => (&label[..i], &label[i + 2..]),
        None => ("", label),
    }
}
