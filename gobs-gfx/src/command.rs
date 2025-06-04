use gobs_core::ImageExtent2D;

use crate::{ImageLayout, Renderer};

pub enum CommandQueueType {
    Graphics,
    Compute,
    Transfer,
}

pub trait Command<R: Renderer> {
    fn new(device: &R::Device, name: &str, ty: CommandQueueType) -> Self;
    fn begin(&self);
    fn end(&self);
    fn begin_label(&self, label: &str);
    fn end_label(&self);
    fn copy_buffer(&self, src: &R::Buffer, dst: &mut R::Buffer, size: usize, offset: usize);
    fn copy_buffer_to_image(&self, src: &R::Buffer, dst: &mut R::Image, width: u32, height: u32);
    fn begin_rendering(
        &self,
        color: Option<&R::Image>,
        extent: ImageExtent2D,
        depth: Option<&R::Image>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    );
    fn end_rendering(&self);
    fn transition_image_layout(&self, image: &mut R::Image, layout: ImageLayout);
    fn copy_image_to_image(
        &self,
        src: &R::Image,
        src_size: ImageExtent2D,
        dst: &mut R::Image,
        dst_size: ImageExtent2D,
    );
    fn copy_image_to_buffer(&self, sec: &R::Image, dst: &mut R::Buffer);
    fn push_constants(&self, pipeline: &R::Pipeline, constants: &[u8]);
    fn set_viewport(&self, width: u32, height: u32);
    fn bind_pipeline(&self, pipeline: &R::Pipeline);
    fn bind_resource(&self, binding_group: &R::BindingGroup);
    fn bind_resource_buffer(&self, buffer: &R::Buffer, pipeline: &R::Pipeline);
    fn bind_index_buffer(&self, buffer: &R::Buffer, offset: usize);
    fn dispatch(&self, x: u32, y: u32, z: u32);
    fn draw_indexed(&self, index_count: usize, instance_count: usize);
    fn reset(&self);
    fn run_immediate<F>(&self, label: &str, callback: F)
    where
        F: Fn(&Self);
    fn run_immediate_mut<F>(&self, label: &str, callback: F)
    where
        F: FnMut(&Self);
    fn submit2(&self, display: &R::Display, frame: usize);
}
