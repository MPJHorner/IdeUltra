//! Centred input modal used for "New File", "New Folder", and "Rename".

use egui::{Align2, Key, RichText};

use crate::editor::language::ColorTheme;
use crate::fs_ops::{validate_name, NameError};
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, modal_frame, primary_button};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamePromptKind {
    NewFile,
    NewFolder,
    Rename,
}

impl NamePromptKind {
    fn title(&self) -> &'static str {
        match self {
            NamePromptKind::NewFile => "New File",
            NamePromptKind::NewFolder => "New Folder",
            NamePromptKind::Rename => "Rename",
        }
    }
    fn confirm_label(&self) -> &'static str {
        match self {
            NamePromptKind::NewFile => "Create file",
            NamePromptKind::NewFolder => "Create folder",
            NamePromptKind::Rename => "Rename",
        }
    }
    fn hint(&self) -> &'static str {
        match self {
            NamePromptKind::NewFile => "filename.ext",
            NamePromptKind::NewFolder => "folder name",
            NamePromptKind::Rename => "new name",
        }
    }
}

pub struct NamePromptState {
    pub kind: NamePromptKind,
    pub context_label: String,
    pub input: String,
    pub just_opened: bool,
}

pub enum NamePromptAction {
    None,
    Confirm(String),
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    state: &mut NamePromptState,
    theme: ColorTheme,
) -> NamePromptAction {
    let mut action = NamePromptAction::None;
    let t = tokens(theme);

    egui::Window::new(state.kind.title())
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(460.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new(state.kind.title())
                    .color(t.text_primary)
                    .size(ts::HEADING_SM)
                    .strong(),
            );
            if !state.context_label.is_empty() {
                ui.label(
                    RichText::new(&state.context_label)
                        .color(t.text_muted)
                        .size(ts::LABEL_SM),
                );
            }
            ui.add_space(space::S2);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.input)
                    .hint_text(state.kind.hint())
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(15.0)),
            );
            if state.just_opened {
                resp.request_focus();
                state.just_opened = false;
            }

            let validation = validate_name(&state.input);
            if let Err(err) = &validation {
                ui.add_space(space::S1);
                ui.label(
                    RichText::new(err.message())
                        .color(t.error)
                        .size(ts::LABEL_SM),
                );
            }

            ui.add_space(space::S3);
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            let valid = validation.is_ok();
            ui.horizontal(|ui| {
                let resp_btn = ui.add_enabled(
                    valid,
                    egui::Button::new(
                        RichText::new(state.kind.confirm_label())
                            .color(t.text_on_accent)
                            .size(ts::LABEL)
                            .strong(),
                    )
                    .min_size(egui::Vec2::new(0.0, 28.0))
                    .fill(t.accent)
                    .rounding(egui::Rounding::same(6.0)),
                );
                if (resp_btn.clicked() || (enter && valid)) && valid {
                    action =
                        NamePromptAction::Confirm(state.input.trim().to_string());
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Cancel").clicked()
                            || ui.input(|i| i.key_pressed(Key::Escape))
                        {
                            action = NamePromptAction::Cancel;
                        }
                    },
                );
            });
        });
    let _ = NameError::Empty;
    action
}
