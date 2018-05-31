use std::sync::Arc;

use data::TreeGraph;
use model::RenderObject;
use scene::camera::Camera;
use scene::light::Light;

pub type SceneGraph = TreeGraph<SceneData>;

pub enum SceneData {
    Object(Arc<RenderObject>),
    Camera(Camera),
    Light(Light),
}

impl From<Arc<RenderObject>> for SceneData {
    fn from(r: Arc<RenderObject>) -> Self {
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
