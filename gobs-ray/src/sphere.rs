use glam::Vec3;
use gobs_core::Color;

use crate::{Hit, Hitable, Ray};

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    color: Color,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, color: Color) -> Box<dyn Hitable> {
        Box::new(Self {
            center,
            radius,
            color,
        })
    }
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, min: f32, max: f32) -> Option<Hit> {
        let d = ray.origin - self.center;

        let a = ray.direction.dot(ray.direction); // > 0
        let b = 2. * ray.direction.dot(d);
        let c = d.dot(d) - self.radius * self.radius;

        let delta = b * b - 4. * a * c;

        if delta > 0. {
            let s1 = 0.5 * (-b - delta.sqrt()) / a;
            let s2 = 0.5 * (-b + delta.sqrt()) / a;

            if s1 >= min && s1 <= max && s1 <= s2 {
                Some(Hit {
                    distance: s1,
                    position: ray.origin + s1 * ray.direction,
                    normal: Vec3::ZERO,
                    color: self.color,
                })
            } else if s2 >= min && s2 <= max {
                Some(Hit {
                    distance: s2,
                    position: ray.origin + s2 * ray.direction,
                    normal: Vec3::ZERO,
                    color: self.color,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
