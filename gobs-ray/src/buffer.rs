use gobs_core::Color;
use rand::seq::SliceRandom;

pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub draw_indexes: Vec<usize>,
    pub framebuffer: Vec<Color>,
}

impl ImageBuffer {
    const PIXEL_PER_CHUNK: usize = 20000;

    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            draw_indexes: Vec::new(),
            framebuffer: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.framebuffer.clear();
        self.draw_indexes.clear();
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.framebuffer
            .iter()
            .flat_map(|c| Into::<[u8; 4]>::into(*c))
            .collect::<Vec<u8>>()
    }

    pub fn add_pixel(&mut self, idx: usize) {
        self.framebuffer.push(Color::BLACK);
        self.draw_indexes.push(idx);
    }

    pub fn update_pixel(&mut self, idx: usize, c: Color) {
        self.framebuffer[idx] = c;
    }

    pub fn prepare(&mut self) {
        let mut rng = rand::thread_rng();
        self.draw_indexes.shuffle(&mut rng)
    }

    pub fn is_complete(&self) -> bool {
        self.draw_indexes.is_empty()
    }

    pub fn get_chunk(&mut self) -> Vec<usize> {
        self.draw_indexes
            .drain(0..Self::PIXEL_PER_CHUNK.min(self.draw_indexes.len()))
            .collect::<Vec<usize>>()
    }
}
