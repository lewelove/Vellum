use crate::server::state::AppState;
use eframe::egui;
use std::sync::Arc;
use super::{home_view, navigation, queue, theme};

#[derive(PartialEq)]
pub enum ActiveTab {
    Home,
    Queue,
}

pub struct VellumApp {
    state: Arc<AppState>,
    active_tab: ActiveTab,
    home_view: home_view::HomeViewController,
}

impl VellumApp {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: Arc::clone(&state),
            active_tab: ActiveTab::Home,
            home_view: home_view::HomeViewController::new(),
        }
    }
}

impl eframe::App for VellumApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut style = (*ui.ctx().global_style()).clone();
        style.visuals.window_fill = theme::BG_MAIN;
        ui.ctx().set_global_style(style);

        egui::Panel::left("nav_bar")
            .exact_size(64.0)
            .frame(egui::Frame::NONE.fill(theme::BG_DRAWER).inner_margin(12.0))
            .show_inside(ui, |ui| {
                navigation::render(ui, &mut self.active_tab);
            });

        match self.active_tab {
            ActiveTab::Home => self.home_view.render(ui, &self.state),
            ActiveTab::Queue => {
                egui::CentralPanel::default().show_inside(ui, |ui| queue::render(ui, &self.state));
            }
        }
    }
}
