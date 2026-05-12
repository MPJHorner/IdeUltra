use std::path::PathBuf;

use egui::{Align2, RichText};

use crate::editor::language::ColorTheme;
use crate::recovery::Recovery;
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, modal_frame, primary_button};

pub enum RecoveryAction {
    None,
    RestoreAll,
    DiscardAll,
    Dismiss,
}

pub fn show(
    ctx: &egui::Context,
    recoveries: &[Recovery],
    theme: ColorTheme,
) -> RecoveryAction {
    let mut action = RecoveryAction::None;
    let t = tokens(theme);
    egui::Window::new("Recover unsaved changes?")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(560.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Recover unsaved changes?")
                    .color(t.text_primary)
                    .size(ts::HEADING_SM)
                    .strong(),
            );
            ui.add_space(space::S1);
            ui.label(
                RichText::new(format!(
                    "IdeUltra found {} unsaved buffer{} from your last session.",
                    recoveries.len(),
                    if recoveries.len() == 1 { "" } else { "s" },
                ))
                .color(t.text_secondary)
                .size(ts::LABEL),
            );
            ui.add_space(space::S3);
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for r in recoveries {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("•").color(t.text_muted).size(ts::BODY),
                            );
                            ui.label(
                                RichText::new(display_path(&r.meta.path))
                                    .color(t.text_primary)
                                    .monospace()
                                    .size(ts::MONO_UI),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "{} bytes",
                                            r.contents.len()
                                        ))
                                        .color(t.text_muted)
                                        .size(ts::CAPTION),
                                    );
                                },
                            );
                        });
                    }
                });
            ui.add_space(space::S3);
            ui.label(
                RichText::new(
                    "Restore loads each buffer into a tab so you can review and save. \
                     Discard deletes the recovery files.",
                )
                .color(t.text_muted)
                .size(ts::LABEL_SM),
            );
            ui.add_space(space::S3);
            ui.horizontal(|ui| {
                if primary_button(ui, theme, "Restore all").clicked() {
                    action = RecoveryAction::RestoreAll;
                }
                if ghost_button(ui, theme, "Discard all").clicked() {
                    action = RecoveryAction::DiscardAll;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Decide later").clicked() {
                            action = RecoveryAction::Dismiss;
                        }
                    },
                );
            });
        });
    action
}

fn display_path(p: &PathBuf) -> String {
    if let Some(home) = directories::UserDirs::new() {
        if let Ok(rel) = p.strip_prefix(home.home_dir()) {
            return format!("~/{}", rel.display());
        }
    }
    p.display().to_string()
}
