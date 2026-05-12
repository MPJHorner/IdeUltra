use egui::{Align, Align2, Color32, Layout, RichText};

use crate::editor::language::ColorTheme;
use crate::editor::EditorTab;
use crate::mru::TabMru;
use crate::style::{space, tokens, ts};
use crate::ui::components::modal_frame;

#[derive(Default)]
pub struct QuickSwitcherState {
    pub open: bool,
    pub selected: usize,
}

pub fn show(
    ctx: &egui::Context,
    tabs: &[EditorTab],
    mru: &TabMru,
    selected: usize,
    theme: ColorTheme,
) {
    if tabs.is_empty() || mru.is_empty() {
        return;
    }
    let t = tokens(theme);
    egui::Window::new("Recent Tabs")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, -40.0])
        .default_width(400.0)
        .frame(modal_frame(ctx, theme))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                RichText::new("Hold ⌃, press Tab to cycle · release to switch")
                    .color(t.text_muted)
                    .size(ts::CAPTION),
            );
            ui.add_space(space::S1);
            ui.separator();
            for (pos, tab_idx) in mru.order.iter().enumerate() {
                let Some(tab) = tabs.get(*tab_idx) else { continue };
                let is_selected = pos == selected;
                let bg = if is_selected {
                    t.accent_bg
                } else {
                    Color32::TRANSPARENT
                };
                let fg = if is_selected { t.accent } else { t.text_primary };
                let dim = if is_selected {
                    t.accent.linear_multiply(0.7)
                } else {
                    t.text_muted
                };
                egui::Frame::default()
                    .fill(bg)
                    .stroke(if is_selected {
                        egui::Stroke::new(1.0, t.accent_border)
                    } else {
                        egui::Stroke::NONE
                    })
                    .rounding(egui::Rounding::same(crate::style::radii::SM))
                    .inner_margin(egui::Margin::symmetric(space::S3, space::S1 + 1.0))
                    .show(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            let dot = if tab.is_dirty() { "● " } else { "" };
                            ui.label(
                                RichText::new(format!("{dot}{}", tab.display_name))
                                    .color(fg)
                                    .size(ts::BODY)
                                    .strong(),
                            );
                            if let Some(parent) =
                                tab.path.parent().and_then(|p| p.to_str())
                            {
                                ui.add_space(space::S2);
                                ui.label(
                                    RichText::new(short_path(parent))
                                        .color(dim)
                                        .size(ts::LABEL_SM),
                                );
                            }
                        });
                    });
            }
        });
}

fn short_path(p: &str) -> String {
    if p.chars().count() > 50 {
        let tail: String = p
            .chars()
            .rev()
            .take(50)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("…{tail}")
    } else {
        p.to_string()
    }
}
