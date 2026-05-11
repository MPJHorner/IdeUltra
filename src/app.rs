use eframe::CreationContext;
use egui::{CentralPanel, Context};

pub struct IdeUltraApp {
    started_at: std::time::Instant,
    first_frame_logged: bool,
}

impl IdeUltraApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            started_at: std::time::Instant::now(),
            first_frame_logged: false,
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

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 40.0);
                ui.heading("IdeUltra");
                ui.label("A snappy, native, local-first code IDE.");
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("v0.1.0 — Deliverable D1: empty window")
                        .small()
                        .weak(),
                );
            });
        });
    }
}
