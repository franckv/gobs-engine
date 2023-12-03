use glam::Vec3;
use gobs_core::Color;

use crate::{Hit, Hitable, Ray};

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    color: Color,
    reflect: f32,
}

impl Sphere {
    pub fn new(
        center: Vec3,
        radius: f32,
        color: Color,
        reflect: f32,
    ) -> Box<dyn Hitable + Send + Sync> {
        Box::new(Self {
            center,
            radius,
            color,
            reflect,
        })
    }
}

impl Hitable for Sphere {
    fn hit_distance(&self, ray: &Ray, min: f32, max: f32) -> Option<f32> {
        let d = ray.origin - self.center;

        let a = ray.direction.dot(ray.direction); // > 0
        let b = 2. * ray.direction.dot(d);
        let c = d.dot(d) - self.radius * self.radius;

        let delta = b * b - 4. * a * c;

        if delta > 0. {
            let t1 = 0.5 * (-b - delta.sqrt()) / a;
            let t2 = 0.5 * (-b + delta.sqrt()) / a;

            if t1 >= min && t1 <= max && t1 <= t2 {
                Some(t1)
            } else if t2 >= min && t2 <= max {
                Some(t2)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn hit(&self, ray: &Ray, min: f32, max: f32) -> Option<Hit> {
        let distance = self.hit_distance(ray, min, max);

        match distance {
            Some(t) => {
                let position = ray.origin + t * ray.direction;
                let normal = (position - self.center).normalize();
                Some(Hit {
                    distance: t,
                    position,
                    normal,
                    color: self.color,
                    reflect: self.reflect,
                })
            }
            None => None,
        }
    }
}
