use std::path::Path;

use once_cell::sync::Lazy;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorTheme {
    Dark,
    Light,
}

impl ColorTheme {
    pub fn syntect_name(self) -> &'static str {
        // Names ship in syntect's default theme set. Verified at runtime.
        match self {
            // base16 ocean dark reads well at small monospace sizes.
            ColorTheme::Dark => "base16-ocean.dark",
            ColorTheme::Light => "InspiredGitHub",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ColorTheme::Dark => "Dark",
            ColorTheme::Light => "Light",
        }
    }
}

/// Pick a syntax for a file path. Falls back to plain text.
pub fn syntax_for_path(path: &Path) -> &'static SyntaxReference {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !ext.is_empty() {
        if let Some(s) = SYNTAX_SET.find_syntax_by_extension(ext) {
            return s;
        }
    }
    // Try by filename (e.g. "Makefile", "Dockerfile").
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if let Some(s) = SYNTAX_SET.find_syntax_by_token(name) {
            return s;
        }
    }
    SYNTAX_SET.find_syntax_plain_text()
}

/// Human-readable language label for the status bar.
pub fn language_label(syntax: &SyntaxReference) -> &str {
    if syntax.name.is_empty() {
        "Plain Text"
    } else {
        &syntax.name
    }
}
