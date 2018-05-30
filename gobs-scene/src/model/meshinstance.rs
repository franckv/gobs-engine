use std::sync::Arc;

use cgmath::{Matrix, Matrix3, Matrix4, SquareMatrix, Vector4};

use model::Color;
use model::Mesh;
use model::Texture;

#[derive(Copy, Clone)]
pub struct Instance {
    pub transform: [[f32; 4]; 4],
    pub normal_transform: [[f32; 3]; 3],
    pub color: [f32; 4],
    pub region: [f32; 4],
}

pub struct MeshInstanceBuilder {
    mesh: Arc<Mesh>,
    color: Color,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl MeshInstanceBuilder {
    pub fn new(mesh: Arc<Mesh>) -> MeshInstanceBuilder {
        MeshInstanceBuilder {
            mesh: mesh,
            color: Color::white(),
            texture: None,
            region: [0.0, 0.0, 1.0, 1.0]
        }
    }

    pub fn color(mut self, color: Color) -> MeshInstanceBuilder {
        self.color = color;

        self
    }

    pub fn texture(mut self, texture: Arc<Texture>) -> MeshInstanceBuilder {
        self.texture = Some(texture);

        self
    }

    pub fn region(mut self, region: [f32; 4]) -> MeshInstanceBuilder {
        self.region = region;

        self
    }

    pub fn atlas(self, i: usize, j: usize, tile_size: [usize; 2]) -> MeshInstanceBuilder {
        let (ustep, vstep) = {
            let texture = self.texture.as_ref().unwrap();
            let img_size = texture.size();

            (tile_size[0] as f32 / img_size[0] as f32, tile_size[1] as f32 / img_size[1] as f32)
        };

        let i = i as f32;
        let j = j as f32;

        self.region([i * ustep, j * vstep, (i + 1.0) * ustep, (j + 1.0) * vstep])
    }

    pub fn build(self) -> MeshInstance {
        MeshInstance::new(self.mesh.clone(), self.color, self.texture, self.region)
    }
}

pub struct MeshInstance {
    mesh: Arc<Mesh>,
    color: Color,
    matrix: Matrix4<f32>,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl MeshInstance {
    fn new(mesh: Arc<Mesh>, color: Color, texture: Option<Arc<Texture>>,
        region: [f32; 4]) -> MeshInstance {
        MeshInstance {
            mesh: mesh,
            color: color,
            matrix: Matrix4::identity(),
            texture: texture,
            region: region
        }
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

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    pub fn get_instance_data(&self) -> Instance {
        let normal_transform = Matrix3::from_cols(
            self.matrix.x.truncate(),
            self.matrix.y.truncate(),
            self.matrix.z.truncate(),
        ).invert().unwrap().transpose();

        Instance {
            transform: self.matrix.into(),
            normal_transform: normal_transform.into(),
            color: self.color.into(),
            region: self.region,
        }
    }

    pub fn transform(&mut self, trans: &Matrix4<f32>) {
        self.matrix = trans * self.matrix;
    }

    pub fn scale(&mut self, scale_x: f32, scale_y: f32, scale_z: f32) {
        self.transform(&Matrix4::from_diagonal(Vector4::new(scale_x, scale_y, scale_z, 1.)));
    }

    pub fn translate(&mut self, vector: (f32, f32, f32)) {
        self.transform(&Matrix4::from_translation(vector.into()));
    }
}
