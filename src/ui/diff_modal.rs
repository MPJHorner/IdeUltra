use egui::{Color32, FontFamily, FontId, RichText, ScrollArea, Ui};

use crate::diff::{DiffKind, DiffSummary};
use crate::editor::language::ColorTheme;
use crate::style::{radii, space, tokens, ts};
use crate::ui::components::{ghost_button, hint_row, modal_frame, primary_button};

pub enum DiffModalAction {
    None,
    Reload,
    KeepMine,
    Close,
}

pub fn show(
    ctx: &egui::Context,
    file_name: &str,
    summary: &DiffSummary,
    theme: ColorTheme,
) -> DiffModalAction {
    let mut action = DiffModalAction::None;
    let mut close_clicked = false;
    let t = tokens(theme);

    egui::Window::new(format!("Changes on disk — {file_name}"))
        .title_bar(false)
        .collapsible(false)
        .resizable(true)
        .default_width(760.0)
        .default_height(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Changes on disk — {file_name}"))
                        .color(t.text_primary)
                        .size(ts::HEADING_SM)
                        .strong(),
                );
            });
            ui.label(
                RichText::new(format!(
                    "+{} added  ·  −{} removed  ·  {} unchanged",
                    summary.added, summary.removed, summary.equal,
                ))
                .color(t.text_muted)
                .size(ts::LABEL_SM),
            );
            ui.add_space(space::S2);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_lines(ui, theme, summary);
                });

            ui.add_space(space::S2);
            ui.separator();
            ui.horizontal(|ui| {
                if primary_button(ui, theme, "Reload from disk").clicked() {
                    action = DiffModalAction::Reload;
                }
                if ghost_button(ui, theme, "Keep mine").clicked() {
                    action = DiffModalAction::KeepMine;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ghost_button(ui, theme, "Close").clicked() {
                            close_clicked = true;
                        }
                        hint_row(ui, theme, &["esc close"]);
                    },
                );
            });
        });

    if close_clicked && matches!(action, DiffModalAction::None) {
        action = DiffModalAction::Close;
    }
    action
}

fn render_lines(ui: &mut Ui, theme: ColorTheme, summary: &DiffSummary) {
    let t = tokens(theme);
    let mono = FontId::new(ts::MONO_CODE, FontFamily::Monospace);
    let gutter = FontId::new(ts::MONO_UI, FontFamily::Monospace);
    for line in &summary.lines {
        let (bg, marker, marker_color) = match line.kind {
            DiffKind::Added => (
                t.success_bg,
                "+",
                t.success,
            ),
            DiffKind::Removed => (
                t.error_bg,
                "−",
                t.error,
            ),
            DiffKind::Equal => (
                Color32::TRANSPARENT,
                " ",
                t.text_muted,
            ),
        };
        egui::Frame::default()
            .fill(bg)
            .inner_margin(egui::Margin {
                left: space::S2,
                right: space::S2,
                top: 1.0,
                bottom: 1.0,
            })
            .rounding(egui::Rounding::same(radii::XS))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{:>4} {:>4}",
                            line.old_line.map(|n| n.to_string()).unwrap_or_default(),
                            line.new_line.map(|n| n.to_string()).unwrap_or_default(),
                        ))
                        .font(gutter.clone())
                        .color(t.text_muted),
                    );
                    ui.label(
                        RichText::new(format!(" {marker} "))
                            .font(gutter.clone())
                            .color(marker_color),
                    );
                    ui.label(RichText::new(&line.text).font(mono.clone()));
                });
            });
    }
    if summary.lines.is_empty() {
        ui.label(
            RichText::new("No differences")
                .color(t.text_muted)
                .size(ts::LABEL),
        );
    }
}
