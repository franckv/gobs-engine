use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use gobs_scene as scene;
use gobs_vulkan as backend;

use scene::model::Transform;

use backend::buffer::{Buffer, BufferUsage};
use backend::command::CommandBuffer;
use backend::device::Device;
use backend::sync::{Fence, Semaphore};

use super::context::Context;

pub struct Frame {
    device: Arc<Device>,
    pub wait_image: Semaphore,
    pub wait_command: Semaphore,
    pub submit_fence: Fence,
    pub command_buffer: CommandBuffer,
    pub view_proj_buffer: Buffer<Transform>,
    instance_buffers: HashMap<Uuid, Buffer<Transform>>,
    pub dirty: bool,
    max_instances: usize
}

impl Frame {
    pub fn new(context: &Context, frame_count: usize,
               max_instances: usize) -> Vec<Self> {
        (0..frame_count).map(|_| {
            let view_proj_buffer = Buffer::new(1, BufferUsage::Uniform,
                                               context.device());
            let instance_buffers = HashMap::new();

            Frame {
                device: context.device(),
                wait_image: Semaphore::new(context.device()),
                wait_command: Semaphore::new(context.device()),
                submit_fence: Fence::new(context.device(), true),
                command_buffer: CommandBuffer::new(
                    context.device(),
                    context.command_pool().clone()),
                view_proj_buffer,
                instance_buffers,
                dirty: true,
                max_instances
            }
        }).collect()
    }

    pub fn instance_buffer_mut(&mut self, id: Uuid) -> &mut Buffer<Transform> {
        log::trace!("Updating instance {}", id);
        if !self.instance_buffers.contains_key(&id) {
            let buffer = Buffer::new(self.max_instances,
                                     BufferUsage::Instance,
                                     self.device.clone());
            self.instance_buffers.insert(id, buffer);
        }
        self.instance_buffers.get_mut(&id).unwrap()
    }

    pub fn instance_buffer(&self, id: &Uuid) -> &Buffer<Transform> {
        log::trace!("Using instance {}", id);
        &self.instance_buffers.get(id).unwrap()
    }
}
