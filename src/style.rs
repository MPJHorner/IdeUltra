//! Centralised design tokens and egui visuals.
//!
//! This file is the runtime expression of [`STYLE_GUIDE.md`](../../STYLE_GUIDE.md).
//! Every UI surface should read its colors, spacing, radii, and shadows
//! from here — never from hard-coded hex literals scattered through the
//! widget code.
//!
//! Two public entry points:
//!
//!   1. [`tokens(theme)`] — returns the semantic [`Tokens`] for the
//!      requested theme. Cheap (returns a `&'static` reference).
//!   2. [`apply(ctx, theme)`] — walks the tokens and configures egui's
//!      `Visuals` + `Style::spacing` so built-in widgets render to spec.

use egui::epaint::Shadow;
use egui::{Color32, Context, Rounding, Stroke, Visuals};
use once_cell::sync::Lazy;

use crate::editor::language::ColorTheme;

// ───────────────────────────────────────────────────────────────────────
// Token type
// ───────────────────────────────────────────────────────────────────────

/// All design tokens, resolved for one theme.
///
/// Member names map directly to STYLE_GUIDE.md §2. Add new tokens here
/// only when the style guide gains a new entry — keep guide and code in
/// lock-step.
///
/// `#[allow(dead_code)]` is intentional: every field is a public surface
/// that downstream UI files may use. Many are consumed today; a few
/// (notably the VCS palette beyond modified/added/deleted) await the
/// features they belong to.
#[allow(dead_code)]
pub struct Tokens {
    // surfaces (dark → light layering)
    pub bg_canvas: Color32,
    pub bg_chrome: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,
    pub bg_modal: Color32,
    pub bg_subtle: Color32,
    pub bg_hover: Color32,
    pub bg_active: Color32,

    // foreground
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub text_on_accent: Color32,

    // borders
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,

    // accent
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_bg: Color32,
    pub accent_border: Color32,

    // semantic status
    pub error: Color32,
    pub error_bg: Color32,
    pub error_border: Color32,
    pub warning: Color32,
    pub warning_bg: Color32,
    pub warning_border: Color32,
    pub success: Color32,
    pub success_bg: Color32,
    pub success_border: Color32,
    pub info: Color32,
    pub info_bg: Color32,
    pub info_border: Color32,

    // vcs
    pub vcs_added: Color32,
    pub vcs_modified: Color32,
    pub vcs_deleted: Color32,
    pub vcs_renamed: Color32,
    pub vcs_untracked: Color32,
    pub vcs_conflict: Color32,
    pub vcs_ignored: Color32,

    // elevation
    pub shadow_sm: Shadow,
    pub shadow_md: Shadow,
    pub shadow_lg: Shadow,

    // theme flag — handy for branches that depend on light vs dark
    pub is_dark: bool,
}

/// Spacing tokens (4 px grid).
#[allow(dead_code)]
pub mod space {
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 8.0;
    pub const S3: f32 = 12.0;
    pub const S4: f32 = 16.0;
    pub const S5: f32 = 20.0;
    pub const S6: f32 = 24.0;
    pub const S8: f32 = 32.0;
    pub const S10: f32 = 40.0;
    pub const S12: f32 = 48.0;
    pub const S16: f32 = 64.0;
}

/// Radii tokens.
#[allow(dead_code)]
pub mod radii {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 10.0;
    pub const XL: f32 = 14.0;
}

/// Type-scale tokens in px.
#[allow(dead_code)]
pub mod ts {
    pub const CAPTION: f32 = 11.0;
    pub const LABEL_SM: f32 = 12.0;
    pub const LABEL: f32 = 13.0;
    pub const BODY: f32 = 14.0;
    pub const BODY_LG: f32 = 16.0;
    pub const HEADING_SM: f32 = 18.0;
    pub const HEADING_MD: f32 = 22.0;
    pub const HEADING_LG: f32 = 28.0;
    pub const MONO_CODE: f32 = 13.5;
    pub const MONO_UI: f32 = 12.0;
    pub const KEYCAP: f32 = 11.0;
}

// ───────────────────────────────────────────────────────────────────────
// Token constructors
// ───────────────────────────────────────────────────────────────────────

static DARK: Lazy<Tokens> = Lazy::new(dark_tokens);
static LIGHT: Lazy<Tokens> = Lazy::new(light_tokens);

pub fn tokens(theme: ColorTheme) -> &'static Tokens {
    match theme {
        ColorTheme::Dark => &DARK,
        ColorTheme::Light => &LIGHT,
    }
}

