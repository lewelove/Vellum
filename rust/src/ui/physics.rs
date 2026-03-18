#[derive(Default)]
pub struct PhysicsEngine {
    pub current_y: f64,
    pub target_y: f64,
    pub velocity: f64,
    
    pub container_width: u32,
    pub container_height: u32,
    pub cols: u32,
    pub row_height: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub card_size: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            current_y: 0.0,
            target_y: 0.0,
            velocity: 0.0,
            container_width: 0,
            container_height: 0,
            cols: 1,
            row_height: 260.0,
            gap_x: 30.0,
            gap_y: 16.0,
            card_size: 190.0,
        }
    }

    pub fn update_layout(&mut self, width: u32, height: u32) {
        self.container_width = width;
        self.container_height = height;
        
        let available_width = width as f32 - 40.0;
        self.cols = ((available_width + self.gap_x) / (self.card_size + self.gap_x)).floor() as u32;
        self.cols = self.cols.max(1);
    }

    pub fn scroll(&mut self, delta: f64) {
        self.target_y = (self.target_y + delta).max(0.0);
    }

    pub fn tick(&mut self) {
        let stiffness = 0.15;
        let damping = 0.82;
        
        let force = (self.target_y - self.current_y) * stiffness;
        self.velocity = (self.velocity + force) * damping;
        self.current_y += self.velocity;

        if (self.target_y - self.current_y).abs() < 0.01 && self.velocity.abs() < 0.01 {
            self.current_y = self.target_y;
            self.velocity = 0.0;
        }
    }

    pub fn get_item_pos(&self, index: usize) -> [f32; 2] {
        let row = index / self.cols as usize;
        let col = index % self.cols as usize;
        
        let grid_width = (self.cols as f32 * self.card_size) + ((self.cols - 1) as f32 * self.gap_x);
        let x_offset = (self.container_width as f32 - grid_width) / 2.0;
        
        let x = x_offset + (col as f32 * (self.card_size + self.gap_x));
        let y = self.gap_y + (row as f32 * self.row_height);
        
        [x, y]
    }

    pub fn get_visible_range(&self, total_items: usize) -> (usize, usize) {
        let start_row = (self.current_y / self.row_height as f64).floor() as isize - 1;
        let end_row = ((self.current_y + self.container_height as f64) / self.row_height as f64).ceil() as isize + 1;
        
        let start = (start_row.max(0) as usize * self.cols as usize).min(total_items);
        let end = (end_row.max(0) as usize * self.cols as usize).min(total_items);
        
        (start, end)
    }
}
