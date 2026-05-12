use std::path::{Path, PathBuf};

use eframe::CreationContext;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};

use std::time::Duration;

use crate::editor::language::{language_label, ColorTheme};
use crate::editor::position::{char_index_at_line_start, detect_line_ending, line_col_at_char};
use crate::editor::EditorTab;
use crate::find::replace_all as do_replace_all;
use crate::persistence::{self, Loaded, Saver, SessionState, Settings, WindowState};
use crate::ui::editor_panel::Jump;
use crate::ui::find_bar::{self, FindAction, FindState};
use crate::command_palette::CommandId;
use crate::diff::{line_diff, DiffSummary};
use crate::keymap::{Keymap, KeymapPreset};
use crate::mru::TabMru;
use crate::recent::RecentFiles;
use crate::recovery::{Recovery, RecoveryStore, RECOVERY_DEBOUNCE_MS};
use crate::ui::diff_modal::{self, DiffModalAction};
use crate::ui::command_palette_modal::{
    self, CommandPaletteAction, CommandPaletteState,
};
use crate::ui::finder_modal::{self, FinderAction, FinderState};
use crate::ui::project_search_panel::{
    self, ProjectSearchAction, ProjectSearchState,
};
use crate::ui::close_confirm_modal::{self, CloseConfirmAction};
use crate::ui::delete_confirm_modal::{self, DeleteAction};
use crate::ui::keymap_picker::{self, KeymapPickerAction};
use crate::ui::name_prompt_modal::{self, NamePromptAction, NamePromptKind, NamePromptState};
use crate::ui::preferences_window::{self, PreferencesAction, PreferencesView};
use crate::ui::quick_switcher::{self, QuickSwitcherState};
use crate::ui::replace_confirm_modal::{self, ReplaceConfirmAction};
use crate::ui::recovery_modal::{self, RecoveryAction};
use crate::ui::{
    editor_panel,
    sidebar::{self, SidebarAction},
    tabs::{self, TabAction},
};
use crate::workspace::watcher::{Change, ChangeKind};
use crate::workspace::Workspace;

const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

pub struct IdeUltraApp {
    started_at: std::time::Instant,
    first_frame_logged: bool,
    workspace: Option<Workspace>,
    tabs: Vec<EditorTab>,
    active_tab: usize,
    sidebar_width: f32,
    status_message: Option<(String, std::time::Instant)>,
    theme: ColorTheme,
    find: FindState,
    last_find_signature: Option<u64>,
    sidebar_visible: bool,
    zoom: f32,
    caret_line_col: Option<(usize, usize)>,
    goto_open: bool,
    goto_input: String,
    pending_goto_char: Option<usize>,
    saver: Saver,
    window_state: WindowState,
    finder: FinderState,
    project_search: ProjectSearchState,
    palette: CommandPaletteState,
    recovery_store: Option<RecoveryStore>,
    /// Recoveries surfaced at startup, awaiting the user's choice.
    pending_recoveries: Vec<Recovery>,
    /// Per-tab cooldown timestamps so we don't fsync on every keystroke.
    last_recovery_write: std::collections::HashMap<PathBuf, std::time::Instant>,
    markdown_preview: bool,
    /// Active diff modal: (tab index, summary, file name shown in title).
    diff_modal: Option<(usize, DiffSummary, String)>,
    recent_files: RecentFiles,
    recent_workspaces: RecentFiles,
    autosave_on_focus_loss: bool,
    /// Last-known viewport focus state. Auto-save fires on true → false.
    was_focused: bool,
    trim_whitespace_on_save: bool,
    ensure_final_newline_on_save: bool,
    indent_style: crate::persistence::IndentStyle,
    soft_wrap: bool,
    check_for_updates: bool,
    keymap: Keymap,
    keymap_preset: KeymapPreset,
    /// Whether the user has explicitly chosen a keymap. False on first run.
    keymap_chosen: bool,
    /// Show the keymap picker as a modal (first-run or user-requested).
    keymap_picker_open: bool,
    /// When set, the close-confirm modal is shown for these tab indices.
    /// Resolves one tab at a time so the user can decide per-buffer.
    pending_close: Vec<usize>,
    /// After auto-pair inserts a closer, the next frame walks the cursor
    /// back one step so the caret sits between the brackets.
    autopair_backstep_pending: bool,
    /// Tab most-recently-used order. Driven by every active-tab change
    /// (typing, click, ⌘1..9, sidebar click) and consumed by Ctrl+Tab.
    tab_mru: TabMru,
    quick_switcher: QuickSwitcherState,
    preferences_open: bool,
    /// Active tab of the *non-focused* pane when the editor is split.
    /// `None` ⇒ no split; a single pane renders.
    pane2_active: Option<usize>,
    /// Which pane is currently focused. Only meaningful when `pane2_active`
    /// is `Some`. `active_tab` always refers to the focused pane's tab.
    focused_right: bool,
    /// When set, the replace-in-project confirmation modal is showing.
    replace_confirm_open: bool,
    /// Counter for "Untitled N" filenames. Resets every launch.
    next_untitled_n: usize,
    /// Result of the background update check, surfaced as a banner when set.
    pending_update: Option<crate::updater::UpdateInfo>,
    /// Channel where the update-check background thread delivers its result.
    update_rx: Option<crossbeam_channel::Receiver<Option<crate::updater::UpdateInfo>>>,
    /// Active sidebar-driven name prompt (New File, New Folder, Rename).
    /// `parent_or_path` is the directory to create under, or the existing
    /// path being renamed.
    name_prompt: Option<(NamePromptState, std::path::PathBuf)>,
    /// Active delete-confirm modal: (path, is_dir).
    delete_confirm: Option<(std::path::PathBuf, bool)>,
}

impl IdeUltraApp {
    pub fn new(cc: &CreationContext<'_>, loaded: Loaded) -> Self {
        // Apply the persisted zoom to the egui context right away — otherwise
        // the first frame paints at 1.0 and snaps a frame later.
        cc.egui_ctx.set_zoom_factor(loaded.settings.zoom);

        let mut app = Self {
            started_at: std::time::Instant::now(),
            first_frame_logged: false,
            workspace: None,
            tabs: Vec::new(),
            active_tab: 0,
            sidebar_width: loaded.settings.sidebar_width,
            status_message: None,
            theme: loaded.settings.theme,
            find: FindState::default(),
            last_find_signature: None,
            sidebar_visible: loaded.settings.sidebar_visible,
            zoom: loaded.settings.zoom,
            caret_line_col: None,
            goto_open: false,
            goto_input: String::new(),
            pending_goto_char: None,
            saver: Saver::new(loaded.paths.clone()),
            window_state: loaded.session.window.clone(),
            finder: FinderState::default(),
            project_search: ProjectSearchState::default(),
            palette: CommandPaletteState::default(),
            recovery_store: RecoveryStore::from_project_dirs(),
            pending_recoveries: Vec::new(),
            last_recovery_write: Default::default(),
            markdown_preview: loaded.settings.markdown_preview,
            diff_modal: None,
            recent_files: loaded.session.recent_files.clone(),
            recent_workspaces: loaded.session.recent_workspaces.clone(),
            autosave_on_focus_loss: loaded.settings.autosave_on_focus_loss,
            was_focused: true,
            trim_whitespace_on_save: loaded.settings.trim_trailing_whitespace_on_save,
            ensure_final_newline_on_save: loaded.settings.ensure_final_newline_on_save,
            indent_style: loaded.settings.indent_style,
            soft_wrap: loaded.settings.soft_wrap,
            check_for_updates: loaded.settings.check_for_updates,
            keymap: Keymap::for_preset(loaded.settings.keymap_preset),
            keymap_preset: loaded.settings.keymap_preset,
            keymap_chosen: loaded.settings.keymap_chosen,
            keymap_picker_open: !loaded.settings.keymap_chosen,
            pending_close: Vec::new(),
            autopair_backstep_pending: false,
            tab_mru: TabMru::default(),
            quick_switcher: QuickSwitcherState::default(),
            preferences_open: false,
            pane2_active: loaded.session.pane2_active,
            focused_right: loaded.session.focused_right,
            replace_confirm_open: false,
            next_untitled_n: 1,
            pending_update: None,
            update_rx: None,
            name_prompt: None,
            delete_confirm: None,
        };
        if loaded.settings.check_for_updates {
            app.spawn_update_check();
        }
        // Seed the MRU from the restored tabs so Ctrl+Tab works on first
        // launch. Order: active tab first, then the others in tab order.
        for i in 0..app.tabs.len() {
            app.tab_mru.touch(i);
        }
        if app.active_tab < app.tabs.len() {
            app.tab_mru.touch(app.active_tab);
        }

        // Surface anything left over from a previous crash / force-quit.
        // We don't restore automatically — the user decides.
        if let Some(store) = &app.recovery_store {
            app.pending_recoveries = store.scan();
            if !app.pending_recoveries.is_empty() {
                tracing::info!(
                    count = app.pending_recoveries.len(),
                    "found recoverable buffers"
                );
            }
        }

        // Restore the last workspace (if any) and the tabs that were open.
        if let Some(folder) = &loaded.session.last_folder {
            if folder.is_dir() {
                if let Ok(ws) = Workspace::open(folder) {
                    app.workspace = Some(ws);
                }
            }
        }
        for path in &loaded.session.open_tabs {
            // open_file mutates app state; we intentionally don't mark dirty
            // here — restoration shouldn't trigger an immediate save.
            if let Ok(tab) = EditorTab::open(path) {
                app.tabs.push(tab);
            }
        }
        if loaded.session.active_tab < app.tabs.len() {
            app.active_tab = loaded.session.active_tab;
        }

        app
    }

    fn current_settings(&self) -> Settings {
        Settings {
            theme: self.theme,
            zoom: self.zoom,
            sidebar_width: self.sidebar_width,
            sidebar_visible: self.sidebar_visible,
            markdown_preview: self.markdown_preview,
            autosave_on_focus_loss: self.autosave_on_focus_loss,
            keymap_preset: self.keymap_preset,
            keymap_chosen: self.keymap_chosen,
            trim_trailing_whitespace_on_save: self.trim_whitespace_on_save,
            ensure_final_newline_on_save: self.ensure_final_newline_on_save,
            indent_style: self.indent_style,
            soft_wrap: self.soft_wrap,
            check_for_updates: self.check_for_updates,
        }
    }

    fn toggle_split(&mut self) {
        if self.tabs.is_empty() {
            self.flash("Open a file first");
            return;
        }
        if self.pane2_active.is_some() {
            // Collapse: keep whichever pane is currently focused; close the other.
            self.pane2_active = None;
            self.focused_right = false;
        } else {
            // Open split: secondary pane starts on the same tab so the user
            // can move it elsewhere without losing their place.
            self.pane2_active = Some(self.active_tab);
        }
    }