fn dark_tokens() -> Tokens {
    Tokens {
        bg_canvas: Color32::from_rgb(0x0F, 0x11, 0x16),
        bg_chrome: Color32::from_rgb(0x13, 0x16, 0x1D),
        bg_surface: Color32::from_rgb(0x19, 0x1D, 0x26),
        bg_elevated: Color32::from_rgb(0x1F, 0x24, 0x30),
        bg_modal: Color32::from_rgb(0x22, 0x28, 0x3A),
        bg_subtle: Color32::from_rgba_premultiplied(8, 8, 8, 8), // ~3%α white tint approx
        bg_hover: Color32::from_rgba_premultiplied(10, 10, 10, 10), // ~4%α
        bg_active: Color32::from_rgba_premultiplied(20, 20, 20, 20), // ~8%α

        text_primary: Color32::from_rgb(0xE6, 0xE8, 0xEC),
        text_secondary: Color32::from_rgb(0xA8, 0xAD, 0xB8),
        text_muted: Color32::from_rgb(0x6E, 0x73, 0x7D),
        text_disabled: Color32::from_rgb(0x4A, 0x4E, 0x58),
        text_on_accent: Color32::from_rgb(0x04, 0x10, 0x1F),

        border_subtle: Color32::from_rgba_premultiplied(15, 15, 15, 15),
        border_default: Color32::from_rgba_premultiplied(25, 25, 25, 25),
        border_strong: Color32::from_rgba_premultiplied(40, 40, 40, 40),

        accent: Color32::from_rgb(0x7A, 0xA2, 0xF7),
        accent_hover: Color32::from_rgb(0x8F, 0xB3, 0xF9),
        accent_bg: Color32::from_rgba_premultiplied(15, 19, 30, 30),
        accent_border: Color32::from_rgba_premultiplied(36, 49, 75, 75),

        error: Color32::from_rgb(0xE5, 0x48, 0x4D),
        error_bg: Color32::from_rgba_premultiplied(23, 7, 8, 26),
        error_border: Color32::from_rgba_premultiplied(57, 18, 19, 64),
        warning: Color32::from_rgb(0xF5, 0xA5, 0x24),
        warning_bg: Color32::from_rgba_premultiplied(24, 17, 4, 26),
        warning_border: Color32::from_rgba_premultiplied(61, 41, 9, 64),
        success: Color32::from_rgb(0x46, 0xA7, 0x58),
        success_bg: Color32::from_rgba_premultiplied(7, 17, 9, 26),
        success_border: Color32::from_rgba_premultiplied(17, 42, 22, 64),
        info: Color32::from_rgb(0x5E, 0xB1, 0xF0),
        info_bg: Color32::from_rgba_premultiplied(9, 18, 24, 26),
        info_border: Color32::from_rgba_premultiplied(23, 44, 60, 64),

        vcs_added: Color32::from_rgb(0x27, 0xA6, 0x57),
        vcs_modified: Color32::from_rgb(0xD3, 0xB0, 0x20),
        vcs_deleted: Color32::from_rgb(0xE0, 0x6C, 0x76),
        vcs_renamed: Color32::from_rgb(0xB0, 0x83, 0xDA),
        vcs_untracked: Color32::from_rgb(0x5E, 0xB1, 0xF0),
        vcs_conflict: Color32::from_rgb(0xFF, 0x6B, 0x6B),
        vcs_ignored: Color32::from_rgb(0x78, 0x78, 0x78),

        shadow_sm: Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 2.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        },
        shadow_md: Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(90),
        },
        shadow_lg: Shadow {
            offset: egui::vec2(0.0, 16.0),
            blur: 40.0,
            spread: 0.0,
            color: Color32::from_black_alpha(115),
        },

        is_dark: true,
    }
}

fn light_tokens() -> Tokens {
    Tokens {
        bg_canvas: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        bg_chrome: Color32::from_rgb(0xFA, 0xFB, 0xFC),
        bg_surface: Color32::from_rgb(0xF4, 0xF5, 0xF8),
        bg_elevated: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        bg_modal: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        bg_subtle: Color32::from_rgba_premultiplied(0, 0, 0, 6),
        bg_hover: Color32::from_rgba_premultiplied(0, 0, 0, 10),
        bg_active: Color32::from_rgba_premultiplied(0, 0, 0, 18),

        text_primary: Color32::from_rgb(0x0E, 0x11, 0x17),
        text_secondary: Color32::from_rgb(0x47, 0x55, 0x69),
        text_muted: Color32::from_rgb(0x7A, 0x84, 0x93),
        text_disabled: Color32::from_rgb(0xB7, 0xBF, 0xC9),
        text_on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),

        border_subtle: Color32::from_rgba_premultiplied(0, 0, 0, 16),
        border_default: Color32::from_rgba_premultiplied(0, 0, 0, 26),
        border_strong: Color32::from_rgba_premultiplied(0, 0, 0, 42),

        accent: Color32::from_rgb(0x3B, 0x82, 0xF6),
        accent_hover: Color32::from_rgb(0x25, 0x63, 0xEB),
        accent_bg: Color32::from_rgba_premultiplied(6, 13, 26, 26),
        accent_border: Color32::from_rgba_premultiplied(18, 39, 74, 77),

        error: Color32::from_rgb(0xDC, 0x26, 0x26),
        error_bg: Color32::from_rgba_premultiplied(22, 4, 4, 26),
        error_border: Color32::from_rgba_premultiplied(55, 10, 10, 64),
        warning: Color32::from_rgb(0xD9, 0x77, 0x06),
        warning_bg: Color32::from_rgba_premultiplied(22, 12, 1, 26),
        warning_border: Color32::from_rgba_premultiplied(54, 30, 2, 64),
        success: Color32::from_rgb(0x05, 0x96, 0x69),
        success_bg: Color32::from_rgba_premultiplied(0, 15, 11, 26),
        success_border: Color32::from_rgba_premultiplied(1, 38, 27, 64),
        info: Color32::from_rgb(0x09, 0x69, 0xDA),
        info_bg: Color32::from_rgba_premultiplied(1, 11, 22, 26),
        info_border: Color32::from_rgba_premultiplied(2, 26, 55, 64),

        vcs_added: Color32::from_rgb(0x16, 0x80, 0x3D),
        vcs_modified: Color32::from_rgb(0xB1, 0x83, 0x00),
        vcs_deleted: Color32::from_rgb(0xC0, 0x36, 0x3F),
        vcs_renamed: Color32::from_rgb(0x80, 0x52, 0xCC),
        vcs_untracked: Color32::from_rgb(0x09, 0x69, 0xDA),
        vcs_conflict: Color32::from_rgb(0xDC, 0x26, 0x26),
        vcs_ignored: Color32::from_rgb(0x98, 0x98, 0x98),

        shadow_sm: Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 2.0,
            spread: 0.0,
            color: Color32::from_black_alpha(15),
        },
        shadow_md: Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 14.0,
            spread: 0.0,
            color: Color32::from_black_alpha(25),
        },
        shadow_lg: Shadow {
            offset: egui::vec2(0.0, 18.0),
            blur: 50.0,
            spread: 0.0,
            color: Color32::from_black_alpha(45),
        },

        is_dark: false,
    }
}

