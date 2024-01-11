use std::sync::Arc;

use gobs_vulkan as vk;
use vk::{
    command::{CommandBuffer, CommandPool},
    device::Device,
    instance::Instance,
    queue::Queue,
    surface::Surface,
};
use winit::window::Window;

pub struct Context {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub immediate_cmd: CommandBuffer,
}

impl Context {
    pub fn new(name: &str, window: Window) -> Self {
        let instance = Instance::new(name, 1);
        let surface = Surface::new(instance.clone(), window);
        let physical_device = instance.find_adapter(&surface);

        log::info!("Using adapter {}", physical_device.name);

        let queue_family = instance.find_family(&physical_device, &surface).unwrap();

        let device = Device::new(instance.clone(), physical_device, &queue_family);

        let queue = Queue::new(device.clone(), queue_family);

        let immediate_cmd_pool = CommandPool::new(device.clone(), &queue.family);
        let immediate_cmd = CommandBuffer::new(device.clone(), queue.clone(), immediate_cmd_pool);

        Context {
            instance,
            device,
            queue,
            surface,
            immediate_cmd,
        }
    }
}
