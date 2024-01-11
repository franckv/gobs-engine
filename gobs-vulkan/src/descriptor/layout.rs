use std::ptr;
use std::sync::Arc;

use ash::vk;

use crate::device::Device;

#[derive(Copy, Clone)]
pub enum DescriptorType {
    Uniform,
    UniformDynamic,
    ImageSampler,
    StorageImage,
}

impl Into<vk::DescriptorType> for DescriptorType {
    fn into(self) -> vk::DescriptorType {
        match self {
            DescriptorType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::UniformDynamic => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            DescriptorType::ImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
        }
    }
}

#[derive(Copy, Clone)]
pub enum DescriptorStage {
    Compute,
    Vertex,
    Fragment,
    All,
}

impl Into<vk::ShaderStageFlags> for DescriptorStage {
    fn into(self) -> vk::ShaderStageFlags {
        match self {
            DescriptorStage::Compute => vk::ShaderStageFlags::COMPUTE,
            DescriptorStage::Vertex => vk::ShaderStageFlags::VERTEX,
            DescriptorStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            DescriptorStage::All => vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        }
    }
}

pub struct DescriptorSetLayoutBinding {
    pub ty: DescriptorType,
    pub stage: DescriptorStage,
}

pub struct DescriptorSetLayoutBuilder {
    bindings: Vec<DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayoutBuilder {
    pub fn new() -> Self {
        DescriptorSetLayoutBuilder {
            bindings: Vec::new(),
        }
    }

    pub fn binding(mut self, ty: DescriptorType, stage: DescriptorStage) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding { ty, stage });

        self
    }

    pub fn build(mut self, device: Arc<Device>) -> Arc<DescriptorSetLayout> {
        let mut bindings = Vec::new();
        bindings.append(&mut self.bindings);

        DescriptorSetLayout::new(device, bindings)
    }
}

pub struct DescriptorSetLayout {
    device: Arc<Device>,
    pub(crate) layout: vk::DescriptorSetLayout,
    pub bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayout {
    pub fn new(device: Arc<Device>, bindings: Vec<DescriptorSetLayoutBinding>) -> Arc<Self> {
        let bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
            .iter()
            .enumerate()
            .map(|(idx, binding)| vk::DescriptorSetLayoutBinding {
                binding: idx as u32,
                descriptor_type: binding.ty.into(),
                descriptor_count: 1,
                p_immutable_samplers: ptr::null(),
                stage_flags: binding.stage.into(),
            })
            .collect();

        let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
        };

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
        })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        log::debug!("Drop DescriptorSetLayout");

        unsafe {
            self.device
                .raw()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
