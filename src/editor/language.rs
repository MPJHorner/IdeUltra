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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rust_extension_resolves_to_rust_syntax() {
        let s = syntax_for_path(&PathBuf::from("src/main.rs"));
        assert_eq!(language_label(s), "Rust");
    }

    #[test]
    fn unknown_extension_falls_back_to_plain_text() {
        let s = syntax_for_path(&PathBuf::from("notes.idontexist"));
        // syntect calls plain text "Plain Text".
        assert_eq!(s.name, "Plain Text");
    }

    #[test]
    fn no_extension_falls_back_to_plain_text() {
        let s = syntax_for_path(&PathBuf::from("LICENSE"));
        // No extension and no token match → plain text.
        // (syntect doesn't ship a LICENSE syntax.)
        assert_eq!(s.name, "Plain Text");
    }

    #[test]
    fn theme_labels_are_human_readable() {
        assert_eq!(ColorTheme::Dark.label(), "Dark");
        assert_eq!(ColorTheme::Light.label(), "Light");
    }

    #[test]
    fn theme_set_actually_contains_named_themes() {
        // If syntect renames or drops these, we want to know at test time
        // not at first user-visible paint.
        assert!(THEME_SET
            .themes
            .contains_key(ColorTheme::Dark.syntect_name()));
        assert!(THEME_SET
            .themes
            .contains_key(ColorTheme::Light.syntect_name()));
    }
}
