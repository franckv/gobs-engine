use serde::{Deserialize, Serialize};

use gobs_render_hal::{AlignMode, Attribute, UniformLayout};

use crate::UniformData;

// TODO: Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialDataPropData {
    DiffuseColor([f32; 4]),
    EmissionColor([f32; 4]),
    SpecularColor([f32; 4]),
    SpecularPower(f32),
    DiffuseIndex(u32),
    NormalIndex(u32),
    EmissionIndex(u32),
    SpecularIndex(u32),
}

// TODO: Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialDataProp {
    DiffuseColor,
    EmissionColor,
    SpecularColor,
    SpecularPower,
    DiffuseIndex,
    NormalIndex,
    EmissionIndex,
    SpecularIndex,
}

#[derive(Clone, Debug)]
pub struct MaterialDataLayout {
    layout: Vec<MaterialDataProp>,
    uniform_layout: UniformLayout,
    pub(crate) material_indexing: bool,
}

impl MaterialDataLayout {
    pub fn new(material_indexing: bool) -> Self {
        let mode = if material_indexing {
            AlignMode::Std430
        } else {
            AlignMode::Std140
        };

        Self {
            layout: Vec::new(),
            uniform_layout: UniformLayout::new(mode),
            material_indexing,
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
            MaterialDataProp::DiffuseIndex => {
                self.uniform_layout.prop("diffuse index", Attribute::U32)
            }
            MaterialDataProp::NormalIndex => {
                self.uniform_layout.prop("diffuse index", Attribute::U32)
            }
            MaterialDataProp::EmissionIndex => {
                self.uniform_layout.prop("diffuse index", Attribute::U32)
            }
            MaterialDataProp::SpecularIndex => {
                self.uniform_layout.prop("diffuse index", Attribute::U32)
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

#[derive(Clone, Debug, Default)]
pub struct MaterialConstantData {
    pub diffuse_color: [f32; 4],
    pub emission_color: [f32; 4],
    pub specular_color: [f32; 4],
    pub specular_power: f32,
    pub diffuse_index: u32,
    pub normal_index: u32,
    pub emission_index: u32,
    pub specular_index: u32,
}
