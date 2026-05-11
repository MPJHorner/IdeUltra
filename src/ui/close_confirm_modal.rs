use egui::{Align2, RichText};

pub enum CloseConfirmAction {
    None,
    Save,
    Discard,
    Cancel,
}

/// Modal shown when the user tries to close (or quit) with a dirty buffer.
/// `name` is the filename the user sees; `count` lets us pluralize the
/// title when bulk-closing.
pub fn show(ctx: &egui::Context, name: &str, count: usize) -> CloseConfirmAction {
    let mut action = CloseConfirmAction::None;
    let title = if count > 1 {
        format!("Save {count} unsaved buffers?")
    } else {
        format!("Save changes to {name}?")
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(420.0)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(if count > 1 {
                    "Your changes will be lost if you don't save them."
                } else {
                    "Your edits will be lost if you don't save them."
                })
                .small(),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    action = CloseConfirmAction::Save;
                }
                if ui.button("Don't Save").clicked() {
                    action = CloseConfirmAction::Discard;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Cancel").clicked()
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
