use crate::{GfxBindingGroup, GfxBuffer, GfxDevice, GfxImage, GfxPipeline, ImageExtent2D, ImageLayout};

pub trait Command {
    fn new(device: &GfxDevice, name: &str) -> Self;
    fn begin_label(&self, label: &str);
    fn end_label(&self);
    fn copy_buffer(&self, src: &GfxBuffer, dst: &GfxBuffer, size: usize, offset: usize);
    fn copy_buffer_to_image(&self, src: &GfxBuffer, dst: &GfxImage, width: u32, height: u32);
    fn begin_rendering(
        &self,
        color: Option<&GfxImage>,
        extent: ImageExtent2D,
        depth: Option<&GfxImage>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    );
    fn end_rendering(&self);
    fn transition_image_layout(&self, image: &mut GfxImage, layout: ImageLayout);
    fn push_constants(&self, pipeline: &GfxPipeline, constants: &[u8]);
    fn set_viewport(&self, width: u32, height: u32);
    fn bind_pipeline(&self, pipeline: &GfxPipeline);
    fn bind_resource(&self, binding_group: &GfxBindingGroup);
    fn bind_resource_buffer(&self, buffer: &GfxBuffer, pipeline: &GfxPipeline);
    fn bind_index_buffer(&self, buffer: &GfxBuffer, offset: usize);
    fn dispatch(&self, x: u32, y: u32, z: u32);
    fn draw_indexed(&self, index_count: usize, instance_count: usize);
    fn reset(&mut self);
}