// ───────────────────────────────────────────────────────────────────────
// egui apply
// ───────────────────────────────────────────────────────────────────────

/// Configure egui's `Visuals` + `Style::spacing` from the active tokens.
/// Call once per frame after the theme is determined.
pub fn apply(ctx: &Context, theme: ColorTheme) {
    let t = tokens(theme);
    let mut v = if t.is_dark { Visuals::dark() } else { Visuals::light() };

    v.dark_mode = t.is_dark;
    v.panel_fill = t.bg_chrome;
    v.window_fill = t.bg_modal;
    v.window_stroke = Stroke::new(1.0, t.border_subtle);
    v.window_rounding = Rounding::same(radii::LG);
    v.window_shadow = t.shadow_lg;
    v.popup_shadow = t.shadow_md;
    v.menu_rounding = Rounding::same(radii::MD);
    v.extreme_bg_color = if t.is_dark {
        Color32::from_rgb(0x0A, 0x0C, 0x12)
    } else {
        Color32::from_rgb(0xF4, 0xF6, 0xFA)
    };
    v.faint_bg_color = t.bg_surface;
    v.code_bg_color = if t.is_dark {
        Color32::from_rgb(0x0E, 0x11, 0x18)
    } else {
        Color32::from_rgb(0xF4, 0xF6, 0xFA)
    };

    v.override_text_color = Some(t.text_primary);
    v.hyperlink_color = t.accent;

    // 4-state widget lifecycle.
    v.widgets.noninteractive.bg_fill = t.bg_elevated;
    v.widgets.noninteractive.weak_bg_fill = t.bg_elevated;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border_subtle);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_secondary);
    v.widgets.noninteractive.rounding = Rounding::same(radii::SM);

    v.widgets.inactive.bg_fill = t.bg_elevated;
    v.widgets.inactive.weak_bg_fill = t.bg_subtle;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border_subtle);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text_primary);
    v.widgets.inactive.rounding = Rounding::same(radii::SM);

    v.widgets.hovered.bg_fill = t.bg_hover;
    v.widgets.hovered.weak_bg_fill = t.bg_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.border_default);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text_primary);
    v.widgets.hovered.rounding = Rounding::same(radii::SM);

    v.widgets.active.bg_fill = t.bg_active;
    v.widgets.active.weak_bg_fill = t.bg_active;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.border_strong);
    v.widgets.active.fg_stroke = Stroke::new(1.5, t.text_primary);
    v.widgets.active.rounding = Rounding::same(radii::SM);

    v.widgets.open.bg_fill = t.bg_active;
    v.widgets.open.bg_stroke = Stroke::new(1.0, t.border_strong);
    v.widgets.open.fg_stroke = Stroke::new(1.0, t.text_primary);
    v.widgets.open.rounding = Rounding::same(radii::SM);

    v.selection.bg_fill = t.accent_bg;
    v.selection.stroke = Stroke::new(1.0, t.accent);

    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(space::S2, space::S1);
        s.spacing.button_padding = egui::vec2(space::S3, space::S1 + 2.0);
        s.spacing.menu_margin = egui::Margin::symmetric(space::S2, space::S1);
        s.spacing.window_margin = egui::Margin::ZERO;
        s.spacing.indent = 18.0;
        s.spacing.scroll.bar_width = 10.0;
        s.spacing.scroll.handle_min_length = 32.0;
    });
}
