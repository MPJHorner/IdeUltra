use egui::{Align2, RichText};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, modal_frame, primary_button};

pub enum ReplaceConfirmAction {
    None,
    Confirm,
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    file_count: usize,
    match_count: usize,
    query: &str,
    replacement: &str,
    theme: ColorTheme,
) -> ReplaceConfirmAction {
    let mut action = ReplaceConfirmAction::None;
    let t = tokens(theme);
    egui::Window::new("Replace in project?")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(560.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Replace in project?")
                    .color(t.text_primary)
                    .size(ts::HEADING_SM)
                    .strong(),
            );
            ui.add_space(space::S1);
            ui.label(
                RichText::new(format!(
                    "Replace {match_count} match(es) across {file_count} file(s)."
                ))
                .color(t.text_secondary)
                .size(ts::LABEL),
            );
            ui.add_space(space::S2);
            ui.label(
                RichText::new(format!("Search:      «{query}»"))
                    .color(t.text_secondary)
                    .monospace()
                    .size(ts::MONO_UI),
            );
            ui.label(
                RichText::new(format!("Replace with: «{replacement}»"))
                    .color(t.text_secondary)
                    .monospace()
                    .size(ts::MONO_UI),
            );
            ui.add_space(space::S3);
            ui.label(
                RichText::new(
                    "Files are written through to disk. There is no project-wide undo — \
                     make sure your changes are committed or backed up first.",
                )
                .color(t.warning)
                .size(ts::LABEL_SM),
            );
            ui.add_space(space::S3);
            ui.horizontal(|ui| {
                if primary_button(ui, theme, "Replace All").clicked() {
                    action = ReplaceConfirmAction::Confirm;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            action = ReplaceConfirmAction::Cancel;
                        }
                    },
                );
            });
        });
    action
}
