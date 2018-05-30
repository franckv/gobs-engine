use std::sync::Arc;
use std::slice::Iter;

use model::MeshInstance;
use scene::camera::Camera;
use scene::light::{Light, LightBuilder};

pub struct SceneGraph {
    camera: Camera,
    light: Light,
    instances: Vec<Arc<MeshInstance>>
}

impl SceneGraph {
    pub fn new() -> SceneGraph {
        SceneGraph {
            camera: Camera::new([0., 0., 0.].into()),
            light: LightBuilder::new().build(),
            instances: Vec::new()
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn set_light(&mut self, light: Light) {
        self.light = light;
    }

    pub fn add_instance(&mut self, instance: Arc<MeshInstance>) {
        self.instances.push(instance);
    }

    pub fn instances(&self) -> Iter<Arc<MeshInstance>> {
        self.instances.iter()
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }
}
