use serde::{Deserialize, Serialize};

use gobs_render_hal::{
    AlignMode, Attribute, BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType,
    UniformLayout,
};

use crate::UniformData;

// TODO: Emissive, Specular, Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialDataPropData {
    DiffuseColor([f32; 4]),
    EmissionColor([f32; 4]),
    SpecularColor([f32; 4]),
    SpecularPower(f32),
}

// TODO: Emissive, Specular, Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialDataProp {
    DiffuseColor,
    EmissionColor,
    SpecularColor,
    SpecularPower,
}

#[derive(Clone, Debug)]
pub struct MaterialDataLayout {
    layout: Vec<MaterialDataProp>,
    uniform_layout: UniformLayout,
}

impl MaterialDataLayout {
    pub fn new(mode: AlignMode) -> Self {
        Self {
            layout: Vec::new(),
            uniform_layout: UniformLayout::new(mode),
        }
    }
}

impl UniformData<MaterialDataProp> for MaterialDataLayout {
    fn prop(mut self, prop: MaterialDataProp) -> Self {
        self.layout.push(prop);

        self.uniform_layout = match prop {
            MaterialDataProp::DiffuseColor => {
                self.uniform_layout.prop("diffuse color", Attribute::Vec4F)
            }
            MaterialDataProp::EmissionColor => {
                self.uniform_layout.prop("emission color", Attribute::Vec4F)
            }
            MaterialDataProp::SpecularColor => {
                self.uniform_layout.prop("specular color", Attribute::Vec4F)
            }
            MaterialDataProp::SpecularPower => {
                self.uniform_layout.prop("specular power", Attribute::F32)
            }
        };

        self
    }

    fn layout(&self) -> &[MaterialDataProp] {
        &self.layout
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }
}

impl MaterialDataLayout {
    pub fn bindings_layout(&self) -> BindingGroupLayout {
        BindingGroupLayout::new(BindingGroupType::MaterialData).add_binding(
            DescriptorType::Uniform,
            DescriptorStage::Fragment,
            1,
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialConstantData {
    pub diffuse_color: [f32; 4],
    pub emission_color: [f32; 4],
    pub specular_color: [f32; 4],
    pub specular_power: f32,
}
