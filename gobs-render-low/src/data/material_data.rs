use gobs_gfx::{
    BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType, GfxBindingGroupLayout,
};
use serde::{Deserialize, Serialize};

use crate::{
    GfxContext, UniformData,
    data::{UniformLayout, UniformProp},
};

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
    uniform_layout: UniformLayout,
}

impl UniformData<MaterialDataProp, ()> for MaterialDataLayout {
    fn prop(mut self, prop: MaterialDataProp) -> Self {
        self.layout.push(prop);

        match prop {
            MaterialDataProp::DiffuseColor => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("diffuse color", UniformProp::Vec4F)
            }
            MaterialDataProp::EmissionColor => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("emission color", UniformProp::Vec4F)
            }
            MaterialDataProp::SpecularColor => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("specular color", UniformProp::Vec4F)
            }
            MaterialDataProp::SpecularPower => {
                self.uniform_layout = self.uniform_layout.prop("specular power", UniformProp::F32)
            }
        }

        self
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }

    fn copy_data(&self, _ctx: &GfxContext, _: &(), buffer: &mut Vec<u8>) {
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

        layout.copy_data(&props, buffer)
    }

    fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }
}

impl MaterialDataLayout {
    pub fn bindings_layout(&self) -> GfxBindingGroupLayout {
        GfxBindingGroupLayout::new(BindingGroupType::MaterialData)
            .add_binding(DescriptorType::Uniform, DescriptorStage::Fragment)
    }
}
