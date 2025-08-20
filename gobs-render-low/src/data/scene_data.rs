use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, Transform};
use gobs_resource::entity::{camera::Camera, light::Light};

use crate::data::{UniformLayout, UniformProp, UniformPropData};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SceneDataProp {
    CameraPosition,
    CameraViewProj,
    CameraViewPort,
    LightDirection,
    LightColor,
    LightAmbientColor,
}

pub struct SceneDataLayout {
    layout: Vec<SceneDataProp>,
    uniform_layout: UniformLayout,
}

impl SceneDataLayout {
    pub fn builder() -> SceneDataLayoutBuilder {
        SceneDataLayoutBuilder::new()
    }

    pub fn data(&self, scene_data: &SceneData) -> Vec<u8> {
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
                        scene_data.light_transform.translation().normalize().into(),
                    ));
                }
                SceneDataProp::LightColor => {
                    props.push(UniformPropData::Vec4F(scene_data.light.colour.into()));
                }
                SceneDataProp::LightAmbientColor => {
                    props.push(UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]));
                }
            }
        }

        layout.data(&props)
    }

    pub fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }
}

pub struct SceneDataLayoutBuilder {
    layout: Vec<SceneDataProp>,
}

impl SceneDataLayoutBuilder {
    fn new() -> Self {
        Self {
            layout: Default::default(),
        }
    }

    pub fn prop(mut self, prop: SceneDataProp) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn build(self) -> SceneDataLayout {
        let mut uniform_layout = UniformLayout::new();

        for prop in &self.layout {
            match prop {
                SceneDataProp::CameraPosition => {
                    uniform_layout = uniform_layout.prop("camera_position", UniformProp::Vec3F)
                }
                SceneDataProp::CameraViewProj => {
                    uniform_layout = uniform_layout.prop("view_proj", UniformProp::Mat4F)
                }
                SceneDataProp::CameraViewPort => {
                    uniform_layout = uniform_layout.prop("screen_size", UniformProp::Vec2F)
                }
                SceneDataProp::LightDirection => {
                    uniform_layout = uniform_layout.prop("light_direction", UniformProp::Vec3F)
                }
                SceneDataProp::LightColor => {
                    uniform_layout = uniform_layout.prop("light_color", UniformProp::Vec4F)
                }
                SceneDataProp::LightAmbientColor => {
                    uniform_layout = uniform_layout.prop("ambient_color", UniformProp::Vec4F)
                }
            };
        }

        SceneDataLayout {
            layout: self.layout,
            uniform_layout,
        }
    }
}

pub struct SceneData<'data> {
    pub camera_transform: &'data Transform,
    pub camera: &'data Camera,
    pub light_transform: &'data Transform,
    pub light: &'data Light,
    pub extent: ImageExtent2D,
}