    /// Click / activate in pane `right`. If that's the non-focused pane,
    /// swap the active tabs so `self.active_tab` still names the focused
    /// pane's active buffer.
    fn focus_pane(&mut self, want_right: bool, new_active: usize) {
        let split = self.pane2_active.is_some();
        if !split {
            self.active_tab = new_active;
            return;
        }
        let currently_right = self.focused_right;
        if want_right == currently_right {
            // Same pane — just update its active tab.
            self.active_tab = new_active;
        } else {
            // Switching pane: the OLD active_tab becomes pane2_active.
            let prev_active = self.active_tab;
            self.active_tab = new_active;
            self.pane2_active = Some(prev_active);
            self.focused_right = want_right;
        }
    }

    /// When a tab is removed globally, clamp `pane2_active` so it never
    /// points past the end. If we'd land on the same slot as `active_tab`
    /// after clamping, the split collapses.
    fn fix_pane2_after_removal(&mut self) {
        let Some(p2) = self.pane2_active else { return };
        if self.tabs.is_empty() {
            self.pane2_active = None;
            self.focused_right = false;
            return;
        }
        let clamped = p2.min(self.tabs.len() - 1);
        if clamped == self.active_tab {
            // Both panes would land on the same tab — collapse the split.
            self.pane2_active = None;
            self.focused_right = false;
        } else {
            self.pane2_active = Some(clamped);
        }
    }

    fn switch_keymap(&mut self, preset: KeymapPreset) {
        self.keymap_preset = preset;
        self.keymap = Keymap::for_preset(preset);
        self.keymap_chosen = true;
        self.saver.mark_dirty();
        self.flash(format!("Keymap: {}", preset.label()));
    }

    fn handle_focus_autosave(&mut self, ctx: &Context) {
        let now_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if self.was_focused && !now_focused && self.autosave_on_focus_loss {
            self.autosave_all_dirty();
        }
        // Coming back to focus is a good cheap moment to refresh git status —
        // the user may have committed or branched while we were in the background.
        if !self.was_focused && now_focused {
            if let Some(ws) = self.workspace.as_mut() {
                ws.refresh_git_status();
            }
        }
        self.was_focused = now_focused;
    }

    fn autosave_all_dirty(&mut self) {
        let mut saved = 0usize;
        let mut errors = 0usize;
        let trim = self.trim_whitespace_on_save;
        let final_nl = self.ensure_final_newline_on_save;
        for tab in self.tabs.iter_mut() {
            if !tab.is_dirty() {
                continue;
            }
            // Untitled tabs need user input for path — skip on autosave.
            if tab.is_untitled {
                continue;
            }
            tab.apply_save_normalization(trim, final_nl);
            match tab.save() {
                Ok(_) => {
                    saved += 1;
                    if let Some(store) = &self.recovery_store {
                        let _ = store.clear(&tab.path);
                    }
                    tab.last_recovered_hash = None;
                }
                Err(err) => {
                    errors += 1;
                    tracing::warn!(
                        error = %err,
                        file = %tab.path.display(),
                        "autosave failed",
                    );
                }
            }
        }
        if saved > 0 || errors > 0 {
            tracing::info!(saved, errors, "focus-loss autosave");
            self.flash(format!(
                "Auto-saved {saved} buffer(s){}",
                if errors > 0 {
                    format!(" ({errors} failed)")
                } else {
                    String::new()
                }
            ));
        }
    }

