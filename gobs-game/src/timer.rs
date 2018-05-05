use time;

pub struct Timer {
    last_tick: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            last_tick: time::precise_time_ns()
        }
    }

    pub fn delta(&mut self) -> u64 {
        let tick = time::precise_time_ns();
        let delta = tick - self.last_tick;
        self.last_tick = tick;

        delta
    }

    pub fn reset(&mut self) {
        self.last_tick = time::precise_time_ns();
    }
}
