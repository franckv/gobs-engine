use glam::Vec3;
use uuid::Uuid;

use crate::Color;

pub type LightId = Uuid;

pub struct Light {
    pub id: LightId,
    pub position: Vec3,
    pub colour: Color,
}

impl Light {
    pub fn new<V: Into<Vec3>>(position: V, colour: Color) -> Self {
        let position: Vec3 = position.into();

        Light {
            id: Uuid::new_v4(),
            position,
            colour,
        }
    }

    pub fn update<V: Into<Vec3>>(&mut self, position: V) {
        self.position = position.into();
    }
}
