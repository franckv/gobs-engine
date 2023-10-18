#[derive(Clone, Copy, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    pub fn white() -> Color {
        Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    }

    pub fn red() -> Color {
        Color {
            r: 1.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    }

    pub fn green() -> Color {
        Color {
            r: 0.,
            g: 1.,
            b: 0.,
            a: 1.,
        }
    }

    pub fn blue() -> Color {
        Color {
            r: 0.,
            g: 0.,
            b: 1.,
            a: 1.,
        }
    }

    pub fn yellow() -> Color {
        Color {
            r: 1.,
            g: 1.,
            b: 0.,
            a: 1.,
        }
    }
}

impl From<Color> for [f32; 3] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b]
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

impl From<Color> for [u8; 3] {
    fn from(c: Color) -> Self {
        let r = (c.r * 255.0) as u8;
        let g = (c.g * 255.0) as u8;
        let b = (c.b * 255.0) as u8;
        [r, g, b]
    }
}

impl From<Color> for [u8; 4] {
    fn from(c: Color) -> Self {
        let r = (c.r * 255.0) as u8;
        let g = (c.g * 255.0) as u8;
        let b = (c.b * 255.0) as u8;
        let a = (c.a * 255.0) as u8;
        [r, g, b, a]
    }
}
