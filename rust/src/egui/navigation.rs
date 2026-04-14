use super::app::ActiveTab;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, active_tab: &mut ActiveTab) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);

        let nav_btn = |ui: &mut egui::Ui, is_active: bool, uri: &'static str, bytes: &'static [u8]| {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click());
            
            let bg_color = if is_active {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
            } else if response.hovered() {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 2)
            };

            let stroke_color = if is_active {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
            } else if response.hovered() {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            };

            ui.painter().rect(
                rect,
                10.0,
                bg_color,
                egui::Stroke::new(2.0, stroke_color),
                egui::StrokeKind::Inside,
            );

            let tint = if is_active || response.hovered() {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_white_alpha(77)
            };

            let image = egui::Image::from_bytes(uri, bytes)
                .fit_to_exact_size(egui::vec2(24.0, 24.0))
                .tint(tint);
            
            let img_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(24.0, 24.0));
            image.paint_at(ui, img_rect);

            response
        };

        if nav_btn(
            ui, 
            *active_tab == ActiveTab::Home, 
            "bytes://house.svg", 
            include_bytes!("../../public/icons/24px/house.svg")
        ).clicked() {
            *active_tab = ActiveTab::Home;
        }

        ui.add_space(12.0);

        if nav_btn(
            ui, 
            *active_tab == ActiveTab::Queue, 
            "bytes://queue_music.svg", 
            include_bytes!("../../public/icons/24px/queue_music.svg")
        ).clicked() {
            *active_tab = ActiveTab::Queue;
        }
    });
}
