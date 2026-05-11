use std::path::PathBuf;

use egui::Ui;

use crate::recovery::Recovery;

pub enum RecoveryAction {
    None,
    RestoreAll,
    DiscardAll,
    /// User dismissed the modal without choosing. Treat as "ask me again".
    Dismiss,
}

/// Render a startup modal listing the recoverable buffers. Returns the
/// user's choice for the parent to handle.
pub fn show(ctx: &egui::Context, recoveries: &[Recovery]) -> RecoveryAction {
    let mut action = RecoveryAction::None;
    egui::Window::new("Recover unsaved changes?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(520.0)
        .show(ctx, |ui| {
            ui.label(format!(
                "IdeUltra found {} unsaved buffer{} from your last session.",
                recoveries.len(),
                if recoveries.len() == 1 { "" } else { "s" },
            ));
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for r in recoveries {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(
                                egui::RichText::new(display_path(&r.meta.path))
                                    .monospace(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} bytes",
                                            r.contents.len()
                                        ))
                                        .small()
                                        .weak(),
                                    );
                                },
                            );
                        });
                    }
                });
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "Restore loads each buffer into a tab so you can review and save. \
                     Discard deletes the recovery files.",
                )
                .small()
                .weak(),
            );
            ui.separator();
            buttons(ui, &mut action);
        });
    action
}

fn buttons(ui: &mut Ui, action: &mut RecoveryAction) {
    ui.horizontal(|ui| {
        if ui.button("Restore all").clicked() {
            *action = RecoveryAction::RestoreAll;
        }
        if ui.button("Discard all").clicked() {
            *action = RecoveryAction::DiscardAll;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Decide later").clicked() {
                *action = RecoveryAction::Dismiss;
            }
        });
    });
}

fn display_path(p: &PathBuf) -> String {
    if let Some(home) = directories::UserDirs::new() {
        if let Ok(rel) = p.strip_prefix(home.home_dir()) {
            return format!("~/{}", rel.display());
        }
    }
    p.display().to_string()
}
