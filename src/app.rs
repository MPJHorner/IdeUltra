use std::path::{Path, PathBuf};

use eframe::CreationContext;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};

use crate::editor::EditorTab;
use crate::ui::{
    editor_panel,
    sidebar::{self, SidebarAction},
    tabs::{self, TabAction},
};
use crate::workspace::Workspace;

pub struct IdeUltraApp {
    started_at: std::time::Instant,
    first_frame_logged: bool,
    workspace: Option<Workspace>,
    tabs: Vec<EditorTab>,
    active_tab: usize,
    sidebar_width: f32,
    status_message: Option<(String, std::time::Instant)>,
}

impl IdeUltraApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            started_at: std::time::Instant::now(),
            first_frame_logged: false,
            workspace: None,
            tabs: Vec::new(),
            active_tab: 0,
            sidebar_width: 260.0,
            status_message: None,
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            match Workspace::open(&path) {
                Ok(ws) => {
                    tracing::info!(root = %ws.root.display(), "workspace opened");
                    self.workspace = Some(ws);
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
        // Re-focus if already open.
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = idx;
            return;
        }
        match EditorTab::open(path) {
            Ok(tab) => {
                tracing::info!(file = %path.display(), "file opened");
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
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
        // NOTE: D3/D4 close is unconditional. The dirty-discard confirm
        // dialog is wired in D8 once we have native confirm helpers.
        self.tabs.remove(idx);
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.tabs.is_empty() {
            self.active_tab = 0;
        }
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

        ctx.input_mut(|i| {
            use egui::{Key, KeyboardShortcut, Modifiers};
            let cmd = Modifiers::COMMAND;
            let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
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
        }
        if prev_tab && !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
        if let Some(n) = go_to {
            if n < self.tabs.len() {
                self.active_tab = n;
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

impl eframe::App for IdeUltraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.first_frame_logged {
            let elapsed = self.started_at.elapsed();
            tracing::info!(elapsed_ms = elapsed.as_millis() as u64, "first frame");
            self.first_frame_logged = true;
        }

        self.handle_shortcuts(ctx);
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
                    ui.label(
                        egui::RichText::new(format!(
                            "{}{}  ·  {} bytes",
                            tab.path.display(),
                            dirty,
                            tab.buffer.text.len()
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

        // Expire the status message so the next frame redraws clean.
        if let Some((_, t)) = self.status_message {
            if t.elapsed() >= std::time::Duration::from_secs(3) {
                self.status_message = None;
            }
        }

        // ── sidebar ──────────────────────────────────────────────────────
        let mut file_to_open: Option<PathBuf> = None;
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
        if let Some(path) = file_to_open {
            self.open_file(&path);
        }

        // ── central panel: tabs + editor ─────────────────────────────────
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
                            egui::RichText::new("⌘O open file   ·   ⇧⌘O open folder")
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

            // Tab strip
            let action = tabs::show(ui, &self.tabs, self.active_tab);
            ui.separator();
            match action {
                TabAction::Activate(i) => self.active_tab = i,
                TabAction::Close(i) => self.close_tab(i),
                TabAction::None => {}
            }

            // Editor
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                editor_panel::show(ui, tab);
            }
        });
    }
}
