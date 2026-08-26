const DAMPING_COEFFICIENT: f32 = 0.18;
const POSITION_EPSILON: f32 = 0.01;

pub struct ScrollEngine {
    pub current_y: f32,
    pub target_slot: f32,
}

impl Default for ScrollEngine {
    fn default() -> Self {
        Self {
            current_y: 0.0,
            target_slot: 0.0,
        }
    }
}

impl ScrollEngine {
    pub fn update(&mut self, row_height: f32, dpr: f32) {
        let ideal_target_y = self.target_slot * row_height;
        let snapped_target_y = if dpr > 0.0 {
            (ideal_target_y * dpr).round() / dpr
        } else {
            ideal_target_y
        };

        let diff = snapped_target_y - self.current_y;
        let velocity = diff * DAMPING_COEFFICIENT;

        if diff.abs() < POSITION_EPSILON {
            self.current_y = snapped_target_y;
        } else {
            self.current_y += velocity;
        }
    }

    pub fn scroll_step(&mut self, direction: f32, max_slots: f32) {
        let base = self.target_slot.round();
        self.target_slot = (base + direction).clamp(0.0, max_slots);
    }

    pub fn scroll_row(&mut self, delta: f32, max_slots: f32) {
        self.target_slot = (self.target_slot + delta).clamp(0.0, max_slots);
    }

    pub const fn sync_to_slot(&mut self, slot: f32) {
        self.target_slot = slot;
    }
}
