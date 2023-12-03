use gobs_core::Color;
use rand::seq::SliceRandom;

pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub framebuffer: Vec<Color>,
    strategy: ChunkStrategyData,
}

impl ImageBuffer {
    pub fn new(width: u32, height: u32, strategy: ChunkStrategy) -> Self {
        Self {
            width,
            height,
            framebuffer: Vec::new(),
            strategy: ChunkStrategy::new(strategy, width, height),
        }
    }

    pub fn reset(&mut self) {
        self.framebuffer.clear();

        for _ in 0..(self.height as usize * self.width as usize) {
            self.framebuffer.push(Color::BLACK);
        }

        self.strategy.reset(self.width, self.height);
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.framebuffer
            .iter()
            .flat_map(|c| Into::<[u8; 4]>::into(*c))
            .collect::<Vec<u8>>()
    }

    pub fn update_pixel(&mut self, idx: usize, c: Color) {
        self.framebuffer[idx] = c;
    }

    pub fn is_complete(&self) -> bool {
        self.strategy.is_complete()
    }

    pub fn get_chunk(&mut self) -> Vec<usize> {
        self.strategy.get_chunk()
    }
}

pub enum ChunkStrategy {
    RANDOM,
    LINE,
    BOX,
}

impl ChunkStrategy {
    pub fn new(strategy: ChunkStrategy, width: u32, height: u32) -> ChunkStrategyData {
        match strategy {
            ChunkStrategy::RANDOM => ChunkStrategyData::RANDOM(RandomChunk::new()),
            ChunkStrategy::LINE => ChunkStrategyData::LINE(LineChunk::new()),
            ChunkStrategy::BOX => ChunkStrategyData::BOX(BoxChunk::new(width, height)),
        }
    }
}

pub enum ChunkStrategyData {
    RANDOM(RandomChunk),
    LINE(LineChunk),
    BOX(BoxChunk),
}

impl ChunkStrategyData {
    pub fn reset(&mut self, width: u32, height: u32) {
        match self {
            ChunkStrategyData::RANDOM(ref mut strategy) => {
                strategy.reset(width, height);
            }
            ChunkStrategyData::LINE(ref mut strategy) => {
                strategy.reset(width, height);
            }
            ChunkStrategyData::BOX(ref mut strategy) => {
                strategy.reset(width, height);
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            ChunkStrategyData::RANDOM(ref strategy) => strategy.is_complete(),
            ChunkStrategyData::LINE(ref strategy) => strategy.is_complete(),
            ChunkStrategyData::BOX(ref strategy) => strategy.is_complete(),
        }
    }

    pub fn get_chunk(&mut self) -> Vec<usize> {
        match self {
            ChunkStrategyData::RANDOM(ref mut strategy) => strategy.get_chunk(),
            ChunkStrategyData::LINE(ref mut strategy) => strategy.get_chunk(),
            ChunkStrategyData::BOX(ref mut strategy) => strategy.get_chunk(),
        }
    }
}

pub struct RandomChunk {
    draw_indexes: Vec<usize>,
}

impl RandomChunk {
    const PIXEL_PER_CHUNK: usize = 20000;

    pub fn new() -> Self {
        Self {
            draw_indexes: Vec::new(),
        }
    }

    fn reset(&mut self, width: u32, height: u32) {
        self.draw_indexes.clear();

        for i in 0..width * height {
            self.draw_indexes.push(i as usize);
        }
        let mut rng = rand::thread_rng();
        self.draw_indexes.shuffle(&mut rng)
    }

    fn is_complete(&self) -> bool {
        self.draw_indexes.is_empty()
    }

    fn get_chunk(&mut self) -> Vec<usize> {
        self.draw_indexes
            .drain(0..Self::PIXEL_PER_CHUNK.min(self.draw_indexes.len()))
            .collect::<Vec<usize>>()
    }
}

pub struct LineChunk {
    draw_indexes: Vec<usize>,
}

impl LineChunk {
    const PIXEL_PER_CHUNK: usize = 1920;

    pub fn new() -> Self {
        Self {
            draw_indexes: Vec::new(),
        }
    }

    fn reset(&mut self, width: u32, height: u32) {
        self.draw_indexes.clear();

        for i in 0..width * height {
            self.draw_indexes.push(i as usize);
        }
    }

    fn is_complete(&self) -> bool {
        self.draw_indexes.is_empty()
    }

    fn get_chunk(&mut self) -> Vec<usize> {
        self.draw_indexes
            .drain(0..Self::PIXEL_PER_CHUNK.min(self.draw_indexes.len()))
            .collect::<Vec<usize>>()
    }
}

pub struct BoxChunk {
    cols: u32,
    rows: u32,
    draw_boxes: Vec<Vec<usize>>,
}

impl BoxChunk {
    const BOX_WIDTH: u32 = 128;
    const BOX_HEIGHT: u32 = 128;

    pub fn new(width: u32, height: u32) -> Self {
        let cols = width.div_ceil(Self::BOX_WIDTH);
        let rows = height.div_ceil(Self::BOX_HEIGHT);

        Self {
            cols,
            rows,
            draw_boxes: Vec::new(),
        }
    }

    pub fn reset(&mut self, width: u32, height: u32) {
        self.draw_boxes.clear();

        for j in 0..self.rows {
            for i in 0..self.cols {
                let mut chunk = Vec::new();

                let x_min = i * Self::BOX_WIDTH;
                let x_max = (x_min + Self::BOX_WIDTH).min(width);
                let y_min = j * Self::BOX_HEIGHT;
                let y_max = (y_min + Self::BOX_HEIGHT).min(height);

                for x in x_min..x_max {
                    for y in y_min..y_max {
                        chunk.push((x + y * width) as usize);
                    }
                }
                self.draw_boxes.push(chunk)
            }
        }

        let mut rng = rand::thread_rng();
        self.draw_boxes.shuffle(&mut rng)
    }

    fn is_complete(&self) -> bool {
        log::debug!("{} boxes to draw", self.draw_boxes.len());
        self.draw_boxes.is_empty()
    }

    fn get_chunk(&mut self) -> Vec<usize> {
        self.draw_boxes.pop().unwrap()
    }
}
