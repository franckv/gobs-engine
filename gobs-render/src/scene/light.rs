use cgmath::{Point3, Vector3, Vector4};

use model::Color;

pub struct LightBuilder {
    position: Vector4<f32>,
    color: Color,
    ambient: Color
}

impl LightBuilder {
    pub fn new() -> Self {
        LightBuilder {
            position: Vector4::unit_z(),
            color: Color::white(),
            ambient: Color::new(0.1, 0.1, 0.1, 1.)
        }
    }

    pub fn directional(mut self, dir: Vector3<f32>) -> Self {
        self.position = -dir.extend(0.);

        self
    }

    pub fn point(mut self, pos: Point3<f32>) -> Self {
        self.position = Vector4::new(pos.x, pos.y, pos.z, 1.);

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn ambient(mut self, ambient: Color) -> Self {
        self.ambient = ambient;

        self
    }

    pub fn build(self) -> Light {
        Light::new(self.position, self.color, self.ambient)
    }
}

pub struct Light {
    position: Vector4<f32>,
    color: Color,
    ambient: Color
}


impl Light {
    fn new(position: Vector4<f32>, color: Color, ambient: Color) -> Self {
        Light {
            position: position,
            color: color,
            ambient: ambient
        }
    }

    pub fn direction(&self) -> &Vector4<f32> {
        &self.position
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn ambient(&self) -> &Color {
        &self.ambient
    }
}
