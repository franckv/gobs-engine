use gobs_core::ImageExtent2D;

use crate::{BindResource, Handle, ImageLayout, RenderHAL};

pub enum CommandQueueType {
    Graphics,
    Compute,
    Transfer,
}

pub trait CommandBuffer {
    fn begin(&mut self, frame_number: usize);
    fn end(&mut self);
    fn begin_label(&mut self, label: &str);
    fn end_label(&mut self);
    #[allow(clippy::too_many_arguments)]
    fn begin_rendering(
        &mut self,
        hal: &dyn RenderHAL,
        color: Option<Handle>,
        extent: ImageExtent2D,
        depth: Option<Handle>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    );
    fn end_rendering(&mut self);
    fn copy_buffer_to_buffer(
        &mut self,
        hal: &dyn RenderHAL,
        src: Handle,
        dst: Handle,
        size: usize,
        src_offset: u64,
        dst_offset: u64,
    );
    fn copy_buffer_to_image(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64);
    fn copy_image_to_buffer(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64);
    fn copy_image_to_image(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle);
    fn dispatch(&mut self, x: u32, y: u32, z: u32);
    fn draw_indexed(&mut self, index_count: usize, instance_count: usize);
    fn bind_pipeline(&mut self, hal: &dyn RenderHAL, pipeline: Handle);
    fn bind_vertex_buffer(&mut self, hal: &dyn RenderHAL, buffer: Handle);
    fn bind_index_buffer(&mut self, hal: &dyn RenderHAL, buffer: Handle);
    fn bind_resource(&mut self, hal: &mut dyn RenderHAL, pipeline: Handle, resource: &BindResource);
    fn push_constants(&mut self, hal: &dyn RenderHAL, pipeline: Handle, constants: &[u8]);
    fn wait(&self);
    fn reset(&mut self);
    fn run_immediate(&mut self, label: &str, callback: &dyn Fn(&dyn CommandBuffer));
    fn run_immediate_mut(&mut self, label: &str, callback: &mut dyn FnMut(&mut dyn CommandBuffer));
    fn set_viewport(&mut self, width: u32, height: u32);
    fn submit2(&self, hal: &dyn RenderHAL, frame: usize);
    fn transition_image_layout(
        &mut self,
        hal: &mut dyn RenderHAL,
        image: Handle,
        layout: ImageLayout,
    );
}
