#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageExtent2D {
    pub width: u32,
    pub height: u32,
}

impl ImageExtent2D {
    pub fn new(width: u32, height: u32) -> Self {
        ImageExtent2D { width, height }
    }

    pub fn size(self) -> u32 {
        self.width * self.height
    }
}

impl From<(u32, u32)> for ImageExtent2D {
    fn from(value: (u32, u32)) -> Self {
        ImageExtent2D::new(value.0, value.1)
    }
}

impl Into<(u32, u32)> for ImageExtent2D {
    fn into(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Into<(f32, f32)> for ImageExtent2D {
    fn into(self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }
}

impl Into<[f32; 2]> for ImageExtent2D {
    fn into(self) -> [f32; 2] {
        [self.width as f32, self.height as f32]
    }
}
