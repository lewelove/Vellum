use crate::server::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub fn render(ui: &mut egui::Ui, state: &Arc<AppState>) {
    ui.add_space(12.0);

    let nav_button = |ui: &mut egui::Ui, text: &str| {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
        
        if response.hovered() {
            ui.painter().rect_filled(rect, 8.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 13));
        }

        ui.painter().text(
            rect.right_center() - egui::vec2(12.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            text,
            egui::FontId::proportional(14.0),
            super::theme::TEXT_MAIN,
        );

        response
    };

    ui.vertical(|ui| {
        ui.set_width(ui.available_width());
        
        if nav_button(ui, "Media Library").clicked() {}
        ui.add_space(4.0);
        if nav_button(ui, "Recently Added").clicked() {}
    });

    ui.add_space(12.0);
    ui.painter().hline(
        ui.min_rect().left()..=(ui.min_rect().right() - 24.0),
        ui.cursor().top(),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 13)),
    );
    ui.add_space(12.0);

    let control_button = |ui: &mut egui::Ui, text: &str, uri: &'static str, icon_bytes: &'static [u8]| {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width() - 24.0, 36.0), egui::Sense::click());
        
        let bg_color = if response.hovered() {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 13)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 2)
        };

        let stroke_color = if response.hovered() {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
        };

        ui.painter().rect(
            rect,
            8.0,
            bg_color,
            egui::Stroke::new(2.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        let icon_img = egui::Image::from_bytes(uri, icon_bytes)
            .fit_to_exact_size(egui::vec2(20.0, 20.0))
            .tint(super::theme::TEXT_MUTED);

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            egui::vec2(20.0, 20.0)
        );
        icon_img.paint_at(ui, icon_rect);

        ui.painter().text(
            egui::pos2(rect.left() + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(14.0),
            super::theme::TEXT_MUTED,
        );
        
        response
    };

    ui.horizontal(|ui| {
        control_button(ui, "Default", "bytes://swap_vert.svg", include_bytes!("../../public/icons/20px/swap_vert.svg"));
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        control_button(ui, "Genre", "bytes://stack.svg", include_bytes!("../../public/icons/20px/stack.svg"));
    });

    ui.add_space(12.0);
    ui.painter().hline(
        ui.min_rect().left()..=(ui.min_rect().right() - 24.0),
        ui.cursor().top(),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 13)),
    );
    ui.add_space(12.0);

    if let Ok(lib) = state.library.try_read() {
        ui.label(
            egui::RichText::new(format!("Total Albums: {}", lib.albums.len()))
                .color(super::theme::TEXT_MUTED)
                .size(13.0)
        );
    }
}
