use std::{ffi::CString, sync::Arc};

use ash::vk;

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

pub enum PipelineStage {
    AllCommands,
    AllGraphics,
    TopOfPipe,
    Compute,
    Vertex,
    Fragment,
    BottomOfPipe,
}

impl Into<vk::PipelineStageFlags> for PipelineStage {
    fn into(self) -> vk::PipelineStageFlags {
        match self {
            PipelineStage::AllCommands => vk::PipelineStageFlags::ALL_COMMANDS,
            PipelineStage::AllGraphics => vk::PipelineStageFlags::ALL_GRAPHICS,
            PipelineStage::TopOfPipe => vk::PipelineStageFlags::TOP_OF_PIPE,
            PipelineStage::Compute => vk::PipelineStageFlags::COMPUTE_SHADER,
            PipelineStage::Vertex => vk::PipelineStageFlags::VERTEX_SHADER,
            PipelineStage::Fragment => vk::PipelineStageFlags::FRAGMENT_SHADER,
            PipelineStage::BottomOfPipe => vk::PipelineStageFlags::BOTTOM_OF_PIPE,
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

    pub fn info(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(match self.shader.ty {
                ShaderType::Compute => vk::ShaderStageFlags::COMPUTE,
                ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
                ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
            })
            .module(self.shader.raw())
            .name(&self.entry)
    }
}

#[derive(Debug)]
pub struct Pipeline {
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
        tracing::debug!(target: "memory", "Drop pipeline");
        unsafe {
            self.device.raw().destroy_pipeline(self.pipeline, None);
        }
    }
}
