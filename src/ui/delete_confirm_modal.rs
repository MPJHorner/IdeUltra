use std::path::PathBuf;

use egui::{Align2, RichText};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, modal_frame, primary_button};

pub enum DeleteAction {
    None,
    Confirm,
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    target: &PathBuf,
    is_dir: bool,
    theme: ColorTheme,
) -> DeleteAction {
    let mut action = DeleteAction::None;
    let t = tokens(theme);
    let title = if is_dir {
        "Delete folder?"
    } else {
        "Delete file?"
    };
    egui::Window::new(title)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(480.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new(title)
                    .color(t.text_primary)
                    .size(ts::HEADING_SM)
                    .strong(),
            );
            ui.add_space(space::S2);
            ui.label(
                RichText::new(target.display().to_string())
                    .color(t.text_secondary)
                    .monospace()
                    .size(ts::MONO_UI),
            );
            ui.add_space(space::S2);
            ui.label(
                RichText::new(if is_dir {
                    "The folder and everything inside it will be moved to the system trash."
                } else {
                    "The file will be moved to the system trash."
                })
                .color(t.text_muted)
                .size(ts::LABEL_SM),
            );
            ui.add_space(space::S3);
            ui.horizontal(|ui| {
                if primary_button(ui, theme, "Delete").clicked() {
                    action = DeleteAction::Confirm;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            action = DeleteAction::Cancel;
                        }
                    },
                );
            });
        });
    action
}
