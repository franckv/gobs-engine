use std::ptr;
use std::sync::Arc;

use ash::vk;
use serde::{Deserialize, Serialize};

use crate::device::Device;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DescriptorType {
    Uniform,
    UniformDynamic,
    ImageSampler,
    StorageImage,
    Sampler,
    SampledImage,
}

impl From<DescriptorType> for vk::DescriptorType {
    fn from(val: DescriptorType) -> Self {
        match val {
            DescriptorType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::UniformDynamic => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            DescriptorType::ImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            DescriptorType::Sampler => vk::DescriptorType::SAMPLER,
            DescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DescriptorStage {
    Compute,
    Vertex,
    Fragment,
    All,
}

impl From<DescriptorStage> for vk::ShaderStageFlags {
    fn from(val: DescriptorStage) -> Self {
        match val {
            DescriptorStage::Compute => vk::ShaderStageFlags::COMPUTE,
            DescriptorStage::Vertex => vk::ShaderStageFlags::VERTEX,
            DescriptorStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            DescriptorStage::All => vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        }
    }
}

#[derive(Debug)]
pub struct DescriptorSetLayoutBinding {
    pub ty: DescriptorType,
    pub stage: DescriptorStage,
}

pub struct DescriptorSetLayoutBuilder {
    bindings: Vec<DescriptorSetLayoutBinding>,
    set: u32,
}

impl DescriptorSetLayoutBuilder {
    fn new(set: u32) -> Self {
        DescriptorSetLayoutBuilder {
            bindings: Vec::new(),
            set,
        }
    }

    pub fn binding(mut self, ty: DescriptorType, stage: DescriptorStage) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding { ty, stage });

        self
    }

    pub fn build(mut self, device: Arc<Device>, push: bool) -> Arc<DescriptorSetLayout> {
        let mut bindings = Vec::new();
        bindings.append(&mut self.bindings);

        DescriptorSetLayout::new(device, bindings, self.set, push)
    }
}

#[derive(Debug)]
pub struct DescriptorSetLayout {
    device: Arc<Device>,
    pub(crate) layout: vk::DescriptorSetLayout,
    pub bindings: Vec<DescriptorSetLayoutBinding>,
    pub set: u32,
}

impl DescriptorSetLayout {
    pub fn builder(set: u32) -> DescriptorSetLayoutBuilder {
        DescriptorSetLayoutBuilder::new(set)
    }

    fn new(
        device: Arc<Device>,
        bindings: Vec<DescriptorSetLayoutBinding>,
        set: u32,
        push: bool,
    ) -> Arc<Self> {
        let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
            .iter()
            .enumerate()
            .map(|(idx, binding)| vk::DescriptorSetLayoutBinding {
                binding: idx as u32,
                descriptor_type: binding.ty.into(),
                descriptor_count: 1,
                p_immutable_samplers: ptr::null(),
                stage_flags: binding.stage.into(),
                _marker: std::marker::PhantomData,
            })
            .collect();

        let mut descriptor_info = vk::DescriptorSetLayoutCreateInfo::default();

        if !vk_bindings.is_empty() {
            descriptor_info = descriptor_info.bindings(&vk_bindings);
        }

        if push {
            descriptor_info =
                descriptor_info.flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR);
        }

        let layout = unsafe {
            device
                .raw()
                .create_descriptor_set_layout(&descriptor_info, None)
                .unwrap()
        };

        Arc::new(DescriptorSetLayout {
            device,
            layout,
            bindings,
            set,
        })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop DescriptorSetLayout");

        unsafe {
            self.device
                .raw()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
