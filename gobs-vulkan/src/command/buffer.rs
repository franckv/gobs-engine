use std::sync::Arc;

use ash::vk::{self, AttachmentLoadOp, AttachmentStoreOp, RenderingAttachmentInfo, RenderingInfo};

use crate::buffer::Buffer;
use crate::command::CommandPool;
use crate::descriptor::DescriptorSet;
use crate::device::Device;
use crate::framebuffer::Framebuffer;
use crate::image::{Image, ImageExtent2D, ImageLayout};
use crate::pipeline::Pipeline;
use crate::queue::Queue;
use crate::sync::{Fence, Semaphore};
use crate::Wrap;

pub trait IndexType: Copy {
    fn get_index_type() -> vk::IndexType;
}

impl IndexType for u32 {
    fn get_index_type() -> vk::IndexType {
        vk::IndexType::UINT32
    }
}

/// Store command to be executed by a device
pub struct CommandBuffer {
    device: Arc<Device>,
    pool: Arc<CommandPool>,
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(device: Arc<Device>, pool: Arc<CommandPool>) -> Self {
        let buffer_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pool.raw())
            .level(vk::CommandBufferLevel::PRIMARY);

        let mut command_buffers =
            unsafe { device.raw().allocate_command_buffers(&buffer_info).unwrap() };

        assert!(command_buffers.len() == 1);

        CommandBuffer {
            device,
            pool,
            command_buffer: command_buffers.remove(0),
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
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.device
                .raw()
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap();
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
        color: &Image,
        extent: ImageExtent2D,
        depth: Option<&Image>,
        clear: bool,
        clear_color: [f32; 4],
    ) {
        let color_load_op = if clear {
            AttachmentLoadOp::CLEAR
        } else {
            AttachmentLoadOp::LOAD
        };

        let rendering_info = RenderingInfo::builder()
            .render_area(vk::Rect2D::builder().extent(extent.into()).build())
            .layer_count(1)
            .color_attachments(&[RenderingAttachmentInfo::builder()
                .image_view(color.image_view)
                .image_layout(color.layout.into())
                .load_op(color_load_op)
                .store_op(AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: clear_color,
                    },
                })
                .build()])
            .build();

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

        let renderpass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(framebuffer.renderpass().raw())
            .framebuffer(framebuffer.raw())
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: dim.into(),
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

    pub fn bind_vertex_buffer<T: Copy>(&self, binding: usize, buffer: &Buffer<T>) {
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

    pub fn bind_index_buffer<T: IndexType>(&self, buffer: &Buffer<T>) {
        unsafe {
            self.device.raw().cmd_bind_index_buffer(
                self.command_buffer,
                buffer.raw(),
                0,
                T::get_index_type(),
            )
        }
    }

    pub fn bind_descriptor_set(&self, set: &DescriptorSet, pipeline: &Pipeline) {
        unsafe {
            self.device.raw().cmd_bind_descriptor_sets(
                self.command_buffer,
                pipeline.bind_point,
                pipeline.layout.raw(),
                0,
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

    pub fn copy_buffer<T: Copy>(&self, src: &Buffer<T>, dst: &Buffer<T>, size: usize) {
        let copy_info = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: size as u64,
        };

        unsafe {
            self.device.raw().cmd_copy_buffer(
                self.command_buffer,
                src.raw(),
                dst.raw(),
                &[copy_info],
            );
        }
    }

    pub fn copy_image_to_image(
        &self,
        src: &Image,
        src_size: ImageExtent2D,
        dst: &Image,
        dst_size: ImageExtent2D,
    ) {
        log::trace!(
            "Blitting image {}/{} to {}/{}",
            src_size.width,
            src_size.height,
            dst_size.width,
            dst_size.height
        );

        let blit_region = vk::ImageBlit2::builder()
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder()
                    .x(src_size.width as i32)
                    .y(src_size.height as i32)
                    .z(1)
                    .build(),
            ])
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder()
                    .x(dst_size.width as i32)
                    .y(dst_size.height as i32)
                    .z(1)
                    .build(),
            ])
            .src_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(src.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0)
                    .build(),
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(dst.usage.into())
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0)
                    .build(),
            )
            .build();

        let blit_info = vk::BlitImageInfo2::builder()
            .dst_image(dst.raw())
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_image(src.raw())
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .filter(vk::Filter::LINEAR)
            .regions(&[blit_region])
            .build();

        unsafe {
            self.device
                .raw()
                .cmd_blit_image2(self.command_buffer, &blit_info);
        }
    }

    pub fn copy_buffer_to_image<T: Copy>(
        &self,
        src: &Buffer<T>,
        dst: &Image,
        width: u32,
        height: u32,
    ) {
        let image_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);

        let copy_info = vk::BufferImageCopy::builder()
            .image_subresource(*image_subresource)
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
        let barrier_info = vk::ImageMemoryBarrier2::builder()
            .old_layout(image.layout.into())
            .new_layout(dst_layout.into())
            .image(image.raw())
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(image.usage.into())
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        let dep_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&[barrier_info])
            .build();

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

    pub fn submit(&self, queue: &Queue, wait: &Semaphore, signal: &Semaphore, fence: &Fence) {
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[self.command_buffer])
            .wait_semaphores(&[wait.raw()])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .signal_semaphores(&[signal.raw()])
            .build();

        unsafe {
            self.device
                .raw()
                .queue_submit(queue.queue, &[submit_info], fence.raw())
                .unwrap();
        }
    }

    pub fn submit2(&self, queue: &Queue, wait: &Semaphore, signal: &Semaphore, fence: &Fence) {
        let command_info = vk::CommandBufferSubmitInfo::builder()
            .command_buffer(self.command_buffer)
            .device_mask(0)
            .build();

        let wait_info = vk::SemaphoreSubmitInfo::builder()
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR)
            .semaphore(wait.raw())
            .device_index(0)
            .value(1)
            .build();

        let signal_info = vk::SemaphoreSubmitInfo::builder()
            .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
            .semaphore(signal.raw())
            .device_index(0)
            .value(1)
            .build();

        let submit_info = vk::SubmitInfo2::builder()
            .command_buffer_infos(&[command_info])
            .wait_semaphore_infos(&[wait_info])
            .signal_semaphore_infos(&[signal_info])
            .build();

        unsafe {
            self.device
                .raw()
                .queue_submit2(queue.queue, &[submit_info], fence.raw())
                .unwrap();
        }
    }

    pub fn submit_now(&self, queue: &Queue, wait: &Semaphore) {
        let wait_command = Semaphore::new(queue.device());
        let fence = Fence::new(queue.device(), false);

        self.submit(queue, wait, &wait_command, &fence);

        queue.wait();
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        log::info!("Drop command buffer");

        let buffers = [self.command_buffer];

        unsafe {
            self.device
                .raw()
                .free_command_buffers(self.pool.raw(), &buffers);
        }
    }
}
