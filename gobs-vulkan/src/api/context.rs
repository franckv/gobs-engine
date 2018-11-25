use std::sync::Arc;

use backend::command::{CommandBuffer, CommandPool};
use backend::device::Device;
use backend::instance::Instance;
use backend::queue::Queue;
use backend::surface::Surface;

pub struct Context {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Queue,
    command_pool: Arc<CommandPool>,
}

impl Context {
    pub fn new(instance: Arc<Instance>, surface: &Arc<Surface>) -> Arc<Self> {
        let p_device = instance.find_adapter(&surface);

        info!("Using adapter: {}", p_device.name);

        let family = instance.find_family(&p_device, &surface).unwrap();

        let device = Device::new(instance.clone(),
                                 p_device, family.clone());

        let queue = Queue::new(device.clone());

        let command_pool = CommandPool::new(
            device.clone(),
            &family);

        Arc::new(Context {
            instance,
            device,
            queue,
            command_pool
        })
    }

    pub fn command_pool(&self) -> Arc<CommandPool> { self.command_pool.clone() }

    pub fn command_pool_ref(&self) -> &Arc<CommandPool> { &self.command_pool }

    pub fn instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn device_ref(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn get_command_buffer(&self) -> CommandBuffer {
        CommandBuffer::new(
            self.device.clone(),
            self.command_pool.clone())
    }
}
