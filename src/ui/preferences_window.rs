//! Preferences window. Hand-rolled UI built on top of the components
//! module so every section follows the style guide.

use egui::{Align2, RichText, Slider, Ui};

use crate::editor::language::ColorTheme;
use crate::keymap::KeymapPreset;
use crate::persistence::IndentStyle;
use crate::style::{space, tokens, ts};
use crate::ui::components::{ghost_button, hint_row, modal_frame, section};

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
    let theme = view.theme;
    let t = tokens(theme);

    egui::Window::new("Preferences")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(580.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = space::S2;

            // Title row.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Preferences")
                        .color(t.text_primary)
                        .size(ts::HEADING_SM)
                        .strong(),
                );
            });
            ui.add_space(space::S1);

            section(ui, theme, "Appearance", |ui| {
                ui.horizontal(|ui| {
                    label(ui, theme, "Theme");
                    ui.add_space(space::S2);
                    for opt in [ColorTheme::Dark, ColorTheme::Light] {
                        if ui.radio(view.theme == opt, opt.label()).clicked() {
                            action = PreferencesAction::SetTheme(opt);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    label(ui, theme, "Zoom");
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
                    if ghost_button(ui, theme, "Reset").clicked() {
                        action = PreferencesAction::ResetZoom;
                    }
                });
                let mut on = view.markdown_preview;
                if ui
                    .checkbox(&mut on, "Show markdown preview side-pane (⌥⌘M)")
                    .changed()
                {
                    action = PreferencesAction::SetMarkdownPreview(on);
                }
            });

            section(ui, theme, "Editor", |ui| {
                ui.horizontal(|ui| {
                    label(ui, theme, "Indent");
                    ui.add_space(space::S2);
                    for opt in [
                        IndentStyle::Spaces(2),
                        IndentStyle::Spaces(4),
                        IndentStyle::Spaces(8),
                        IndentStyle::Tab,
                    ] {
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

            section(ui, theme, "Files", |ui| {
                let mut on = view.autosave;
                if ui
                    .checkbox(
                        &mut on,
                        "Auto-save dirty buffers when the window loses focus",
                    )
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
                    RichText::new(
                        "Crash-recovery snapshots are written every second a buffer is dirty.",
                    )
                    .color(t.text_muted)
                    .size(ts::LABEL_SM),
                );
            });

            section(ui, theme, "Keymap", |ui| {
                ui.horizontal(|ui| {
                    label(ui, theme, "Preset");
                    ui.add_space(space::S2);
                    for opt in KeymapPreset::all() {
                        if ui.radio(view.keymap == *opt, opt.label()).clicked() {
                            action = PreferencesAction::SwitchKeymap(*opt);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ghost_button(ui, theme, "Open Keymap Picker…").clicked() {
                        action = PreferencesAction::OpenKeymapPicker;
                    }
                });
                ui.label(
                    RichText::new(view.keymap.description())
                        .color(t.text_muted)
                        .size(ts::LABEL_SM),
                );
            });

            if let Some(dir) = view.state_dir {
                section(ui, theme, "Storage", |ui| {
                    ui.label(
                        RichText::new("State files live in:")
                            .color(t.text_muted)
                            .size(ts::LABEL_SM),
                    );
                    ui.label(
                        RichText::new(dir.display().to_string())
                            .color(t.text_secondary)
                            .monospace()
                            .size(ts::MONO_UI),
                    );
                });
            }

            ui.add_space(space::S2);
            ui.separator();
            ui.horizontal(|ui| {
                hint_row(ui, theme, &["esc close", "Changes apply immediately"]);
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Close").clicked()
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

fn label(ui: &mut Ui, theme: ColorTheme, text: &str) {
    ui.label(
        RichText::new(text)
            .color(tokens(theme).text_secondary)
            .size(ts::LABEL),
    );
}
