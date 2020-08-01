use std::sync::Arc;

use crate::data::TreeGraph;
use crate::model::Model;
use super::camera::Camera;
use super::light::Light;

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
