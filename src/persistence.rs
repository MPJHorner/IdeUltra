//! On-disk persistence for IdeUltra: Settings and SessionState.
//!
//! Two human-readable JSON files under
//! `~/Library/Application Support/com.mpjhorner.IdeUltra/`. Writes are
//! atomic (write to `.tmp`, rename) and debounced by the caller.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::editor::language::ColorTheme;
use crate::keymap::KeymapPreset;
use crate::recent::RecentFiles;

/// How a leading-indent unit is rendered when the user hits Tab on a
/// multi-line selection (Tab key on single caret still inserts a `\t`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndentStyle {
    Tab,
    Spaces(u8),
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(4)
    }
}

impl IndentStyle {
    pub fn as_string(self) -> String {
        match self {
            IndentStyle::Tab => "\t".to_string(),
            IndentStyle::Spaces(n) => " ".repeat(n as usize),
        }
    }
    pub fn label(self) -> String {
        match self {
            IndentStyle::Tab => "Tab".to_string(),
            IndentStyle::Spaces(n) => format!("{n} spaces"),
        }
    }
}

/// User-tunable preferences. Survives across sessions; never touched
/// during normal editing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: ColorTheme,
    pub zoom: f32,
    pub sidebar_width: f32,
    pub sidebar_visible: bool,
    /// Whether the markdown preview pane is shown next to .md tabs.
    #[serde(default = "default_markdown_preview")]
    pub markdown_preview: bool,
    /// Save every dirty buffer when the window loses focus.
    #[serde(default = "default_autosave_on_focus_loss")]
    pub autosave_on_focus_loss: bool,
    /// Which preset (Default / VS Code / PhpStorm) drives the global
    /// keyboard shortcuts.
    #[serde(default = "default_keymap_preset")]
    pub keymap_preset: KeymapPreset,
    /// Whether the user has explicitly chosen a preset. False on a fresh
    /// install — drives the first-run keymap picker modal.
    #[serde(default)]
    pub keymap_chosen: bool,
    #[serde(default = "default_trim_whitespace")]
    pub trim_trailing_whitespace_on_save: bool,
    #[serde(default = "default_ensure_final_newline")]
    pub ensure_final_newline_on_save: bool,
    #[serde(default)]
    pub indent_style: IndentStyle,
    #[serde(default)]
    pub soft_wrap: bool,
}

fn default_trim_whitespace() -> bool {
    true
}

fn default_ensure_final_newline() -> bool {
    true
}

fn default_keymap_preset() -> KeymapPreset {
    KeymapPreset::Default
}

fn default_markdown_preview() -> bool {
    true
}

fn default_autosave_on_focus_loss() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ColorTheme::Dark,
            zoom: 1.0,
            sidebar_width: 260.0,
            sidebar_visible: true,
            markdown_preview: true,
            autosave_on_focus_loss: true,
            keymap_preset: KeymapPreset::Default,
            keymap_chosen: false,
            trim_trailing_whitespace_on_save: true,
            ensure_final_newline_on_save: true,
            indent_style: IndentStyle::default(),
            soft_wrap: false,
        }
    }
}

/// What the user had open when they last quit. Restored on relaunch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionState {
    pub window: WindowState,
    pub last_folder: Option<PathBuf>,
    /// Paths only — unsaved buffers live in the recovery store, not here.
    pub open_tabs: Vec<PathBuf>,
    pub active_tab: usize,
    #[serde(default)]
    pub recent_files: RecentFiles,
    #[serde(default)]
    pub pane2_active: Option<usize>,
    #[serde(default)]
    pub focused_right: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub size: [f32; 2],
    pub pos: Option<[f32; 2]>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            size: [1200.0, 800.0],
            pos: None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────
// Loader
// ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PersistencePaths {
    pub settings: PathBuf,
    pub session: PathBuf,
}

