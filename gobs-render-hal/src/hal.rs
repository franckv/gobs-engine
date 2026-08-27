use std::{any::Any, sync::Arc};

use slotmap::new_key_type;
use winit::window::Window;

use gobs_core::{
    GobsConfig, ImageExtent2D, ImageFormat, SamplerFilter, data::data_buffer::SliceBuffer,
};

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
    config: GobsConfig,
    validation: bool,
) -> Box<dyn RenderHAL> {
    Box::new(VulkanHAL::new(name, window, config, validation))
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BufferType {
    Vertex,
    Index,
    Instance,
    Staging,
    StagingDst,
    Uniform,
    Storage,
}

pub trait RenderHAL {
    fn new_frame(&mut self, frame_number: usize);
    fn frame_id(&self, frame_number: usize) -> usize;
    fn frames_in_flight(&self) -> usize;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn create_buffer(&mut self, name: &str, size: usize, ty: BufferType) -> Handle;
    fn upload_buffer(&mut self, buffer: Handle, data: &[u8], offset: u64);
    fn upload_buffer_with(
        &mut self,
        buffer: Handle,
        offset: usize,
        len: usize,
        f: &mut dyn FnMut(&mut dyn RenderHAL, &mut SliceBuffer),
    );
    fn get_buffer_address(&self, buffer: Handle) -> u64;
    fn destroy_buffer(&mut self, buffer: Handle);
    fn allocate_material_index(&mut self) -> usize;
    fn update_material_data(&mut self, index: usize, data: &[u8]);
    fn release_material_index(&mut self, index: usize);
    fn get_material_offset(&self, index: usize) -> Option<u32>;
    fn get_instance_buffer(&self, frame_id: usize) -> Handle;
    fn get_instance_buffer_size(&self) -> usize;
    fn allocate_instance(&mut self, frame_id: usize, size: usize) -> usize;
    fn update_instance_data(&mut self, frame_id: usize, offset: u64, data: &[u8]);

    fn create_image(
        &mut self,
        name: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Handle;
    fn invalidate_image(&mut self, image: Handle);
    fn get_image_extent(&self, image: Handle) -> ImageExtent2D;
    fn destroy_image(&mut self, image: Handle);
    fn allocate_texture_index(&mut self) -> usize;
    fn update_texture_index(&mut self, index: usize, image: Handle);
    fn release_texture_index(&mut self, index: usize);
    fn max_textures(&self) -> usize;

    fn create_sampler(&mut self, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Handle;
    fn destroy_sampler(&mut self, sampler: Handle);

    fn create_command_buffer(&mut self, name: &str, ty: CommandQueueType)
    -> Box<dyn CommandBuffer>;

    fn create_graphics_pipeline(&self, name: &str) -> Box<dyn GraphicsPipelineBuilder>;
    fn create_compute_pipeline(&self, name: &str) -> Box<dyn ComputePipelineBuilder>;
    fn destroy_pipeline(&mut self, pipeline: Handle);

    fn get_pipeline_object_layout(&self, pipeline: Handle) -> Arc<ObjectDataLayout>;
    fn get_pipeline_push_layout(&self, pipeline: Handle) -> Arc<ObjectDataLayout>;
    fn get_pipeline_descriptor_types(&self, pipeline: Handle) -> Vec<BindingGroupType>;
    fn get_pipeline_descriptor_layout(
        &self,
        pipeline: Handle,
        binding_group_type: &BindingGroupType,
    ) -> Option<Arc<BindingGroupLayout>>;
    fn get_pipeline_vertex_attributes(&self, pipeline: Handle) -> VertexAttribute;

    fn acquire(&mut self, frame: usize) -> Result<(), RenderBackendError>;
    fn present(&mut self) -> Result<(), RenderBackendError>;
    fn resize(&mut self);
    fn request_redraw(&mut self);
    fn is_minimized(&self) -> bool;
    fn lock_mouse(&mut self, lock: bool);

    fn get_render_target(&self) -> Option<Handle>;
    fn get_extent(&self) -> ImageExtent2D;

    fn wait(&mut self);

    fn info(&self);
}
