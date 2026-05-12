//! Design tokens and egui visuals for IdeUltra.
//!
//! IdeUltra's house style is "**luxury light**" — white and light grey
//! surfaces, deep-ink accent, almost-invisible chrome, generous
//! whitespace, no neon. Reference points: Linear light, Vercel Geist
//! light, Stripe docs, iA Writer. The visual answer to "what does a
//! 2026 code editor look like when it isn't trying to look like a
//! 2010 code editor?"
//!
//! Light is the default. Dark is the refined "night mode" complement —
//! charcoal/zinc rather than indigo, equally restrained.
//!
//! Two public entry points:
//!
//!   1. [`tokens(theme)`] — semantic tokens (`&'static Tokens`).
//!   2. [`apply(ctx, theme)`] — wire those tokens into egui's
//!      `Visuals` + `Style::spacing`.

use egui::epaint::Shadow;
use egui::{Color32, Context, Rounding, Stroke, Visuals};
use once_cell::sync::Lazy;

use crate::editor::language::ColorTheme;

// ───────────────────────────────────────────────────────────────────────
// Token type
// ───────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct Tokens {
    // surface ramp
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

    // accent — deep-ink charcoal in light, soft ivory in dark
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_bg: Color32,
    pub accent_border: Color32,

    // semantic status (deep / muted, not neon)
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

    pub is_dark: bool,
}

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

#[allow(dead_code)]
pub mod radii {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 10.0;
    pub const XL: f32 = 14.0;
}

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

static LIGHT: Lazy<Tokens> = Lazy::new(light_tokens);
static DARK: Lazy<Tokens> = Lazy::new(dark_tokens);

pub fn tokens(theme: ColorTheme) -> &'static Tokens {
    match theme {
        ColorTheme::Light => &LIGHT,
        ColorTheme::Dark => &DARK,
    }
}

/// LUXURY LIGHT — the hero theme.
///
/// White editor canvas, near-white panels, ink-charcoal text, deep-ink
/// accent (looks black until you put it next to true black). Almost
/// invisible borders, near-zero shadows. The code carries the color.
fn light_tokens() -> Tokens {
    Tokens {
        // surface ramp — purposely tiny lightness deltas
        bg_canvas: Color32::WHITE,                       // editor
        bg_chrome: Color32::from_rgb(0xFB, 0xFB, 0xFC),  // sidebar, status bar
        bg_surface: Color32::from_rgb(0xF5, 0xF5, 0xF7), // find bar, tabs strip
        bg_elevated: Color32::WHITE,                     // popovers
        bg_modal: Color32::WHITE,                        // modals
        bg_subtle: Color32::from_rgba_premultiplied(0, 0, 0, 5),
        bg_hover: Color32::from_rgba_premultiplied(0, 0, 0, 8),
        bg_active: Color32::from_rgba_premultiplied(0, 0, 0, 14),

        // foreground — deep ink, never pure black (saves the eyes on white)
        text_primary: Color32::from_rgb(0x0A, 0x0A, 0x0A),
        text_secondary: Color32::from_rgb(0x44, 0x44, 0x48),
        text_muted: Color32::from_rgb(0x88, 0x88, 0x92),
        text_disabled: Color32::from_rgb(0xC0, 0xC0, 0xC6),
        text_on_accent: Color32::WHITE,

        // borders — barely there
        border_subtle: Color32::from_rgba_premultiplied(0, 0, 0, 10),
        border_default: Color32::from_rgba_premultiplied(0, 0, 0, 18),
        border_strong: Color32::from_rgba_premultiplied(0, 0, 0, 32),

        // accent — deep ink. Reads as "really nice black" against #FFF.
        accent: Color32::from_rgb(0x18, 0x18, 0x1B),
        accent_hover: Color32::from_rgb(0x27, 0x27, 0x2A),
        accent_bg: Color32::from_rgba_premultiplied(0, 0, 0, 8),
        accent_border: Color32::from_rgba_premultiplied(0, 0, 0, 28),

        // semantic status — deep, never saturated
        error: Color32::from_rgb(0xB9, 0x1C, 0x1C),
        error_bg: Color32::from_rgba_premultiplied(15, 2, 2, 18),
        error_border: Color32::from_rgba_premultiplied(50, 8, 8, 50),
        warning: Color32::from_rgb(0xB4, 0x53, 0x09),
        warning_bg: Color32::from_rgba_premultiplied(15, 7, 1, 18),
        warning_border: Color32::from_rgba_premultiplied(50, 23, 4, 50),
        success: Color32::from_rgb(0x15, 0x80, 0x3D),
        success_bg: Color32::from_rgba_premultiplied(2, 11, 5, 18),
        success_border: Color32::from_rgba_premultiplied(6, 34, 16, 50),
        info: Color32::from_rgb(0x1D, 0x4E, 0xD8),
        info_bg: Color32::from_rgba_premultiplied(3, 7, 19, 18),
        info_border: Color32::from_rgba_premultiplied(8, 21, 55, 50),

        // vcs — slightly more vivid than status (gutters need to read)
        vcs_added: Color32::from_rgb(0x15, 0x80, 0x3D),
        vcs_modified: Color32::from_rgb(0xA1, 0x6A, 0x05),
        vcs_deleted: Color32::from_rgb(0xB9, 0x1C, 0x1C),
        vcs_renamed: Color32::from_rgb(0x68, 0x3F, 0xB6),
        vcs_untracked: Color32::from_rgb(0x1D, 0x4E, 0xD8),
        vcs_conflict: Color32::from_rgb(0xB9, 0x1C, 0x1C),
        vcs_ignored: Color32::from_rgb(0x9A, 0x9A, 0xA0),

        // elevation — soft, almost imperceptible
        shadow_sm: Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 1.0,
            spread: 0.0,
            color: Color32::from_black_alpha(8),
        },
        shadow_md: Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(14),
        },
        shadow_lg: Shadow {
            offset: egui::vec2(0.0, 14.0),
            blur: 40.0,
            spread: 0.0,
            color: Color32::from_black_alpha(22),
        },

        is_dark: false,
    }
}

