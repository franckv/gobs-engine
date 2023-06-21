use time;

pub struct Timer {
    last_tick: time::OffsetDateTime,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            last_tick: time::OffsetDateTime::now_utc()
        }
    }

    pub fn delta(&mut self) -> i128 {
        let tick = time::OffsetDateTime::now_utc();
        let delta = tick - self.last_tick;
        self.last_tick = tick;

        delta.whole_nanoseconds()
    }

    pub fn reset(&mut self) {
        self.last_tick = time::OffsetDateTime::now_utc();
    }
}
