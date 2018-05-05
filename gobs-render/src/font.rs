use std::io::Read;
use std::fs::File;

use rusttype::{Font as RFont, FontCollection, Scale, point, PositionedGlyph};

use color::Color;

pub struct Font<'a> {
    size: usize,
    font: RFont<'a>
}

impl<'a> Font<'a> {
    pub fn new(size: usize, path: &str) -> Self {
        let font = Self::load_font(path);

        Font {
            size: size,
            font: font
        }
    }

   pub fn load_font(path: &str) -> RFont<'static> {
        let mut f = File::open(path).unwrap();
        let mut v = Vec::new();
        f.read_to_end(&mut v).expect("Cannot read font");

        let collection = FontCollection::from_bytes(v);

        collection.unwrap().into_font().unwrap()
    }

    pub fn get_glyphs(&'a self, text: &str) -> Vec<PositionedGlyph<'a>> {
        let scale = Scale {x: self.size as f32 * 2., y: self.size as f32};

        let v_metrics = self.font.v_metrics(scale);

        let offset = point(0., v_metrics.ascent);

        self.font.layout(text, scale, offset).collect()
    }

    pub fn get_width(glyphs: &Vec<PositionedGlyph>) -> usize {
        let g = glyphs.last().unwrap();
        (g.position().x as f32 + g.unpositioned().h_metrics().advance_width).ceil() as usize
    }

    pub fn rasterize(glyphs: &Vec<PositionedGlyph>, width: usize, height: usize, color: Color) -> Vec<u8> {
        let mut image = vec![0u8; width * height * 4];

        let rgb: [u8; 3] = color.into();

        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let c = (255.0 * v) as u8;
                    let x = x as i32  + bb.min.x;
                    let y = y as i32  + bb.min.y;

                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        let x = x as usize;
                        let y = y as usize;
                        let idx = 4 * (x + y * width);
                        image[idx] = rgb[0];
                        image[idx + 1] = rgb[1];
                        image[idx + 2] = rgb[2];
                        image[idx + 3] = c;
                    }
                })
            }
        }

        image
    }
}
