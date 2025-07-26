use std::ffi::CString;
use std::sync::Arc;

use ash::vk::{self, Extent3D, ImageSubresourceLayers, Offset3D};
use bytemuck::Pod;

use gobs_core::{ImageExtent2D, logger};

use crate::buffers::Buffer;
use crate::command::CommandPool;
use crate::descriptor::DescriptorSet;
use crate::device::Device;
use crate::framebuffer::Framebuffer;
use crate::images::{Image, ImageLayout};
use crate::pipelines::{Pipeline, PipelineLayout, PipelineStage};
use crate::query::QueryPool;
use crate::queue::Queue;
use crate::sync::{Fence, Semaphore};
use crate::{Wrap, debug};

pub trait IndexType: Copy {
    fn get_index_type() -> vk::IndexType;
    fn size() -> usize;
}

impl IndexType for u32 {
    fn get_index_type() -> vk::IndexType {
        vk::IndexType::UINT32
    }
    fn size() -> usize {
        4
    }
}

/// Store command to be executed by a device
pub struct CommandBuffer {
    pub device: Arc<Device>,
    queue: Arc<Queue>,
    pool: Arc<CommandPool>,
    command_buffer: vk::CommandBuffer,
    pub fence: Fence,
}

impl CommandBuffer {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pool: Arc<CommandPool>,
        label: &str,
    ) -> Self {
        let buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(pool.raw())
            .level(vk::CommandBufferLevel::PRIMARY);

        let mut command_buffers =
            unsafe { device.raw().allocate_command_buffers(&buffer_info).unwrap() };

        assert!(command_buffers.len() == 1);

        let command_buffer = command_buffers.remove(0);

        let command_label = format!("[Command Buffer] {label}");

        debug::add_label(device.clone(), &command_label, command_buffer);

        CommandBuffer {
            device: device.clone(),
            queue,
            pool,
            command_buffer,
            fence: Fence::new(device, true, "Command buffer"),
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device
                .raw()
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap()
        };
    }

    pub fn begin(&self) {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .raw()
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap();
        }
    }

    pub fn begin_label(&self, label: &str) {
        let label = CString::new(label).unwrap();
        let label_info = vk::DebugUtilsLabelEXT::default().label_name(&label);

        unsafe {
            self.device
                .debug_utils_device
                .cmd_begin_debug_utils_label(self.command_buffer, &label_info);
        }
    }

    pub fn end_label(&self) {
        unsafe {
            self.device
                .debug_utils_device
                .cmd_end_debug_utils_label(self.command_buffer);
        }
    }

    pub fn insert_label(&self, label: &str) {
        let label = CString::new(label).unwrap();
        let label_info = vk::DebugUtilsLabelEXT::default().label_name(&label);

        unsafe {
            self.device
                .debug_utils_device
                .cmd_insert_debug_utils_label(self.command_buffer, &label_info);
        }
    }

    pub fn clear_color(&self, image: &Image, color: [f32; 4]) {
        let color_value = vk::ClearColorValue { float32: color };

        unsafe {
            self.device.raw().cmd_clear_color_image(
                self.command_buffer,
                image.raw(),
                vk::ImageLayout::GENERAL,
                &color_value,
                &[vk::ImageSubresourceRange {
                    aspect_mask: image.usage.into(),
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }],
            )
        }
    }

    pub fn begin_rendering(
        &self,
        color: Option<&Image>,
        extent: ImageExtent2D,
        depth: Option<&Image>,
        color_clear: bool,
        depth_clear: bool,
        clear_color: [f32; 4],
        depth_clear_color: f32,
    ) {
        let color_load_op = if color_clear {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        };

        let mut color_info = vec![];
        if let Some(color) = color {
            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(color.image_view)
                .image_layout(color.layout.into())
                .load_op(color_load_op)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: clear_color,
                    },
                });
            color_info.push(color_attachment);
        }

        let depth_load_op = if depth_clear {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        };

        let mut depth_info = vec![];
        if let Some(depth) = depth {
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(depth.image_view)
                .image_layout(depth.layout.into())
                .load_op(depth_load_op)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: depth_clear_color,
                        stencil: 0,
                    },
                });
            depth_info.push(depth_attachment);
        }

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D::default().extent(vk::Extent2D {
                width: extent.width,
                height: extent.height,
            }))
            .layer_count(1);

        let rendering_info = match color_info.first() {
            Some(color_attachment) => {
                rendering_info.color_attachments(std::slice::from_ref(color_attachment))
            }
            None => rendering_info,
        };

        let rendering_info = match depth_info.first() {
            Some(depth_attachment) => rendering_info.depth_attachment(depth_attachment),
            None => rendering_info,
        };

        unsafe {
            self.device
                .raw()
                .cmd_begin_rendering(self.command_buffer, &rendering_info);
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.device.raw().cmd_end_rendering(self.command_buffer);
        }
    }

    pub fn start_render_pass(&self, framebuffer: &Framebuffer) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.,
                    stencil: 0,
                },
            },
        ];

        let dim = framebuffer.dimensions();

        let renderpass_info = vk::RenderPassBeginInfo::default()
            .render_pass(framebuffer.renderpass().raw())
            .framebuffer(framebuffer.raw())
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: dim.width,
                    height: dim.height,
                },
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.raw().cmd_begin_render_pass(
                self.command_buffer,
                &renderpass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        unsafe {
            self.device.raw().cmd_bind_pipeline(
                self.command_buffer,
                pipeline.bind_point,
                pipeline.raw(),
            );
        }
    }

    pub fn bind_vertex_buffer<T: Copy>(&self, binding: usize, buffer: &Buffer) {
        let bindings = [buffer.raw()];
        let offsets = [0];

        unsafe {
            self.device.raw().cmd_bind_vertex_buffers(
                self.command_buffer,
                binding as u32,
                &bindings,
                &offsets,
            )
        }
    }

    pub fn bind_index_buffer<T: IndexType>(&self, buffer: &Buffer, offset: usize) {
        let index_size = T::size();

        unsafe {
            self.device.raw().cmd_bind_index_buffer(
                self.command_buffer,
                buffer.raw(),
                (index_size * offset) as vk::DeviceSize,
                T::get_index_type(),
            )
        }
    }

    pub fn bind_descriptor_set(&self, set: &DescriptorSet, first_set: u32, pipeline: &Pipeline) {
        unsafe {
            self.device.raw().cmd_bind_descriptor_sets(
                self.command_buffer,
                pipeline.bind_point,
                pipeline.layout.raw(),
                first_set,
                &[set.raw()],
                &[],
            );
        }
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe { self.device.raw().cmd_dispatch(self.command_buffer, x, y, z) }
    }

    pub fn draw(&self, vertex_count: usize) {
        unsafe {
            self.device
                .raw()
                .cmd_draw(self.command_buffer, vertex_count as u32, 1, 0, 0);
        }
    }

    pub fn draw_indexed(&self, index_count: usize, instance_count: usize) {
        unsafe {
            self.device.raw().cmd_draw_indexed(
                self.command_buffer,
                index_count as u32,
                instance_count as u32,
                0,
                0,
                0,
            );
        }
    }

    pub fn push_constants<T: Pod>(&self, layout: Arc<PipelineLayout>, constants: &[T]) {
        unsafe {
            self.device.raw().cmd_push_constants(
                self.command_buffer,
                layout.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(constants),
            );
        }
    }

    pub fn reset_query_pool(&self, pool: &QueryPool, first_query: u32, query_count: u32) {
        unsafe {
            self.device.raw().cmd_reset_query_pool(
                self.command_buffer,
                pool.pool,
                first_query,
                query_count,
            );
        }
    }

    pub fn write_timestamp(&self, pool: &QueryPool, stage: PipelineStage, query: u32) {
        unsafe {
            self.device.raw().cmd_write_timestamp(
                self.command_buffer,
                stage.into(),
                pool.pool,
                query,
            );
        }
    }

    pub fn set_viewport(&self, width: u32, height: u32) {
        let viewports = vk::Viewport {
            x: 0.,
            y: 0.,
            width: width as f32,
            height: height as f32,
            min_depth: 0.,
            max_depth: 1.,
        };

        let scissors = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width, height },
        };

        unsafe {
            self.device
                .raw()
                .cmd_set_viewport(self.command_buffer, 0, &[viewports]);
            self.device
                .raw()
                .cmd_set_scissor(self.command_buffer, 0, &[scissors]);
        }
    }

    pub fn copy_buffer(&self, src: &Buffer, dst: &Buffer, size: usize, offset: usize) {
        let copy_info = vk::BufferCopy::default()
            .src_offset(offset as u64)
            .dst_offset(0)
            .size(size as u64);

        unsafe {
            self.device.raw().cmd_copy_buffer(
                self.command_buffer,
                src.raw(),
                dst.raw(),
                &[copy_info],
            );
        }
    }

    pub fn copy_image_to_buffer(&self, src: &Image, dst: &Buffer) {
        let copy_info = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_image_height(src.extent.height)
            .buffer_row_length(src.extent.width)
            .image_offset(Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(Extent3D {
                width: src.extent.width,
                height: src.extent.height,
                depth: 1,
            })
            .image_subresource(
                ImageSubresourceLayers::default()
                    .aspect_mask(src.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            );

        unsafe {
            self.device.raw().cmd_copy_image_to_buffer(
                self.command_buffer,
                src.raw(),
                src.layout.into(),
                dst.raw(),
                &[copy_info],
            );
        }
    }

    pub fn copy_image_to_image(&self, src: &Image, dst: &Image) {
        assert!(src.format.pixel_size() == dst.format.pixel_size());

        let copy_region = vk::ImageCopy2::default()
            .src_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(src.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(dst.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            )
            .extent(Extent3D {
                width: src.extent.width,
                height: src.extent.height,
                depth: 1,
            });

        let copy_info = vk::CopyImageInfo2::default()
            .dst_image(dst.raw())
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_image(src.raw())
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .regions(std::slice::from_ref(&copy_region));

        unsafe {
            self.device
                .raw()
                .cmd_copy_image2(self.command_buffer, &copy_info);
        }
    }

    pub fn copy_image_to_image_blit(
        &self,
        src: &Image,
        src_size: ImageExtent2D,
        dst: &Image,
        dst_size: ImageExtent2D,
    ) {
        tracing::trace!(target: logger::RENDER,
            "Blitting image {}/{} to {}/{}",
            src_size.width,
            src_size.height,
            dst_size.width,
            dst_size.height
        );

        let blit_region = vk::ImageBlit2::default()
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::default()
                    .x(src_size.width as i32)
                    .y(src_size.height as i32)
                    .z(1),
            ])
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::default()
                    .x(dst_size.width as i32)
                    .y(dst_size.height as i32)
                    .z(1),
            ])
            .src_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(src.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(dst.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            );

        let blit_info = vk::BlitImageInfo2::default()
            .dst_image(dst.raw())
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_image(src.raw())
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .filter(vk::Filter::LINEAR)
            .regions(std::slice::from_ref(&blit_region));

        unsafe {
            self.device
                .raw()
                .cmd_blit_image2(self.command_buffer, &blit_info);
        }
    }

    pub fn copy_buffer_to_image(&self, src: &Buffer, dst: &Image, width: u32, height: u32) {
        let image_subresource = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);

        let copy_info = vk::BufferImageCopy::default()
            .image_subresource(image_subresource)
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });

        unsafe {
            self.device.raw().cmd_copy_buffer_to_image(
                self.command_buffer,
                src.raw(),
                dst.raw(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&copy_info),
            );
        }
    }

    pub fn transition_image_layout(&self, image: &mut Image, dst_layout: ImageLayout) {
        tracing::trace!(target: logger::RENDER,
            "Transition [{}] from {:?} to {:?}",
            &image.label,
            image.layout,
            dst_layout
        );

        let barrier_info = vk::ImageMemoryBarrier2::default()
            .old_layout(image.layout.into())
            .new_layout(dst_layout.into())
            .image(image.raw())
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(image.usage.into())
                    .base_mip_level(0)
                    .level_count(vk::REMAINING_MIP_LEVELS)
                    .base_array_layer(0)
                    .layer_count(vk::REMAINING_ARRAY_LAYERS),
            );

        let dep_info = vk::DependencyInfo::default()
            .image_memory_barriers(std::slice::from_ref(&barrier_info));

        unsafe {
            self.device
                .raw()
                .cmd_pipeline_barrier2(self.command_buffer, &dep_info);
        }

        image.layout = dst_layout
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device.raw().cmd_end_render_pass(self.command_buffer);
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device
                .raw()
                .end_command_buffer(self.command_buffer)
                .unwrap();
        }
    }

    pub fn submit2(&self, wait: Option<&Semaphore>, signal: Option<&Semaphore>) {
        let command_info = vk::CommandBufferSubmitInfo::default()
            .command_buffer(self.command_buffer)
            .device_mask(0);

        let mut submit_info =
            vk::SubmitInfo2::default().command_buffer_infos(std::slice::from_ref(&command_info));

        let mut wait_info = Vec::new();
        if let Some(wait) = wait {
            wait_info.push(
                vk::SemaphoreSubmitInfo::default()
                    .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR)
                    .semaphore(wait.raw())
                    .device_index(0)
                    .value(1),
            );

            submit_info = submit_info.wait_semaphore_infos(&wait_info);
        };

        let mut signal_info = Vec::new();
        if let Some(signal) = signal {
            signal_info.push(
                vk::SemaphoreSubmitInfo::default()
                    .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
                    .semaphore(signal.raw())
                    .device_index(0)
                    .value(1),
            );

            submit_info = submit_info.signal_semaphore_infos(&signal_info);
        };

        unsafe {
            self.device
                .raw()
                .queue_submit2(
                    self.queue.queue,
                    std::slice::from_ref(&submit_info),
                    self.fence.raw(),
                )
                .unwrap();
        }
    }

    pub fn immediate<F>(&self, callback: F)
    where
        F: Fn(&CommandBuffer),
    {
        tracing::debug!(target: logger::RENDER, "Submit immediate command");
        self.fence.reset();
        assert!(!self.fence.signaled());

        self.reset();

        self.begin();

        callback(self);

        self.end();

        self.submit2(None, None);

        self.fence.wait();
        tracing::debug!(target: logger::RENDER, "Immediate command done");
    }

    pub fn immediate_mut<F>(&self, mut callback: F)
    where
        F: FnMut(&CommandBuffer),
    {
        tracing::debug!(target: logger::RENDER, "Submit immediate command");
        self.fence.reset();
        assert!(!self.fence.signaled());

        self.reset();

        self.begin();

        callback(self);

        self.end();

        self.submit2(None, None);

        self.fence.wait();
        tracing::debug!(target: logger::RENDER, "Immediate command done");
    }
}

impl Wrap<vk::CommandBuffer> for CommandBuffer {
    fn raw(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop command buffer");

        unsafe {
            self.device
                .raw()
                .free_command_buffers(self.pool.raw(), &[self.command_buffer]);
        }
    }
}
