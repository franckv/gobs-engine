use std::sync::Arc;
use std::vec::Vec;

use image;
use image::{ImageBuffer, GenericImageView, Pixel};
use uuid::Uuid;

use super::Color;

pub struct Texture {
    id: Uuid,
    data: Vec<u8>,
    size: [usize; 2]
}

impl Texture {
    pub fn from_file(path: &str) -> Arc<Self> {
        let img = image::open(path).expect("Texture file not found");

        let (width, height) = (img.width() as usize, img.height() as usize);

        let img_data = img.to_rgba().into_raw();

        Self::from_raw(img_data, width, height)
    }

    pub fn from_color(color: Color) -> Arc<Self> {
        let c: [u8; 4] = color.into();
        let pixel = image::Rgba::from_channels(c[0], c[1], c[2], c[3]);

        let img = ImageBuffer::from_pixel(1, 1, pixel);

        let img_data = img.into_raw();

        Self::from_raw(img_data, 1, 1)
    }

    pub fn from_raw(data: Vec<u8>, width: usize, height: usize) -> Arc<Self> {
        Arc::new(Texture {
            id: Uuid::new_v4(),
            data: data,
            size: [width, height]
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn size(&self) -> [usize; 2] {
        self.size
    }
}
