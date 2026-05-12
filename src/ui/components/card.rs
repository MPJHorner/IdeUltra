//! Card: a rounded surface with optional border + padding. The base
//! container for non-interactive groupings (settings sections,
//! sidebar group headers, find bar).

use egui::{Margin, Rounding, Stroke, Ui};

use crate::editor::language::ColorTheme;
use crate::style::{radii, space, tokens};

pub struct Card {
    pub padding: Margin,
    pub rounding: Rounding,
    pub stroke_strength: u8,
    pub elevated: bool,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            padding: Margin::symmetric(space::S4, space::S3),
            rounding: Rounding::same(radii::MD),
            stroke_strength: 1,
            elevated: false,
        }
    }
}

impl Card {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn elevated(mut self) -> Self {
        self.elevated = true;
        self
    }
    pub fn padded(mut self, m: Margin) -> Self {
        self.padding = m;
        self
    }
    pub fn no_border(mut self) -> Self {
        self.stroke_strength = 0;
        self
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        theme: ColorTheme,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let t = tokens(theme);
        let fill = if self.elevated { t.bg_elevated } else { t.bg_surface };
        let stroke = if self.stroke_strength == 0 {
            Stroke::NONE
        } else {
            Stroke::new(1.0, t.border_subtle)
        };
        let frame = egui::Frame::default()
            .fill(fill)
            .stroke(stroke)
            .rounding(self.rounding)
            .inner_margin(self.padding);
        let mut result = None;
        frame.show(ui, |ui| {
            result = Some(content(ui));
        });
        result.expect("frame.show calls its closure exactly once")
    }
}