    fn transform_active_text<F: FnOnce(&str) -> String>(&mut self, f: F) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.buffer.text = f(&tab.buffer.text);
        }
    }

    fn current_session(&self) -> SessionState {
        SessionState {
            window: self.window_state.clone(),
            last_folder: self.workspace.as_ref().map(|w| w.root.clone()),
            open_tabs: self.tabs.iter().map(|t| t.path.clone()).collect(),
            active_tab: self.active_tab,
            recent_files: self.recent_files.clone(),
            pane2_active: self.pane2_active,
            focused_right: self.focused_right,
            recent_workspaces: self.recent_workspaces.clone(),
        }
    }

    fn drain_watcher(&mut self) {
        let Some(ws) = self.workspace.as_mut() else {
            return;
        };
        let Some(watcher) = ws.watcher.as_ref() else {
            return;
        };
        let events = watcher.drain();
        if events.is_empty() {
            return;
        }

        // Invalidate every affected tree dir once per frame.
        let mut affected_dirs: std::collections::HashSet<PathBuf> = Default::default();
        let mut structural_change = false;
        for ev in &events {
            if let Some(parent) = ev.path.parent() {
                affected_dirs.insert(parent.to_path_buf());
            }
            affected_dirs.insert(ev.path.clone());
            if matches!(
                ev.kind,
                ChangeKind::Created | ChangeKind::Removed | ChangeKind::Renamed
            ) {
                structural_change = true;
            }
        }
        for d in affected_dirs {
            ws.tree.invalidate_containing(&d);
        }
        // Drop the fuzzy-find index on structural changes so the next ⌘P
        // sees the current shape of the workspace. The walk is fast.
        if structural_change {
            ws.invalidate_index();
        }

        // Apply to any open tab whose path matches an event.
        for ev in events {
            self.apply_external_change(ev);
        }
    }

    fn apply_external_change(&mut self, ev: Change) {
        let Some(idx) = self.tabs.iter().position(|t| t.path == ev.path) else {
            return;
        };
        match ev.kind {
            ChangeKind::Removed => {
                tracing::info!(file = %ev.path.display(), "external removal — marking tab");
                if let Some(tab) = self.tabs.get_mut(idx) {
                    // Don't auto-close: the user might want to recover from buffer.
                    tab.external_change = true;
                }
            }
            ChangeKind::Modified | ChangeKind::Renamed | ChangeKind::Created => {
                let dirty = self.tabs[idx].is_dirty();
                if !dirty {
                    if let Err(err) = self.tabs[idx].reload_from_disk() {
                        tracing::warn!(error = %err, "reload_from_disk failed");
                        self.tabs[idx].external_change = true;
                    } else {
                        tracing::info!(file = %ev.path.display(), "reloaded clean tab");
                    }
                } else {
                    self.tabs[idx].external_change = true;
                }
            }
            ChangeKind::Other => {}
        }
    }

    fn read_window_state(&mut self, ctx: &Context) {
        let new_state = ctx.input(|i| {
            let vp = i.viewport();
            let size = vp
                .inner_rect
                .map(|r| [r.size().x, r.size().y])
                .unwrap_or(self.window_state.size);
            let pos = vp
                .outer_rect
                .map(|r| [r.min.x, r.min.y])
                .or(self.window_state.pos);
            WindowState { size, pos }
        });
        if new_state.size != self.window_state.size || new_state.pos != self.window_state.pos {
            self.window_state = new_state;
            self.saver.mark_dirty();
        }
    }

    fn apply_theme(&self, ctx: &Context) {
        crate::style::apply(ctx, self.theme);
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.open_workspace(&path);
        }
    }

    fn open_workspace(&mut self, path: &Path) {
        match Workspace::open(path) {
            Ok(ws) => {
                tracing::info!(root = %ws.root.display(), "workspace opened");
                self.recent_workspaces.push(ws.root.clone());
                self.workspace = Some(ws);
                self.saver.mark_dirty();
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to open workspace");
                self.flash(format!("Could not open folder: {err}"));
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open_file(&path);
        }
    }

    fn open_file(&mut self, path: &Path) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = idx;
            self.tab_mru.touch(idx);
            self.recent_files.push(path.to_path_buf());
            self.saver.mark_dirty();
            return;
        }
        match EditorTab::open(path) {
            Ok(tab) => {
                tracing::info!(file = %path.display(), "file opened");
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                self.tab_mru.touch(self.active_tab);
                self.recent_files.push(path.to_path_buf());
                self.saver.mark_dirty();
            }
            Err(err) => {
                tracing::warn!(file = %path.display(), error = %err, "open failed");
                self.flash(format!("Could not open file: {err}"));
            }
        }
    }

    fn save_active(&mut self) {
        // Untitled tabs route through Save As — we don't have a real path yet.
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.is_untitled {
                self.save_as_active();
                return;
            }
        }
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.apply_save_normalization(
                self.trim_whitespace_on_save,
                self.ensure_final_newline_on_save,
            );
            match tab.save() {
                Ok(_) => {
                    let msg = format!("Saved {}", tab.display_name);
                    tracing::info!(file = %tab.path.display(), "saved");
                    if let Some(store) = &self.recovery_store {
                        let _ = store.clear(&tab.path);
                    }
                    tab.last_recovered_hash = None;
                    if let Some(ws) = self.workspace.as_mut() {
                        ws.refresh_git_status();
                    }
                    self.flash(msg);
                }
                Err(err) => {
                    tracing::warn!(error = %err, "save failed");
                    self.flash(format!("Save failed: {err}"));
                }
            }
        }
    }

    fn write_recoveries(&mut self) {
        let Some(store) = self.recovery_store.as_ref() else {
            return;
        };
        let now = std::time::Instant::now();
        for tab in self.tabs.iter_mut() {
            if !tab.is_dirty() {
                continue;
            }
            // Untitled buffers don't have a real path — the recovery store
            // key would collide across launches. Skip until Save As lands.
            if tab.is_untitled {
                continue;
            }
            // Per-tab debounce.
            if let Some(last) = self.last_recovery_write.get(&tab.path) {
                if now.duration_since(*last)
                    < std::time::Duration::from_millis(RECOVERY_DEBOUNCE_MS)
                {
                    continue;
                }
            }
            // Skip when content is unchanged from the last snapshot.
            let h = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hh = DefaultHasher::new();
                tab.buffer.text.hash(&mut hh);
                hh.finish()
            };
            if tab.last_recovered_hash == Some(h) {
                self.last_recovery_write.insert(tab.path.clone(), now);
                continue;
            }
            match store.write(&tab.path, &tab.buffer.text) {
                Ok(_) => {
                    tab.last_recovered_hash = Some(h);
                    self.last_recovery_write.insert(tab.path.clone(), now);
                }
                Err(err) => {
                    tracing::warn!(error = %err, path = %tab.path.display(), "recovery write failed");
                }
            }
        }
    }

    fn open_diff_for_active_tab(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };
        let on_disk = match std::fs::read_to_string(&tab.path) {
            Ok(s) => s,
            Err(err) => {
                self.flash(format!("Could not read for diff: {err}"));
                return;
            }
        };
        let summary = line_diff(&on_disk, &tab.buffer.text);
        self.diff_modal = Some((self.active_tab, summary, tab.display_name.clone()));
    }

    fn apply_recovery_action(&mut self, action: RecoveryAction) {
        match action {
            RecoveryAction::None => {}
            RecoveryAction::Dismiss => {
                // Drop the list so we don't keep rendering the modal. The
                // recovery files stay on disk for the next launch.
                self.pending_recoveries.clear();
            }
            RecoveryAction::DiscardAll => {
                if let Some(store) = &self.recovery_store {
                    if let Err(err) = store.clear_all() {
                        tracing::warn!(error = %err, "clear_all failed");
                    }
                }
                self.pending_recoveries.clear();
                self.flash("Discarded unsaved drafts");
            }
            RecoveryAction::RestoreAll => {
                let mut restored = 0usize;
                let recoveries = std::mem::take(&mut self.pending_recoveries);
                for r in recoveries {
                    // If the file is already open in a tab, just push the
                    // recovered text into the existing buffer.
                    if let Some(idx) =
                        self.tabs.iter().position(|t| t.path == r.meta.path)
                    {
                        self.tabs[idx].buffer.text = r.contents;
                        restored += 1;
                    } else {
                        let tab = EditorTab::from_recovered(r.meta.path, r.contents);
                        self.tabs.push(tab);
                        self.active_tab = self.tabs.len() - 1;
                        restored += 1;
                    }
                }
                self.flash(format!("Restored {restored} buffer(s) — save when ready"));
            }
        }
    }

    /// Public entry-point for closing a tab. Defers to the dirty-confirm
    /// modal if the buffer has unsaved changes.
    fn request_close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        if self.tabs[idx].is_dirty() {
            // Queue and surface the modal; we'll get the user's answer next
            // frame. Multiple queued tabs prompt one at a time.
            if !self.pending_close.contains(&idx) {
                self.pending_close.push(idx);
            }
            return;
        }
        self.force_close_tab(idx);
    }

    fn force_close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        if let Some(tab) = self.tabs.get(idx) {
            if let Some(store) = &self.recovery_store {
                let _ = store.clear(&tab.path);
            }
        }
        self.tabs.remove(idx);
        self.tab_mru.removed(idx);
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.tabs.is_empty() {
            self.active_tab = 0;
        }
        // Rebase pane2_active too, then collapse if it now coincides.
        if let Some(p2) = self.pane2_active {
            if p2 == idx {
                // The removed tab WAS the secondary pane's active — collapse.
                self.pane2_active = None;
                self.focused_right = false;
            } else if p2 > idx {
                self.pane2_active = Some(p2 - 1);
            }
        }
        self.fix_pane2_after_removal();
        if !self.tabs.is_empty() {
            self.tab_mru.touch(self.active_tab);
        }
        self.saver.mark_dirty();
    }

    fn close_others(&mut self, keep_idx: usize) {
        if keep_idx >= self.tabs.len() {
            return;
        }
        // Build list of indices to close in *descending* order so removals
        // don't shift the others. Skip the kept one.
        let to_close: Vec<usize> = (0..self.tabs.len())
            .rev()
            .filter(|i| *i != keep_idx)
            .collect();
        for i in to_close {
            self.request_close_tab(i);
        }
    }

    fn close_all_tabs(&mut self) {
        let to_close: Vec<usize> = (0..self.tabs.len()).rev().collect();
        for i in to_close {
            self.request_close_tab(i);
        }
    }

    fn copy_to_clipboard(&mut self, ctx: &Context, text: String) {
        ctx.copy_text(text.clone());
        self.flash(format!("Copied: {text}"));
    }

    fn reveal_in_finder(&mut self, path: &Path) {
        match std::process::Command::new("open").arg("-R").arg(path).status() {
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, "reveal in finder failed");
                self.flash(format!("Reveal failed: {err}"));
            }
        }
    }

    fn flash(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    fn handle_text_input_autopair(&mut self, ctx: &Context) {
        // Auto-pair only fires when the user is actively typing into the
        // editor of an existing tab — not when another widget has focus.
        let Some(tab_idx) = self.tabs.get(self.active_tab).map(|_| self.active_tab) else {
            return;
        };
        let path = self.tabs[tab_idx].path.clone();
        let editor_id = egui::Id::new(("ide_editor", path.as_path()));
        let has_focus = ctx.memory(|m| m.has_focus(editor_id));
        if !has_focus {
            return;
        }

        // Snapshot cursor + text so we can decide based on context.
        let cursor = egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
            .and_then(|s| s.cursor.char_range())
            .map(|r| r.primary.index);
        let Some(cursor_char) = cursor else { return };
        let text = self.tabs[tab_idx].buffer.text.clone();

        ctx.input_mut(|i| {
            for ev in i.events.iter_mut() {
                let egui::Event::Text(s) = ev else { continue };
                // Only single-character inserts qualify.
                let Some(first) = s.chars().next() else { continue };
                if s.chars().count() != 1 {
                    continue;
                }

                // Skip-closer: typing `)` right before an existing `)` just
                // moves the caret past it.
                if crate::autopair::should_skip_closer(&text, cursor_char, first) {
                    *s = String::new();
                    self.autopair_backstep_pending = false;
                    // We don't actually skip the cursor forward here; the
                    // user's right-arrow / next-input does that. Emptying
                    // the text event swallows the duplicate insertion.
                    continue;
                }

                // Auto-pair: replace `(` with `()`, etc., and queue a
                // back-step so the caret sits between them next frame.
                if let Some((_, closer)) =
                    crate::autopair::pair_for_opener(first)
                {
                    if crate::autopair::should_auto_pair(&text, cursor_char, first)
                    {
                        let mut paired = String::with_capacity(2);
                        paired.push(first);
                        paired.push(closer);
                        *s = paired;
                        self.autopair_backstep_pending = true;
                    }
                }
            }
        });
    }

    fn apply_autopair_backstep(&mut self, ctx: &Context) {
        if !self.autopair_backstep_pending {
            return;
        }
        self.autopair_backstep_pending = false;
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };
        let editor_id = egui::Id::new(("ide_editor", tab.path.as_path()));
        let Some(mut state) =
            egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
        else {
            return;
        };
        let Some(range) = state.cursor.char_range() else {
            return;
        };
        if range.primary.index == 0 {
            return;
        }
        let new_pos = range.primary.index.saturating_sub(1);
        use egui::text::{CCursor, CCursorRange};
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(
                CCursor::new(new_pos),
                CCursor::new(new_pos),
            )));
        state.store(ctx, editor_id);
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        // 1. Modal-context Esc and modal Enter/Arrow keys live here because
        //    they're context-sensitive (Esc on find bar ≠ Esc anywhere else).
        ctx.input_mut(|i| {
            use egui::Key;
            if self.find.open && i.key_pressed(Key::Escape) {
                self.find.close();
            }
            if self.project_search.open
                && i.key_pressed(Key::Escape)
                && !self.find.open
                && !self.finder.open
                && !self.goto_open
            {
                self.project_search.close();
            }
        });

        // 1b. Tab / Shift+Tab on a multi-line selection indents the selection
        //     rather than inserting a literal tab. Single-caret Tab falls
        //     through to the TextEdit (we don't consume the event then).
        let active_path = self
            .tabs
            .get(self.active_tab)
            .map(|t| t.path.clone());
        if let Some(path) = &active_path {
            let editor_id = egui::Id::new(("ide_editor", path.as_path()));
            let selection = egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
                .and_then(|s| s.cursor.char_range())
                .map(|r| (r.primary.index, r.secondary.index));
            let multi_line = selection
                .map(|sel| {
                    let text = self
                        .tabs
                        .get(self.active_tab)
                        .map(|t| t.buffer.text.as_str())
                        .unwrap_or("");
                    crosses_newline(text, sel)
                })
                .unwrap_or(false);
            if multi_line {
                let mut did_indent = false;
                let mut did_dedent = false;
                ctx.input_mut(|i| {
                    use egui::{Key, Modifiers};
                    if i.consume_key(Modifiers::SHIFT, Key::Tab)
                        || i.consume_key(Modifiers::NONE, Key::Tab)
                            && i.modifiers.shift
                    {
                        did_dedent = true;
                    } else if i.consume_key(Modifiers::NONE, Key::Tab) {
                        did_indent = true;
                    }
                });
                if did_indent {
                    self.apply_indent(ctx, false);
                } else if did_dedent {
                    self.apply_indent(ctx, true);
                }
            }
        }

        // 2. Tab navigation (Cmd+[ / Cmd+] / Cmd+1..9 / Ctrl+Tab MRU) is
        //    fixed across presets — these are universal conventions.
        let mut activate: Option<usize> = None;
        ctx.input_mut(|i| {
            use egui::{Key, KeyboardShortcut, Modifiers};
            let cmd = Modifiers::COMMAND;
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::CloseBracket))
                && !self.tabs.is_empty()
            {
                let n = self.tabs.len();
                activate = Some((self.active_tab + 1) % n);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::OpenBracket))
                && !self.tabs.is_empty()
            {
                let n = self.tabs.len();
                activate = Some(if self.active_tab == 0 { n - 1 } else { self.active_tab - 1 });
            }
            for (n, key) in [
                Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5,
                Key::Num6, Key::Num7, Key::Num8, Key::Num9,
            ]
            .iter()
            .enumerate()
            {
                if i.consume_shortcut(&KeyboardShortcut::new(cmd, *key)) {
                    if n < self.tabs.len() {
                        activate = Some(n);
                    }
                }
            }
            if i.consume_shortcut(&KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::P,
            )) {
                self.palette.open();
            }

            // Ctrl+Tab — quick-switcher. While Ctrl is held, each Tab press
            // advances the MRU selection. We open on the first press; on
            // Ctrl-release we commit and activate the chosen tab.
            let ctrl = i.modifiers.ctrl;
            let tab_pressed = i.consume_key(Modifiers::NONE, Key::Tab) && ctrl
                || i.consume_key(Modifiers::CTRL, Key::Tab);
            let shift_tab_pressed = i.consume_key(Modifiers::SHIFT, Key::Tab) && ctrl
                || i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
            if (tab_pressed || shift_tab_pressed) && self.tabs.len() >= 2 {
                if !self.quick_switcher.open {
                    self.quick_switcher.open = true;
                    // Start at MRU position 1 — that's the next-most-recent,
                    // i.e. the most useful first jump from the current tab.
                    self.quick_switcher.selected = 1.min(self.tab_mru.len().saturating_sub(1));
                } else {
                    let step = if shift_tab_pressed { -1 } else { 1 };
                    if let Some(next) =
                        self.tab_mru.cycle(self.quick_switcher.selected, step)
                    {
                        self.quick_switcher.selected = next;
                    }
                }
            }
            // Ctrl released → commit and close.
            if self.quick_switcher.open && !ctrl {
                if let Some(idx) = self.tab_mru.tab_at(self.quick_switcher.selected) {
                    activate = Some(idx);
                }
                self.quick_switcher.open = false;
            }
        });
        if let Some(idx) = activate {
            self.active_tab = idx;
            self.tab_mru.touch(idx);
            self.saver.mark_dirty();
        }

        // 3. Everything else is driven by the active keymap preset. We
        //    collect the IDs to fire, then dispatch outside the input scope.
        let bindings: Vec<(CommandId, egui::KeyboardShortcut)> =
            self.keymap.iter().cloned().collect();
        let mut to_fire: Vec<CommandId> = Vec::new();
        ctx.input_mut(|i| {
            for (id, sc) in &bindings {
                if i.consume_shortcut(sc) {
                    to_fire.push(*id);
                }
            }
        });
        for id in to_fire {
            self.dispatch_command(ctx, id);
        }
    }

    fn dispatch_command(&mut self, ctx: &Context, id: CommandId) {
        match id {
            CommandId::OpenFile => self.open_file_dialog(),
            CommandId::OpenFolder => self.open_folder_dialog(),
            CommandId::Save => self.save_active(),
            CommandId::CloseTab => self.request_close_tab(self.active_tab),
            CommandId::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            CommandId::Find => self.find.open_find(),
            CommandId::FindReplace => self.find.open_replace(),
            CommandId::SearchInProject => self.open_project_search(),
            CommandId::GoToFile => self.open_finder(),
            CommandId::GoToLine => {
                if !self.tabs.is_empty() {
                    self.goto_open = true;
                    self.goto_input.clear();
                }
            }
            CommandId::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                self.saver.mark_dirty();
            }
            CommandId::ZoomIn => {
                self.zoom = (self.zoom + 0.1).clamp(0.5, 3.0);
                ctx.set_zoom_factor(self.zoom);
                self.saver.mark_dirty();
            }
            CommandId::ZoomOut => {
                self.zoom = (self.zoom - 0.1).clamp(0.5, 3.0);
                ctx.set_zoom_factor(self.zoom);
                self.saver.mark_dirty();
            }
            CommandId::ZoomReset => {
                self.zoom = 1.0;
                ctx.set_zoom_factor(self.zoom);
                self.saver.mark_dirty();
            }
            CommandId::ThemeDark => {
                self.theme = ColorTheme::Dark;
                self.saver.mark_dirty();
            }
            CommandId::ThemeLight => {
                self.theme = ColorTheme::Light;
                self.saver.mark_dirty();
            }
            CommandId::ToggleMarkdownPreview => {
                self.markdown_preview = !self.markdown_preview;
                self.saver.mark_dirty();
            }
            CommandId::SortLines => {
                self.transform_active_text(crate::transforms::sort_lines);
            }
            CommandId::SortLinesReverse => {
                self.transform_active_text(crate::transforms::sort_lines_reverse);
            }
            CommandId::UniqueLines => {
                self.transform_active_text(crate::transforms::unique_lines);
            }
            CommandId::UpperCase => {
                self.transform_active_text(crate::transforms::to_upper);
            }
            CommandId::LowerCase => {
                self.transform_active_text(crate::transforms::to_lower);
            }
            CommandId::ToggleAutosaveOnFocusLoss => {
                self.autosave_on_focus_loss = !self.autosave_on_focus_loss;
                self.saver.mark_dirty();
                self.flash(if self.autosave_on_focus_loss {
                    "Auto-save on focus loss: on"
                } else {
                    "Auto-save on focus loss: off"
                });
            }
            CommandId::ChooseKeymap => {
                self.keymap_picker_open = true;
            }
            CommandId::KeymapDefault => self.switch_keymap(KeymapPreset::Default),
            CommandId::KeymapVsCode => self.switch_keymap(KeymapPreset::VsCode),
            CommandId::KeymapPhpStorm => self.switch_keymap(KeymapPreset::PhpStorm),
            CommandId::ToggleLineComment => self.toggle_line_comment(ctx),
            CommandId::OpenPreferences => self.preferences_open = !self.preferences_open,
            CommandId::ToggleSplit => self.toggle_split(),
            CommandId::SelectNextOccurrence => self.select_next_occurrence(ctx),
            CommandId::NewUntitled => self.new_untitled(),
            CommandId::SaveAs => self.save_as_active(),
        }
    }

    fn handle_sidebar_request(&mut self, ctx: &Context, req: SidebarAction) {
        match req {
            SidebarAction::None | SidebarAction::OpenFile(_) => {}
            SidebarAction::NewFileIn(dir) => {
                let display = self.workspace_relative_display(&dir);
                self.name_prompt = Some((
                    NamePromptState {
                        kind: NamePromptKind::NewFile,
                        context_label: format!("in {display}"),
                        input: String::new(),
                        just_opened: true,
                    },
                    dir,
                ));
            }
            SidebarAction::NewFolderIn(dir) => {
                let display = self.workspace_relative_display(&dir);
                self.name_prompt = Some((
                    NamePromptState {
                        kind: NamePromptKind::NewFolder,
                        context_label: format!("in {display}"),
                        input: String::new(),
                        just_opened: true,
                    },
                    dir,
                ));
            }
            SidebarAction::Rename(p, _is_dir) => {
                let current = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                self.name_prompt = Some((
                    NamePromptState {
                        kind: NamePromptKind::Rename,
                        context_label: format!("renaming {current}"),
                        input: current,
                        just_opened: true,
                    },
                    p,
                ));
            }
            SidebarAction::Delete(p, is_dir) => {
                self.delete_confirm = Some((p, is_dir));
            }
            SidebarAction::Reveal(p) => self.reveal_in_finder(&p),
            SidebarAction::CopyPath(p) => {
                let s = p.display().to_string();
                self.copy_to_clipboard(ctx, s);
            }
        }
    }

    fn workspace_relative_display(&self, p: &Path) -> String {
        if let Some(ws) = &self.workspace {
            if let Ok(rel) = p.strip_prefix(&ws.root) {
                return format!("{}/{}", ws.display_name(), rel.display());
            }
        }
        p.display().to_string()
    }

    fn apply_name_prompt(&mut self, name: String) {
        let Some((state, target)) = self.name_prompt.take() else {
            return;
        };
        match state.kind {
            NamePromptKind::NewFile => {
                let new_path = match crate::fs_ops::resolve_under(&target, &name) {
                    Ok(p) => p,
                    Err(e) => {
                        self.flash(format!("Invalid name: {}", e.message()));
                        return;
                    }
                };
                if new_path.exists() {
                    self.flash("A file with that name already exists");
                    return;
                }
                if let Err(err) = std::fs::write(&new_path, b"") {
                    self.flash(format!("Create failed: {err}"));
                    return;
                }
                self.invalidate_after_fs_change(&target);
                self.open_file(&new_path);
                self.flash(format!("Created {name}"));
            }
            NamePromptKind::NewFolder => {
                let new_path = match crate::fs_ops::resolve_under(&target, &name) {
                    Ok(p) => p,
                    Err(e) => {
                        self.flash(format!("Invalid name: {}", e.message()));
                        return;
                    }
                };
                if new_path.exists() {
                    self.flash("A folder with that name already exists");
                    return;
                }
                if let Err(err) = std::fs::create_dir(&new_path) {
                    self.flash(format!("Create failed: {err}"));
                    return;
                }
                self.invalidate_after_fs_change(&target);
                self.flash(format!("Created folder {name}"));
            }
            NamePromptKind::Rename => {
                let new_path = match crate::fs_ops::rename_target(&target, &name) {
                    Ok(p) => p,
                    Err(e) => {
                        self.flash(format!("Invalid name: {}", e.message()));
                        return;
                    }
                };
                if new_path == target {
                    return; // No-op rename.
                }
                if new_path.exists() {
                    self.flash("A file with that name already exists");
                    return;
                }
                if let Err(err) = std::fs::rename(&target, &new_path) {
                    self.flash(format!("Rename failed: {err}"));
                    return;
                }
                // Rewrite the path on any open tab pointing at the old location.
                for tab in self.tabs.iter_mut() {
                    if tab.path == target {
                        tab.path = new_path.clone();
                        tab.display_name = new_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("untitled")
                            .to_string();
                        tab.syntax_name = crate::editor::language::syntax_for_path(&new_path).name.clone();
                    }
                }
                if let Some(parent) = target.parent() {
                    self.invalidate_after_fs_change(parent);
                }
                self.flash(format!("Renamed to {name}"));
            }
        }
    }

    fn perform_delete(&mut self) {
        let Some((path, is_dir)) = self.delete_confirm.take() else {
            return;
        };
        // Best-effort move to trash. Fall back to fs::remove_dir_all /
        // remove_file if the user doesn't have `trash` available.
        let result = if is_dir {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(err) = result {
            self.flash(format!("Delete failed: {err}"));
            return;
        }
        // Close any open tab that pointed at the removed file.
        let mut to_close: Vec<usize> = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.path == path || tab.path.starts_with(&path) {
                to_close.push(i);
            }
        }
        for i in to_close.into_iter().rev() {
            self.force_close_tab(i);
        }
        if let Some(parent) = path.parent() {
            self.invalidate_after_fs_change(parent);
        }
        self.flash(format!(
            "Deleted {}",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("")
        ));
    }

    fn invalidate_after_fs_change(&mut self, dir: &Path) {
        if let Some(ws) = self.workspace.as_mut() {
            ws.tree.invalidate_containing(dir);
            ws.invalidate_index();
            ws.refresh_git_status();
        }
    }

    fn spawn_update_check(&mut self) {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let current = env!("CARGO_PKG_VERSION").to_string();
        std::thread::spawn(move || {
            let result =
                crate::updater::check(crate::updater::REPO, &current);
            let payload = match result {
                Ok(info) => info,
                Err(err) => {
                    tracing::debug!(error = %err, "update check failed");
                    None
                }
            };
            let _ = tx.send(payload);
        });
        self.update_rx = Some(rx);
    }

    fn drain_update_check(&mut self) {
        let Some(rx) = self.update_rx.as_ref() else {
            return;
        };
        if let Ok(payload) = rx.try_recv() {
            if let Some(info) = payload {
                tracing::info!(
                    current = %info.current,
                    latest = %info.latest,
                    "update available"
                );
                self.pending_update = Some(info);
            }
            self.update_rx = None;
        }
    }

    fn new_untitled(&mut self) {
        let tab = EditorTab::new_untitled(self.next_untitled_n);
        self.next_untitled_n += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.tab_mru.touch(self.active_tab);
        self.saver.mark_dirty();
        self.flash("New buffer — use Save As (⇧⌘S) to write to disk");
    }

    fn save_as_active(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };
        let mut dialog = rfd::FileDialog::new().set_file_name(&tab.display_name);
        // Default to the workspace root when one's open so the picker
        // starts somewhere useful.
        if let Some(ws) = &self.workspace {
            dialog = dialog.set_directory(&ws.root);
        }
        let Some(path) = dialog.save_file() else {
            return;
        };

        // Apply on-save normalisation before writing — same as the
        // normal Save path.
        let trim = self.trim_whitespace_on_save;
        let final_nl = self.ensure_final_newline_on_save;
        let Some(tab) = self.tabs.get_mut(self.active_tab) else {
            return;
        };
        tab.apply_save_normalization(trim, final_nl);
        match tab.save_as(&path) {
            Ok(_) => {
                let name = tab.display_name.clone();
                tracing::info!(file = %path.display(), "saved as");
                if let Some(store) = &self.recovery_store {
                    let _ = store.clear(&path);
                }
                tab.last_recovered_hash = None;
                self.recent_files.push(path.clone());
                if let Some(ws) = self.workspace.as_mut() {
                    ws.refresh_git_status();
                    ws.invalidate_index();
                }
                self.flash(format!("Saved {name}"));
            }
            Err(err) => {
                tracing::warn!(error = %err, "save_as failed");
                self.flash(format!("Save As failed: {err}"));
            }
        }
    }

    fn select_next_occurrence(&mut self, ctx: &Context) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };
        let editor_id = egui::Id::new(("ide_editor", tab.path.as_path()));
        let Some(mut state) =
            egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
        else {
            return;
        };
        let range = state.cursor.char_range();
        let buf = &tab.buffer.text;
        // Determine the needle: existing selection text, else the word at cursor.
        let (needle, start_char) = match range {
            Some(r) if r.primary.index != r.secondary.index => {
                let (a, b) = if r.primary.index <= r.secondary.index {
                    (r.primary.index, r.secondary.index)
                } else {
                    (r.secondary.index, r.primary.index)
                };
                let s: String = buf.chars().skip(a).take(b - a).collect();
                (s, b)
            }
            _ => {
                let caret = range
                    .map(|r| r.primary.index)
                    .unwrap_or(0);
                let Some(w) = crate::select_next::word_at(buf, caret) else {
                    self.flash("No word at cursor");
                    return;
                };
                let word: String = buf.chars().skip(w.start).take(w.end - w.start).collect();
                (word, w.start)
            }
        };
        // Find next occurrence (case-sensitive by convention; Cmd+D on
        // VS Code is case-sensitive too).
        let Some(hit) = crate::select_next::find_next(buf, start_char, &needle, true) else {
            self.flash("No more occurrences");
            return;
        };
        // Select the match. Scroll-to-me happens because TextEdit moves
        // the cursor on the next frame.
        use egui::text::{CCursor, CCursorRange};
        state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(hit.start),
            CCursor::new(hit.end),
        )));
        state.store(ctx, editor_id);
        ctx.request_repaint();
    }

    fn apply_indent(&mut self, ctx: &Context, dedent: bool) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else {
            return;
        };
        let editor_id = egui::Id::new(("ide_editor", tab.path.as_path()));
        let selection = egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
            .and_then(|s| s.cursor.char_range())
            .map(|r| (r.primary.index, r.secondary.index))
            .unwrap_or((0, 0));
        let unit_string = self.indent_style.as_string();
        let unit = unit_string.as_str();
        let result = if dedent {
            crate::indent::dedent(&tab.buffer.text, selection, unit)
        } else {
            crate::indent::indent(&tab.buffer.text, selection, unit)
        };
        tab.buffer.text = result.new_text;
        if let Some(mut state) =
            egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
        {
            use egui::text::{CCursor, CCursorRange};
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(result.new_range.0),
                CCursor::new(result.new_range.1),
            )));
            state.store(ctx, editor_id);
        }
    }

    fn toggle_line_comment(&mut self, ctx: &Context) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else {
            return;
        };
        let syntax = tab.syntax();
        let Some(token) = crate::comment::line_comment_token(syntax) else {
            self.flash(format!(
                "No line comment for {}",
                crate::editor::language::language_label(syntax)
            ));
            return;
        };

        // Pull the current caret range from the TextEdit (char indices).
        let editor_id = egui::Id::new(("ide_editor", tab.path.as_path()));
        let selection = egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
            .and_then(|s| s.cursor.char_range())
            .map(|r| (r.primary.index, r.secondary.index))
            .unwrap_or((0, 0));

        let result =
            crate::comment::toggle_line_comment(&tab.buffer.text, selection, token);
        tab.buffer.text = result.new_text;

        // Restore selection so the user can keep toggling without losing place.
        if let Some(mut state) =
            egui::widgets::text_edit::TextEditState::load(ctx, editor_id)
        {
            use egui::text::{CCursor, CCursorRange};
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(result.new_range.0),
                CCursor::new(result.new_range.1),
            )));
            state.store(ctx, editor_id);
        }
    }

    fn open_project_search(&mut self) {
        if self.workspace.is_none() {
            self.flash("Open a folder first (⇧⌘O)");
            return;
        }
        self.project_search.open();
    }

    fn execute_replace_all_in_project(&mut self) {
        let Some(outcome) = self.project_search.outcome.clone() else {
            return;
        };
        if outcome.hits.is_empty() {
            return;
        }
        let query = self.project_search.query.clone();
        let replacement = self.project_search.replacement.clone();
        let options = self.project_search.options;

        let mut files_changed = 0usize;
        let mut replacements_done = 0usize;
        let mut errors = 0usize;

        for file_hits in outcome.hits.iter() {
            let path = file_hits.path.clone();
            // Prefer the in-memory buffer if a tab is open — that way unsaved
            // edits don't get clobbered.
            let source_text =
                if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
                    self.tabs[idx].buffer.text.clone()
                } else {
                    match std::fs::read_to_string(&path) {
                        Ok(t) => t,
                        Err(err) => {
                            tracing::warn!(error = %err, path = %path.display(), "read for replace failed");
                            errors += 1;
                            continue;
                        }
                    }
                };

            let (new_text, count) =
                match crate::find::replace_all(&source_text, &query, &replacement, options) {
                    Ok(r) => r,
                    Err(err) => {
                        tracing::warn!(error = ?err, path = %path.display(), "replace failed");
                        errors += 1;
                        continue;
                    }
                };
            if count == 0 {
                continue;
            }
            // Write through.
            if let Err(err) = std::fs::write(&path, new_text.as_bytes()) {
                tracing::warn!(error = %err, path = %path.display(), "write for replace failed");
                errors += 1;
                continue;
            }
            files_changed += 1;
            replacements_done += count;

            // Update any open tab on this file to match the new contents.
            if let Some(idx) = self.tabs.iter_mut().position(|t| t.path == path) {
                self.tabs[idx].buffer.text = new_text;
                self.tabs[idx].buffer.mark_clean();
                self.tabs[idx].external_change = false;
                self.tabs[idx].last_recovered_hash = None;
                if let Some(store) = &self.recovery_store {
                    let _ = store.clear(&path);
                }
            }
        }

        // Invalidate the workspace's git status + file index so the sidebar
        // reflects the new modified-list and the next search is fresh.
        if let Some(ws) = self.workspace.as_mut() {
            ws.refresh_git_status();
            ws.invalidate_index();
        }
        // Drop the stale outcome — the user can run search again to see what's left.
        self.project_search.outcome = None;
        self.project_search.dirty = true;

        let msg = if errors > 0 {
            format!(
                "Replaced {replacements_done} match(es) in {files_changed} file(s); {errors} failed"
            )
        } else {
            format!("Replaced {replacements_done} match(es) in {files_changed} file(s)")
        };
        tracing::info!(
            replacements = replacements_done,
            files = files_changed,
            errors,
            "project replace complete"
        );
        self.flash(msg);
    }

    fn run_project_search(&mut self) {
        let Some(ws) = self.workspace.as_mut() else {
            return;
        };
        ws.ensure_index();
        let Some(index) = ws.file_index.as_ref() else {
            return;
        };
        let outcome = crate::project_search::search_workspace(
            index,
            &self.project_search.query,
            self.project_search.options,
        );
        tracing::info!(
            query = %self.project_search.query,
            matches = outcome.total_matches,
            files = outcome.hits.len(),
            scanned = outcome.files_scanned,
            skipped = outcome.files_skipped,
            truncated = outcome.truncated,
            "project search complete"
        );
        self.project_search.outcome = Some(outcome);
        self.project_search.dirty = false;
    }

    fn create_and_open(&mut self, path: &Path) {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    self.flash(format!("Create dir failed: {err}"));
                    return;
                }
            }
        }
        if let Err(err) = std::fs::write(path, b"") {
            self.flash(format!("Create file failed: {err}"));
            return;
        }
        if let Some(parent) = path.parent() {
            self.invalidate_after_fs_change(parent);
        }
        self.open_file(path);
        self.flash(format!(
            "Created {}",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("")
        ));
    }

    fn open_finder(&mut self) {
        let Some(ws) = self.workspace.as_mut() else {
            self.flash("Open a folder first (⇧⌘O)");
            return;
        };
        ws.ensure_index();
        let ws_root = ws.root.clone();
        if let Some(index) = ws.file_index.as_ref() {
            self.finder.open();
            self.finder
                .refresh(index, &self.recent_files.entries, Some(&ws_root));
        }
    }

    fn apply_goto(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if let Ok(line) = self.goto_input.trim().parse::<usize>() {
                if line >= 1 {
                    let char_idx = char_index_at_line_start(&tab.buffer.text, line);
                    self.pending_goto_char = Some(char_idx);
                    self.goto_open = false;
                }
            }
        }
    }

    fn refresh_find_if_needed(&mut self) {
        if !self.find.open {
            return;
        }
        let Some(tab) = self.tabs.get(self.active_tab) else {
            self.find.matches.clear();
            return;
        };
        let signature = find_signature(
            self.active_tab,
            &tab.buffer.text,
            &self.find.query,
            self.find.options,
        );
        if self.last_find_signature != Some(signature) {
            self.find.refresh(&tab.buffer.text);
            self.last_find_signature = Some(signature);
        }
    }

    fn apply_find_action(&mut self, action: FindAction) {
        match action {
            FindAction::None => {}
            FindAction::Next => self.find.next(),
            FindAction::Prev => self.find.prev(),
            FindAction::Close => self.find.close(),
            FindAction::ReplaceCurrent => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Some(range) = self.find.matches.get(self.find.current).cloned() {
                        let (new_text, _) = crate::find::replace_one(
                            &tab.buffer.text,
                            range,
                            &self.find.replacement,
                        );
                        tab.buffer.text = new_text;
                        self.last_find_signature = None;
                    }
                }
            }
            FindAction::ReplaceAll => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    match do_replace_all(
                        &tab.buffer.text,
                        &self.find.query,
                        &self.find.replacement,
                        self.find.options,
                    ) {
                        Ok((new_text, count)) => {
                            tab.buffer.text = new_text;
                            self.flash(format!("Replaced {count} occurrence(s)"));
                            self.last_find_signature = None;
                        }
                        Err(err) => {
                            self.flash(format!("Replace failed: {err:?}"));
                        }
                    }
                }
            }
        }
    }

    fn window_title(&self) -> String {
        let ws = self
            .workspace
            .as_ref()
            .map(|w| w.display_name())
            .unwrap_or_default();
        let tab = self.tabs.get(self.active_tab);
        let dot = tab.filter(|t| t.is_dirty()).map(|_| "● ").unwrap_or("");
        let file = tab.map(|t| t.display_name.as_str()).unwrap_or("");
        match (file.is_empty(), ws.is_empty()) {
            (true, true) => "IdeUltra".to_string(),
            (true, false) => format!("{ws} — IdeUltra"),
            (false, true) => format!("{dot}{file} — IdeUltra"),
            (false, false) => format!("{dot}{file} — {ws} — IdeUltra"),
        }
    }
}

