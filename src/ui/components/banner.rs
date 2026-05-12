//! Banner: top-of-window strip used for the update notifier and any
//! future status callouts.

use egui::{Frame, Margin, RichText, Stroke, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{space, tokens, ts};

pub struct Banner;

impl Banner {
    /// `accent`-tinted banner with a leading message and a closure for
    /// right-aligned action buttons. The closure is called inside a
    /// right-to-left horizontal layout so buttons render right-to-left
    /// in source order.
    pub fn accent(
        ui: &mut Ui,
        theme: ColorTheme,
        message: &str,
        actions: impl FnOnce(&mut Ui),
    ) {
        let t = tokens(theme);
        Frame::default()
            .fill(t.accent_bg)
            .stroke(Stroke::new(1.0, t.accent_border))
            .inner_margin(Margin::symmetric(space::S4, space::S2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(message)
                            .color(t.accent)
                            .size(ts::LABEL)
                            .strong(),
                    );
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        actions,
                    );
                });
            });
    }

    /// Warning-tinted banner (amber) used for the external-change banner
    /// and other "you should look at this" calls-to-action.
    pub fn warning(
        ui: &mut Ui,
        theme: ColorTheme,
        message: &str,
        actions: impl FnOnce(&mut Ui),
    ) {
        let t = tokens(theme);
        Frame::default()
            .fill(t.warning_bg)
            .stroke(Stroke::new(1.0, t.warning_border))
            .inner_margin(Margin::symmetric(space::S4, space::S2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(message)
                            .color(t.warning)
                            .size(ts::LABEL)
                            .strong(),
                    );
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        actions,
                    );
                });
            });
    }
}
