use std::sync::Arc;

use data::TreeGraph;
use model::Model;
use scene::camera::Camera;
use scene::light::Light;

pub type SceneGraph = TreeGraph<SceneData>;

pub enum SceneData {
    Object(Arc<Model>),
    Camera(Camera),
    Light(Light),
}

impl From<Arc<Model>> for SceneData {
    fn from(r: Arc<Model>) -> Self {
        SceneData::Object(r)
    }
}

impl From<Camera> for SceneData {
    fn from(c: Camera) -> Self {
        SceneData::Camera(c)
    }
}

impl From<Light> for SceneData {
    fn from(l: Light) -> Self {
        SceneData::Light(l)
    }
}
