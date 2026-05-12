//! Preferences window — a real UI for the settings that previously
//! required hand-editing `settings.json`. All changes apply live; close
//! the window via Esc, the close button, or pressing Cmd+, again.

use egui::{Align2, RichText, Slider, Ui};

use crate::editor::language::ColorTheme;
use crate::keymap::KeymapPreset;
use crate::persistence::IndentStyle;

pub enum PreferencesAction {
    None,
    Close,
    SetTheme(ColorTheme),
    SetZoom(f32),
    SetAutosaveOnFocusLoss(bool),
    SetMarkdownPreview(bool),
    OpenKeymapPicker,
    SwitchKeymap(KeymapPreset),
    ResetZoom,
    SetTrimWhitespace(bool),
    SetEnsureFinalNewline(bool),
    SetIndentStyle(IndentStyle),
    SetSoftWrap(bool),
}

pub struct PreferencesView<'a> {
    pub theme: ColorTheme,
    pub zoom: f32,
    pub autosave: bool,
    pub markdown_preview: bool,
    pub keymap: KeymapPreset,
    pub state_dir: Option<&'a std::path::Path>,
    pub trim_whitespace: bool,
    pub ensure_final_newline: bool,
    pub indent_style: IndentStyle,
    pub soft_wrap: bool,
}

pub fn show(ctx: &egui::Context, view: &PreferencesView<'_>) -> PreferencesAction {
    let mut action = PreferencesAction::None;
    egui::Window::new("Preferences")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(540.0)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            section(ui, "Appearance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Theme");
                    ui.add_space(8.0);
                    for opt in [ColorTheme::Dark, ColorTheme::Light] {
                        let label = opt.label();
                        if ui.radio(view.theme == opt, label).clicked() {
                            action = PreferencesAction::SetTheme(opt);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Zoom");
                    let mut zoom = view.zoom;
                    let resp = ui.add(
                        Slider::new(&mut zoom, 0.5..=3.0)
                            .step_by(0.1)
                            .fixed_decimals(1)
                            .text("×"),
                    );
                    if resp.changed() {
                        action = PreferencesAction::SetZoom(zoom);
                    }
                    if ui.small_button("Reset").clicked() {
                        action = PreferencesAction::ResetZoom;
                    }
                });
                ui.horizontal(|ui| {
                    let mut on = view.markdown_preview;
                    if ui.checkbox(&mut on, "Show markdown preview side-pane (⌥⌘M)").changed() {
                        action = PreferencesAction::SetMarkdownPreview(on);
                    }
                });
            });

            section(ui, "Editor", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Indent style");
                    ui.add_space(8.0);
                    let options = [
                        IndentStyle::Spaces(2),
                        IndentStyle::Spaces(4),
                        IndentStyle::Spaces(8),
                        IndentStyle::Tab,
                    ];
                    for opt in options {
                        if ui.radio(view.indent_style == opt, opt.label()).clicked() {
                            action = PreferencesAction::SetIndentStyle(opt);
                        }
                    }
                });
                let mut wrap = view.soft_wrap;
                if ui
                    .checkbox(&mut wrap, "Soft-wrap long lines in the editor")
                    .changed()
                {
                    action = PreferencesAction::SetSoftWrap(wrap);
                }
            });

            section(ui, "Files", |ui| {
                let mut on = view.autosave;
                if ui
                    .checkbox(&mut on, "Auto-save dirty buffers when the window loses focus")
                    .changed()
                {
                    action = PreferencesAction::SetAutosaveOnFocusLoss(on);
                }
                let mut trim = view.trim_whitespace;
                if ui
                    .checkbox(&mut trim, "Trim trailing whitespace on save")
                    .changed()
                {
                    action = PreferencesAction::SetTrimWhitespace(trim);
                }
                let mut nl = view.ensure_final_newline;
                if ui
                    .checkbox(&mut nl, "Ensure a final newline on save")
                    .changed()
                {
                    action = PreferencesAction::SetEnsureFinalNewline(nl);
                }
                ui.label(
                    RichText::new("Crash-recovery snapshots are written separately every second a buffer is dirty, and are surfaced on next launch.")
                        .small()
                        .weak(),
                );
            });

            section(ui, "Keymap", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Active preset");
                    ui.add_space(8.0);
                    for opt in KeymapPreset::all() {
                        if ui.radio(view.keymap == *opt, opt.label()).clicked() {
                            action = PreferencesAction::SwitchKeymap(*opt);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Open Keymap Picker…").clicked() {
                        action = PreferencesAction::OpenKeymapPicker;
                    }
                });
                ui.label(
                    RichText::new(view.keymap.description())
                        .small()
                        .weak(),
                );
            });

            if let Some(dir) = view.state_dir {
                section(ui, "Storage", |ui| {
                    ui.label(
                        RichText::new("State files (settings, session, recovery) live in:")
                            .small()
                            .weak(),
                    );
                    ui.label(
                        RichText::new(dir.display().to_string())
                            .monospace()
                            .size(12.0),
                    );
                });
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Changes apply immediately; Esc to close.")
                        .small()
                        .weak(),
                );
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Close").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            action = PreferencesAction::Close;
                        }
                    },
                );
            });
        });
    action
}

fn section(ui: &mut Ui, title: &str, f: impl FnOnce(&mut Ui)) {
    ui.vertical(|ui| {
        ui.label(RichText::new(title).strong().size(13.5));
        ui.add_space(2.0);
        egui::Frame::default()
            .fill(ui.visuals().widgets.inactive.bg_fill)
            .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                f(ui);
            });
    });
}
