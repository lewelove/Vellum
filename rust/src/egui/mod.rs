pub mod home_view;
pub mod logic;
pub mod app;
pub mod navigation;
pub mod queue;
pub mod theme;
pub mod text;

use crate::server::state::AppState;
use anyhow::{anyhow, Result};
use eframe::egui;
use std::sync::Arc;

pub fn run(state: Arc<AppState>) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Vellum"),
        ..Default::default()
    };

    eframe::run_native(
        "Vellum",
        options,
        Box::new(|_cc| {
            Ok(Box::new(app::VellumApp::new(state)))
        }),
    )
    .map_err(|e| anyhow!("eframe error: {:?}", e))
}
