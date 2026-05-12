use std::path::PathBuf;

use egui::{Id, Rect, ScrollArea, Sense, Ui};

use crate::editor::EditorTab;

pub enum TabAction {
    None,
    Activate(usize),
    Close(usize),
    CloseOthers(usize),
    CloseAll,
    CopyPath(PathBuf),
    RevealInFinder(PathBuf),
    /// Move the tab at `from` so it sits immediately before the one
    /// currently at `to` (or to the end if `to == tabs.len()`).
    Reorder { from: usize, to: usize },
}

pub fn show(ui: &mut Ui, tabs: &[EditorTab], active: usize) -> TabAction {
    let mut action = TabAction::None;
    // Track per-tab rects so we can resolve a drag-release back to a
    // target index after the loop.
    let mut tab_rects: Vec<(usize, Rect)> = Vec::with_capacity(tabs.len());
    let drag_id = Id::new("ide_tab_drag");
    let dragging_idx: Option<usize> = ui.ctx().data(|d| d.get_temp(drag_id));

    ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (idx, tab) in tabs.iter().enumerate() {
                    let is_active = idx == active;
                    let is_being_dragged = dragging_idx == Some(idx);
                    let dot = if tab.is_dirty() { "● " } else { "" };
                    let label = format!("{dot}{}", tab.display_name);

                    let resp = ui
                        .selectable_label(is_active, label)
                        .interact(Sense::click_and_drag());
                    tab_rects.push((idx, resp.rect));

                    if resp.clicked() && !resp.drag_started() {
                        action = TabAction::Activate(idx);
                    }
                    if resp.middle_clicked() {
                        action = TabAction::Close(idx);
                    }

                    if resp.drag_started() {
                        ui.ctx().data_mut(|d| d.insert_temp(drag_id, idx));
                    }
                    if is_being_dragged {
                        // Subtle visual cue while dragging — paint a faint
                        // accent border around the source.
                        ui.painter().rect_stroke(
                            resp.rect,
                            egui::Rounding::same(3.0),
                            egui::Stroke::new(1.0, ui.visuals().selection.stroke.color),
                        );
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

    // Resolve the drag on release: which tab rect contains the pointer?
    if let Some(from) = dragging_idx {
        let pointer_released = ui.input(|i| i.pointer.any_released());
        if pointer_released {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                let target = tab_rects
                    .iter()
                    .find(|(_, rect)| rect.contains(pos))
                    .map(|(idx, _)| *idx);
                if let Some(target) = target {
                    if target != from {
                        action = TabAction::Reorder { from, to: target };
                    }
                }
            }
            ui.ctx().data_mut(|d| d.remove::<usize>(drag_id));
        }
    }

    action
}
