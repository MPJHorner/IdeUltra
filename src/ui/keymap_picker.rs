//! First-run modal: pick a keymap preset. Also reused as the body of
//! "Settings: Choose Keymap…" later.

use egui::{Align2, RichText};

use crate::editor::language::ColorTheme;
use crate::keymap::KeymapPreset;
use crate::style::{radii, space, tokens, ts};
use crate::ui::components::modal_frame;

pub enum KeymapPickerAction {
    None,
    Choose(KeymapPreset),
}

pub fn show(
    ctx: &egui::Context,
    current: KeymapPreset,
    first_run: bool,
    theme: ColorTheme,
) -> KeymapPickerAction {
    let mut action = KeymapPickerAction::None;
    let t = tokens(theme);
    let title = if first_run {
        "Welcome to IdeUltra"
    } else {
        "Choose a Keymap"
    };
    egui::Window::new(title)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(580.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.label(
                RichText::new(title)
                    .color(t.text_primary)
                    .size(ts::HEADING_MD)
                    .strong(),
            );
            if first_run {
                ui.add_space(space::S1);
                ui.label(
                    RichText::new(
                        "Pick a keyboard layout. You can change this later from \
                         Edit → Keymap or the command palette.",
                    )
                    .color(t.text_secondary)
                    .size(ts::LABEL),
                );
            }
            ui.add_space(space::S3);
            ui.spacing_mut().item_spacing.y = space::S2;
            for preset in KeymapPreset::all() {
                let is_current = *preset == current;
                if preset_card(ui, theme, *preset, is_current).clicked() {
                    action = KeymapPickerAction::Choose(*preset);
                }
            }
            ui.add_space(space::S2);
            ui.label(
                RichText::new(
                    "This only sets your shortcuts. Theme, font size, and \
                     everything else stay the same.",
                )
                .color(t.text_muted)
                .size(ts::LABEL_SM),
            );
        });
    action
}

fn preset_card(
    ui: &mut egui::Ui,
    theme: ColorTheme,
    preset: KeymapPreset,
    selected: bool,
) -> egui::Response {
    let t = tokens(theme);
    let (fill, stroke) = if selected {
        (
            t.accent_bg,
            egui::Stroke::new(1.5, t.accent),
        )
    } else {
        (
            t.bg_elevated,
            egui::Stroke::new(1.0, t.border_default),
        )
    };
    egui::Frame::default()
        .fill(fill)
        .stroke(stroke)
        .rounding(egui::Rounding::same(radii::MD))
        .inner_margin(egui::Margin::symmetric(space::S4, space::S3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(preset.label())
                            .color(t.text_primary)
                            .size(ts::BODY)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(preset.description())
                            .color(t.text_muted)
                            .size(ts::LABEL_SM),
                    );
                });
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::TOP),
                    |ui| {
                        if selected {
                            ui.label(
                                RichText::new("✓ Current")
                                    .color(t.accent)
                                    .size(ts::LABEL_SM)
                                    .strong(),
                            );
                        }
                    },
                );
            });
        })
        .response
        .interact(egui::Sense::click())
}
