use std::{io::pipe, sync::Arc};

use gobs_core::{ImageExtent2D, logger};
use gobs_vulkan::{self as vk, descriptor::DescriptorSetUpdates};

use crate::{
    BindResource, BindingGroupLayout, CommandQueueType, Handle, ImageLayout, RenderHAL,
    UniformData as _,
    backend::{
        VulkanHAL, VulkanHALExt,
        vulkan::pipeline::{self, VkPipeline},
    },
    command::CommandBuffer,
};

pub struct VkCommandBuffer {
    pub(crate) command: vk::CommandBuffer,
    pub frame_number: usize,
    pub fence: vk::sync::Fence,
}

impl VkCommandBuffer {
    pub fn new(device: Arc<vk::Device>, name: &str, queue: Arc<vk::Queue>) -> Self {
        let command_pool = vk::CommandPool::new(device.clone(), &queue.family);

        let command = vk::CommandBuffer::new(device.clone(), queue, command_pool, name);

        VkCommandBuffer {
            command,
            frame_number: 0,
            fence: vk::sync::Fence::new(device.clone(), true, "Command buffer"),
        }
    }
}

impl CommandBuffer for VkCommandBuffer {
    fn begin(&mut self, frame_number: usize) {
        self.command.begin();
        self.frame_number = frame_number;
    }

    fn end(&mut self) {
        self.command.end();
    }

    fn begin_label(&mut self, label: &str) {
        self.command.begin_label(label);
    }

    fn end_label(&mut self) {
        self.command.end_label();
    }

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

    fn end_rendering(&mut self) {
        self.command.end_rendering();
    }

    fn copy_buffer_to_buffer(
        &mut self,
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

    fn copy_buffer_to_image(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64) {
        let hal = hal.get();

        let src = hal.registry.buffers.get(src).unwrap();
        let dst = hal.registry.images.get(dst).unwrap();

        self.command
            .copy_buffer_to_image(&src.buffer, dst, src.offset + offset);
    }

    fn copy_image_to_buffer(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle, offset: u64) {
        let hal = hal.get();

        let src = hal.registry.images.get(src).unwrap();
        let dst = &hal.registry.buffers.get(dst).unwrap();
        self.command
            .copy_image_to_buffer(src, &dst.buffer, dst.offset + offset);
    }

    fn copy_image_to_image(&mut self, hal: &dyn RenderHAL, src: Handle, dst: Handle) {
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
            self.command.copy_image_to_image_blit(src, dst);
        } else {
            tracing::debug!(target: logger::RENDER, "Copy from {:?} to {:?}", src.format, dst.format);
            self.command.copy_image_to_image(src, dst);
        }
    }

    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.command.dispatch(x, y, z);
    }

    fn draw_indexed(&mut self, index_count: usize, instance_count: usize) {
        self.command.draw_indexed(index_count, instance_count);
    }

    fn bind_pipeline(&mut self, hal: &dyn RenderHAL, pipeline: Handle) {
        let hal = hal.get();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap().pipeline;

        tracing::debug!(target: logger::RENDER, "Binding pipeline {}", &pipeline.label);
        self.command.bind_pipeline(pipeline);
    }

    fn bind_index_buffer(&mut self, hal: &dyn RenderHAL, buffer: Handle) {
        let hal = hal.get();

        let index_view = hal.registry.buffers.get(buffer).unwrap();
        self.command
            .bind_index_buffer::<u32>(&index_view.buffer, index_view.offset);
    }

    fn bind_resource(
        &mut self,
        hal: &mut dyn RenderHAL,
        pipeline: Handle,
        resource: &BindResource,
    ) {
        let mut hal = hal.get_mut();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap();

        let binding_type = resource.layout.binding_group_type;

        Self::validate_layout(pipeline, &resource.layout);

        if resource.layout.binding_group_type.is_push() {
            hal.bindings.push_descriptor(
                hal.device.clone(),
                &hal.registry,
                resource,
                &pipeline.pipeline,
                &self.command,
            );
        } else {
            let frame_id = hal.frame_id(self.frame_number);
            let ds = hal
                .bindings
                .get_ds(hal.device.clone(), &hal.registry, resource, frame_id);

            let set = binding_type.set();
            self.command
                .bind_descriptor_set(&ds, set, &pipeline.pipeline);
        }
    }

    fn push_constants(&mut self, hal: &dyn RenderHAL, pipeline: Handle, constants: &[u8]) {
        let mut hal = hal.get();

        let pipeline = &hal.registry.pipelines.get(pipeline).unwrap();

        debug_assert!(constants.len() == pipeline.push_layout.uniform_layout().size());

        self.command
            .push_constants(pipeline.pipeline.layout.clone(), constants);
    }

    fn wait(&self) {
        self.fence.wait();
    }

    fn reset(&mut self) {
        if self.fence.signaled() {
            self.fence.reset();
            debug_assert!(!self.fence.signaled());
        } else {
            tracing::warn!(target: logger::SYNC, "Fence unsignaled");
        }
        self.command.reset();
    }

    fn run_immediate(&mut self, label: &str, callback: &dyn Fn(&dyn CommandBuffer)) {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None, &self.fence);

        self.wait();
    }

    fn run_immediate_mut(&mut self, label: &str, callback: &mut dyn FnMut(&mut dyn CommandBuffer)) {
        self.reset();

        self.command.begin();
        self.command.begin_label(label);
        callback(self);
        self.command.end_label();
        self.command.end();
        self.command.submit2(None, None, &self.fence);

        self.wait();
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
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

        self.command.submit2(wait, signal, &self.fence);
    }

    fn transition_image_layout(
        &mut self,
        hal: &mut dyn RenderHAL,
        image: Handle,
        layout: ImageLayout,
    ) {
        let mut hal = hal.get_mut();

        let image = hal.registry.images.get_mut(image).unwrap();

        self.command.transition_image_layout(image, layout);
    }
}

impl VkCommandBuffer {
    fn validate_layout(pipeline: &VkPipeline, resource_layout: &BindingGroupLayout) {
        tracing::debug!(target: logger::RENDER, "Bind resource to pipeline {}", pipeline.pipeline.label);
        tracing::debug!(target: logger::RENDER, "Bind descriptors layout {:?}", resource_layout);
        tracing::debug!(target: logger::RENDER, "Bind descriptors set {:?}", resource_layout.binding_group_type);
        tracing::debug!(target: logger::RENDER, "Pipeline descriptors set count {:?}", pipeline.pipeline.layout.descriptor_layouts.len());

        debug_assert!(
            resource_layout.binding_group_type.set()
                < pipeline.pipeline.layout.descriptor_layouts.len() as u32,
            "Pipeline descriptor layout does not have set {}",
            resource_layout.binding_group_type.set()
        );

        debug_assert!(
            pipeline
                .descriptor_layout
                .get(&resource_layout.binding_group_type)
                .is_some_and(|layout| layout.bindings.len() == resource_layout.bindings.len()),
            "BindResource layout is not compatible with pipeline for set {}",
            resource_layout.binding_group_type.set()
        );
    }
}
