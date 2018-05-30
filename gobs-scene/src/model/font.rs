use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;

use cgmath::{Matrix4, SquareMatrix};
use rusttype::{Font as RFont, FontCollection, Scale, point, Rect};

use model::Color;
use model::Texture;

const TEXTURE_SIZE: (usize, usize) = (1024, 1024);

#[derive(Clone)]
pub struct Character {
    c: char,
    region: [f32; 4],
    transform: Matrix4<f32>,
    advance: f32
}

impl Character {
    pub fn region(&self) -> &[f32; 4] {
        &self.region
    }

    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn translate(&mut self, transform: Matrix4<f32>) {
        self.transform = transform * self.transform;
    }
}

pub struct Font {
    texture: Arc<Texture>,
    cache: HashMap<char, Character>
}

impl Font {
    pub fn new(size: usize, path: &str) -> Self {
        let mut f = File::open(path).unwrap();
        let mut v = Vec::new();
        f.read_to_end(&mut v).expect("Cannot read font");

        let collection = FontCollection::from_bytes(v);

        let font = collection.unwrap().into_font().unwrap();
        let scale = Scale {x: size as f32, y: size as f32};

        let (width, height) = TEXTURE_SIZE;

        let mut cache = HashMap::new();

        let image_data = Self::build_texture(&font, &mut cache, scale, width, height);

        let texture = Texture::from_raw(image_data, width, height);

        Font {
            texture: texture,
            cache: cache
        }
    }

    pub fn texture(&self) -> Arc<Texture> {
        self.texture.clone()
    }

    pub fn layout(&self, text: &str) -> Vec<Character> {
        let mut result = Vec::new();

        let mut translate = Matrix4::identity();

        for c in text.nfc() {
            if let Some(character) = self.cache.get(&c) {
                let advance = character.advance;

                translate = Matrix4::from_translation([advance, 0.0, 0.0].into()) * translate;

                let mut character = character.clone();
                character.translate(translate);

                result.push(character);
            }
        }

        result
    }

    fn build_texture(font: &RFont, cache: &mut HashMap<char, Character>,
        scale: Scale, width: usize, height: usize) -> Vec<u8> {
        let mut image = vec![0u8; width * height * 4];

        let v_metrics = font.v_metrics(scale);

        // cursor position
        let mut pos = point(0., v_metrics.ascent);

        // height of a symbol line
        let max_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let rgb: [u8; 3] = Color::white().into();

        for c in 32u8..127u8 {
            let c = c as char;
            let mut glyph = font.glyph(c).scaled(scale);
            let advance = glyph.h_metrics().advance_width;

            let mut glyph = glyph.positioned(pos);

            let bb = match glyph.pixel_bounding_box() {
                Some(rect) => {
                    if rect.max.x > width as i32 {
                        pos = point(0., pos.y + max_height);
                        glyph = glyph.into_unpositioned().positioned(pos);
                    }
                    let rect = glyph.pixel_bounding_box().unwrap();

                    rect
                },
                None => Rect {
                    min: point(0, 0),
                    max: point(1, 1),
                }
            };

            let character = {
                let xmin = bb.min.x as f32 / width as f32;
                let ymin = bb.min.y as f32 / height as f32;
                let xmax = bb.max.x as f32 / width as f32;
                let ymax = bb.max.y as f32 / height as f32;

                let ypos = pos.y / height as f32;

                // align character on origin
                let offset = ypos - (ymin + ymax) / 2.;
                let translate = Matrix4::from_translation([0., offset, 0.].into());

                // resize mesh to fit bounding box
                let transform = Matrix4::from_diagonal(
                    [xmax - xmin, ymax - ymin, 1., 1.].into());

                Character {
                    c: c,
                    region: [xmin, ymin, xmax, ymax],
                    transform: translate * transform,
                    advance: advance / width as f32
                }
            };

            cache.insert(c, character);

            glyph.draw(|x, y, v| {
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
            });
            pos.x += advance;
        }

        image
    }
}