fn display_recent(path: &Path) -> String {
    // Show ~/foo/bar.rs instead of /Users/you/foo/bar.rs when possible.
    if let Some(home) = directories::UserDirs::new() {
        if let Ok(rel) = path.strip_prefix(home.home_dir()) {
            return format!("~/{}", rel.display());
        }
    }
    path.display().to_string()
}

fn crosses_newline(text: &str, selection: (usize, usize)) -> bool {
    let (a, b) = if selection.0 <= selection.1 {
        selection
    } else {
        (selection.1, selection.0)
    };
    if a == b {
        return false;
    }
    text.chars().skip(a).take(b - a).any(|c| c == '\n')
}

fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md" | "markdown" | "mdx" | "mkd")
    )
}

/// Tabs we'd want a word count for in the status bar: markdown plus
/// other prose-shaped extensions.
fn is_prose(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md" | "markdown" | "mdx" | "mkd" | "txt" | "rst" | "adoc")
    )
}

fn find_signature(
    active_tab: usize,
    text: &str,
    query: &str,
    options: crate::find::FindOptions,
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    active_tab.hash(&mut h);
    text.hash(&mut h);
    query.hash(&mut h);
    options.case_sensitive.hash(&mut h);
    options.whole_word.hash(&mut h);
    options.regex.hash(&mut h);
    h.finish()
}

