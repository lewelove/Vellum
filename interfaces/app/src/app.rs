use crate::colors::Palette;
use crate::config::AppConfig;
use crate::library::collection::CollectionStore;
use crate::library::prewarmer::Prewarmer;
use crate::library::sync::SyncEngine;
use crate::library::view::{HomeSubView, ScrollReset, ViewState};
use crate::modules::home_view::album_grid::grid_controller::GridController;
use crate::modules::home_view::sidebar::render_sidebar;
use crate::modules::home_view::{RenderHomeContext, render_home_view};
use crate::modules::navigation_bar::render_navigation_bar;
use crate::navigation::NavigationTab;
use eframe::App;
use egui::{CornerRadius, Pos2, Rect, Ui, UiBuilder, Vec2};
use std::sync::mpsc::Receiver;

pub struct DaleApp {
    sync: SyncEngine,
    collection: CollectionStore,
    view: ViewState,
    prewarmer: Prewarmer,
    config: AppConfig,
    palette: Palette,
    grid_ctrl: GridController,
    active_tab: NavigationTab,
    last_reset_version: u64,
    config_rx: Receiver<serde_json::Value>,
}

impl DaleApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let sync = SyncEngine::start();
        let (config_tx, config_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build configuration loader runtime");

            rt.block_on(async move {
                if let Ok(cfg) = crate::api::fetch_interface_config("default").await {
                    let _ = config_tx.send(cfg);
                }
            });
        });

        Self {
            sync,
            collection: CollectionStore::default(),
            view: ViewState::default(),
            prewarmer: Prewarmer::new(),
            config: AppConfig::default(),
            palette: Palette::default(),
            grid_ctrl: GridController::default(),
            active_tab: NavigationTab::Home,
            last_reset_version: 0,
            config_rx,
        }
    }

    fn apply_loaded_config(&mut self, cfg_json: &serde_json::Value) {
        if let Some(pal) = cfg_json.get("palette") {
            if let Some(s) = pal.get("ok100").and_then(serde_json::Value::as_str)
                && let Some(c) = crate::colors::parse_oklch(s)
            {
                self.palette.ok100 = c;
            }
            if let Some(s) = pal.get("ok200").and_then(serde_json::Value::as_str)
                && let Some(c) = crate::colors::parse_oklch(s)
            {
                self.palette.ok200 = c;
            }
            if let Some(s) = pal.get("ok300").and_then(serde_json::Value::as_str)
                && let Some(c) = crate::colors::parse_oklch(s)
            {
                self.palette.ok300 = c;
            }
            if let Some(s) = pal.get("ok400").and_then(serde_json::Value::as_str)
                && let Some(c) = crate::colors::parse_oklch(s)
            {
                self.palette.ok400 = c;
            }
            if let Some(s) = pal.get("ok500").and_then(serde_json::Value::as_str)
                && let Some(c) = crate::colors::parse_oklch(s)
            {
                self.palette.ok500 = c;
            }
        }
    }
}

impl App for DaleApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Ok(cfg_json) = self.config_rx.try_recv() {
            self.apply_loaded_config(&cfg_json);
        }

        let messages = self.sync.poll_messages();
        for msg in messages {
            self.view
                .handle_inbound_message(&msg, &mut self.collection, &self.sync);
        }

        let albums = self
            .collection
            .map_ids_to_albums(&self.collection.library_view_ids);

        self.prewarmer.prewarm_all(
            &albums,
            &self.config.album_grid.album_card.cover.filter,
            self.config.album_grid.album_card.cover.size,
        );

        self.prewarmer.update(ui.ctx());

        if self.view.reset_version != self.last_reset_version {
            self.grid_ctrl.reset_scroll();
            self.grid_ctrl.text_cache.clear();
            self.last_reset_version = self.view.reset_version;
        }

        ui.input(|i| {
            if i.key_pressed(egui::Key::Num1) || i.key_pressed(egui::Key::H) {
                self.active_tab = NavigationTab::Home;
                self.view.home_sub_view = HomeSubView::Libraries;
                self.view.refresh_view(&self.sync, ScrollReset::Reset);
            }
            if i.key_pressed(egui::Key::Num2) || i.key_pressed(egui::Key::Q) {
                self.active_tab = NavigationTab::Queue;
            }
            if i.key_pressed(egui::Key::S) {
                self.active_tab = NavigationTab::Home;
                self.view.home_sub_view = HomeSubView::Cabinets;
                self.view.refresh_view(&self.sync, ScrollReset::Reset);
            }
        });

        let max_rect = ui.max_rect();
        let full_width = max_rect.width();
        let full_height = max_rect.height();

        let nav_width = 48.0_f32;
        let side_width = self.view.sidebar_width.clamp(160.0, 480.0);
        let grid_width = (full_width - nav_width - side_width).max(100.0);

        let nav_rect = Rect::from_min_size(max_rect.min, Vec2::new(nav_width, full_height));
        let grid_rect = Rect::from_min_size(
            Pos2::new(max_rect.min.x + nav_width, max_rect.min.y),
            Vec2::new(grid_width, full_height),
        );
        let side_rect = Rect::from_min_size(
            Pos2::new(max_rect.min.x + nav_width + grid_width, max_rect.min.y),
            Vec2::new(side_width, full_height),
        );

        let mut nav_ui = ui.new_child(UiBuilder::new().max_rect(nav_rect));
        nav_ui
            .painter()
            .rect_filled(nav_rect, CornerRadius::ZERO, self.palette.ok300);
        render_navigation_bar(&mut nav_ui, &mut self.active_tab, &self.palette);

        let mut grid_ui = ui.new_child(UiBuilder::new().max_rect(grid_rect));
        grid_ui
            .painter()
            .rect_filled(grid_rect, CornerRadius::ZERO, self.palette.ok200);
        match self.active_tab {
            NavigationTab::Home => {
                let mut focused = self.view.focused_album_id.clone();
                let mut on_focus = |id: String| {
                    focused = Some(id);
                };

                let mut home_ctx = RenderHomeContext {
                    sub_view: self.view.home_sub_view,
                    ctrl: &mut self.grid_ctrl,
                    prewarmer: &mut self.prewarmer,
                    config: &self.config,
                    palette: &self.palette,
                    albums: &albums,
                    on_focus: &mut on_focus,
                };

                render_home_view(&mut grid_ui, &mut home_ctx);

                self.view.focused_album_id = focused;
            }
            NavigationTab::Queue => {}
        }

        let mut side_ui = ui.new_child(UiBuilder::new().max_rect(side_rect));
        side_ui
            .painter()
            .rect_filled(side_rect, CornerRadius::ZERO, self.palette.ok300);
        render_sidebar(
            &mut side_ui,
            &mut self.view,
            &self.collection,
            &self.sync,
            &self.palette,
        );

        let target_y =
            self.grid_ctrl.scroll.target_slot * self.grid_ctrl.layout.row_height();
        if (target_y - self.grid_ctrl.scroll.current_y).abs() > 0.01 {
            ui.ctx().request_repaint();
        }
    }
}
