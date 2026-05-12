//! Centred input modal used for "New File", "New Folder", and "Rename".
//! Shows a single line input plus inline validation errors from
//! `crate::fs_ops::validate_name`.

use egui::{Align2, Key, RichText};

use crate::fs_ops::{validate_name, NameError};

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
}

pub struct NamePromptState {
    pub kind: NamePromptKind,
    /// Subtitle showing the directory or current name, e.g. "in src/".
    pub context_label: String,
    pub input: String,
    pub just_opened: bool,
}

pub enum NamePromptAction {
    None,
    Confirm(String),
    Cancel,
}

pub fn show(ctx: &egui::Context, state: &mut NamePromptState) -> NamePromptAction {
    let mut action = NamePromptAction::None;
    egui::Window::new(state.kind.title())
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(420.0)
        .show(ctx, |ui| {
            if !state.context_label.is_empty() {
                ui.label(RichText::new(&state.context_label).small().weak());
                ui.add_space(4.0);
            }
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.input)
                    .hint_text(match state.kind {
                        NamePromptKind::NewFile => "filename.ext",
                        NamePromptKind::NewFolder => "folder name",
                        NamePromptKind::Rename => "new name",
                    })
                    .desired_width(f32::INFINITY),
            );
            if state.just_opened {
                resp.request_focus();
                state.just_opened = false;
            }

            let validation = validate_name(&state.input);
            if let Err(err) = &validation {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(err.message())
                        .small()
                        .color(egui::Color32::from_rgb(220, 90, 90)),
                );
            }

            ui.add_space(8.0);
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            let valid = validation.is_ok();
            ui.horizontal(|ui| {
                let btn = egui::Button::new(state.kind.confirm_label());
                if ui.add_enabled(valid, btn).clicked() || (enter && valid) {
                    action = NamePromptAction::Confirm(state.input.trim().to_string());
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Cancel").clicked()
                            || ui.input(|i| i.key_pressed(Key::Escape))
                        {
                            action = NamePromptAction::Cancel;
                        }
                    },
                );
            });
        });
    let _ = NameError::Empty; // ensure the import isn't pruned
    action
}