impl PersistencePaths {
    pub fn from_project_dirs() -> Option<Self> {
        let dirs = ProjectDirs::from("com", "mpjhorner", "IdeUltra")?;
        let data = dirs.data_dir();
        Some(Self {
            settings: data.join("settings.json"),
            session: data.join("session.json"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Loaded {
    pub settings: Settings,
    pub session: SessionState,
    pub paths: Option<PersistencePaths>,
}

pub fn load() -> Loaded {
    let paths = PersistencePaths::from_project_dirs();
    let settings = paths
        .as_ref()
        .and_then(|p| read_json::<Settings>(&p.settings).ok())
        .unwrap_or_default();
    let session = paths
        .as_ref()
        .and_then(|p| read_json::<SessionState>(&p.session).ok())
        .unwrap_or_default();
    let mut session = filter_missing_tabs(session);
    session.recent_files.prune_missing();
    Loaded {
        settings,
        session,
        paths,
    }
}

/// Drop tabs whose files no longer exist. Surfacing a "file not found"
/// error on launch is worse UX than silently moving on.
fn filter_missing_tabs(mut s: SessionState) -> SessionState {
    let before = s.open_tabs.len();
    s.open_tabs.retain(|p| p.is_file());
    let after = s.open_tabs.len();
    if before != after {
        tracing::warn!(
            dropped = before - after,
            "skipped tabs whose files no longer exist"
        );
    }
    if s.active_tab >= s.open_tabs.len() {
        s.active_tab = 0;
    }
    s
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}

// ────────────────────────────────────────────────────────────────────────
// Saver
// ────────────────────────────────────────────────────────────────────────

/// Tracks dirtiness and debounces writes so we don't fsync on every keystroke.
pub struct Saver {
    paths: Option<PersistencePaths>,
    dirty: bool,
    last_save: Instant,
}

impl Saver {
    pub fn new(paths: Option<PersistencePaths>) -> Self {
        Self {
            paths,
            dirty: false,
            last_save: Instant::now(),
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Save if dirty AND at least `debounce` has elapsed since the last save.
    /// Cheap to call every frame.
    pub fn maybe_save(
        &mut self,
        settings: &Settings,
        session: &SessionState,
        debounce: Duration,
    ) {
        if !self.dirty || self.last_save.elapsed() < debounce {
            return;
        }
        if let Err(err) = self.save_now(settings, session) {
            tracing::warn!(error = %err, "persistence save failed");
        } else {
            self.dirty = false;
            self.last_save = Instant::now();
        }
    }

    pub fn save_now(&self, settings: &Settings, session: &SessionState) -> Result<()> {
        let Some(paths) = &self.paths else {
            anyhow::bail!("no project dirs available");
        };
        if let Some(parent) = paths.settings.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("mkdir {}", parent.display()))?;
        }
        write_json_atomic(&paths.settings, settings)?;
        write_json_atomic(&paths.session, session)?;
        Ok(())
    }
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &bytes).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn settings_default_round_trip() {
        let s = Settings::default();
        let bytes = serde_json::to_vec(&s).unwrap();
        let s2: Settings = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(s.theme, s2.theme);
        assert_eq!(s.zoom, s2.zoom);
        assert_eq!(s.sidebar_width, s2.sidebar_width);
        assert_eq!(s.sidebar_visible, s2.sidebar_visible);
    }

    #[test]
    fn session_round_trips_through_json() {
        let sess = SessionState {
            window: WindowState {
                size: [1500.0, 900.0],
                pos: Some([100.0, 50.0]),
            },
            last_folder: Some(PathBuf::from("/tmp/proj")),
            open_tabs: vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")],
            active_tab: 1,
            recent_files: Default::default(),
            pane2_active: None,
            focused_right: false,
        };
        let bytes = serde_json::to_vec(&sess).unwrap();
        let parsed: SessionState = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.window.size, [1500.0, 900.0]);
        assert_eq!(parsed.last_folder, Some(PathBuf::from("/tmp/proj")));
        assert_eq!(parsed.open_tabs.len(), 2);
        assert_eq!(parsed.active_tab, 1);
    }

    #[test]
    fn filter_missing_tabs_drops_paths_that_no_longer_exist() {
        let dir = tempdir().unwrap();
        let real = dir.path().join("real.rs");
        fs::write(&real, "").unwrap();
        let fake = dir.path().join("nope.rs");

        let sess = SessionState {
            window: WindowState::default(),
            last_folder: None,
            open_tabs: vec![real.clone(), fake],
            active_tab: 1, // points at the fake one
            recent_files: Default::default(),
            pane2_active: None,
            focused_right: false,
        };
        let filtered = super::filter_missing_tabs(sess);
        assert_eq!(filtered.open_tabs, vec![real]);
        // active_tab is out of range after the drop — clamps to 0.
        assert_eq!(filtered.active_tab, 0);
    }

    #[test]
    fn atomic_write_then_read_round_trips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let s = Settings::default();
        write_json_atomic(&path, &s).unwrap();
        let read: Settings = read_json(&path).unwrap();
        assert_eq!(read.zoom, s.zoom);
        // The temp file should be cleaned up by the rename.
        assert!(!dir.path().join("settings.tmp").exists());
    }

    #[test]
    fn saver_debounces_writes() {
        let dir = tempdir().unwrap();
        let paths = PersistencePaths {
            settings: dir.path().join("settings.json"),
            session: dir.path().join("session.json"),
        };
        let mut saver = Saver::new(Some(paths.clone()));
        let s = Settings::default();
        let sess = SessionState::default();

        saver.mark_dirty();
        // Debounce is 1 hour — should NOT save yet.
        saver.maybe_save(&s, &sess, Duration::from_secs(3600));
        assert!(!paths.settings.exists());

        // Debounce of zero — should save now.
        saver.maybe_save(&s, &sess, Duration::from_secs(0));
        assert!(paths.settings.exists());
    }
}
