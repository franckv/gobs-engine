use gobs_core::{ImageExtent2D, logger};
use gobs_vulkan::{self as vk, descriptor::DescriptorSetUpdates};

use crate::{
    BindResource, BindingGroupLayout, Handle, ImageLayout, RenderHAL,
    backend::{VulkanHAL, VulkanHALExt, vulkan::pipeline},
    command::CommandBuffer,
};

pub struct VkCommandBuffer {
    pub(crate) command: vk::CommandBuffer,
}

impl CommandBuffer for VkCommandBuffer {
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
    ) {
        let hal = hal.get();

        self.command.begin_rendering(
            color.and_then(|image| hal.registry.images.get(image)),
            extent,
            depth.and_then(|image| hal.registry.images.get(image)),
            color_clear,
            depth_clear,
            clear_color,
            depth_clear_color,
        );
    }

    fn end_rendering(&self) {
        self.command.end_rendering();
    }

    fn copy_buffer_to_buffer(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        dst: Handle,
        size: usize,
        src_offset: u64,
        dst_offset: u64,
    ) {
        let hal = hal.get();
        let src = hal.registry.buffers.get(src).unwrap();
        let dst = hal.registry.buffers.get(dst).unwrap();

        self.command.copy_buffer(
            &src.buffer,
            &dst.buffer,
            size,
            src.offset + src_offset,
            dst.offset + dst_offset,
        );
    }

    fn copy_buffer_to_image(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        dst: Handle,
        offset: u64,
        dst_size: ImageExtent2D,
    ) {
        let hal = hal.get();

        let src = hal.registry.buffers.get(src).unwrap();
        let dst = hal.registry.images.get(dst).unwrap();

        self.command.copy_buffer_to_image(
            &src.buffer,
            dst,
            src.offset + offset,
            dst_size.width,
            dst_size.height,
        );
    }

    fn copy_image_to_buffer(&self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64) {
        let hal = hal.get();

        let src = hal.registry.images.get(src).unwrap();
        let dst = &hal.registry.buffers.get(dst).unwrap();
        self.command
            .copy_image_to_buffer(src, &dst.buffer, dst.offset + offset);
    }

    fn copy_image_to_image(
        &self,
        hal: &dyn RenderHAL,
        src: Handle,
        src_size: ImageExtent2D,
        dst: Handle,
        dst_size: ImageExtent2D,
    ) {
        let hal = hal.get();

        let src = hal.registry.images.get(src).unwrap();
        let dst = hal.registry.images.get(dst).unwrap();

        if self
            .command
            .device
            .support_blit(src.format, src.usage, true)
            && self
                .command
                .device
                .support_blit(dst.format, dst.usage, false)
        {
            tracing::debug!(target: logger::RENDER, "Blit from {:?} to {:?}", src.format, dst.format);
            self.command
                .copy_image_to_image_blit(src, src_size, dst, dst_size);
        } else {
            tracing::debug!(target: logger::RENDER, "Copy from {:?} to {:?}", src.format, dst.format);
            self.command.copy_image_to_image(src, dst);
        }
    }

    fn dispatch(&self, x: u32, y: u32, z: u32) {
        self.command.dispatch(x, y, z);
    }

    fn draw_indexed(&self, index_count: usize, instance_count: usize) {
        self.command.draw_indexed(index_count, instance_count);
    }

    fn bind_pipeline(&self, hal: &dyn RenderHAL, pipeline: Handle) {
        let hal = hal.get();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap().pipeline;

        tracing::debug!(target: logger::RENDER, "Binding pipeline {}", &pipeline.label);
        self.command.bind_pipeline(pipeline);
    }

    fn bind_index_buffer(&self, hal: &dyn RenderHAL, buffer: Handle) {
        let hal = hal.get();

        let index_view = hal.registry.buffers.get(buffer).unwrap();
        self.command
            .bind_index_buffer::<u32>(&index_view.buffer, index_view.offset);
    }

    fn bind_resource(&self, hal: &mut dyn RenderHAL, pipeline: Handle, resource: &BindResource) {
        let mut hal = hal.get_mut();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap().pipeline;

        let binding_type = resource.layout.binding_group_type;

        tracing::debug!(target: logger::RENDER, "Bind resource to pipeline {}", pipeline.label);
        tracing::debug!(target: logger::RENDER, "Bind descriptors layout {:?}", resource.layout);
        tracing::debug!(target: logger::RENDER, "Bind descriptors set {:?}", binding_type);
        tracing::debug!(target: logger::RENDER, "Pipeline descriptors set {:?}", pipeline.layout.descriptor_layouts.len());

        debug_assert!(binding_type.set() < pipeline.layout.descriptor_layouts.len() as u32);

        if resource.layout.binding_group_type.is_push() {
            hal.bindings.push_descriptor(
                hal.device.clone(),
                &hal.registry,
                resource,
                pipeline,
                &self.command,
            );
        } else {
            let ds = hal
                .bindings
                .allocate_ds(hal.device.clone(), &hal.registry, resource);

            let set = binding_type.set();
            self.command.bind_descriptor_set(&ds, set, pipeline);
        }
    }

    fn push_constants(&self, hal: &dyn RenderHAL, pipeline: Handle, constants: &[u8]) {
        let mut hal = hal.get();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap().pipeline;

        self.command
            .push_constants(pipeline.layout.clone(), constants);
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

    fn run_immediate(&self, label: &str, callback: &dyn Fn(&dyn CommandBuffer)) {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None);

        self.command.fence.wait();
    }

    fn run_immediate_mut(&self, label: &str, callback: &mut dyn FnMut(&dyn CommandBuffer)) {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None);

        self.command.fence.wait();
    }

    fn set_viewport(&self, width: u32, height: u32) {
        self.command.set_viewport(width, height);
    }

    fn submit2(&self, hal: &dyn RenderHAL, frame: usize) {
        let hal = hal.get();

        let swapchain_idx = hal.display.swapchain_idx;
        tracing::trace!(target: logger::SYNC, "Submit with swapchain semaphore: {}, render semaphore: {}", frame, swapchain_idx);
        let (wait, signal) = if hal.display.swapchain.is_some() {
            let wait = Some(&hal.display.swapchain_semaphores[frame]);
            let signal = Some(&hal.display.render_semaphores[swapchain_idx]);

            (wait, signal)
        } else {
            (None, None)
        };

        self.command.submit2(wait, signal);
    }

    fn transition_image_layout(&self, hal: &mut dyn RenderHAL, image: Handle, layout: ImageLayout) {
        let mut hal = hal.get_mut();

        let image = hal.registry.images.get_mut(image).unwrap();

        self.command.transition_image_layout(image, layout);
    }
}
