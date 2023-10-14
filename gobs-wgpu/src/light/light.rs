use glam::Vec3;

use super::LightResource;

pub struct Light {
    pub position: Vec3,
    pub colour: Vec3,
    pub resource: LightResource
}

impl Light {
    pub fn new<V: Into<Vec3>>(
        mut resource: LightResource,
        position: V,
        colour: V) -> Self {
        
        let position: Vec3 = position.into();
        let colour: Vec3 = colour.into();

        resource.update(position.into(), colour.into());

        Light {
            position,
            colour,
            resource
        }
    }

    pub fn update<V: Into<Vec3>>(&mut self, position: V) {
        self.position = position.into();
        self.resource.update(self.position.into(), self.colour.into());
    }
}