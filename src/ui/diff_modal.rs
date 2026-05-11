use egui::{Color32, FontFamily, FontId, RichText, ScrollArea, Ui};

use crate::diff::{DiffKind, DiffSummary};

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
) -> DiffModalAction {
    let mut action = DiffModalAction::None;
    let mut close_clicked = false;

    egui::Window::new(format!("Changes on disk — {file_name}"))
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .default_height(480.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "+{} added  ·  −{} removed  ·  {} unchanged",
                        summary.added, summary.removed, summary.equal,
                    ))
                    .small()
                    .weak(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });
            ui.separator();

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_lines(ui, summary);
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Reload from disk").clicked() {
                    action = DiffModalAction::Reload;
                }
                if ui.button("Keep mine").clicked() {
                    action = DiffModalAction::KeepMine;
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Close").clicked() {
                            close_clicked = true;
                        }
                    },
                );
            });
        });

    if close_clicked && matches!(action, DiffModalAction::None) {
        action = DiffModalAction::Close;
    }
    action
}

fn render_lines(ui: &mut Ui, summary: &DiffSummary) {
    let font = FontId::new(13.0, FontFamily::Monospace);
    for line in &summary.lines {
        let (bg, marker, fg_dim) = match line.kind {
            DiffKind::Added => (
                Color32::from_rgba_premultiplied(35, 110, 60, 60),
                "+",
                Color32::from_rgb(120, 220, 150),
            ),
            DiffKind::Removed => (
                Color32::from_rgba_premultiplied(150, 50, 50, 60),
                "−",
                Color32::from_rgb(255, 140, 140),
            ),
            DiffKind::Equal => (
                Color32::TRANSPARENT,
                " ",
                ui.visuals().weak_text_color(),
            ),
        };
        let frame = egui::Frame::default()
            .fill(bg)
            .inner_margin(egui::Margin {
                left: 6.0,
                right: 6.0,
                top: 1.0,
                bottom: 1.0,
            });
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{:>4} {:>4}",
                        line.old_line.map(|n| n.to_string()).unwrap_or_default(),
                        line.new_line.map(|n| n.to_string()).unwrap_or_default(),
                    ))
                    .font(font.clone())
                    .color(fg_dim),
                );
                ui.label(
                    RichText::new(format!(" {marker} "))
                        .font(font.clone())
                        .color(fg_dim),
                );
                ui.label(RichText::new(&line.text).font(font.clone()));
            });
        });
    }
    if summary.lines.is_empty() {
        ui.label(RichText::new("No differences").weak());
    }
}
