pub struct ScrollEngine {
    pub current_y: f32,
    pub target_slot: f32,
    pub speed: f32,
}

impl ScrollEngine {
    pub fn new(speed: f32) -> Self {
        Self {
            current_y: 0.0,
            target_slot: 0.0,
            speed,
        }
    }

    pub fn update(&mut self, row_height: f32, dt: f32) -> bool {
        let target_y = self.target_slot * row_height;
        let diff = target_y - self.current_y;

        if diff.abs() < 0.1 {
            if self.current_y != target_y {
                self.current_y = target_y;
                return true;
            }
            false
        } else {
            let factor = 1.0 - (-self.speed * dt).exp();
            self.current_y += diff * factor;
            true
        }
    }

    pub fn scroll_discrete(&mut self, steps: f32, max_slots: f32) {
        self.target_slot = (self.target_slot + steps).clamp(0.0, max_slots);
    }
}
