use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::buffer::Buffer;
use crate::command::CommandPool;
use crate::descriptor::DescriptorSet;
use crate::device::Device;
use crate::framebuffer::Framebuffer;
use crate::image::{Image, ImageLayout};
use crate::pipeline::Pipeline;
use crate::queue::Queue;
use crate::sync::{Semaphore, Fence};
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
        let buffer_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: pool.raw(),
            command_buffer_count: 1,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffer = unsafe {
            device.raw().allocate_command_buffers(&buffer_info).unwrap()[0]
        };

        CommandBuffer {
            device,
            pool,
            command_buffer,
        }
    }

    pub fn begin(&mut self) {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            p_inheritance_info: ptr::null(),
        };

        unsafe {
            self.device.raw().begin_command_buffer(
                self.command_buffer, &begin_info).unwrap();
        }
    }

    pub fn start_render_pass(&mut self, framebuffer: &Framebuffer) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.]
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.,
                    stencil: 0
                }
            }
        ];

        let dim = framebuffer.dimensions();

        let renderpass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: framebuffer.renderpass().raw(),
            framebuffer: framebuffer.raw(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: dim.0,
                    height: dim.1,
                },
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self.device.raw().cmd_begin_render_pass(
                self.command_buffer, &renderpass_info,
                vk::SubpassContents::INLINE);
        }
    }

    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        unsafe {
            self.device.raw().cmd_bind_pipeline(
                self.command_buffer, vk::PipelineBindPoint::GRAPHICS,
                pipeline.raw());
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
                &offsets)
        }
    }

    pub fn bind_index_buffer<T: IndexType>(&self, buffer: &Buffer<T>) {
        unsafe {
            self.device.raw().cmd_bind_index_buffer(
                self.command_buffer,
                buffer.raw(),
                0,
                T::get_index_type())
        }
    }

    pub fn bind_descriptor_set(&self, set: &DescriptorSet, pipeline: &Pipeline,
                               offsets: Vec<u32>) {
        let sets = [set.raw()];

        unsafe {
            self.device.raw().cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout.raw(), 0, &sets,
                offsets.as_ref());
        }
    }

    pub fn draw(&self, vertex_count: usize) {
        unsafe {
            self.device.raw().cmd_draw(
                self.command_buffer,
                vertex_count as u32,
                1,
                0,
                0);
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
                0);
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
            self.device.raw().cmd_set_viewport(
                self.command_buffer, 0, &[viewports]);
            self.device.raw().cmd_set_scissor(
                self.command_buffer, 0, &[scissors]);
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
                &[copy_info]);
        }
    }

    pub fn copy_buffer_to_image<T: Copy>(&self, src: &Buffer<T>, dst: &Image,
                                         width: u32, height: u32) {
        let copy_info = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D {
                x: 0,
                y: 0,
                z: 0,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        };

        unsafe {
            self.device.raw().cmd_copy_buffer_to_image(
                self.command_buffer,
                src.raw(),
                dst.raw(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy_info]);
        }
    }

    pub fn transition_image_layout(&self, image: &Image,
                                   src_layout: ImageLayout,
                                   dst_layout: ImageLayout) {
        let barrier_info = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            old_layout: src_layout.into(),
            new_layout: dst_layout.into(),
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: image.raw(),
            src_access_mask: src_layout.into(),
            dst_access_mask: dst_layout.into(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: image.usage.into(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        unsafe {
            self.device.raw().cmd_pipeline_barrier(
                self.command_buffer,
                src_layout.into(),
                dst_layout.into(),
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_info]);
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device.raw().cmd_end_render_pass(self.command_buffer);
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.raw().end_command_buffer(self.command_buffer).unwrap();
        }
    }

    pub fn submit(&self, queue: &Queue,
                  wait: Option<&Semaphore>,
                  signal: &Semaphore,
                  fence: &Fence) {
        let mut wait_semaphores = Vec::new();
        let mut wait_stages = Vec::new();

        if let Some(semaphore) = wait {
            wait_semaphores.push(semaphore.raw());
            wait_stages.push(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);
        }

        let command_buffers = [self.command_buffer];

        let signal_semaphores = [signal.raw()];

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: if wait_semaphores.is_empty() { ptr::null() } else { wait_semaphores.as_ptr() },
            p_wait_dst_stage_mask: if wait_stages.is_empty() { ptr::null() } else { wait_stages.as_ptr() },
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        };

        unsafe {
            self.device.raw().queue_submit(
                queue.queue, &[submit_info],
                fence.raw()).unwrap();
        }
    }

    pub fn submit_now(&self, queue: &Queue,
                      wait: Option<&Semaphore>) {
        let wait_command = Semaphore::new(queue.device());
        let fence = Fence::new(queue.device(), false);

        self.submit(queue, wait, &wait_command, &fence);

        queue.wait();
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        trace!("Drop command buffer");
        let buffers = [self.command_buffer];

        unsafe {
            self.device.raw().free_command_buffers(
                self.pool.raw(),
                &buffers);
        }
    }
}
