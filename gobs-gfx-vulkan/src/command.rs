use gobs_core::ImageExtent2D;
use gobs_gfx::{Command, ImageLayout};
use gobs_vulkan as vk;

use crate::{display::VkDisplay, VkBindingGroup, VkBuffer, VkDevice, VkImage, VkPipeline};

pub struct VkCommand {
    pub(crate) command: vk::command::CommandBuffer,
}

impl Command for VkCommand {
    type GfxBindingGroup = VkBindingGroup;
    type GfxBuffer = VkBuffer;
    type GfxDevice = VkDevice;
    type GfxDisplay = VkDisplay;
    type GfxImage = VkImage;
    type GfxPipeline = VkPipeline;

    fn new(device: &VkDevice, name: &str) -> Self {
        let command_pool =
            vk::command::CommandPool::new(device.device.clone(), &device.queue.family);
        let command = vk::command::CommandBuffer::new(
            device.device.clone(),
            device.queue.clone(),
            command_pool,
            name,
        );

        Self { command }
    }

    fn begin(&self) {
        self.command.begin();
    }

    fn end(&self) {
        self.command.end();
    }

    fn begin_label(&self, label: &str) {
        self.command.begin_label(label);
    }

    fn end_label(&self) {
        self.command.end_label();
    }

    fn copy_buffer(&self, src: &VkBuffer, dst: &VkBuffer, size: usize, offset: usize) {
        self.command
            .copy_buffer(&src.buffer, &dst.buffer, size, offset);
    }

    fn copy_buffer_to_image(&self, src: &VkBuffer, dst: &VkImage, width: u32, height: u32) {
        self.command
            .copy_buffer_to_image(&src.buffer, &dst.image, width, height);
    }

    fn begin_rendering(
        &self,
        color: Option<&VkImage>,
        extent: ImageExtent2D,
        depth: Option<&VkImage>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    ) {
        self.command.begin_rendering(
            color.map(|image| &image.image),
            extent,
            depth.map(|image| &image.image),
            color_clear,
            depth_clear,
            clear_color,
            depth_clear_color,
        );
    }

    fn end_rendering(&self) {
        self.command.end_rendering();
    }

    fn transition_image_layout(&self, image: &mut VkImage, layout: ImageLayout) {
        self.command
            .transition_image_layout(&mut image.image, layout);
    }

    fn copy_image_to_image(
        &self,
        src: &VkImage,
        src_size: ImageExtent2D,
        dst: &VkImage,
        dst_size: ImageExtent2D,
    ) {
        self.command
            .copy_image_to_image(&src.image, src_size, &dst.image, dst_size);
    }

    fn push_constants(&self, pipeline: &VkPipeline, constants: &[u8]) {
        self.command
            .push_constants(pipeline.pipeline.layout.clone(), constants);
    }

    fn set_viewport(&self, width: u32, height: u32) {
        self.command.set_viewport(width, height);
    }

    fn bind_pipeline(&self, pipeline: &VkPipeline) {
        tracing::debug!("Binding pipeline {}", pipeline.name);
        self.command.bind_pipeline(&pipeline.pipeline);
    }

    fn bind_resource(&self, binding_group: &VkBindingGroup) {
        let set = binding_group.bind_group_type.set();
        self.command
            .bind_descriptor_set(&binding_group.ds, set, &binding_group.pipeline.pipeline);
    }

    fn bind_resource_buffer(&self, buffer: &VkBuffer, pipeline: &VkPipeline) {
        vk::descriptor::DescriptorSetUpdates::new(self.command.device.clone())
            .bind_buffer(&buffer.buffer, 0, buffer.buffer.size)
            .push_descriptors(&self.command, &pipeline.pipeline, 0);
    }

    fn bind_index_buffer(&self, buffer: &VkBuffer, offset: usize) {
        self.command
            .bind_index_buffer::<u32>(&buffer.buffer, offset);
    }

    fn dispatch(&self, x: u32, y: u32, z: u32) {
        self.command.dispatch(x, y, z);
    }

    fn draw_indexed(&self, index_count: usize, instance_count: usize) {
        self.command.draw_indexed(index_count, instance_count);
    }

    fn reset(&mut self) {
        self.command.fence.wait();

        if self.command.fence.signaled() {
            self.command.fence.reset();
            debug_assert!(!self.command.fence.signaled());
        } else {
            tracing::warn!("Fence unsignaled");
        }
        self.command.reset();
    }

    fn submit2(&self, display: &VkDisplay, frame: usize) {
        tracing::trace!("Submit with semaphore {}", frame);
        let swapchain_semaphore = Some(&display.swapchain_semaphores[frame]);
        let render_semaphore = Some(&display.render_semaphores[frame]);
        self.command.submit2(swapchain_semaphore, render_semaphore);
    }
}
