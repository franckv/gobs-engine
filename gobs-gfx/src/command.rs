use crate::{ImageExtent2D, ImageLayout};

pub trait Command {
    type GfxBindingGroup;
    type GfxBuffer;
    type GfxDevice;
    type GfxDisplay;
    type GfxImage;
    type GfxPipeline;

    fn new(device: &Self::GfxDevice, name: &str) -> Self;
    fn begin(&self);
    fn end(&self);
    fn begin_label(&self, label: &str);
    fn end_label(&self);
    fn copy_buffer(&self, src: &Self::GfxBuffer, dst: &Self::GfxBuffer, size: usize, offset: usize);
    fn copy_buffer_to_image(
        &self,
        src: &Self::GfxBuffer,
        dst: &Self::GfxImage,
        width: u32,
        height: u32,
    );
    fn begin_rendering(
        &self,
        color: Option<&Self::GfxImage>,
        extent: ImageExtent2D,
        depth: Option<&Self::GfxImage>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    );
    fn end_rendering(&self);
    fn transition_image_layout(&self, image: &mut Self::GfxImage, layout: ImageLayout);
    fn copy_image_to_image(
        &self,
        src: &Self::GfxImage,
        src_size: ImageExtent2D,
        dst: &Self::GfxImage,
        dst_size: ImageExtent2D,
    );
    fn push_constants(&self, pipeline: &Self::GfxPipeline, constants: &[u8]);
    fn set_viewport(&self, width: u32, height: u32);
    fn bind_pipeline(&self, pipeline: &Self::GfxPipeline);
    fn bind_resource(&self, binding_group: &Self::GfxBindingGroup);
    fn bind_resource_buffer(&self, buffer: &Self::GfxBuffer, pipeline: &Self::GfxPipeline);
    fn bind_index_buffer(&self, buffer: &Self::GfxBuffer, offset: usize);
    fn dispatch(&self, x: u32, y: u32, z: u32);
    fn draw_indexed(&self, index_count: usize, instance_count: usize);
    fn reset(&mut self);
    fn submit2(&self, display: &Self::GfxDisplay, frame: usize);
}
