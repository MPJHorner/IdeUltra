use std::path::{Path, PathBuf};

use eframe::CreationContext;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};

use std::time::Duration;

use crate::editor::language::{language_label, ColorTheme};
use crate::editor::position::{char_index_at_line_start, detect_line_ending, line_col_at_char};
use crate::editor::EditorTab;
use crate::find::replace_all as do_replace_all;
use crate::persistence::{Loaded, Saver, SessionState, Settings, WindowState};
use crate::ui::editor_panel::Jump;
use crate::ui::find_bar::{self, FindAction, FindState};
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
        };

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
        }
    }

    fn current_session(&self) -> SessionState {
        SessionState {
            window: self.window_state.clone(),
            last_folder: self.workspace.as_ref().map(|w| w.root.clone()),
            open_tabs: self.tabs.iter().map(|t| t.path.clone()).collect(),
            active_tab: self.active_tab,
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
        for ev in &events {
            if let Some(parent) = ev.path.parent() {
                affected_dirs.insert(parent.to_path_buf());
            }
            // Also invalidate the path itself if it's a known directory.
            affected_dirs.insert(ev.path.clone());
        }
        for d in affected_dirs {
            ws.tree.invalidate_containing(&d);
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
        match self.theme {
            ColorTheme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ColorTheme::Light => ctx.set_visuals(egui::Visuals::light()),
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            match Workspace::open(&path) {
                Ok(ws) => {
                    tracing::info!(root = %ws.root.display(), "workspace opened");
                    self.workspace = Some(ws);
                    self.saver.mark_dirty();
                }
                Err(err) => {
                    tracing::warn!(error = %err, "failed to open workspace");
                    self.flash(format!("Could not open folder: {err}"));
                }
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
            self.saver.mark_dirty();
            return;
        }
        match EditorTab::open(path) {
            Ok(tab) => {
                tracing::info!(file = %path.display(), "file opened");
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                self.saver.mark_dirty();
            }
            Err(err) => {
                tracing::warn!(file = %path.display(), error = %err, "open failed");
                self.flash(format!("Could not open file: {err}"));
            }
        }
    }

    fn save_active(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            match tab.save() {
                Ok(_) => {
                    let msg = format!("Saved {}", tab.display_name);
                    tracing::info!(file = %tab.path.display(), "saved");
                    self.flash(msg);
                }
                Err(err) => {
                    tracing::warn!(error = %err, "save failed");
                    self.flash(format!("Save failed: {err}"));
                }
            }
        }
    }

    fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        self.tabs.remove(idx);
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.tabs.is_empty() {
            self.active_tab = 0;
        }
        self.saver.mark_dirty();
    }

    fn flash(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        let mut open_folder = false;
        let mut open_file = false;
        let mut save = false;
        let mut close = false;
        let mut next_tab = false;
        let mut prev_tab = false;
        let mut go_to: Option<usize> = None;
        let mut find_open = false;
        let mut find_replace_open = false;
        let mut find_close = false;
        let mut toggle_sidebar = false;
        let mut zoom_in = false;
        let mut zoom_out = false;
        let mut zoom_reset = false;
        let mut open_goto = false;

        ctx.input_mut(|i| {
            use egui::{Key, KeyboardShortcut, Modifiers};
            let cmd = Modifiers::COMMAND;
            let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
            let cmd_alt = Modifiers::COMMAND | Modifiers::ALT;
            if i.consume_shortcut(&KeyboardShortcut::new(cmd_shift, Key::O)) {
                open_folder = true;
            } else if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::O)) {
                open_file = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::S)) {
                save = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::W)) {
                close = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::CloseBracket)) {
                next_tab = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::OpenBracket)) {
                prev_tab = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd_alt, Key::F)) {
                find_replace_open = true;
            } else if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::F)) {
                find_open = true;
            }
            if self.find.open && i.key_pressed(Key::Escape) {
                find_close = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::B)) {
                toggle_sidebar = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::G)) {
                open_goto = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Equals))
                || i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Plus))
            {
                zoom_in = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Minus)) {
                zoom_out = true;
            }
            if i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Num0)) {
                zoom_reset = true;
            }
            for (n, key) in [
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
                Key::Num8,
                Key::Num9,
            ]
            .iter()
            .enumerate()
            {
                if i.consume_shortcut(&KeyboardShortcut::new(cmd, *key)) {
                    go_to = Some(n);
                }
            }
        });

        if open_folder {
            self.open_folder_dialog();
        }
        if open_file {
            self.open_file_dialog();
        }
        if save {
            self.save_active();
        }
        if close {
            self.close_tab(self.active_tab);
        }
        if next_tab && !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.saver.mark_dirty();
        }
        if prev_tab && !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.saver.mark_dirty();
        }
        if let Some(n) = go_to {
            if n < self.tabs.len() {
                self.active_tab = n;
                self.saver.mark_dirty();
            }
        }
        if find_open {
            self.find.open_find();
        }
        if find_replace_open {
            self.find.open_replace();
        }
        if find_close {
            self.find.close();
        }
        if toggle_sidebar {
            self.sidebar_visible = !self.sidebar_visible;
            self.saver.mark_dirty();
        }
        if zoom_in {
            self.zoom = (self.zoom + 0.1).clamp(0.5, 3.0);
            ctx.set_zoom_factor(self.zoom);
            self.saver.mark_dirty();
        }
        if zoom_out {
            self.zoom = (self.zoom - 0.1).clamp(0.5, 3.0);
            ctx.set_zoom_factor(self.zoom);
            self.saver.mark_dirty();
        }
        if zoom_reset {
            self.zoom = 1.0;
            ctx.set_zoom_factor(self.zoom);
            self.saver.mark_dirty();
        }
        if open_goto && !self.tabs.is_empty() {
            self.goto_open = true;
            self.goto_input.clear();
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
        match self.tabs.get(self.active_tab) {
            Some(tab) => {
                let dot = if tab.is_dirty() { "● " } else { "" };
                format!("{dot}{} — IdeUltra", tab.display_name)
            }
            None => "IdeUltra".to_string(),
        }
    }
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

        self.handle_shortcuts(ctx);
        self.apply_theme(ctx);
        self.read_window_state(ctx);
        self.drain_watcher();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        // ── menu bar ─────────────────────────────────────────────────────
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File…  ⌘O").clicked() {
                        ui.close_menu();
                        self.open_file_dialog();
                    }
                    if ui.button("Open Folder…  ⇧⌘O").clicked() {
                        ui.close_menu();
                        self.open_folder_dialog();
                    }
                    ui.separator();
                    if ui.button("Save  ⌘S").clicked() {
                        ui.close_menu();
                        self.save_active();
                    }
                    if ui.button("Close Tab  ⌘W").clicked() {
                        ui.close_menu();
                        self.close_tab(self.active_tab);
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
                    if ui.button("Go to Line…  ⌘G").clicked() {
                        ui.close_menu();
                        if !self.tabs.is_empty() {
                            self.goto_open = true;
                            self.goto_input.clear();
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Toggle Sidebar  ⌘B").clicked() {
                        ui.close_menu();
                        self.sidebar_visible = !self.sidebar_visible;
                        self.saver.mark_dirty();
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
                    ui.label(
                        egui::RichText::new(format!(
                            "{}{}  ·  {}  ·  {}  ·  UTF-8  ·  {} bytes  ·  {}",
                            tab.path.display(),
                            dirty,
                            pos,
                            ending,
                            tab.buffer.text.len(),
                            lang,
                        ))
                        .small()
                        .weak(),
                    );
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

        // ── sidebar ──────────────────────────────────────────────────────
        let mut file_to_open: Option<PathBuf> = None;
        if self.sidebar_visible {
            if let Some(ws) = self.workspace.as_mut() {
                SidePanel::left("sidebar")
                    .resizable(true)
                    .default_width(self.sidebar_width)
                    .min_width(160.0)
                    .max_width(600.0)
                    .show(ctx, |ui| {
                        self.sidebar_width = ui.available_width();
                        let action = sidebar::show(ui, &mut ws.tree);
                        if let SidebarAction::OpenFile(path) = action {
                            file_to_open = Some(path);
                        }
                    });
            }
        }
        if let Some(path) = file_to_open {
            self.open_file(&path);
        }

        // Keep matches fresh before rendering the bar (so the count reflects
        // the current buffer & query).
        self.refresh_find_if_needed();

        // ── central panel: tabs + find bar + editor ──────────────────────
        let theme = self.theme;
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

            // Tabs
            let tab_action = tabs::show(ui, &self.tabs, self.active_tab);
            ui.separator();
            match tab_action {
                TabAction::Activate(i) => {
                    self.active_tab = i;
                    self.saver.mark_dirty();
                }
                TabAction::Close(i) => self.close_tab(i),
                TabAction::None => {}
            }

            // Find bar
            if self.find.open {
                let action = find_bar::show(ui, &mut self.find);
                self.apply_find_action(action);
            }

            // External-change banner (only for the active tab)
            let mut banner_reload = false;
            let mut banner_dismiss = false;
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

            // Editor
            let jump: Option<Jump> = if self.find.scroll_pending {
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
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                let res = editor_panel::show(ui, tab, theme, jump);
                self.caret_line_col = res
                    .caret_char_index
                    .map(|ci| line_col_at_char(&tab.buffer.text, ci));
            }
        });

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

