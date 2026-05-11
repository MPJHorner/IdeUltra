use std::path::PathBuf;

use egui::{ScrollArea, Sense, Ui};

use crate::editor::EditorTab;

pub enum TabAction {
    None,
    Activate(usize),
    Close(usize),
    CloseOthers(usize),
    CloseAll,
    CopyPath(PathBuf),
    RevealInFinder(PathBuf),
}

pub fn show(ui: &mut Ui, tabs: &[EditorTab], active: usize) -> TabAction {
    let mut action = TabAction::None;
    ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (idx, tab) in tabs.iter().enumerate() {
                    let is_active = idx == active;
                    let dot = if tab.is_dirty() { "● " } else { "" };
                    let label = format!("{dot}{}", tab.display_name);

                    let resp = ui.selectable_label(is_active, label);
                    if resp.clicked() {
                        action = TabAction::Activate(idx);
                    }
                    if resp.middle_clicked() {
                        action = TabAction::Close(idx);
                    }
                    resp.context_menu(|ui| {
                        if ui.button("Close").clicked() {
                            action = TabAction::Close(idx);
                            ui.close_menu();
                        }
                        if ui.button("Close Others").clicked() {
                            action = TabAction::CloseOthers(idx);
                            ui.close_menu();
                        }
                        if ui.button("Close All").clicked() {
                            action = TabAction::CloseAll;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Copy Path").clicked() {
                            action = TabAction::CopyPath(tab.path.clone());
                            ui.close_menu();
                        }
                        if ui.button("Reveal in Finder").clicked() {
                            action = TabAction::RevealInFinder(tab.path.clone());
                            ui.close_menu();
                        }
                    });

                    // Tiny close affordance next to each tab.
                    let close = ui.add(
                        egui::Button::new(egui::RichText::new("×").small())
                            .small()
                            .frame(false)
                            .sense(Sense::click()),
                    );
                    if close.clicked() {
                        action = TabAction::Close(idx);
                    }
                    ui.separator();
                }
            });
        });
    action
}
