use glam::{Vec3, Vec4};
use gobs_core::Transform;

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingBox {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
}

impl BoundingBox {
    pub fn from_corners(bottom_left: Vec3, top_right: Vec3) -> Self {
        Self {
            x_min: bottom_left.x,
            x_max: top_right.x,
            y_min: bottom_left.y,
            y_max: top_right.y,
            z_min: bottom_left.z,
            z_max: top_right.z,
        }
    }

    pub fn bottom_left(&self) -> Vec3 {
        (self.x_min, self.y_min, self.z_min).into()
    }

    pub fn top_right(&self) -> Vec3 {
        (self.x_max, self.y_max, self.z_max).into()
    }

    pub fn extends(&mut self, pos: Vec3) {
        if pos.x < self.x_min {
            self.x_min = pos.x;
        }
        if pos.x > self.x_max {
            self.x_max = pos.x;
        }
        if pos.y < self.y_min {
            self.y_min = pos.y;
        }
        if pos.y > self.y_max {
            self.y_max = pos.y;
        }
        if pos.z < self.z_min {
            self.z_min = pos.z;
        }
        if pos.z > self.z_max {
            self.z_max = pos.z;
        }
    }

    pub fn extends_box(&mut self, other: BoundingBox) {
        if other.x_min < self.x_min {
            self.x_min = other.x_min;
        }
        if other.x_max > self.x_max {
            self.x_max = other.x_max;
        }
        if other.y_min < self.y_min {
            self.y_min = other.y_min;
        }
        if other.y_max > self.y_max {
            self.y_max = other.y_max;
        }
        if other.z_min < self.z_min {
            self.z_min = other.z_min;
        }
        if other.z_max > self.z_max {
            self.z_max = other.z_max;
        }
    }

    pub fn transform(&mut self, transform: Transform) -> Self {
        let min_pos = Vec4::new(self.x_min, self.y_min, self.z_min, 1.);
        let max_pos = Vec4::new(self.x_max, self.y_max, self.z_max, 1.);

        let t_min = transform * min_pos;
        let t_max = transform * max_pos;

        Self::from_corners(t_min.truncate(), t_max.truncate())
    }
}

pub trait Bounded {
    fn boundings(&self) -> BoundingBox;
}
