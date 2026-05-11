//! Command palette: a single enum of dispatchable actions, a static
//! registry, and the fuzzy-ranking glue.
//!
//! The point of the enum is that adding a new action is a compile error
//! anywhere that doesn't handle it — we can't ship a palette button
//! that does nothing.

use crate::finder::score;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    OpenFile,
    OpenFolder,
    Save,
    CloseTab,
    Quit,
    Find,
    FindReplace,
    SearchInProject,
    GoToFile,
    GoToLine,
    ToggleSidebar,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ThemeDark,
    ThemeLight,
    ToggleMarkdownPreview,
    SortLines,
    SortLinesReverse,
    UniqueLines,
    UpperCase,
    LowerCase,
    ToggleAutosaveOnFocusLoss,
    ChooseKeymap,
    KeymapDefault,
    KeymapVsCode,
    KeymapPhpStorm,
    ToggleLineComment,
    OpenPreferences,
}

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub id: CommandId,
    pub label: &'static str,
    pub keys: &'static str,
}

/// Single source of truth for everything the palette exposes. Order is
/// the natural fallback when no query is typed.
pub fn all_commands() -> &'static [CommandEntry] {
    &[
        CommandEntry {
            id: CommandId::OpenFile,
            label: "File: Open File…",
            keys: "⌘O",
        },
        CommandEntry {
            id: CommandId::OpenFolder,
            label: "File: Open Folder…",
            keys: "⇧⌘O",
        },
        CommandEntry {
            id: CommandId::Save,
            label: "File: Save",
            keys: "⌘S",
        },
        CommandEntry {
            id: CommandId::CloseTab,
            label: "File: Close Tab",
            keys: "⌘W",
        },
        CommandEntry {
            id: CommandId::Quit,
            label: "File: Quit",
            keys: "⌘Q",
        },
        CommandEntry {
            id: CommandId::Find,
            label: "Edit: Find…",
            keys: "⌘F",
        },
        CommandEntry {
            id: CommandId::FindReplace,
            label: "Edit: Find & Replace…",
            keys: "⌥⌘F",
        },
        CommandEntry {
            id: CommandId::SearchInProject,
            label: "Edit: Search in Project…",
            keys: "⇧⌘F",
        },
        CommandEntry {
            id: CommandId::GoToFile,
            label: "Go: Go to File…",
            keys: "⌘P",
        },
        CommandEntry {
            id: CommandId::GoToLine,
            label: "Go: Go to Line…",
            keys: "⌘G",
        },
        CommandEntry {
            id: CommandId::ToggleSidebar,
            label: "View: Toggle Sidebar",
            keys: "⌘B",
        },
        CommandEntry {
            id: CommandId::ZoomIn,
            label: "View: Zoom In",
            keys: "⌘=",
        },
        CommandEntry {
            id: CommandId::ZoomOut,
            label: "View: Zoom Out",
            keys: "⌘-",
        },
        CommandEntry {
            id: CommandId::ZoomReset,
            label: "View: Reset Zoom",
            keys: "⌘0",
        },
        CommandEntry {
            id: CommandId::ThemeDark,
            label: "View: Theme: Dark",
            keys: "",
        },
        CommandEntry {
            id: CommandId::ThemeLight,
            label: "View: Theme: Light",
            keys: "",
        },
        CommandEntry {
            id: CommandId::ToggleMarkdownPreview,
            label: "View: Toggle Markdown Preview",
            keys: "⌥⌘M",
        },
        CommandEntry {
            id: CommandId::SortLines,
            label: "Edit: Sort Lines",
            keys: "",
        },
        CommandEntry {
            id: CommandId::SortLinesReverse,
            label: "Edit: Sort Lines (Reverse)",
            keys: "",
        },
        CommandEntry {
            id: CommandId::UniqueLines,
            label: "Edit: Unique Lines",
            keys: "",
        },
        CommandEntry {
            id: CommandId::UpperCase,
            label: "Edit: Transform to Uppercase",
            keys: "",
        },
        CommandEntry {
            id: CommandId::LowerCase,
            label: "Edit: Transform to Lowercase",
            keys: "",
        },
        CommandEntry {
            id: CommandId::ToggleAutosaveOnFocusLoss,
            label: "Settings: Toggle Auto-save on Focus Loss",
            keys: "",
        },
        CommandEntry {
            id: CommandId::ChooseKeymap,
            label: "Settings: Choose Keymap…",
            keys: "",
        },
        CommandEntry {
            id: CommandId::KeymapDefault,
            label: "Settings: Keymap: IdeUltra Default",
            keys: "",
        },
        CommandEntry {
            id: CommandId::KeymapVsCode,
            label: "Settings: Keymap: VS Code",
            keys: "",
        },
        CommandEntry {
            id: CommandId::KeymapPhpStorm,
            label: "Settings: Keymap: PhpStorm",
            keys: "",
        },
        CommandEntry {
            id: CommandId::ToggleLineComment,
            label: "Edit: Toggle Line Comment",
            keys: "⌘/",
        },
        CommandEntry {
            id: CommandId::OpenPreferences,
            label: "Settings: Open Preferences…",
            keys: "⌘,",
        },
    ]
}

#[derive(Debug, Clone)]
pub struct RankedCommand {
    pub entry: CommandEntry,
    pub score: i32,
}

/// Rank commands against a query. Empty query returns the natural order.
pub fn rank(query: &str, limit: usize) -> Vec<RankedCommand> {
    if query.is_empty() {
        return all_commands()
            .iter()
            .take(limit)
            .map(|e| RankedCommand {
                entry: e.clone(),
                score: 0,
            })
            .collect();
    }
    let q = query.to_lowercase();
    let mut ranked: Vec<RankedCommand> = all_commands()
        .iter()
        .filter_map(|e| {
            let s = score(&e.label.to_lowercase(), &q)?;
            Some(RankedCommand {
                entry: e.clone(),
                score: s,
            })
        })
        .collect();
    ranked.sort_by(|a, b| b.score.cmp(&a.score));
    ranked.truncate(limit);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_commands_have_unique_ids() {
        let ids: HashSet<_> = all_commands().iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), all_commands().len());
    }

    #[test]
    fn all_commands_have_nonempty_labels() {
        for e in all_commands() {
            assert!(!e.label.is_empty(), "empty label for {:?}", e.id);
        }
    }

    #[test]
    fn empty_query_returns_natural_order_up_to_limit() {
        let r = rank("", 3);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].entry.id, CommandId::OpenFile);
        assert_eq!(r[1].entry.id, CommandId::OpenFolder);
    }

    #[test]
    fn query_matches_prefix_commands() {
        let r = rank("save", 5);
        assert!(r.iter().any(|c| c.entry.id == CommandId::Save));
    }

    #[test]
    fn query_is_case_insensitive() {
        let upper = rank("SAVE", 5);
        let lower = rank("save", 5);
        assert_eq!(upper.len(), lower.len());
    }

    #[test]
    fn unmatchable_query_returns_empty() {
        let r = rank("zzzzzzzzzz_no_match", 5);
        assert!(r.is_empty());
    }

    #[test]
    fn theme_commands_rank_for_partial_query() {
        let r = rank("theme dark", 5);
        assert!(r.iter().any(|c| c.entry.id == CommandId::ThemeDark));
    }
}
