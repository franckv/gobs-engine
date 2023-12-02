use glam::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn reflect(&self, position: Vec3, normal: Vec3) -> Self {
        Self::new(
            position,
            self.direction - 2. * normal.dot(self.direction) * normal,
        )
    }
}
