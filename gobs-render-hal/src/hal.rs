use std::any::Any;

use slotmap::new_key_type;
use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat, SamplerFilter};

use crate::{
    BindingGroupLayout, BindingGroupType, CommandQueueType, ImageUsage, ObjectDataLayout,
    RenderBackendError, VertexAttribute,
    backend::VulkanHAL,
    command::CommandBuffer,
    pipeline::{ComputePipelineBuilder, GraphicsPipelineBuilder},
};

new_key_type! { pub struct Handle; }

pub fn create_hal(
    name: &str,
    window: Option<Window>,
    frames_in_flight: usize,
    validation: bool,
) -> Box<dyn RenderHAL> {
    Box::new(VulkanHAL::new(name, window, frames_in_flight, validation))
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BufferType {
    Vertex,
    Index,
    Staging,
    StagingDst,
    Uniform,
}

pub trait RenderHAL {
    fn new_frame(&mut self, frame_number: usize);

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn create_buffer(&mut self, name: &str, size: usize, ty: BufferType) -> Handle;
    fn upload_buffer(&mut self, buffer: Handle, data: &[u8], offset: u64);
    fn resize_buffer(&mut self, buffer: Handle, size: usize);
    fn get_buffer_address(&self, buffer: Handle) -> u64;

    fn create_image(
        &mut self,
        name: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Handle;
    fn create_sampler(&mut self, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Handle;
    fn invalidate_image(&mut self, image: Handle);
    fn get_image_extent(&self, image: Handle) -> ImageExtent2D;

    fn create_command_buffer(&mut self, name: &str, ty: CommandQueueType)
    -> Box<dyn CommandBuffer>;

    fn create_graphics_pipeline(&self, name: &str) -> Box<dyn GraphicsPipelineBuilder>;
    fn create_compute_pipeline(&self, name: &str) -> Box<dyn ComputePipelineBuilder>;

    fn get_pipeline_object_layout(&self, pipeline: Handle) -> &ObjectDataLayout;
    fn get_pipeline_descriptor_types(&self, pipeline: Handle) -> Vec<BindingGroupType>;
    fn get_pipeline_descriptor_layout(
        &self,
        pipeline: Handle,
        binding_group_type: &BindingGroupType,
    ) -> Option<&BindingGroupLayout>;
    fn get_pipeline_vertex_attributes(&self, pipeline: Handle) -> VertexAttribute;

    fn acquire(&mut self, frame: usize) -> Result<(), RenderBackendError>;
    fn present(&mut self) -> Result<(), RenderBackendError>;
    fn resize(&mut self);
    fn request_redraw(&mut self);
    fn is_minimized(&self) -> bool;

    fn get_render_target(&self) -> Option<Handle>;
    fn get_extent(&self) -> ImageExtent2D;

    fn wait(&mut self);
}
