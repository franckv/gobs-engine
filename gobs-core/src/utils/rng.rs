use rand::Rng;

pub struct RngPool {
    index: usize,
    pool: Vec<f32>,
}

impl RngPool {
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            index: 0,
            pool: (0..size)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect::<Vec<f32>>(),
        }
    }

    pub fn next(&mut self) -> f32 {
        let r = self.pool[self.index];
        self.index = (self.index + 1) % self.pool.len();

        r
    }
}
