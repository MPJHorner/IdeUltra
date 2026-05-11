use eframe::CreationContext;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};

use crate::ui::sidebar::{self, SidebarAction};
use crate::workspace::Workspace;

pub struct IdeUltraApp {
    started_at: std::time::Instant,
    first_frame_logged: bool,
    workspace: Option<Workspace>,
    sidebar_width: f32,
    last_opened_file: Option<std::path::PathBuf>,
}

impl IdeUltraApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            started_at: std::time::Instant::now(),
            first_frame_logged: false,
            workspace: None,
            sidebar_width: 260.0,
            last_opened_file: None,
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
                }
            }
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

        // Cmd+O shortcut for opening a folder (MVP: folder picker only; file picker comes in D3).
        let open_folder = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::O,
            ))
        });
        if open_folder {
            self.open_folder_dialog();
        }

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Folder…").clicked() {
                        ui.close_menu();
                        self.open_folder_dialog();
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
                        tracing::info!(file = %path.display(), "file clicked (open in D3)");
                        self.last_opened_file = Some(path);
                    }
                });
        }

        CentralPanel::default().show(ctx, |ui| {
            if self.workspace.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 60.0);
                    ui.heading("IdeUltra");
                    ui.add_space(8.0);
                    ui.label("A snappy, native, local-first code IDE.");
                    ui.add_space(20.0);
                    if ui.button("Open Folder…").clicked() {
                        self.open_folder_dialog();
                    }
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("⌘O  to open a folder")
                            .small()
                            .weak(),
                    );
                });
            } else if let Some(path) = &self.last_opened_file {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 40.0);
                    ui.label(
                        egui::RichText::new("Selected (editor lands in D3):")
                            .small()
                            .weak(),
                    );
                    ui.label(egui::RichText::new(path.display().to_string()).monospace());
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 20.0);
                    ui.label(
                        egui::RichText::new("Pick a file in the sidebar →")
                            .small()
                            .weak(),
                    );
                });
            }
        });
    }
}
