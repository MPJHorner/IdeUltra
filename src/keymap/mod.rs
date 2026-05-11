//! Keymap presets — pick between an IdeUltra default, a VS Code-inspired
//! layout, or a PhpStorm-inspired layout. The mapping is `CommandId →
//! KeyboardShortcut`. `app::handle_shortcuts` iterates the keymap once
//! per frame and dispatches via `app::dispatch_command`.
//!
//! Modifying or extending the presets is a single function edit; nothing
//! else in the app needs to know which preset is active.

use egui::{Key, KeyboardShortcut, Modifiers};
use serde::{Deserialize, Serialize};

use crate::command_palette::CommandId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeymapPreset {
    Default,
    VsCode,
    PhpStorm,
}

impl KeymapPreset {
    pub fn label(self) -> &'static str {
        match self {
            KeymapPreset::Default => "IdeUltra Default",
            KeymapPreset::VsCode => "VS Code",
            KeymapPreset::PhpStorm => "PhpStorm / JetBrains",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            KeymapPreset::Default => "Mac-native defaults. Cmd+P for files, Cmd+Shift+P for the command palette, Cmd+F for find.",
            KeymapPreset::VsCode => "VS Code on macOS. Adds Ctrl+R for Open Recent, Ctrl+G for Go to Line, Cmd+H for Replace.",
            KeymapPreset::PhpStorm => "JetBrains macOS keymap. Cmd+E for Recent Files, Cmd+Shift+O for Go to File, Cmd+Shift+A for the action search, Cmd+R for Replace, Cmd+L for Go to Line.",
        }
    }

    pub fn all() -> &'static [KeymapPreset] {
        &[KeymapPreset::Default, KeymapPreset::VsCode, KeymapPreset::PhpStorm]
    }
}

#[derive(Debug, Clone)]
pub struct Keymap {
    pub preset: KeymapPreset,
    bindings: Vec<(CommandId, KeyboardShortcut)>,
}

