use crate::server::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub fn render(ui: &mut egui::Ui, _state: &Arc<AppState>) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        ui.heading("Queue");
    });
}
