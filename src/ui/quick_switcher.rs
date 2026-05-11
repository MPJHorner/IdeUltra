use egui::{Align, Align2, Color32, Layout, RichText};

use crate::editor::EditorTab;
use crate::mru::TabMru;

#[derive(Default)]
pub struct QuickSwitcherState {
    pub open: bool,
    /// Position in the MRU list. 0 = most-recent, etc.
    pub selected: usize,
}

pub fn show(ctx: &egui::Context, tabs: &[EditorTab], mru: &TabMru, selected: usize) {
    if tabs.is_empty() || mru.is_empty() {
        return;
    }
    egui::Window::new("Recent Tabs")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, -40.0])
        .default_width(380.0)
        .frame(modal_frame(ctx))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                RichText::new("Hold ⌃, press Tab to cycle, release to switch")
                    .small()
                    .weak(),
            );
            ui.separator();
            for (pos, tab_idx) in mru.order.iter().enumerate() {
                let Some(tab) = tabs.get(*tab_idx) else { continue };
                let row_bg = if pos == selected {
                    ui.visuals().selection.bg_fill
                } else {
                    Color32::TRANSPARENT
                };
                let row_fg = if pos == selected {
                    ui.visuals().selection.stroke.color
                } else {
                    ui.visuals().text_color()
                };
                let dim = if pos == selected {
                    ui.visuals().selection.stroke.color.linear_multiply(0.7)
                } else {
                    ui.visuals().weak_text_color()
                };
                let frame = egui::Frame::default()
                    .fill(row_bg)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 5.0));
                frame.show(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        let dot = if tab.is_dirty() { "● " } else { "" };
                        ui.label(
                            RichText::new(format!("{dot}{}", tab.display_name))
                                .strong()
                                .color(row_fg),
                        );
                        if let Some(parent) =
                            tab.path.parent().and_then(|p| p.to_str())
                        {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(short_path(parent))
                                    .size(12.0)
                                    .color(dim),
                            );
                        }
                    });
                });
            }
        });
}

fn modal_frame(ctx: &egui::Context) -> egui::Frame {
    let v = ctx.style().visuals.clone();
    egui::Frame::window(&ctx.style())
        .fill(v.window_fill)
        .stroke(egui::Stroke::new(1.0, v.widgets.noninteractive.bg_stroke.color))
        .rounding(egui::Rounding::same(10.0))
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(80),
        })
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
}

fn short_path(p: &str) -> String {
    // Trim to ~50 chars from the start, with an ellipsis prefix.
    if p.chars().count() > 50 {
        let tail: String = p.chars().rev().take(50).collect::<Vec<_>>().into_iter().rev().collect();
        format!("…{tail}")
    } else {
        p.to_string()
    }
}
