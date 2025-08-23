use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, Transform};
use gobs_resource::entity::{camera::Camera, light::Light};

use crate::{
    GfxContext, UniformData,
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

impl UniformData<SceneDataProp, SceneData<'_>> for SceneDataLayout {
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

    fn copy_data(&self, _ctx: Option<&GfxContext>, scene_data: &SceneData, buffer: &mut Vec<u8>) {
        let layout = self.uniform_layout();

        let mut props = Vec::new();

        for prop in &self.layout {
            match prop {
                SceneDataProp::CameraPosition => {
                    props.push(UniformPropData::Vec3F(
                        scene_data.camera_transform.translation().into(),
                    ));
                }
                SceneDataProp::CameraViewProj => {
                    props.push(UniformPropData::Mat4F(
                        scene_data
                            .camera
                            .view_proj(scene_data.camera_transform.translation())
                            .to_cols_array_2d(),
                    ));
                }
                SceneDataProp::CameraViewPort => {
                    props.push(UniformPropData::Vec2F(scene_data.extent.into()));
                }
                SceneDataProp::LightDirection => {
                    props.push(UniformPropData::Vec3F(
                        scene_data
                            .light_transform
                            .unwrap()
                            .translation()
                            .normalize()
                            .into(),
                    ));
                }
                SceneDataProp::LightColor => {
                    props.push(UniformPropData::Vec4F(
                        scene_data.light.unwrap().colour.into(),
                    ));
                }
                SceneDataProp::LightAmbientColor => {
                    props.push(UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]));
                }
            }
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
