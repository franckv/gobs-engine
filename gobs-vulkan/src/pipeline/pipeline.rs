use std::{ffi::CString, sync::Arc};

use ash::vk;
use uuid::Uuid;

use crate::device::Device;
use crate::pipeline::PipelineLayout;
use crate::Wrap;

use super::{ComputePipelineBuilder, GraphicsPipelineBuilder, Shader, ShaderType};

pub struct Rect2D {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect2D {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Rect2D {
            x,
            y,
            width,
            height,
        }
    }

    pub fn raw(&self) -> vk::Rect2D {
        vk::Rect2D {
            offset: vk::Offset2D {
                x: self.x,
                y: self.y,
            },
            extent: vk::Extent2D {
                width: self.width,
                height: self.height,
            },
        }
    }
}

pub struct ShaderStage {
    entry: CString,
    shader: Shader,
}

impl ShaderStage {
    pub fn new(entry: &str, shader: Shader) -> Self {
        ShaderStage {
            entry: CString::new(entry).unwrap(),
            shader,
        }
    }

    pub fn info(&self) -> vk::PipelineShaderStageCreateInfoBuilder {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(match self.shader.ty {
                ShaderType::Compute => vk::ShaderStageFlags::COMPUTE,
                ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
                ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
            })
            .module(self.shader.raw())
            .name(&self.entry)
    }
}

pub type PipelineId = Uuid;

pub struct Pipeline {
    pub id: PipelineId,
    pub(crate) device: Arc<Device>,
    pub layout: Arc<PipelineLayout>,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) bind_point: vk::PipelineBindPoint,
}

impl Pipeline {
    pub fn graphics_builder(device: Arc<Device>) -> GraphicsPipelineBuilder {
        GraphicsPipelineBuilder::new(device)
    }

    pub fn compute_builder(device: Arc<Device>) -> ComputePipelineBuilder {
        ComputePipelineBuilder::new(device)
    }
}

impl Wrap<vk::Pipeline> for Pipeline {
    fn raw(&self) -> vk::Pipeline {
        self.pipeline
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        log::debug!("Drop pipeline");
        unsafe {
            self.device.raw().destroy_pipeline(self.pipeline, None);
        }
    }
}
