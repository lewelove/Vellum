use crate::server::state::AppState;
use eframe::egui;
use std::sync::Arc;

use super::{album_grid, navigation, queue, sidebar, theme};

#[derive(PartialEq)]
pub enum ActiveTab {
    Home,
    Queue,
}

pub struct VellumApp {
    state: Arc<AppState>,
    active_tab: ActiveTab,
    album_grid: album_grid::GridController,
}

impl VellumApp {
    pub fn new(state: Arc<AppState>) -> Self {
        let damping = 0.18;
        Self {
            state: Arc::clone(&state),
            active_tab: ActiveTab::Home,
            album_grid: album_grid::GridController::new(damping),
        }
    }
}

impl eframe::App for VellumApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().options_mut(|opt| {
            opt.tessellation_options.round_text_to_pixels = false;
        });

        let mut style = (*ui.ctx().global_style()).clone();
        style.visuals.window_fill = theme::BG_MAIN;
        style.visuals.panel_fill = theme::BG_MAIN;
        ui.ctx().set_global_style(style);

        egui::Panel::left("nav_bar")
            .exact_size(64.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_DRAWER)
                    .inner_margin(12.0),
            )
            .show_inside(ui, |ui| {
                navigation::render(ui, &mut self.active_tab);
            });

        if self.active_tab == ActiveTab::Home {
            egui::Panel::right("sidebar")
                .resizable(true)
                .default_size(200.0)
                .size_range(140.0..=400.0)
                .frame(egui::Frame::NONE.fill(theme::BG_DRAWER))
                .show_inside(ui, |ui| {
                    sidebar::render(ui, &self.state);
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::BG_MAIN))
            .show_inside(ui, |ui| match self.active_tab {
                ActiveTab::Home => self.album_grid.render(ui, &self.state),
                ActiveTab::Queue => queue::render(ui, &self.state),
            });
    }
}
