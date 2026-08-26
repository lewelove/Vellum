pub mod album_grid;
pub mod cabinets_view;
pub mod libraries_view;
pub mod sidebar;

use self::album_grid::grid_controller::GridController;
use self::cabinets_view::render_cabinets_view;
use self::libraries_view::render_libraries_view;
use crate::colors::Palette;
use crate::config::AppConfig;
use crate::library::collection::AlbumSummary;
use crate::library::prewarmer::Prewarmer;
use crate::library::view::HomeSubView;
use egui::Ui;

pub struct RenderHomeContext<'a, F> {
    pub sub_view: HomeSubView,
    pub ctrl: &'a mut GridController,
    pub prewarmer: &'a mut Prewarmer,
    pub config: &'a AppConfig,
    pub palette: &'a Palette,
    pub albums: &'a [AlbumSummary],
    pub on_focus: &'a mut F,
}

pub fn render_home_view<F: FnMut(String)>(
    ui: &mut Ui,
    ctx: &mut RenderHomeContext<'_, F>,
) {
    match ctx.sub_view {
        HomeSubView::Libraries => {
            render_libraries_view(ui, ctx);
        }
        HomeSubView::Cabinets => {
            render_cabinets_view(ui, ctx);
        }
    }
}
