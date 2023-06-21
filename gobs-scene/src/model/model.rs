use std::sync::Arc;

use super::{Color, Mesh, Texture};

pub struct ModelBuilder {
    mesh: Arc<Mesh>,
    color: Color,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl ModelBuilder {
    pub fn new(mesh: Arc<Mesh>) -> ModelBuilder {
        ModelBuilder {
            mesh,
            color: Color::white(),
            texture: None,
            region: [0.0, 0.0, 1.0, 1.0]
        }
    }

    pub fn color(mut self, color: Color) -> ModelBuilder {
        self.color = color;

        self
    }

    pub fn texture(mut self, texture: Arc<Texture>) -> ModelBuilder {
        self.texture = Some(texture);

        self
    }

    pub fn region(mut self, region: [f32; 4]) -> ModelBuilder {
        self.region = region;

        self
    }

    pub fn atlas(self, i: usize, j: usize, tile_size: [usize; 2]) -> ModelBuilder {
        let (ustep, vstep) = {
            let texture = self.texture.as_ref().unwrap();
            let img_size = texture.size();

            (tile_size[0] as f32 / img_size[0] as f32, tile_size[1] as f32 / img_size[1] as f32)
        };

        let i = i as f32;
        let j = j as f32;

        self.region([i * ustep, j * vstep, (i + 1.0) * ustep, (j + 1.0) * vstep])
    }

    pub fn build(self) -> Arc<Model> {
        Model::new(self.mesh.clone(), self.color, self.texture, self.region)
    }
}

pub struct Model {
    mesh: Arc<Mesh>,
    color: Color,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl Model {
    fn new(mesh: Arc<Mesh>, color: Color, texture: Option<Arc<Texture>>, region: [f32; 4])
    -> Arc<Model> {
        Arc::new(Model {
            mesh,
            color,
            texture,
            region
        })
    }

    pub fn mesh(&self) -> Arc<Mesh> {
        self.mesh.clone()
    }

    pub fn texture(&self) -> Option<Arc<Texture>> {
        self.texture.clone()
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn region(&self) -> &[f32; 4] {
        &self.region
    }
}
