use std::sync::Arc;

use winit::window::Window;

use crate::backend::command::{CommandBuffer, CommandPool};
use crate::backend::device::Device;
use crate::backend::instance::Instance;
use crate::backend::queue::Queue;
use crate::backend::renderpass::RenderPass;
use crate::backend::surface::Surface;

use super::display::Display;

pub struct Context {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Queue,
    command_pool: Arc<CommandPool>,
}

impl Context {
    pub fn new(name: &str, window: Window) -> (Arc<Self>, Display) {
        let instance = Instance::new(name, 0);

        let surface = Surface::new(instance.clone(), window);

        let p_device = instance.find_adapter(&surface);

        info!("Using adapter: {}", p_device.name);

        let family = instance.find_family(&p_device, &surface).unwrap();

        let format = Display::get_surface_format(&surface,
                                                 &p_device);

        let device = Device::new(instance.clone(),
                                 p_device, family.clone());

        let renderpass = RenderPass::new(
            device.clone(), format.format);

        let queue = Queue::new(device.clone());

        let command_pool = CommandPool::new(
            device.clone(),
            &family);

        let context = Arc::new(Context {
            instance,
            device,
            queue,
            command_pool
        });

        let display = Display::new(context.clone(),
                                   surface.clone(),
                                   format,
                                   renderpass.clone());

        (context, display)
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
