use serde::{Deserialize, Serialize};

use crate::{BindingGroupLayout, BindingGroupType, DescriptorStage, DescriptorType};

// TODO: Emissive, Specular, Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TextureDataProp {
    Diffuse,
    Normal,
    Emission,
    Specular,
}

#[derive(Clone, Debug, Default)]
pub struct TextureDataLayout {
    pub layout: Vec<TextureDataProp>,
}

impl TextureDataLayout {
    pub fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }

    pub fn bindings_layout(&self) -> BindingGroupLayout {
        let mut layout = BindingGroupLayout::new(BindingGroupType::MaterialTextures);

        for _ in &self.layout {
            layout = layout
                .add_binding(DescriptorType::SampledImage, DescriptorStage::Fragment)
                .add_binding(DescriptorType::Sampler, DescriptorStage::Fragment);
        }

        layout
    }

    pub fn prop(mut self, prop: TextureDataProp) -> Self {
        self.layout.push(prop);

        self
    }
}