impl eframe::App for IdeUltraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.first_frame_logged {
            let elapsed = self.started_at.elapsed();
            tracing::info!(elapsed_ms = elapsed.as_millis() as u64, "first frame");
            self.first_frame_logged = true;
        }

        self.handle_text_input_autopair(ctx);
        self.handle_shortcuts(ctx);
        self.apply_theme(ctx);
        self.read_window_state(ctx);
        self.handle_focus_autosave(ctx);
        self.drain_watcher();
        self.drain_update_check();
        self.write_recoveries();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        // ── menu bar ─────────────────────────────────────────────────────
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New  ⌘N").clicked() {
                        ui.close_menu();
                        self.new_untitled();
                    }
                    if ui.button("Open File…  ⌘O").clicked() {
                        ui.close_menu();
                        self.open_file_dialog();
                    }
                    if ui.button("Open Folder…  ⇧⌘O").clicked() {
                        ui.close_menu();
                        self.open_folder_dialog();
                    }
                    // Open Recent submenu — top 10 recent files.
                    let mut recent_to_open: Option<PathBuf> = None;
                    ui.menu_button("Open Recent", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label(
                                egui::RichText::new("Nothing yet").small().weak(),
                            );
                        } else {
                            for p in self.recent_files.top(10) {
                                let label = display_recent(p);
                                if ui.button(label).clicked() {
                                    recent_to_open = Some(p.to_path_buf());
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui.button("Clear Recent").clicked() {
                                self.recent_files.entries.clear();
                                self.saver.mark_dirty();
                                ui.close_menu();
                            }
                        }
                    });
                    if let Some(p) = recent_to_open {
                        self.open_file(&p);
                    }
                    // Open Recent Folder submenu — top 10 workspaces.
                    let mut workspace_to_open: Option<PathBuf> = None;
                    ui.menu_button("Open Recent Folder", |ui| {
                        if self.recent_workspaces.is_empty() {
                            ui.label(
                                egui::RichText::new("Nothing yet").small().weak(),
                            );
                        } else {
                            for p in self.recent_workspaces.top(10) {
                                let label = display_recent(p);
                                if ui.button(label).clicked() {
                                    workspace_to_open = Some(p.to_path_buf());
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui.button("Clear").clicked() {
                                self.recent_workspaces.entries.clear();
                                self.saver.mark_dirty();
                                ui.close_menu();
                            }
                        }
                    });
                    if let Some(p) = workspace_to_open {
                        self.open_workspace(&p);
                    }
                    ui.separator();
                    if ui.button("Save  ⌘S").clicked() {
                        ui.close_menu();
                        self.save_active();
                    }
                    if ui.button("Save As…  ⇧⌘S").clicked() {
                        ui.close_menu();
                        self.save_as_active();
                    }
                    if ui.button("Close Tab  ⌘W").clicked() {
                        ui.close_menu();
                        self.request_close_tab(self.active_tab);
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Find  ⌘F").clicked() {
                        ui.close_menu();
                        self.find.open_find();
                    }
                    if ui.button("Find & Replace  ⌥⌘F").clicked() {
                        ui.close_menu();
                        self.find.open_replace();
                    }
                    ui.separator();
                    if ui.button("Go to File…  ⌘P").clicked() {
                        ui.close_menu();
                        self.open_finder();
                    }
                    if ui.button("Search in Project…  ⇧⌘F").clicked() {
                        ui.close_menu();
                        self.open_project_search();
                    }
                    ui.separator();
                    if ui.button("Command Palette  ⇧⌘P").clicked() {
                        ui.close_menu();
                        self.palette.open();
                    }
                    if ui.button("Go to Line…  ⌘G").clicked() {
                        ui.close_menu();
                        if !self.tabs.is_empty() {
                            self.goto_open = true;
                            self.goto_input.clear();
                        }
                    }
                    ui.separator();
                    // Keymap submenu — switch the global shortcut set.
                    let current = self.keymap_preset;
                    ui.menu_button("Keymap", |ui| {
                        for preset in KeymapPreset::all() {
                            let label = format!(
                                "{} {}",
                                if *preset == current { "●" } else { " " },
                                preset.label(),
                            );
                            if ui.button(label).clicked() {
                                ui.close_menu();
                                self.switch_keymap(*preset);
                            }
                        }
                        ui.separator();
                        if ui.button("Open Keymap Picker…").clicked() {
                            ui.close_menu();
                            self.keymap_picker_open = true;
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Toggle Sidebar  ⌘B").clicked() {
                        ui.close_menu();
                        self.sidebar_visible = !self.sidebar_visible;
                        self.saver.mark_dirty();
                    }
                    if ui.button("Toggle Markdown Preview  ⌥⌘M").clicked() {
                        ui.close_menu();
                        self.markdown_preview = !self.markdown_preview;
                        self.saver.mark_dirty();
                    }
                    if ui.button("Toggle Split Editor  ⌘\\").clicked() {
                        ui.close_menu();
                        self.toggle_split();
                    }
                    ui.separator();
                    if ui.button("Zoom In  ⌘=").clicked() {
                        self.zoom = (self.zoom + 0.1).clamp(0.5, 3.0);
                        ui.ctx().set_zoom_factor(self.zoom);
                        self.saver.mark_dirty();
                    }
                    if ui.button("Zoom Out  ⌘-").clicked() {
                        self.zoom = (self.zoom - 0.1).clamp(0.5, 3.0);
                        ui.ctx().set_zoom_factor(self.zoom);
                        self.saver.mark_dirty();
                    }
                    if ui.button("Reset Zoom  ⌘0").clicked() {
                        self.zoom = 1.0;
                        ui.ctx().set_zoom_factor(self.zoom);
                        self.saver.mark_dirty();
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Theme").small().weak());
                    for opt in [ColorTheme::Dark, ColorTheme::Light] {
                        if ui.radio(self.theme == opt, opt.label()).clicked() {
                            self.theme = opt;
                            self.saver.mark_dirty();
                            ui.close_menu();
                        }
                    }
                });
                ui.add_space(8.0);
                if let Some(ws) = &self.workspace {
                    ui.label(
                        egui::RichText::new(ws.display_name())
                            .small()
                            .weak(),
                    );
                }
            });
        });

        // ── update banner (under the menu bar) ──────────────────────────
        if self.pending_update.is_some() {
            let mut open_url: Option<String> = None;
            let mut dismiss = false;
            TopBottomPanel::top("update_banner").show(ctx, |ui| {
                let info = self.pending_update.as_ref().unwrap();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Update available: {} → {}",
                            info.current, info.latest,
                        ))
                        .color(ui.visuals().hyperlink_color)
                        .strong(),
                    );
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui.small_button("Dismiss").clicked() {
                                dismiss = true;
                            }
                            if ui.button("View release").clicked() {
                                open_url = Some(info.html_url.clone());
                            }
                        },
                    );
                });
            });
            if let Some(url) = open_url {
                crate::updater::open_in_browser(&url);
                self.pending_update = None;
            }
            if dismiss {
                self.pending_update = None;
            }
        }

        // ── status bar (bottom) ──────────────────────────────────────────
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    let dirty = if tab.is_dirty() { " · modified" } else { "" };
                    let lang = language_label(tab.syntax());
                    let ending = detect_line_ending(&tab.buffer.text).label();
                    let pos = self
                        .caret_line_col
                        .map(|(l, c)| format!("Ln {l}, Col {c}"))
                        .unwrap_or_else(|| "—".to_string());
                    let mut left = format!(
                        "{}{}  ·  {}  ·  {}  ·  UTF-8  ·  {} bytes  ·  {}",
                        tab.path.display(),
                        dirty,
                        pos,
                        ending,
                        tab.buffer.text.len(),
                        lang,
                    );
                    // For prose-y tabs, append word count + reading time.
                    if is_prose(&tab.path) {
                        let stats = crate::wordcount::analyze(&tab.buffer.text);
                        if stats.words > 0 {
                            left = format!(
                                "{left}  ·  {} words  ·  ~{} min read",
                                stats.words, stats.minutes
                            );
                        }
                    }
                    ui.label(egui::RichText::new(left).small().weak());
                } else {
                    ui.label(
                        egui::RichText::new("No file open")
                            .small()
                            .weak(),
                    );
                }
                if let Some((msg, t)) = &self.status_message {
                    if t.elapsed() < std::time::Duration::from_secs(3) {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(egui::RichText::new(msg).small());
                            },
                        );
                    }
                }
            });
        });

        if let Some((_, t)) = self.status_message {
            if t.elapsed() >= std::time::Duration::from_secs(3) {
                self.status_message = None;
            }
        }

        // ── sidebar (file tree OR project search) ───────────────────────
        let mut file_to_open: Option<PathBuf> = None;
        let mut project_jump: Option<(PathBuf, std::ops::Range<usize>)> = None;
        let mut run_search = false;
        let mut sidebar_request: Option<SidebarAction> = None;
        if self.sidebar_visible || self.project_search.open {
            if let Some(ws) = self.workspace.as_mut() {
                SidePanel::left("sidebar")
                    .resizable(true)
                    .default_width(self.sidebar_width)
                    .min_width(220.0)
                    .max_width(600.0)
                    .show(ctx, |ui| {
                        self.sidebar_width = ui.available_width();
                        if self.project_search.open {
                            let action =
                                project_search_panel::show(ui, &mut self.project_search);
                            match action {
                                ProjectSearchAction::None => {}
                                ProjectSearchAction::Run => run_search = true,
                                ProjectSearchAction::OpenAt { path, byte_range } => {
                                    project_jump = Some((path, byte_range));
                                }
                                ProjectSearchAction::ReplaceAll => {
                                    self.replace_confirm_open = true;
                                }
                            }
                        } else {
                            let action = sidebar::show(
                                ui,
                                &mut ws.tree,
                                ws.git_status.as_ref(),
                            );
                            match action {
                                SidebarAction::None => {}
                                SidebarAction::OpenFile(path) => {
                                    file_to_open = Some(path);
                                }
                                SidebarAction::NewFileIn(dir) => {
                                    sidebar_request = Some(SidebarAction::NewFileIn(dir));
                                }
                                SidebarAction::NewFolderIn(dir) => {
                                    sidebar_request = Some(SidebarAction::NewFolderIn(dir));
                                }
                                SidebarAction::Rename(p, is_dir) => {
                                    sidebar_request = Some(SidebarAction::Rename(p, is_dir));
                                }
                                SidebarAction::Delete(p, is_dir) => {
                                    sidebar_request = Some(SidebarAction::Delete(p, is_dir));
                                }
                                SidebarAction::Reveal(p) => {
                                    sidebar_request = Some(SidebarAction::Reveal(p));
                                }
                                SidebarAction::CopyPath(p) => {
                                    sidebar_request = Some(SidebarAction::CopyPath(p));
                                }
                            }
                        }
                    });
            }
        }
        if run_search {
            self.run_project_search();
        }
        if let Some(path) = file_to_open {
            self.open_file(&path);
        }
        if let Some(req) = sidebar_request {
            self.handle_sidebar_request(ctx, req);
        }
        // Pull the project-search jump out of the borrow scope before opening.
        let project_jump_pending = project_jump.map(|(p, r)| {
            self.open_file(&p);
            r
        });

        // Keep matches fresh before rendering the bar (so the count reflects
        // the current buffer & query).
        self.refresh_find_if_needed();

        // ── central panel: tabs + find bar + editor ──────────────────────
        let theme = self.theme;
        let self_soft_wrap = self.soft_wrap;
        CentralPanel::default().show(ctx, |ui| {
            if self.tabs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 60.0);
                    ui.heading("IdeUltra");
                    ui.add_space(8.0);
                    if self.workspace.is_none() {
                        ui.label("A snappy, native, local-first code IDE.");
                        ui.add_space(16.0);
                        if ui.button("Open Folder…").clicked() {
                            self.open_folder_dialog();
                        }
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("⌘O open file  ·  ⇧⌘O open folder  ·  ⌘F find")
                                .small()
                                .weak(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Pick a file in the sidebar →")
                                .small()
                                .weak(),
                        );
                    }
                });
                return;
            }

            // Tabs strip. When split, render TWO strips side-by-side so
            // each pane can have its own active tab. Clicking in either
            // strip both focuses that pane AND activates the clicked tab.
            if let Some(p2) = self.pane2_active {
                let (left_idx, right_idx) = if self.focused_right {
                    (p2, self.active_tab)
                } else {
                    (self.active_tab, p2)
                };
                let mut acts: [TabAction; 2] = [TabAction::None, TabAction::None];
                ui.columns(2, |cols| {
                    acts[0] = tabs::show(&mut cols[0], &self.tabs, left_idx);
                    acts[1] = tabs::show(&mut cols[1], &self.tabs, right_idx);
                });
                ui.separator();
                for (col_i, act) in acts.into_iter().enumerate() {
                    let want_right = col_i == 1;
                    match act {
                        TabAction::Activate(i) => {
                            self.focus_pane(want_right, i);
                            self.saver.mark_dirty();
                        }
                        TabAction::Close(i) => self.request_close_tab(i),
                        TabAction::CloseOthers(i) => self.close_others(i),
                        TabAction::CloseAll => self.close_all_tabs(),
                        TabAction::CopyPath(p) => {
                            let s = p.display().to_string();
                            self.copy_to_clipboard(ctx, s);
                        }
                        TabAction::RevealInFinder(p) => self.reveal_in_finder(&p),
                        TabAction::None => {}
                    }
                }
            } else {
                let tab_action = tabs::show(ui, &self.tabs, self.active_tab);
                ui.separator();
                match tab_action {
                    TabAction::Activate(i) => {
                        self.focus_pane(false, i);
                        self.saver.mark_dirty();
                    }
                    TabAction::Close(i) => self.request_close_tab(i),
                    TabAction::CloseOthers(i) => self.close_others(i),
                    TabAction::CloseAll => self.close_all_tabs(),
                    TabAction::CopyPath(p) => {
                        let s = p.display().to_string();
                        self.copy_to_clipboard(ctx, s);
                    }
                    TabAction::RevealInFinder(p) => self.reveal_in_finder(&p),
                    TabAction::None => {}
                }
            }

            // Find bar
            if self.find.open {
                let action = find_bar::show(ui, &mut self.find);
                self.apply_find_action(action);
            }

            // External-change banner (only for the active tab)
            let mut banner_reload = false;
            let mut banner_dismiss = false;
            let mut banner_view_diff = false;
            if let Some(tab) = self.tabs.get(self.active_tab) {
                if tab.external_change {
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(255, 220, 120))
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(
                                        "This file changed on disk while you had unsaved edits.",
                                    )
                                    .color(egui::Color32::BLACK),
                                );
                                if ui.button("View diff").clicked() {
                                    banner_view_diff = true;
                                }
                                if ui.button("Reload from disk").clicked() {
                                    banner_reload = true;
                                }
                                if ui.button("Keep mine").clicked() {
                                    banner_dismiss = true;
                                }
                            });
                        });
                }
            }
            if banner_view_diff {
                self.open_diff_for_active_tab();
            }
            if banner_reload {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    if let Err(err) = tab.reload_from_disk() {
                        self.flash(format!("Reload failed: {err}"));
                    }
                }
            }
            if banner_dismiss {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.external_change = false;
                }
            }

            // Markdown preview side-pane (only for .md tabs, only when enabled).
            let show_preview = self.markdown_preview
                && self
                    .tabs
                    .get(self.active_tab)
                    .map(|t| is_markdown(&t.path))
                    .unwrap_or(false);
            if show_preview {
                let accent = ui.visuals().hyperlink_color;
                let source = self
                    .tabs
                    .get(self.active_tab)
                    .map(|t| t.buffer.text.clone())
                    .unwrap_or_default();
                egui::SidePanel::right("markdown_preview")
                    .resizable(true)
                    .default_width(440.0)
                    .min_width(260.0)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Preview").strong());
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("✕").on_hover_text("Hide preview").clicked() {
                                        self.markdown_preview = false;
                                        self.saver.mark_dirty();
                                    }
                                },
                            );
                        });
                        ui.separator();
                        crate::ui::markdown_preview::show(ui, &source, accent);
                    });
            }

            // Editor (one pane normally, two side-by-side when split).
            let jump: Option<Jump> = if let Some(r) = project_jump_pending {
                Some(Jump::ByteRange(r))
            } else if self.find.scroll_pending {
                self.find.scroll_pending = false;
                self.find
                    .matches
                    .get(self.find.current)
                    .cloned()
                    .map(Jump::ByteRange)
            } else if let Some(c) = self.pending_goto_char.take() {
                Some(Jump::CharIndex(c))
            } else {
                None
            };

            if let Some(p2) = self.pane2_active {
                let active = self.active_tab;
                // Which side renders which pane depends on focused_right.
                let (left_idx, right_idx) = if self.focused_right {
                    (p2, active)
                } else {
                    (active, p2)
                };
                // Apply `jump` only to the FOCUSED pane.
                let (left_jump, right_jump) = if self.focused_right {
                    (None, jump)
                } else {
                    (jump, None)
                };
                let theme_l = theme;
                let theme_r = theme;
                let wrap = self_soft_wrap;
                let focused_right = self.focused_right;

                let mut clicked_left = false;
                let mut clicked_right = false;
                let mut left_caret: Option<usize> = None;
                let mut right_caret: Option<usize> = None;

                ui.columns(2, |cols| {
                    // Left pane
                    let ui_l = &mut cols[0];
                    let resp_l = ui_l.label(
                        egui::RichText::new(if focused_right { "" } else { "▎ focused" })
                            .small()
                            .weak(),
                    );
                    if resp_l.clicked() {
                        clicked_left = true;
                    }
                    if let Some(tab) = self.tabs.get_mut(left_idx) {
                        let res = editor_panel::show(ui_l, tab, theme_l, left_jump, wrap);
                        left_caret = res.caret_char_index;
                    }

                    // Right pane
                    let ui_r = &mut cols[1];
                    let resp_r = ui_r.label(
                        egui::RichText::new(if focused_right { "▎ focused" } else { "" })
                            .small()
                            .weak(),
                    );
                    if resp_r.clicked() {
                        clicked_right = true;
                    }
                    if let Some(tab) = self.tabs.get_mut(right_idx) {
                        let res = editor_panel::show(ui_r, tab, theme_r, right_jump, wrap);
                        right_caret = res.caret_char_index;
                    }
                });

                if clicked_left && self.focused_right {
                    self.focus_pane(false, left_idx);
                }
                if clicked_right && !self.focused_right {
                    self.focus_pane(true, right_idx);
                }
                // Caret position for the status bar comes from the focused pane.
                let focused_caret = if self.focused_right { right_caret } else { left_caret };
                let focused_idx = self.active_tab;
                if let Some(tab) = self.tabs.get(focused_idx) {
                    self.caret_line_col =
                        focused_caret.map(|ci| line_col_at_char(&tab.buffer.text, ci));
                }
            } else {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let res = editor_panel::show(ui, tab, theme, jump, self_soft_wrap);
                    self.caret_line_col = res
                        .caret_char_index
                        .map(|ci| line_col_at_char(&tab.buffer.text, ci));
                }
            }
        });

        // After the TextEdit has applied the pending `()` insertion,
        // walk the caret back one position so it sits between the pair.
        self.apply_autopair_backstep(ctx);

        // Quick switcher overlay (rendered last so it sits on top).
        if self.quick_switcher.open {
            quick_switcher::show(
                ctx,
                &self.tabs,
                &self.tab_mru,
                self.quick_switcher.selected,
            );
        }

        // Preferences window (⌘,).
        if self.preferences_open {
            let state_dir = persistence::PersistencePaths::from_project_dirs()
                .and_then(|p| p.settings.parent().map(|q| q.to_path_buf()));
            let view = PreferencesView {
                theme: self.theme,
                zoom: self.zoom,
                autosave: self.autosave_on_focus_loss,
                markdown_preview: self.markdown_preview,
                keymap: self.keymap_preset,
                state_dir: state_dir.as_deref(),
                trim_whitespace: self.trim_whitespace_on_save,
                ensure_final_newline: self.ensure_final_newline_on_save,
                indent_style: self.indent_style,
                soft_wrap: self.soft_wrap,
            };
            match preferences_window::show(ctx, &view) {
                PreferencesAction::None => {}
                PreferencesAction::Close => self.preferences_open = false,
                PreferencesAction::SetTheme(t) => {
                    self.theme = t;
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetZoom(z) => {
                    self.zoom = z.clamp(0.5, 3.0);
                    ctx.set_zoom_factor(self.zoom);
                    self.saver.mark_dirty();
                }
                PreferencesAction::ResetZoom => {
                    self.zoom = 1.0;
                    ctx.set_zoom_factor(self.zoom);
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetAutosaveOnFocusLoss(b) => {
                    self.autosave_on_focus_loss = b;
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetMarkdownPreview(b) => {
                    self.markdown_preview = b;
                    self.saver.mark_dirty();
                }
                PreferencesAction::OpenKeymapPicker => {
                    self.keymap_picker_open = true;
                }
                PreferencesAction::SwitchKeymap(p) => {
                    self.switch_keymap(p);
                }
                PreferencesAction::SetTrimWhitespace(b) => {
                    self.trim_whitespace_on_save = b;
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetEnsureFinalNewline(b) => {
                    self.ensure_final_newline_on_save = b;
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetIndentStyle(s) => {
                    self.indent_style = s;
                    self.saver.mark_dirty();
                }
                PreferencesAction::SetSoftWrap(b) => {
                    self.soft_wrap = b;
                    self.saver.mark_dirty();
                }
            }
        }

        // ── name prompt (sidebar: New File / Folder / Rename) ───────────
        if self.name_prompt.is_some() {
            let action = {
                let (state, _) = self.name_prompt.as_mut().unwrap();
                name_prompt_modal::show(ctx, state)
            };
            match action {
                NamePromptAction::None => {}
                NamePromptAction::Cancel => self.name_prompt = None,
                NamePromptAction::Confirm(name) => self.apply_name_prompt(name),
            }
        }

        // ── delete-confirm modal ────────────────────────────────────────
        if let Some((path, is_dir)) = self.delete_confirm.as_ref() {
            let target = path.clone();
            let is_dir = *is_dir;
            match delete_confirm_modal::show(ctx, &target, is_dir) {
                DeleteAction::None => {}
                DeleteAction::Cancel => self.delete_confirm = None,
                DeleteAction::Confirm => self.perform_delete(),
            }
        }

        // ── replace-in-project confirmation modal ───────────────────────
        if self.replace_confirm_open {
            let outcome = self.project_search.outcome.as_ref();
            let file_count = outcome.map(|o| o.hits.len()).unwrap_or(0);
            let match_count = outcome.map(|o| o.total_matches).unwrap_or(0);
            let query = self.project_search.query.clone();
            let replacement = self.project_search.replacement.clone();
            match replace_confirm_modal::show(
                ctx,
                file_count,
                match_count,
                &query,
                &replacement,
            ) {
                ReplaceConfirmAction::None => {}
                ReplaceConfirmAction::Cancel => self.replace_confirm_open = false,
                ReplaceConfirmAction::Confirm => {
                    self.replace_confirm_open = false;
                    self.execute_replace_all_in_project();
                }
            }
        }

        // ── diff modal (triggered from external-change banner) ──────────
        if let Some((tab_idx, summary, file_name)) = self.diff_modal.as_ref() {
            let tab_idx = *tab_idx;
            let action = diff_modal::show(ctx, file_name, summary);
            match action {
                DiffModalAction::None => {}
                DiffModalAction::Close => self.diff_modal = None,
                DiffModalAction::Reload => {
                    self.diff_modal = None;
                    if let Some(tab) = self.tabs.get_mut(tab_idx) {
                        if let Err(err) = tab.reload_from_disk() {
                            self.flash(format!("Reload failed: {err}"));
                        }
                    }
                }
                DiffModalAction::KeepMine => {
                    self.diff_modal = None;
                    if let Some(tab) = self.tabs.get_mut(tab_idx) {
                        tab.external_change = false;
                    }
                }
            }
        }

        // ── recovery modal (startup) ────────────────────────────────────
        if !self.pending_recoveries.is_empty() {
            let action = recovery_modal::show(ctx, &self.pending_recoveries);
            if !matches!(action, RecoveryAction::None) {
                self.apply_recovery_action(action);
            }
        }

        // ── dirty-close confirm modal ───────────────────────────────────
        if let Some(&idx) = self.pending_close.first() {
            if idx >= self.tabs.len() {
                self.pending_close.remove(0);
            } else {
                let name = self.tabs[idx].display_name.clone();
                let count = self.pending_close.len();
                let action = close_confirm_modal::show(ctx, &name, count);
                match action {
                    CloseConfirmAction::None => {}
                    CloseConfirmAction::Save => {
                        let prev_active = self.active_tab;
                        self.active_tab = idx;
                        self.save_active();
                        // If save succeeded the tab is now clean; close it.
                        if !self
                            .tabs
                            .get(idx)
                            .map(|t| t.is_dirty())
                            .unwrap_or(false)
                        {
                            self.force_close_tab(idx);
                            self.pending_close.remove(0);
                            // Fix up other pending indices: anything > idx
                            // shifts down by 1.
                            for p in &mut self.pending_close {
                                if *p > idx {
                                    *p -= 1;
                                }
                            }
                        }
                        // Restore active focus where possible.
                        if prev_active < self.tabs.len() {
                            self.active_tab = prev_active;
                        }
                    }
                    CloseConfirmAction::Discard => {
                        self.force_close_tab(idx);
                        self.pending_close.remove(0);
                        for p in &mut self.pending_close {
                            if *p > idx {
                                *p -= 1;
                            }
                        }
                    }
                    CloseConfirmAction::Cancel => {
                        self.pending_close.clear();
                    }
                }
            }
        }

        // ── keymap picker (first-run + on-demand) ───────────────────────
        if self.keymap_picker_open {
            let action =
                keymap_picker::show(ctx, self.keymap_preset, !self.keymap_chosen);
            match action {
                KeymapPickerAction::None => {}
                KeymapPickerAction::Choose(preset) => {
                    self.switch_keymap(preset);
                    self.keymap_picker_open = false;
                }
            }
        }

        // ── command palette (⇧⌘P) ───────────────────────────────────────
        if self.palette.open {
            let action = command_palette_modal::show(ctx, &mut self.palette, &self.keymap);
            match action {
                CommandPaletteAction::None => {}
                CommandPaletteAction::Close => self.palette.close(),
                CommandPaletteAction::Run(id) => {
                    self.palette.close();
                    self.dispatch_command(ctx, id);
                }
            }
        }

        // ── fuzzy file finder modal (⌘P) ────────────────────────────────
        if self.finder.open {
            let recent = self.recent_files.entries.clone();
            let ws_root = self.workspace.as_ref().map(|w| w.root.clone());
            let action = if let Some(ws) = self.workspace.as_ref() {
                if let Some(index) = ws.file_index.as_ref() {
                    self.finder.refresh(index, &recent, ws_root.as_deref());
                    finder_modal::show(ctx, &mut self.finder, index)
                } else {
                    FinderAction::Close
                }
            } else {
                FinderAction::Close
            };
            match action {
                FinderAction::None => {}
                FinderAction::Close => self.finder.close(),
                FinderAction::Open(path) => {
                    self.finder.close();
                    self.open_file(&path);
                }
                FinderAction::Create(path) => {
                    self.finder.close();
                    self.create_and_open(&path);
                }
            }
        }

        // ── go-to-line modal ─────────────────────────────────────────────
        if self.goto_open {
            let mut submitted = false;
            let mut cancelled = false;
            egui::Window::new("Go to Line")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
                .show(ctx, |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.goto_input)
                            .hint_text("Line number")
                            .desired_width(160.0),
                    );
                    resp.request_focus();
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        submitted = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Go").clicked() {
                            submitted = true;
                        }
                        if ui.button("Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            cancelled = true;
                        }
                    });
                });
            if submitted {
                self.apply_goto();
            } else if cancelled {
                self.goto_open = false;
            }
        }

        // ── debounced persistence save ───────────────────────────────────
        // Cheap when not dirty; one fs write at most every SAVE_DEBOUNCE.
        let settings = self.current_settings();
        let session = self.current_session();
        self.saver.maybe_save(&settings, &session, SAVE_DEBOUNCE);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Called by eframe periodically and on shutdown. Use this as a
        // belt-and-braces flush so nothing is lost between debounces.
        let settings = self.current_settings();
        let session = self.current_session();
        if let Err(err) = self.saver.save_now(&settings, &session) {
            tracing::warn!(error = %err, "save_now failed");
        }
    }
}

