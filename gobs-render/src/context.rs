use std::sync::Arc;

use gobs_core::entity::uniform::{UniformLayout, UniformProp};
use winit::window::Window;

use gobs_vulkan as vk;
use vk::{
    alloc::Allocator,
    command::{CommandBuffer, CommandPool},
    device::Device,
    image::ImageFormat,
    instance::Instance,
    queue::Queue,
    surface::Surface,
};

pub struct Context {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub immediate_cmd: CommandBuffer,
    pub allocator: Arc<Allocator>,
    pub color_format: ImageFormat,
    pub depth_format: ImageFormat,
    pub frames_in_flight: usize,
    pub push_layout: Arc<UniformLayout>,
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
        let immediate_cmd = CommandBuffer::new(
            device.clone(),
            queue.clone(),
            immediate_cmd_pool,
            "Immediate",
        );

        let allocator = Allocator::new(device.clone());

        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("normal_matrix", UniformProp::Mat3F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        Context {
            instance,
            device,
            queue,
            surface,
            immediate_cmd,
            allocator,
            color_format: ImageFormat::R16g16b16a16Sfloat,
            depth_format: ImageFormat::D32Sfloat,
            frames_in_flight: 2,
            push_layout,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log::debug!("Drop context");
    }
}