impl Keymap {
    pub fn for_preset(preset: KeymapPreset) -> Self {
        Self {
            preset,
            bindings: build_bindings(preset),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(CommandId, KeyboardShortcut)> {
        self.bindings.iter()
    }

    /// Pretty-printed shortcut for the given command, or `""` if unbound.
    /// Returned strings are used in menus, command palette rows, and
    /// status hints.
    pub fn label_for(&self, id: CommandId) -> String {
        match self.bindings.iter().find(|(c, _)| *c == id) {
            Some((_, sc)) => format_shortcut(*sc),
            None => String::new(),
        }
    }

    /// True if the keymap binds the command at all.
    #[allow(dead_code)] // surface for future menu rendering
    pub fn is_bound(&self, id: CommandId) -> bool {
        self.bindings.iter().any(|(c, _)| *c == id)
    }
}

fn build_bindings(preset: KeymapPreset) -> Vec<(CommandId, KeyboardShortcut)> {
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    let cmd_alt = Modifiers::COMMAND | Modifiers::ALT;
    let ctrl = Modifiers::CTRL;
    let alt = Modifiers::ALT;

    // Universals that don't vary across presets.
    let mut b = vec![
        (CommandId::OpenFile, KeyboardShortcut::new(cmd, Key::O)),
        (CommandId::OpenFolder, KeyboardShortcut::new(cmd_shift, Key::O)),
        (CommandId::Save, KeyboardShortcut::new(cmd, Key::S)),
        (CommandId::CloseTab, KeyboardShortcut::new(cmd, Key::W)),
        (CommandId::Find, KeyboardShortcut::new(cmd, Key::F)),
        (CommandId::SearchInProject, KeyboardShortcut::new(cmd_shift, Key::F)),
        (CommandId::ToggleSidebar, KeyboardShortcut::new(cmd, Key::B)),
        (CommandId::ZoomIn, KeyboardShortcut::new(cmd, Key::Equals)),
        (CommandId::ZoomOut, KeyboardShortcut::new(cmd, Key::Minus)),
        (CommandId::ZoomReset, KeyboardShortcut::new(cmd, Key::Num0)),
        (CommandId::ToggleMarkdownPreview, KeyboardShortcut::new(cmd_alt, Key::M)),
        (CommandId::ToggleLineComment, KeyboardShortcut::new(cmd, Key::Slash)),
        (CommandId::OpenPreferences, KeyboardShortcut::new(cmd, Key::Comma)),
    ];

    match preset {
        KeymapPreset::Default => {
            b.extend([
                (CommandId::FindReplace, KeyboardShortcut::new(cmd_alt, Key::F)),
                (CommandId::GoToFile, KeyboardShortcut::new(cmd, Key::P)),
                (
                    CommandId::SearchInProject,
                    KeyboardShortcut::new(cmd_shift, Key::F),
                ),
                (CommandId::GoToLine, KeyboardShortcut::new(cmd, Key::G)),
                (
                    CommandId::OpenFolder,
                    KeyboardShortcut::new(cmd_shift, Key::O),
                ),
                // Palette has the convention-shifted shortcut.
                (
                    CommandId::OpenFile,
                    KeyboardShortcut::new(cmd, Key::O),
                ),
                (
                    CommandId::ToggleMarkdownPreview,
                    KeyboardShortcut::new(cmd_alt, Key::M),
                ),
                (
                    CommandId::Save,
                    KeyboardShortcut::new(cmd, Key::S),
                ),
                (
                    CommandId::CloseTab,
                    KeyboardShortcut::new(cmd, Key::W),
                ),
                (
                    CommandId::Find,
                    KeyboardShortcut::new(cmd, Key::F),
                ),
                // Command palette as ⇧⌘P (default + VS Code).
                (
                    CommandId::OpenFile,
                    KeyboardShortcut::new(cmd, Key::O),
                ),
            ]);
            // Distinct extras.
            push_unique(&mut b, CommandId::FindReplace, KeyboardShortcut::new(cmd_alt, Key::F));
            // Palette command id has no enum variant here; it's triggered
            // by the keymap-independent ⇧⌘P binding in app.rs.
        }
        KeymapPreset::VsCode => {
            b.extend([
                (CommandId::FindReplace, KeyboardShortcut::new(cmd_alt, Key::F)),
                (CommandId::GoToFile, KeyboardShortcut::new(cmd, Key::P)),
                (CommandId::GoToLine, KeyboardShortcut::new(ctrl, Key::G)),
            ]);
        }
        KeymapPreset::PhpStorm => {
            // JetBrains macOS keymap conventions.
            b.extend([
                // Replace: ⌘R
                (CommandId::FindReplace, KeyboardShortcut::new(cmd, Key::R)),
                // Go to File: ⇧⌘O
                (CommandId::GoToFile, KeyboardShortcut::new(cmd_shift, Key::O)),
                // Go to Line: ⌘L
                (CommandId::GoToLine, KeyboardShortcut::new(cmd, Key::L)),
                // Toggle Project (sidebar): ⌥1 — keeping ⌘B as well for muscle memory.
                (CommandId::ToggleSidebar, KeyboardShortcut::new(alt, Key::Num1)),
            ]);
            // PhpStorm doesn't put OpenFolder on ⇧⌘O (because that's Go to File),
            // so move OpenFolder onto ⌘O and OpenFile onto ⌥⌘O.
            replace_binding(&mut b, CommandId::OpenFile, KeyboardShortcut::new(cmd_alt, Key::O));
            replace_binding(&mut b, CommandId::OpenFolder, KeyboardShortcut::new(cmd, Key::O));
        }
    }

    // Dedupe in favour of the *last* binding so preset overrides win.
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(b.len());
    for (id, sc) in b.into_iter().rev() {
        if seen.insert(id) {
            out.push((id, sc));
        }
    }
    out.reverse();
    out
}

fn push_unique(
    b: &mut Vec<(CommandId, KeyboardShortcut)>,
    id: CommandId,
    sc: KeyboardShortcut,
) {
    if !b.iter().any(|(c, _)| *c == id) {
        b.push((id, sc));
    }
}

fn replace_binding(
    b: &mut Vec<(CommandId, KeyboardShortcut)>,
    id: CommandId,
    sc: KeyboardShortcut,
) {
    b.retain(|(c, _)| *c != id);
    b.push((id, sc));
}

/// Pretty-print a shortcut like `⌘P` or `⌥⌘F`.
pub fn format_shortcut(sc: KeyboardShortcut) -> String {
    let mut out = String::with_capacity(8);
    if sc.modifiers.ctrl {
        out.push('⌃');
    }
    if sc.modifiers.alt {
        out.push('⌥');
    }
    if sc.modifiers.shift {
        out.push('⇧');
    }
    if sc.modifiers.command || sc.modifiers.mac_cmd {
        out.push('⌘');
    }
    out.push_str(&key_name(sc.logical_key));
    out
}

fn key_name(k: Key) -> String {
    use Key::*;
    match k {
        A => "A".into(), B => "B".into(), C => "C".into(), D => "D".into(),
        E => "E".into(), F => "F".into(), G => "G".into(), H => "H".into(),
        I => "I".into(), J => "J".into(), K => "K".into(), L => "L".into(),
        M => "M".into(), N => "N".into(), O => "O".into(), P => "P".into(),
        Q => "Q".into(), R => "R".into(), S => "S".into(), T => "T".into(),
        U => "U".into(), V => "V".into(), W => "W".into(), X => "X".into(),
        Y => "Y".into(), Z => "Z".into(),
        Num0 => "0".into(), Num1 => "1".into(), Num2 => "2".into(),
        Num3 => "3".into(), Num4 => "4".into(), Num5 => "5".into(),
        Num6 => "6".into(), Num7 => "7".into(), Num8 => "8".into(), Num9 => "9".into(),
        Equals => "=".into(),
        Minus => "−".into(),
        Plus => "+".into(),
        OpenBracket => "[".into(),
        CloseBracket => "]".into(),
        Enter => "⏎".into(),
        Escape => "esc".into(),
        Space => "Space".into(),
        Tab => "Tab".into(),
        Backspace => "⌫".into(),
        Delete => "⌦".into(),
        ArrowDown => "↓".into(),
        ArrowUp => "↑".into(),
        ArrowLeft => "←".into(),
        ArrowRight => "→".into(),
        F1 => "F1".into(), F2 => "F2".into(), F3 => "F3".into(), F4 => "F4".into(),
        F5 => "F5".into(), F6 => "F6".into(), F7 => "F7".into(), F8 => "F8".into(),
        F9 => "F9".into(), F10 => "F10".into(), F11 => "F11".into(), F12 => "F12".into(),
        _ => format!("{k:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn shortcuts(p: KeymapPreset) -> std::collections::HashMap<CommandId, KeyboardShortcut> {
        Keymap::for_preset(p).bindings.into_iter().collect()
    }

    #[test]
    fn every_preset_binds_the_universal_commands() {
        for preset in KeymapPreset::all() {
            let m = shortcuts(*preset);
            for id in [
                CommandId::Save,
                CommandId::CloseTab,
                CommandId::Find,
                CommandId::GoToFile,
                CommandId::GoToLine,
                CommandId::FindReplace,
                CommandId::SearchInProject,
            ] {
                assert!(
                    m.contains_key(&id),
                    "{:?} missing binding for {:?}",
                    preset,
                    id
                );
            }
        }
    }

    #[test]
    fn presets_differ_where_expected() {
        let d = shortcuts(KeymapPreset::Default);
        let v = shortcuts(KeymapPreset::VsCode);
        let p = shortcuts(KeymapPreset::PhpStorm);
        // Replace differs.
        assert_ne!(d[&CommandId::FindReplace], p[&CommandId::FindReplace]);
        // Goto-line differs (Default ⌘G, VSCode ⌃G, PhpStorm ⌘L).
        assert_ne!(d[&CommandId::GoToLine], v[&CommandId::GoToLine]);
        assert_ne!(d[&CommandId::GoToLine], p[&CommandId::GoToLine]);
        // Goto-file differs (PhpStorm uses ⇧⌘O).
        assert_ne!(d[&CommandId::GoToFile], p[&CommandId::GoToFile]);
    }

    #[test]
    fn no_duplicate_command_in_a_preset() {
        for preset in KeymapPreset::all() {
            let km = Keymap::for_preset(*preset);
            let ids: HashSet<_> = km.bindings.iter().map(|(c, _)| *c).collect();
            assert_eq!(ids.len(), km.bindings.len(), "duplicate in {:?}", preset);
        }
    }

    #[test]
    fn no_duplicate_shortcut_within_a_preset() {
        for preset in KeymapPreset::all() {
            let km = Keymap::for_preset(*preset);
            let mut seen = HashSet::new();
            for (_, sc) in km.iter() {
                let key = (
                    sc.modifiers.ctrl,
                    sc.modifiers.alt,
                    sc.modifiers.shift,
                    sc.modifiers.command || sc.modifiers.mac_cmd,
                    sc.logical_key,
                );
                assert!(
                    seen.insert(key),
                    "duplicate shortcut in {:?}: {:?}",
                    preset,
                    sc
                );
            }
        }
    }

    #[test]
    fn format_shortcut_uses_glyphs() {
        let sc = KeyboardShortcut::new(Modifiers::COMMAND, Key::P);
        assert_eq!(format_shortcut(sc), "⌘P");
        let sc = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
        assert_eq!(format_shortcut(sc), "⇧⌘O");
        let sc = KeyboardShortcut::new(Modifiers::CTRL, Key::G);
        assert_eq!(format_shortcut(sc), "⌃G");
    }

    #[test]
    fn label_for_returns_empty_for_unbound_command() {
        let km = Keymap::for_preset(KeymapPreset::Default);
        // Theme commands are intentionally unbound by default.
        assert_eq!(km.label_for(CommandId::ThemeDark), "");
    }
}
