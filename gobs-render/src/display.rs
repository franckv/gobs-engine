use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, Features, PhysicalDevice};
use vulkano::swapchain::{Surface, SurfaceTransform, PresentMode, Swapchain};

use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use winit::{EventsLoop, Window, WindowBuilder};

use render::Renderer;

pub struct Display {
    surface: Arc<Surface<Window>>
}

impl Display {
    fn new(surface: Arc<Surface<Window>>) -> Display {
        Display {
            surface: surface
        }
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        self.surface.window().get_inner_size().unwrap()
    }
}

pub fn init() -> (EventsLoop, Display, Renderer) {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("error")
    };

    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

    let dimensions = {
        let (width, height) = surface.window().get_inner_size().unwrap();
        [width, height]
    };

    let physical = PhysicalDevice::enumerate(&instance).next().expect("error");

    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

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

    let (swapchain, images) = {
        let caps = surface.capabilities(physical).expect("error");
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
        dimensions, 1, caps.supported_usage_flags, &queue, SurfaceTransform::Identity, alpha,
        PresentMode::Fifo, true, None).expect("error")
    };

    let renderer = Renderer::new(device, queue, swapchain, images, dimensions);

    (events_loop, Display::new(surface), renderer)
}
