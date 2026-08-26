use crate::colors::Palette;
use crate::library::collection::{CollectionStore, GroupNode};
use crate::library::sync::SyncEngine;
use crate::library::view::{ScrollReset, ViewState};
use egui::{
    Align, Button, Color32, ComboBox, CornerRadius, FontId, Layout, Pos2, Rect, Sense, Ui,
    Vec2,
};

fn render_tree_node(
    ui: &mut Ui,
    node: &GroupNode,
    view: &mut ViewState,
    sync: &SyncEngine,
    palette: &Palette,
    depth: usize,
) {
    let is_active = view.active_filter_val.as_deref() == Some(&node.value);
    let depth_u16 = u16::try_from(depth).unwrap_or(0);
    let indent = f32::from(depth_u16) * 12.0;

    let text_color = if is_active {
        palette.ok100
    } else {
        palette.ok100.gamma_multiply(0.7)
    };

    let bg_color = if is_active {
        palette.ok400
    } else {
        Color32::TRANSPARENT
    };

    let row_height = 28.0;
    let (row_rect, row_response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), row_height),
        Sense::click(),
    );

    if row_response.clicked() {
        if let Some(ref grouper) = view.active_sidebar_grouper {
            view.apply_filter(grouper.clone(), node.value.clone(), sync);
        }
    }

    if is_active || row_response.hovered() {
        let fill = if is_active {
            bg_color
        } else {
            palette.ok400.gamma_multiply(0.3)
        };
        ui.painter()
            .rect_filled(row_rect, CornerRadius::same(6), fill);
    }

    let label_pos = Pos2::new(row_rect.min.x + indent + 8.0, row_rect.min.y + 6.0);
    let font_label = FontId::proportional(13.0);
    ui.painter().text(
        label_pos,
        egui::Align2::LEFT_TOP,
        &node.label,
        font_label,
        text_color,
    );

    if let Some(ref sub) = node.sublabel {
        let sub_color = palette.ok100.gamma_multiply(0.4);
        let font_sub = FontId::proportional(11.0);
        let sub_pos = Pos2::new(row_rect.max.x - 8.0, row_rect.min.y + 7.0);
        ui.painter().text(
            sub_pos,
            egui::Align2::RIGHT_TOP,
            sub,
            font_sub,
            sub_color,
        );
    }

    for child in &node.children {
        render_tree_node(ui, child, view, sync, palette, depth.saturating_add(1));
    }
}

pub fn render_sidebar(
    ui: &mut Ui,
    view: &mut ViewState,
    collection: &CollectionStore,
    sync: &SyncEngine,
    palette: &Palette,
) {
    let side_rect = ui.max_rect();
    let resizer_rect = Rect::from_min_size(side_rect.min, Vec2::new(6.0, side_rect.height()));
    let resizer = ui.interact(resizer_rect, ui.id().with("resizer"), Sense::click_and_drag());
    if resizer.dragged() {
        let delta_x = resizer.drag_delta().x;
        view.sidebar_width = (view.sidebar_width - delta_x).clamp(160.0, 480.0);
    }

    ui.vertical(|ui| {
        ui.add_space(8.0);

        let libs = collection.get_libraries();
        if !libs.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let current_label = libs
                    .iter()
                    .find(|(k, _)| k == &view.active_library)
                    .map_or("Library", |(_, l)| l.as_str());

                ComboBox::from_id_salt("library_select")
                    .selected_text(current_label)
                    .width(ui.available_width() - 16.0)
                    .show_ui(ui, |ui| {
                        for (k, l) in libs {
                            if ui.selectable_value(&mut view.active_library, k, l).clicked() {
                                view.refresh_view(sync, ScrollReset::Reset);
                                view.refresh_sidebar(sync);
                            }
                        }
                    });
            });
        }

        ui.add_space(6.0);

        let groupers = collection.get_groupers();
        if !groupers.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let current_g_label = view
                    .active_sidebar_grouper
                    .as_ref()
                    .and_then(|g_key| groupers.iter().find(|(k, _)| k == g_key))
                    .map_or("Group By", |(_, l)| l.as_str());

                ComboBox::from_id_salt("grouper_select")
                    .selected_text(current_g_label)
                    .width(ui.available_width() - 16.0)
                    .show_ui(ui, |ui| {
                        for (k, l) in groupers {
                            let mut selected = view.active_sidebar_grouper.as_deref() == Some(&k);
                            if ui.selectable_value(&mut selected, true, l).clicked() {
                                view.active_sidebar_grouper = Some(k);
                                view.active_filter_key = None;
                                view.active_filter_val = None;
                                view.refresh_sidebar(sync);
                                view.refresh_view(sync, ScrollReset::Reset);
                            }
                        }
                    });
            });
        }

        ui.add_space(6.0);

        let orders = collection.get_orders();
        if !orders.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let current_o_label = view
                    .active_sort_key
                    .as_ref()
                    .and_then(|o_key| orders.iter().find(|(k, _)| k == o_key))
                    .map_or("Order", |(_, l)| l.as_str());

                ComboBox::from_id_salt("order_select")
                    .selected_text(current_o_label)
                    .width(ui.available_width() - 48.0)
                    .show_ui(ui, |ui| {
                        for (k, l) in orders {
                            let mut selected = view.active_sort_key.as_deref() == Some(&k);
                            if ui.selectable_value(&mut selected, true, l).clicked() {
                                view.active_sort_key = Some(k);
                                view.refresh_view(sync, ScrollReset::Reset);
                            }
                        }
                    });

                let dir_btn = Button::new(if view.is_reverse { "v" } else { "^" })
                    .min_size(Vec2::new(28.0, 24.0))
                    .corner_radius(CornerRadius::same(6));

                if ui.add(dir_btn).clicked() {
                    view.is_reverse = !view.is_reverse;
                    view.refresh_view(sync, ScrollReset::Reset);
                }
            });
        }

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    if let Some(ref g_key) = view.active_sidebar_grouper {
                        if let Some(nodes) = collection.sidebar_groups.get(g_key) {
                            for node in nodes {
                                render_tree_node(ui, node, view, sync, palette, 0);
                            }
                        }
                    }
                });
            });
    });
}
