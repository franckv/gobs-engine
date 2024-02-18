use time;

pub struct Timer {
    last_tick: time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            last_tick: time::Instant::now(),
        }
    }

    pub fn delta(&mut self) -> f32 {
        let tick = time::Instant::now();
        let delta = tick - self.last_tick;
        self.last_tick = tick;

        delta.as_seconds_f32()
    }

    pub fn peek(&self) -> f32 {
        let tick = time::Instant::now();
        let delta = tick - self.last_tick;

        delta.as_seconds_f32()
    }

    pub fn reset(&mut self) {
        self.last_tick = time::Instant::now();
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
