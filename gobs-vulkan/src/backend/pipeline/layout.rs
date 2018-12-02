use std::sync::Arc;

use ash::vk;

#[derive(Copy, Clone)]
pub enum PipelineLayoutBindingType {
    Uniform,
    UniformDynamic,
    ImageSampler,
}

impl Into<vk::DescriptorType> for PipelineLayoutBindingType {
    fn into(self) -> vk::DescriptorType {
        match self {
            PipelineLayoutBindingType::Uniform =>
                vk::DescriptorType::UNIFORM_BUFFER,
            PipelineLayoutBindingType::UniformDynamic =>
                vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            PipelineLayoutBindingType::ImageSampler =>
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PipelineLayoutBindingStage {
    Vertex,
    Fragment,
    All
}

impl Into<vk::ShaderStageFlags> for PipelineLayoutBindingStage {
    fn into(self) -> vk::ShaderStageFlags {
        match self {
            PipelineLayoutBindingStage::Vertex =>
                vk::ShaderStageFlags::VERTEX,
            PipelineLayoutBindingStage::Fragment =>
                vk::ShaderStageFlags::FRAGMENT,
            PipelineLayoutBindingStage::All =>
                vk::ShaderStageFlags::VERTEX |
                    vk::ShaderStageFlags::FRAGMENT
        }
    }
}

pub struct PipelineLayoutBinding {
    pub ty: PipelineLayoutBindingType,
    pub stage: PipelineLayoutBindingStage
}

pub struct PipelineLayoutBuilder {
    bindings: Vec<PipelineLayoutBinding>
}

impl PipelineLayoutBuilder {
    pub fn new() -> Self {
        PipelineLayoutBuilder {
            bindings: Vec::new()
        }
    }

    pub fn binding(mut self, ty: PipelineLayoutBindingType,
                   stage: PipelineLayoutBindingStage) -> Self {
        self.bindings.push(PipelineLayoutBinding {
            ty,
            stage
        });

        self
    }

    pub fn build(mut self) -> Arc<PipelineLayout> {
        let mut bindings = Vec::new();
        bindings.append(&mut self.bindings);

        Arc::new(PipelineLayout {
            bindings
        })
    }
}

pub struct PipelineLayout {
    pub bindings: Vec<PipelineLayoutBinding>
}
