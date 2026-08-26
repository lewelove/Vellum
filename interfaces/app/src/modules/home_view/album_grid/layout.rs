use crate::config::AlbumGridConfig;

const PADDING_OFFSET: f32 = 40.0;

#[derive(Default)]
pub struct LayoutManager {
    pub container_width: f32,
    pub config: AlbumGridConfig,
}

impl LayoutManager {
    #[must_use]
    pub const fn gap_x(&self) -> f32 {
        self.config.spacing.x
    }

    #[must_use]
    pub const fn gap_y(&self) -> f32 {
        self.config.spacing.y
    }

    #[must_use]
    pub fn card_size(&self) -> f32 {
        let size_u16 = u16::try_from(self.config.album_card.cover.size).unwrap_or(200);
        f32::from(size_u16)
    }

    #[must_use]
    pub const fn crease_height(&self) -> f32 {
        self.config.spacing.top
    }

    #[must_use]
    pub fn row_height(&self) -> f32 {
        let text_height = if self.config.album_card.text.enable {
            let title_lh = (self.config.album_card.text.title.size * 1.2).round();
            let artist_lh = (self.config.album_card.text.albumartist.size * 1.2).round();
            self.config.album_card.text.spacing.top
                + title_lh
                + self.config.album_card.text.spacing.middle
                + artist_lh
        } else {
            0.0
        };
        self.gap_y() + self.card_size() + text_height
    }

    #[must_use]
    pub fn cols(&self) -> usize {
        let available = self.container_width - PADDING_OFFSET + self.gap_x();
        let slot = self.card_size() + self.gap_x();
        if slot <= 0.0 || available <= 0.0 {
            return 1;
        }

        let mut cols = 1_usize;
        for i in 1..100_usize {
            let count_u16 = u16::try_from(i).unwrap_or(u16::MAX);
            if f32::from(count_u16) * slot <= available {
                cols = i;
            } else {
                break;
            }
        }
        cols
    }

    #[must_use]
    pub fn grid_width(&self) -> f32 {
        let count_u16 = u16::try_from(self.cols()).unwrap_or(1);
        let count_f = f32::from(count_u16);
        (count_f - 1.0).mul_add(self.gap_x(), count_f * self.card_size())
    }

    #[must_use]
    pub fn top_offset(&self) -> f32 {
        self.crease_height() - self.gap_y()
    }

    #[must_use]
    pub fn get_row_y(&self, index: usize) -> f32 {
        let idx_u16 = u16::try_from(index).unwrap_or(u16::MAX);
        f32::from(idx_u16).mul_add(self.row_height(), self.top_offset())
    }

    #[must_use]
    pub fn get_visible_indices(
        &self,
        scroll_y: f32,
        viewport_height: f32,
        row_count: usize,
    ) -> (usize, usize) {
        if row_count == 0 {
            return (0, 0);
        }

        let rh = self.row_height();
        if rh <= 0.0 {
            return (0, 0);
        }

        let mut start_idx = 0_usize;
        for i in 0..row_count {
            let idx_u16 = u16::try_from(i).unwrap_or(u16::MAX);
            if f32::from(idx_u16).mul_add(rh, rh) <= scroll_y {
                start_idx = i.saturating_add(1);
            } else {
                break;
            }
        }
        let start = start_idx.saturating_sub(4);

        let visible_bottom = scroll_y + viewport_height;
        let mut end_idx = start_idx;
        for i in start_idx..row_count {
            let idx_u16 = u16::try_from(i).unwrap_or(u16::MAX);
            end_idx = i;
            if f32::from(idx_u16) * rh >= visible_bottom {
                break;
            }
        }
        let end = (end_idx.saturating_add(4)).min(row_count.saturating_sub(1));

        (start, end)
    }

    #[must_use]
    pub fn chunk<T: Clone>(&self, items: &[T]) -> Vec<Vec<T>> {
        let columns = self.cols();
        items.chunks(columns).map(<[T]>::to_vec).collect()
    }
}
