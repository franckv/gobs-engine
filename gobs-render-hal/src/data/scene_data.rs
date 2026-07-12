use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, Transform};
use gobs_resource::{camera::Camera, light::Light};

use crate::{
    UniformData,
    data::{UniformLayout, UniformProp, UniformPropData},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SceneDataProp {
    CameraPosition,
    CameraViewProj,
    CameraViewPort,
    LightDirection,
    LightColor,
    LightAmbientColor,
}

#[derive(Clone, Debug, Default)]
pub struct SceneDataLayout {
    layout: Vec<SceneDataProp>,
    uniform_layout: UniformLayout,
}

impl UniformData<SceneDataProp> for SceneDataLayout {
    fn prop(mut self, prop: SceneDataProp) -> Self {
        self.layout.push(prop);

        match prop {
            SceneDataProp::CameraPosition => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("camera_position", UniformProp::Vec3F)
            }
            SceneDataProp::CameraViewProj => {
                self.uniform_layout = self.uniform_layout.prop("view_proj", UniformProp::Mat4F)
            }
            SceneDataProp::CameraViewPort => {
                self.uniform_layout = self.uniform_layout.prop("screen_size", UniformProp::Vec2F)
            }
            SceneDataProp::LightDirection => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("light_direction", UniformProp::Vec3F)
            }
            SceneDataProp::LightColor => {
                self.uniform_layout = self.uniform_layout.prop("light_color", UniformProp::Vec4F)
            }
            SceneDataProp::LightAmbientColor => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("ambient_color", UniformProp::Vec4F)
            }
        }

        self
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn copy_data<F>(&self, buffer: &mut Vec<u8>, get_data: F)
    where
        F: Fn(&SceneDataProp) -> UniformPropData,
    {
        let layout = self.uniform_layout();

        let mut props = Vec::new();

        for prop in &self.layout {
            props.push(get_data(prop));
        }

        layout.copy_data(&props, buffer)
    }

    fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }
}

pub struct SceneData<'data> {
    pub camera_transform: &'data Transform,
    pub camera: &'data Camera,
    pub light_transform: Option<&'data Transform>,
    pub light: Option<&'data Light>,
    pub extent: ImageExtent2D,
}
