//! First-run modal: pick a keymap preset (IdeUltra default, VS Code, PhpStorm).
//! Reused as the body of the "Settings: Choose Keymap" command later.

use egui::{Align2, RichText};

use crate::keymap::KeymapPreset;

pub enum KeymapPickerAction {
    None,
    Choose(KeymapPreset),
}

pub fn show(ctx: &egui::Context, current: KeymapPreset, first_run: bool) -> KeymapPickerAction {
    let mut action = KeymapPickerAction::None;
    let title = if first_run {
        "Welcome to IdeUltra"
    } else {
        "Choose a Keymap"
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(560.0)
        .show(ctx, |ui| {
            if first_run {
                ui.label(
                    "Pick a keyboard layout. You can change this later from \
                     Edit → Keymap or the command palette.",
                );
                ui.add_space(8.0);
            }
            ui.spacing_mut().item_spacing.y = 6.0;
            for preset in KeymapPreset::all() {
                let is_current = *preset == current;
                let resp = preset_card(ui, *preset, is_current);
                if resp.clicked() {
                    action = KeymapPickerAction::Choose(*preset);
                }
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new("This only sets your shortcuts. Theme, font size and everything else stay the same.")
                    .small()
                    .weak(),
            );
        });
    action
}

fn preset_card(ui: &mut egui::Ui, preset: KeymapPreset, selected: bool) -> egui::Response {
    let frame_color = if selected {
        ui.visuals().selection.bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let stroke = if selected {
        egui::Stroke::new(1.5, ui.visuals().selection.stroke.color)
    } else {
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
    };
    egui::Frame::default()
        .fill(frame_color)
        .stroke(stroke)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(preset.label()).strong().size(15.0));
                    ui.label(RichText::new(preset.description()).small().weak());
                });
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::TOP),
                    |ui| {
                        if selected {
                            ui.label(RichText::new("✓ Current").small());
                        }
                    },
                );
            });
        })
        .response
        .interact(egui::Sense::click())
}
