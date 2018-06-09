use std::sync::Arc;

use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, Features, PhysicalDevice};
use vulkano_win;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, WindowBuilder};

use display::Display;

pub struct Context {
    device: Arc<Device>,
    queue: Arc<Queue>
}

impl Context {
    fn new(device: Arc<Device>, queue: Arc<Queue>) -> Arc<Context> {
        Arc::new(Context {
            device: device,
            queue: queue
        })
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
}

pub fn init() -> (EventsLoop, Arc<Context>, Arc<Display>) {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("error")
    };

    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

    let physical = PhysicalDevice::enumerate(&instance).next().expect("error");

    info!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    let queue_family = physical.queue_families().find(|&q| {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).expect("error");

    let (device, mut queues) = {
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };
        Device::new(physical, &Features::none(), &device_ext,
                    [(queue_family, 0.5)].iter().cloned()).expect("error")
    };

    let queue = queues.next().unwrap();

    let context = Context::new(device, queue);

    let display = Display::new(surface);

    (events_loop, context, display)
}
