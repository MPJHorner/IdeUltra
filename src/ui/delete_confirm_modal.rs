//! Confirmation modal for irreversible Delete operations on tree nodes.

use std::path::PathBuf;

use egui::{Align2, RichText};

pub enum DeleteAction {
    None,
    Confirm,
    Cancel,
}

pub fn show(ctx: &egui::Context, target: &PathBuf, is_dir: bool) -> DeleteAction {
    let mut action = DeleteAction::None;
    let title = if is_dir {
        "Delete folder?"
    } else {
        "Delete file?"
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(440.0)
        .show(ctx, |ui| {
            ui.label(RichText::new(target.display().to_string()).monospace());
            ui.add_space(6.0);
            ui.label(
                RichText::new(if is_dir {
                    "The folder and everything inside it will be moved to the system trash."
                } else {
                    "The file will be moved to the system trash."
                })
                .small(),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    action = DeleteAction::Confirm;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Cancel").clicked()
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
