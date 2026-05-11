#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod autopair;
mod command_palette;
mod comment;
mod diff;
mod editor;
mod find;
mod finder;
mod git;
mod indent;
mod keymap;
mod logging;
mod markdown;
mod mru;
mod persistence;
mod project_search;
mod recent;
mod recovery;
mod style;
mod transforms;
mod ui;
mod wordcount;
mod workspace;

use anyhow::Result;
use app::IdeUltraApp;

fn main() -> Result<()> {
    let _guard = logging::init();

    let loaded = persistence::load();
    let size = loaded.session.window.size;
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("IdeUltra")
        .with_inner_size(size)
        .with_min_inner_size([640.0, 400.0]);
    if let Some(pos) = loaded.session.window.pos {
        viewport = viewport.with_position(pos);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "IdeUltra",
        native_options,
        Box::new(move |cc| Ok(Box::new(IdeUltraApp::new(cc, loaded.clone())))),
    )
    .map_err(|e| anyhow::anyhow!("eframe failed: {e}"))?;

    Ok(())
}
