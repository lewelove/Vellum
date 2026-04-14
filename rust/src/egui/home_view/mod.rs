pub mod sidebar;
pub mod modal_drawer;
pub mod album_grid;

use crate::egui::logic::filters;
use crate::egui::logic::sorters;
use crate::server::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub struct HomeViewController {
    pub sidebar: sidebar::SidebarController,
    pub album_grid: album_grid::GridController,
}

impl HomeViewController {
    pub fn new() -> Self {
        Self {
            sidebar: sidebar::SidebarController::default(),
            album_grid: album_grid::GridController::new(0.18),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &Arc<AppState>) {
        let is_modal_open = self.album_grid.last_clicked_id.is_some();

        egui::Panel::right("sidebar")
            .resizable(true)
            .default_size(200.0)
            .size_range(140.0..=400.0)
            .frame(egui::Frame::NONE.fill(crate::egui::theme::BG_DRAWER).inner_margin(12.0))
            .show_inside(ui, |ui| {
                self.sidebar.render(ui, state);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(crate::egui::theme::BG_MAIN))
            .show_inside(ui, |ui| {
                if let Ok(lib) = state.library.try_read() {
                    let mut albums: Vec<&_> = lib.albums.iter()
                        .filter(|a| {
                            if let Some(ref k) = self.sidebar.filter_key {
                                filters::apply_filter(a, k, self.sidebar.filter_val.as_ref().unwrap())
                            } else { true }
                        })
                        .collect();

                    match self.sidebar.sort_key.as_str() {
                        "date_added" => albums.sort_by(|a, b| sorters::sort_date_added(a, b)),
                        _ => albums.sort_by(|a, b| sorters::sort_default(a, b)),
                    }
                    if self.sidebar.sort_reverse { albums.reverse(); }

                    self.album_grid.render(ui, state, &albums);
                }
            });

        let mut close_modal = false;
        if let Some(ref id) = self.album_grid.last_clicked_id {
            if let Ok(lib) = state.library.try_read() {
                if let Some(album) = lib.album_map.get(id) {
                    modal_drawer::render(ui.ctx(), state, album, || {
                        close_modal = true;
                    });
                }
            }
        }
        if close_modal { self.album_grid.last_clicked_id = None; }
    }
}
