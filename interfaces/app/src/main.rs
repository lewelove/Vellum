mod api;
mod app;
mod colors;
mod config;
mod library;
mod modules;
mod navigation;

use app::DaleApp;
use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("Dale"),
        ..Default::default()
    };

    eframe::run_native(
        "dale-app",
        native_options,
        Box::new(|cc| Ok(Box::new(DaleApp::new(cc)))),
    )
}
