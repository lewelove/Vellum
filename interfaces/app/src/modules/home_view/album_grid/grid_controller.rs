use super::layout::LayoutManager;
use super::scroll::ScrollEngine;
use crate::library::collection::AlbumSummary;
use std::collections::HashMap;

pub struct VirtualRow {
    pub y: f32,
    pub data: Vec<AlbumSummary>,
}

#[derive(Default)]
pub struct GridController {
    pub layout: LayoutManager,
    pub scroll: ScrollEngine,
    pub viewport_height: f32,
    pub albums: Vec<AlbumSummary>,
    pub text_cache: HashMap<String, (String, String)>,
}

impl GridController {
    #[must_use]
    pub fn all_rows(&self) -> Vec<Vec<AlbumSummary>> {
        self.layout.chunk(&self.albums)
    }

    #[must_use]
    pub fn visible_rows_count(&self) -> usize {
        let rh = self.layout.row_height();
        if rh <= 0.0 || self.viewport_height <= 0.0 {
            return 1;
        }

        let mut count = 1_usize;
        for i in 1..10_000_usize {
            let idx_u16 = u16::try_from(i).unwrap_or(u16::MAX);
            if f32::from(idx_u16) * rh < self.viewport_height {
                count = i.saturating_add(1);
            } else {
                break;
            }
        }
        count.max(1)
    }

    #[must_use]
    pub fn max_slots(&self) -> f32 {
        let total_rows = self.all_rows().len();
        let visible = self.visible_rows_count();
        let diff = (total_rows.saturating_add(1)).saturating_sub(visible);
        let diff_u16 = u16::try_from(diff).unwrap_or(u16::MAX);
        f32::from(diff_u16)
    }

    #[must_use]
    pub fn virtual_rows(&self) -> Vec<VirtualRow> {
        let rows = self.all_rows();
        if rows.is_empty() {
            return Vec::new();
        }

        let (start, end) = self.layout.get_visible_indices(
            self.scroll.current_y,
            self.viewport_height,
            rows.len(),
        );

        let mut result = Vec::with_capacity(end.saturating_sub(start) + 1);
        for i in start..=end {
            if let Some(row_data) = rows.get(i) {
                result.push(VirtualRow {
                    y: self.layout.get_row_y(i),
                    data: row_data.clone(),
                });
            }
        }
        result
    }

    pub fn update(&mut self, dpr: f32) {
        let max = self.max_slots();
        self.scroll.target_slot = self.scroll.target_slot.clamp(0.0, max);
        self.scroll.update(self.layout.row_height(), dpr);
    }

    pub fn scroll_step(&mut self, direction: f32) {
        let max = self.max_slots();
        self.scroll.scroll_step(direction, max);
    }

    pub fn scroll_row(&mut self, delta: f32) {
        let max = self.max_slots();
        self.scroll.scroll_row(delta, max);
    }

    pub const fn reset_scroll(&mut self) {
        self.scroll.sync_to_slot(0.0);
        self.scroll.current_y = 0.0;
    }
}
