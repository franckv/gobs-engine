use glam::{Vec3, Vec4};
use serde::Serialize;

use gobs_core::Transform;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
}

impl BoundingBox {
    pub fn from_corners(corners: &[Vec3]) -> Self {
        let (mut x_min, mut y_min, mut z_min) = corners[0].into();
        let (mut x_max, mut y_max, mut z_max) = corners[0].into();

        for corner in corners {
            if corner.x < x_min {
                x_min = corner.x;
            }
            if corner.x > x_max {
                x_max = corner.x;
            }
            if corner.y < y_min {
                y_min = corner.y;
            }
            if corner.y > y_max {
                y_max = corner.y;
            }
            if corner.z < z_min {
                z_min = corner.z;
            }
            if corner.z > z_max {
                z_max = corner.z;
            }
        }

        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
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

    pub fn transform(&self, transform: Transform) -> Self {
        let c1 = Vec4::new(self.x_min, self.y_min, self.z_min, 1.);
        let c2 = Vec4::new(self.x_min, self.y_min, self.z_max, 1.);
        let c3 = Vec4::new(self.x_min, self.y_max, self.z_min, 1.);
        let c4 = Vec4::new(self.x_min, self.y_max, self.z_max, 1.);
        let c5 = Vec4::new(self.x_max, self.y_min, self.z_min, 1.);
        let c6 = Vec4::new(self.x_max, self.y_min, self.z_max, 1.);
        let c7 = Vec4::new(self.x_max, self.y_max, self.z_min, 1.);
        let c8 = Vec4::new(self.x_max, self.y_max, self.z_max, 1.);

        let tc1 = transform * c1;
        let tc2 = transform * c2;
        let tc3 = transform * c3;
        let tc4 = transform * c4;
        let tc5 = transform * c5;
        let tc6 = transform * c6;
        let tc7 = transform * c7;
        let tc8 = transform * c8;

        Self::from_corners(&[
            tc1.truncate(),
            tc2.truncate(),
            tc3.truncate(),
            tc4.truncate(),
            tc5.truncate(),
            tc6.truncate(),
            tc7.truncate(),
            tc8.truncate(),
        ])
    }
}

pub trait Bounded {
    fn boundings(&self) -> BoundingBox;
}
