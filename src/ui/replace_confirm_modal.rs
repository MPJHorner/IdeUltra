use egui::{Align2, RichText};

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
) -> ReplaceConfirmAction {
    let mut action = ReplaceConfirmAction::None;
    egui::Window::new("Replace in project?")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(520.0)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(format!(
                    "Replace {match_count} match(es) across {file_count} file(s)."
                ))
                .strong(),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("Search:      «{query}»")).monospace(),
            );
            ui.label(
                RichText::new(format!("Replace with: «{replacement}»")).monospace(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Files are written through to disk. There is no project-wide undo — \
                     please make sure your changes are committed or backed up first.",
                )
                .small(),
            );
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Replace All").clicked() {
                    action = ReplaceConfirmAction::Confirm;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Cancel").clicked()
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
