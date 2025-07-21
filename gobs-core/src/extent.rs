#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

impl From<ImageExtent2D> for (u32, u32) {
    fn from(val: ImageExtent2D) -> Self {
        (val.width, val.height)
    }
}

impl From<ImageExtent2D> for (f32, f32) {
    fn from(val: ImageExtent2D) -> Self {
        (val.width as f32, val.height as f32)
    }
}

impl From<ImageExtent2D> for [f32; 2] {
    fn from(val: ImageExtent2D) -> Self {
        [val.width as f32, val.height as f32]
    }
}
