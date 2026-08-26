use super::RenderHomeContext;
use super::album_grid::render_album_grid;
use egui::Ui;

pub fn render_libraries_view<F: FnMut(String)>(
    ui: &mut Ui,
    ctx: &mut RenderHomeContext<'_, F>,
) {
    render_album_grid(
        ui,
        ctx.ctrl,
        ctx.prewarmer,
        ctx.config,
        ctx.palette,
        ctx.albums,
        ctx.on_focus,
    );
}
