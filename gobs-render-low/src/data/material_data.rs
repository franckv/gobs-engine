use std::sync::Arc;

use gobs_gfx::{
    BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType, GfxBindingGroupLayout,
};
use serde::{Deserialize, Serialize};

use crate::data::{UniformLayout, UniformProp};

// TODO: Emissive, Specular, Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialDataProp {
    DiffuseColor,
    EmissionColor,
    SpecularColor,
    SpecularPower,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialDataLayout {
    layout: Vec<MaterialDataProp>,
    uniform_layout: Arc<UniformLayout>,
}

impl MaterialDataLayout {
    pub fn builder() -> MaterialDataLayoutBuilder {
        MaterialDataLayoutBuilder::new()
    }

    pub fn data(&self) -> Vec<u8> {
        let layout = self.uniform_layout();

        let props = Vec::new();

        for prop in &self.layout {
            match prop {
                MaterialDataProp::DiffuseColor => {}
                MaterialDataProp::EmissionColor => {}
                MaterialDataProp::SpecularColor => {}
                MaterialDataProp::SpecularPower => {}
            }
        }

        layout.data(&props)
    }

    pub fn bindings_layout(&self) -> GfxBindingGroupLayout {
        GfxBindingGroupLayout::new(BindingGroupType::MaterialData)
            .add_binding(DescriptorType::Uniform, DescriptorStage::Fragment)
    }

    pub fn uniform_layout(&self) -> Arc<UniformLayout> {
        self.uniform_layout.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }
}

pub struct MaterialDataLayoutBuilder {
    layout: Vec<MaterialDataProp>,
}

impl MaterialDataLayoutBuilder {
    pub fn new() -> Self {
        Self {
            layout: Default::default(),
        }
    }

    pub fn prop(mut self, prop: MaterialDataProp) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn build(self) -> MaterialDataLayout {
        let mut layout = UniformLayout::builder();

        for prop in &self.layout {
            match prop {
                MaterialDataProp::DiffuseColor => {
                    layout = layout.prop("diffuse color", UniformProp::Vec4F)
                }
                MaterialDataProp::EmissionColor => {
                    layout = layout.prop("emission color", UniformProp::Vec4F)
                }
                MaterialDataProp::SpecularColor => {
                    layout = layout.prop("specular color", UniformProp::Vec4F)
                }
                MaterialDataProp::SpecularPower => {
                    layout = layout.prop("specular power", UniformProp::F32)
                }
            }
        }

        let uniform_layout = layout.build();

        MaterialDataLayout {
            layout: self.layout,
            uniform_layout,
        }
    }
}

impl Default for MaterialDataLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}
