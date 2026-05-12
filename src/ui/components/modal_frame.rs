//! ModalFrame: the floating-surface wrapper used by every modal window
//! (finder, command palette, preferences, picker, confirms).
//!
//! Visual contract per `STYLE_GUIDE.md` §3.2:
//!   * `bg_modal` background
//!   * `radius.lg` corners
//!   * `shadow_lg`
//!   * subtle 1-px border (the shadow does most of the depth work)
//!   * generous inner margin

use egui::Frame;

use crate::editor::language::ColorTheme;
use crate::style::{radii, space, tokens};

pub fn modal_frame(ctx: &egui::Context, theme: ColorTheme) -> Frame {
    let t = tokens(theme);
    Frame::window(&ctx.style())
        .fill(t.bg_modal)
        .stroke(egui::Stroke::new(1.0, t.border_subtle))
        .rounding(egui::Rounding::same(radii::LG))
        .shadow(t.shadow_lg)
        .inner_margin(egui::Margin::symmetric(space::S5, space::S4))
}
