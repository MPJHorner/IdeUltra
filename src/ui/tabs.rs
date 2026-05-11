use egui::{ScrollArea, Ui};

use crate::editor::EditorTab;

pub enum TabAction {
    None,
    Activate(usize),
    Close(usize),
}

pub fn show(ui: &mut Ui, tabs: &[EditorTab], active: usize) -> TabAction {
    let mut action = TabAction::None;
    ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (idx, tab) in tabs.iter().enumerate() {
                    let dot = if tab.is_dirty() { "● " } else { "" };
                    let label = format!("{dot}{}", tab.display_name);
                    let is_active = idx == active;
                    let resp = ui.selectable_label(is_active, label);
                    if resp.clicked() {
                        action = TabAction::Activate(idx);
                    }
                    let close = ui.add(
                        egui::Button::new(egui::RichText::new("×").small())
                            .small()
                            .frame(false),
                    );
                    if close.clicked() {
                        action = TabAction::Close(idx);
                    }
                    if resp.middle_clicked() {
                        action = TabAction::Close(idx);
                    }
                    ui.separator();
                }
            });
        });
    action
}
