use std::sync::Arc;

use crate::{
    alloc::Allocator,
    command::{CommandBuffer, CommandPool},
    device::Device,
    instance::Instance,
    queue::Queue,
};

pub struct Context {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub immediate_cmd: CommandBuffer,
    pub allocator: Arc<Allocator>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        let instance = Instance::new(name, 1);
        let physical_device = instance.find_headless_adapter();

        log::info!("Using adapter {}", physical_device.name);

        let queue_family = instance.find_headless_family(&physical_device).unwrap();

        let device = Device::new(instance.clone(), physical_device, &queue_family);

        let queue = Queue::new(device.clone(), queue_family);

        let immediate_cmd_pool = CommandPool::new(device.clone(), &queue.family);
        let immediate_cmd = CommandBuffer::new(device.clone(), queue.clone(), immediate_cmd_pool);

        let allocator = Allocator::new(device.clone());

        Context {
            instance,
            device,
            queue,
            immediate_cmd,
            allocator,
        }
    }
}
