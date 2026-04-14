pub struct LayoutManager {
    pub container_width: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub card_size: f32,
    pub crease_height: f32,
    pub text_gap_main: f32,
    pub text_gap_lesser: f32,
    pub lh_title: f32,
    pub lh_artist: f32,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self {
            container_width: 0.0,
            gap_x: 30.0,
            gap_y: 16.0,
            card_size: 190.0,
            crease_height: 20.0,
            text_gap_main: 11.0,
            text_gap_lesser: 2.0,
            lh_title: 16.0,
            lh_artist: 14.0,
        }
    }
}

impl LayoutManager {
    pub fn row_height(&self) -> f32 {
        self.gap_y + self.card_size + self.text_gap_main + self.lh_title + self.text_gap_lesser + self.lh_artist
    }

    pub fn cols(&self) -> usize {
        ((self.container_width - 40.0 + self.gap_x) / (self.card_size + self.gap_x)).floor().max(1.0) as usize
    }

    pub fn get_row_y(&self, index: usize) -> f32 {
        (index as f32 * self.row_height()) + (self.crease_height - self.gap_y)
    }
}
