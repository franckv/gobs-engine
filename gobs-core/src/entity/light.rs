use glam::Vec3;
use uuid::Uuid;

pub type LightId = Uuid;

pub struct Light {
    pub id: LightId,
    pub position: Vec3,
    pub colour: Vec3,
}

impl Light {
    pub fn new<V: Into<Vec3>>(position: V, colour: V) -> Self {
        let position: Vec3 = position.into();
        let colour: Vec3 = colour.into();

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
