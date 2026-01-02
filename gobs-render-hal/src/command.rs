use gobs_core::ImageExtent2D;

use crate::{BindResource, Handle, ImageLayout, RenderHAL};

pub enum CommandQueueType {
    Graphics,
    Compute,
    Transfer,
}

pub trait CommandBuffer {
    fn begin(&self);
    fn end(&self);
    fn begin_label(&self, label: &str);
    fn end_label(&self);
    #[allow(clippy::too_many_arguments)]
    fn begin_rendering(
        &self,
        hal: &dyn RenderHAL,
        color: Option<Handle>,
        extent: ImageExtent2D,
        depth: Option<Handle>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    );
    fn end_rendering(&self);
    fn copy_buffer_to_buffer(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        dst: Handle,
        size: usize,
        src_offset: u64,
        dst_offset: u64,
    );
    fn copy_buffer_to_image(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        dst: Handle,
        offset: u64,
        dst_size: ImageExtent2D,
    );
    fn copy_image_to_buffer(&self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64);
    fn copy_image_to_image(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        src_size: ImageExtent2D,
        dst: Handle,
        dst_size: ImageExtent2D,
    );
    fn dispatch(&self, x: u32, y: u32, z: u32);
    fn draw_indexed(&self, index_count: usize, instance_count: usize);
    fn bind_pipeline(&self, hal: &dyn RenderHAL, pipeline: Handle);
    fn bind_index_buffer(&self, hal: &dyn RenderHAL, buffer: Handle);
    fn bind_resource(&self, hal: &mut dyn RenderHAL, pipeline: Handle, resource: &BindResource);
    fn push_constants(&self, hal: &dyn RenderHAL, pipeline: Handle, constants: &[u8]);
    fn reset(&self);
    fn run_immediate(&self, label: &str, callback: &dyn Fn(&dyn CommandBuffer));
    fn run_immediate_mut(&self, label: &str, callback: &mut dyn FnMut(&dyn CommandBuffer));
    fn set_viewport(&self, width: u32, height: u32);
    fn submit2(&self, hal: &dyn RenderHAL, frame: usize);
    fn transition_image_layout(&self, hal: &mut dyn RenderHAL, image: Handle, layout: ImageLayout);
}
