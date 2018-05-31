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

pub struct RenderObjectBuilder {
    mesh: Arc<Mesh>,
    color: Color,
    matrix: Matrix4<f32>,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl RenderObjectBuilder {
    pub fn new(mesh: Arc<Mesh>) -> RenderObjectBuilder {
        RenderObjectBuilder {
            mesh: mesh,
            color: Color::white(),
            matrix: Matrix4::identity(),
            texture: None,
            region: [0.0, 0.0, 1.0, 1.0]
        }
    }

    pub fn color(mut self, color: Color) -> RenderObjectBuilder {
        self.color = color;

        self
    }

    pub fn texture(mut self, texture: Arc<Texture>) -> RenderObjectBuilder {
        self.texture = Some(texture);

        self
    }

    pub fn region(mut self, region: [f32; 4]) -> RenderObjectBuilder {
        self.region = region;

        self
    }

    pub fn atlas(self, i: usize, j: usize, tile_size: [usize; 2]) -> RenderObjectBuilder {
        let (ustep, vstep) = {
            let texture = self.texture.as_ref().unwrap();
            let img_size = texture.size();

            (tile_size[0] as f32 / img_size[0] as f32, tile_size[1] as f32 / img_size[1] as f32)
        };

        let i = i as f32;
        let j = j as f32;

        self.region([i * ustep, j * vstep, (i + 1.0) * ustep, (j + 1.0) * vstep])
    }

    pub fn transform(mut self, matrix: Matrix4<f32>) -> RenderObjectBuilder {
        self.matrix = matrix * self.matrix;

        self
    }

    pub fn translate(self, vector: (f32, f32, f32)) -> RenderObjectBuilder {
        self.transform(Matrix4::from_translation(vector.into()))
    }

    pub fn scale(self, scale_x: f32, scale_y: f32, scale_z: f32) -> RenderObjectBuilder {
        self.transform(Matrix4::from_diagonal(Vector4::new(scale_x, scale_y, scale_z, 1.)))
    }

    pub fn build(self) -> Arc<RenderObject> {
        RenderObject::new(self.mesh.clone(), self.color, self.matrix, self.texture,
            self.region)
    }
}

pub struct RenderObject {
    mesh: Arc<Mesh>,
    color: Color,
    matrix: Matrix4<f32>,
    texture: Option<Arc<Texture>>,
    region: [f32; 4]
}

impl RenderObject {
    fn new(mesh: Arc<Mesh>, color: Color, matrix: Matrix4<f32>,
        texture: Option<Arc<Texture>>, region: [f32; 4]) -> Arc<RenderObject> {
        Arc::new(RenderObject {
            mesh: mesh,
            color: color,
            matrix: matrix,
            texture: texture,
            region: region
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

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    pub fn get_instance_data(&self, global_transform: Matrix4<f32>) -> Instance {
        let transform = global_transform * self.matrix;

        let normal_transform = Matrix3::from_cols(
            transform.x.truncate(),
            transform.y.truncate(),
            transform.z.truncate(),
        ).invert().unwrap().transpose();

        Instance {
            transform: transform.into(),
            normal_transform: normal_transform.into(),
            color: self.color.into(),
            region: self.region,
        }
    }
}
