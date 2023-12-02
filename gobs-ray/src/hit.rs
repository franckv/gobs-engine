use glam::Vec3;
use gobs_core::Color;

use crate::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Hit {
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Color,
    pub reflect: f32,
}

pub trait Hitable {
    fn hit(&self, ray: &Ray, min: f32, max: f32) -> Option<Hit>;
    fn hit_distance(&self, ray: &Ray, min: f32, max: f32) -> Option<f32>;
}
