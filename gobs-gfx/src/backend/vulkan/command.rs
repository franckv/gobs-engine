use gobs_core::{ImageExtent2D, logger};
use gobs_vulkan as vk;

use crate::backend::vulkan::{
    bindgroup::VkBindingGroup, buffer::VkBuffer, device::VkDevice, display::VkDisplay,
    image::VkImage, pipeline::VkPipeline, renderer::VkRenderer,
};
use crate::command::CommandQueueType;
use crate::{Command, Image, ImageLayout};

pub struct VkCommand {
    pub(crate) command: vk::command::CommandBuffer,
}

impl Command<VkRenderer> for VkCommand {
    fn new(device: &VkDevice, name: &str, ty: CommandQueueType) -> Self {
        let queue = device.get_queue(ty);

        let command_pool = vk::command::CommandPool::new(device.device.clone(), &queue.family);

        let command =
            vk::command::CommandBuffer::new(device.device.clone(), queue, command_pool, name);

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

    fn copy_buffer(&self, src: &VkBuffer, dst: &mut VkBuffer, size: usize, offset: usize) {
        self.command
            .copy_buffer(&src.buffer, &dst.buffer, size, offset);
    }

    fn copy_buffer_to_image(&self, src: &VkBuffer, dst: &mut VkImage, width: u32, height: u32) {
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
        dst: &mut VkImage,
        dst_size: ImageExtent2D,
    ) {
        if self
            .command
            .device
            .support_blit(src.format(), src.usage(), true)
            && self
                .command
                .device
                .support_blit(dst.format(), dst.usage(), false)
        {
            tracing::debug!(target: logger::RENDER, "Blit from {:?} to {:?}", src.format(), dst.format());
            self.command
                .copy_image_to_image_blit(&src.image, src_size, &dst.image, dst_size);
        } else {
            tracing::debug!(target: logger::RENDER, "Copy from {:?} to {:?}", src.format(), dst.format());
            self.command.copy_image_to_image(&src.image, &dst.image);
        }
    }

    fn copy_image_to_buffer(&self, src: &VkImage, dst: &mut VkBuffer) {
        self.command.copy_image_to_buffer(&src.image, &dst.buffer);
    }

    fn push_constants(&self, pipeline: &VkPipeline, constants: &[u8]) {
        self.command
            .push_constants(pipeline.pipeline.layout.clone(), constants);
    }

    fn set_viewport(&self, width: u32, height: u32) {
        self.command.set_viewport(width, height);
    }

    fn bind_pipeline(&self, pipeline: &VkPipeline) {
        tracing::debug!(target: logger::RENDER, "Binding pipeline {}", pipeline.name);
        self.command.bind_pipeline(&pipeline.pipeline);
    }

    fn bind_resource(&self, binding_group: &VkBindingGroup, pipeline: &VkPipeline) {
        let set = binding_group.bind_group_type.set();
        self.command
            .bind_descriptor_set(&binding_group.ds, set, &pipeline.pipeline);
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

    fn reset(&self) {
        self.command.fence.wait();

        if self.command.fence.signaled() {
            self.command.fence.reset();
            debug_assert!(!self.command.fence.signaled());
        } else {
            tracing::warn!(target: logger::SYNC, "Fence unsignaled");
        }
        self.command.reset();
    }

    fn run_immediate<F>(&self, label: &str, callback: F)
    where
        F: Fn(&VkCommand),
    {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None);

        self.command.fence.wait();
    }

    fn run_immediate_mut<F>(&self, label: &str, mut callback: F)
    where
        F: FnMut(&VkCommand),
    {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None);

        self.command.fence.wait();
    }

    fn submit2(&self, display: &VkDisplay, frame: usize) {
        let swapchain_idx = display.swapchain_idx;
        tracing::trace!(target: logger::SYNC, "Submit with swapchain semaphore: {}, render semaphore: {}", frame, swapchain_idx);
        let (wait, signal) = if display.swapchain.is_some() {
            let wait = Some(&display.swapchain_semaphores[frame]);
            let signal = Some(&display.render_semaphores[swapchain_idx]);

            (wait, signal)
        } else {
            (None, None)
        };

        self.command.submit2(wait, signal);
    }
}
