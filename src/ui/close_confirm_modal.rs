use egui::{Align2, RichText};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, modal_frame, primary_button};

pub enum CloseConfirmAction {
    None,
    Save,
    Discard,
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    name: &str,
    count: usize,
    theme: ColorTheme,
) -> CloseConfirmAction {
    let mut action = CloseConfirmAction::None;
    let t = tokens(theme);
    let title = if count > 1 {
        format!("Save {count} unsaved buffers?")
    } else {
        format!("Save changes to {name}?")
    };
    egui::Window::new(&title)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(440.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new(&title)
                    .color(t.text_primary)
                    .size(ts::HEADING_SM)
                    .strong(),
            );
            ui.add_space(space::S2);
            ui.label(
                RichText::new(if count > 1 {
                    "Your changes will be lost if you don't save them."
                } else {
                    "Your edits will be lost if you don't save them."
                })
                .color(t.text_secondary)
                .size(ts::LABEL),
            );
            ui.add_space(space::S3);
            ui.horizontal(|ui| {
                if primary_button(ui, theme, "Save").clicked() {
                    action = CloseConfirmAction::Save;
                }
                if ghost_button(ui, theme, "Don't Save").clicked() {
                    action = CloseConfirmAction::Discard;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            action = CloseConfirmAction::Cancel;
                        }
                    },
                );
            });
        });
    action
}
