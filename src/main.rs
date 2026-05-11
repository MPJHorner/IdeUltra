#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod editor;
mod find;
mod logging;
mod ui;
mod workspace;

use anyhow::Result;
use app::IdeUltraApp;

fn main() -> Result<()> {
    let _guard = logging::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("IdeUltra")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([640.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "IdeUltra",
        native_options,
        Box::new(|cc| Ok(Box::new(IdeUltraApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe failed: {e}"))?;

    Ok(())
}