/// Dark — refined charcoal/zinc complement. Designed to feel like the
/// same product at night, not a different app.
fn dark_tokens() -> Tokens {
    Tokens {
        bg_canvas: Color32::from_rgb(0x0B, 0x0B, 0x0D),
        bg_chrome: Color32::from_rgb(0x10, 0x10, 0x12),
        bg_surface: Color32::from_rgb(0x16, 0x16, 0x19),
        bg_elevated: Color32::from_rgb(0x1B, 0x1B, 0x1E),
        bg_modal: Color32::from_rgb(0x1E, 0x1E, 0x22),
        bg_subtle: Color32::from_rgba_premultiplied(8, 8, 8, 8),
        bg_hover: Color32::from_rgba_premultiplied(12, 12, 12, 12),
        bg_active: Color32::from_rgba_premultiplied(22, 22, 22, 22),

        text_primary: Color32::from_rgb(0xF0, 0xF0, 0xF2),
        text_secondary: Color32::from_rgb(0xAE, 0xAE, 0xB2),
        text_muted: Color32::from_rgb(0x72, 0x72, 0x76),
        text_disabled: Color32::from_rgb(0x48, 0x48, 0x4C),
        text_on_accent: Color32::from_rgb(0x0B, 0x0B, 0x0D),

        border_subtle: Color32::from_rgba_premultiplied(18, 18, 18, 18),
        border_default: Color32::from_rgba_premultiplied(32, 32, 32, 32),
        border_strong: Color32::from_rgba_premultiplied(58, 58, 58, 58),

        // ivory accent — looks white against zinc backgrounds
        accent: Color32::from_rgb(0xF4, 0xF4, 0xF6),
        accent_hover: Color32::WHITE,
        accent_bg: Color32::from_rgba_premultiplied(28, 28, 28, 28),
        accent_border: Color32::from_rgba_premultiplied(80, 80, 80, 80),

        error: Color32::from_rgb(0xF2, 0x67, 0x6C),
        error_bg: Color32::from_rgba_premultiplied(28, 11, 12, 30),
        error_border: Color32::from_rgba_premultiplied(80, 32, 34, 80),
        warning: Color32::from_rgb(0xF4, 0xB5, 0x4A),
        warning_bg: Color32::from_rgba_premultiplied(28, 19, 7, 30),
        warning_border: Color32::from_rgba_premultiplied(80, 56, 22, 80),
        success: Color32::from_rgb(0x6E, 0xCC, 0x84),
        success_bg: Color32::from_rgba_premultiplied(10, 22, 13, 30),
        success_border: Color32::from_rgba_premultiplied(30, 66, 38, 80),
        info: Color32::from_rgb(0x6E, 0xA8, 0xF7),
        info_bg: Color32::from_rgba_premultiplied(10, 17, 28, 30),
        info_border: Color32::from_rgba_premultiplied(30, 52, 82, 80),

        vcs_added: Color32::from_rgb(0x6E, 0xCC, 0x84),
        vcs_modified: Color32::from_rgb(0xE3, 0xC0, 0x60),
        vcs_deleted: Color32::from_rgb(0xF2, 0x67, 0x6C),
        vcs_renamed: Color32::from_rgb(0xC1, 0x9F, 0xF0),
        vcs_untracked: Color32::from_rgb(0x6E, 0xA8, 0xF7),
        vcs_conflict: Color32::from_rgb(0xFF, 0x6F, 0x6F),
        vcs_ignored: Color32::from_rgb(0x66, 0x66, 0x6A),

        shadow_sm: Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 2.0,
            spread: 0.0,
            color: Color32::from_black_alpha(80),
        },
        shadow_md: Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 14.0,
            spread: 0.0,
            color: Color32::from_black_alpha(75),
        },
        shadow_lg: Shadow {
            offset: egui::vec2(0.0, 18.0),
            blur: 48.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        },

        is_dark: true,
    }
}

// ───────────────────────────────────────────────────────────────────────
// egui apply
// ───────────────────────────────────────────────────────────────────────

pub fn apply(ctx: &Context, theme: ColorTheme) {
    let t = tokens(theme);
    let mut v = if t.is_dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    v.dark_mode = t.is_dark;
    v.panel_fill = t.bg_chrome;
    v.window_fill = t.bg_modal;
    v.window_stroke = Stroke::new(1.0, t.border_subtle);
    v.window_rounding = Rounding::same(radii::LG);
    v.window_shadow = t.shadow_lg;
    v.popup_shadow = t.shadow_md;
    v.menu_rounding = Rounding::same(radii::MD);
    v.extreme_bg_color = t.bg_surface;
    v.faint_bg_color = t.bg_surface;
    v.code_bg_color = t.bg_surface;

    v.override_text_color = Some(t.text_primary);
    v.hyperlink_color = t.accent;

    // 4-state widget lifecycle — accent stays close to text color so
    // the chrome feels monochrome.
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
        // Slightly more generous than before — luxury feels = breathing room.
        s.spacing.item_spacing = egui::vec2(space::S2, space::S1 + 2.0);
        s.spacing.button_padding = egui::vec2(space::S3, space::S2);
        s.spacing.menu_margin = egui::Margin::symmetric(space::S2, space::S1);
        s.spacing.window_margin = egui::Margin::ZERO;
        s.spacing.indent = 20.0;
        s.spacing.scroll.bar_width = 10.0;
        s.spacing.scroll.handle_min_length = 36.0;
    });
}
