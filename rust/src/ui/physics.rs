#[derive(Clone, Debug)]
pub struct PhysicsEngine {
    pub current_y: f64,
    pub target_slot: f64,
    pub wheel_accumulator: f64,
    
    pub container_width: u32,
    pub container_height: u32,
    pub cols: u32,
    pub row_height: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub card_size: f32,
    pub crease_height: f32,
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self {
            current_y: 0.0,
            target_slot: 0.0,
            wheel_accumulator: 0.0,
            container_width: 0,
            container_height: 0,
            cols: 1,
            row_height: 249.0,
            gap_x: 30.0,
            gap_y: 16.0,
            card_size: 190.0,
            crease_height: 20.0,
        }
    }
}

impl PhysicsEngine {
    pub fn update_layout(&mut self, width: u32, height: u32) {
        self.container_width = width;
        self.container_height = height;
        
        let available_width = width as f32 - 40.0;
        self.cols = ((available_width + self.gap_x) / (self.card_size + self.gap_x)).floor() as u32;
        self.cols = self.cols.max(1);
    }

    pub fn scroll(&mut self, delta: f64, max_slots: f64) {
        self.wheel_accumulator += delta;
        
        if self.wheel_accumulator.abs() > 40.0 {
            let direction = if self.wheel_accumulator > 0.0 { 1.0 } else { -1.0 };
            let base = self.target_slot.round();
            self.target_slot = (base + direction).clamp(0.0, max_slots);
            self.wheel_accumulator = 0.0;
        }
    }

    pub fn tick(&mut self) {
        let damping = 0.18;
        let ideal_target_y = self.target_slot * f64::from(self.row_height);
        
        let diff = ideal_target_y - self.current_y;
        
        if diff.abs() < 0.01 {
            self.current_y = ideal_target_y;
        } else {
            self.current_y += diff * damping;
        }
    }

    pub fn get_top_offset(&self) -> f32 {
        self.crease_height - self.gap_y
    }

    pub fn get_item_pos(&self, index: usize) -> [f32; 2] {
        let row = index / self.cols as usize;
        let col = index % self.cols as usize;
        
        let grid_width = (self.cols as f32 * self.card_size) + ((self.cols - 1) as f32 * self.gap_x);
        let x_offset = (self.container_width as f32 - grid_width) / 2.0;
        
        let x = x_offset + (col as f32 * (self.card_size + self.gap_x));
        let y = self.get_top_offset() + (row as f32 * self.row_height);
        
        [x, y]
    }

    pub fn get_visible_range(&self, total_items: usize) -> (usize, usize) {
        if total_items == 0 || self.row_height <= 1.0 {
            return (0, 0);
        }

        let start_row = (self.current_y / f64::from(self.row_height)).floor() as isize;
        let visible_rows = (f64::from(self.container_height) / f64::from(self.row_height)).ceil() as isize;

        let buffer = 4;
        let start = (start_row - buffer).max(0) as usize * self.cols as usize;
        let end = (start_row + visible_rows + buffer).max(0) as usize * self.cols as usize;
        
        (start.min(total_items), end.min(total_items))
    }
}
