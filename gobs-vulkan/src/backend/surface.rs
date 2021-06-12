use std;
use std::sync::Arc;

use winit::window::Window;

use ash::vk;

use crate::backend::device::Device;
use crate::backend::image::{ColorSpace, ImageFormat};
use crate::backend::instance::Instance;
use crate::backend::physical::PhysicalDevice;
use crate::backend::queue::QueueFamily;
use crate::backend::swapchain::PresentationMode;
use crate::backend::Wrap;

#[derive(Copy, Clone, Debug)]
pub struct SurfaceFormat {
    pub format: ImageFormat,
    pub color_space: ColorSpace,
}

pub struct SurfaceCapabilities {
    pub min_image_count: usize,
    pub max_image_count: usize,
    pub width: u32,
    pub height: u32,
}

pub struct Surface {
    instance: Arc<Instance>,
    window: Window,
    surface: vk::SurfaceKHR
}

impl Surface {
    pub fn new(instance: Arc<Instance>, window: Window) -> Arc<Self> {
        let surface = unsafe {
            ash_window::create_surface(&instance.entry, &instance.instance, &window, None).unwrap()
        };

        Arc::new(Surface {
            instance: instance,
            window,
            surface,
        })
    }

    pub fn family_supported(&self, p_device: &PhysicalDevice,
                            family: &QueueFamily) -> bool {
        unsafe {
            self.instance.surface_loader.get_physical_device_surface_support(
            p_device.raw(), family.index, self.surface).unwrap()
        }
    }

    pub fn get_available_format(&self, p_device: &PhysicalDevice) -> Vec<SurfaceFormat> {
        let mut results = Vec::new();

        let formats = unsafe {
            self.instance.surface_loader
                .get_physical_device_surface_formats(
                    p_device.raw(), self.surface).unwrap()
        };

        for format in formats {
            let format = SurfaceFormat {
                format: format.format.into(),
                color_space: format.color_space.into(),
            };
            results.push(format);
        }

        results
    }

    pub fn get_available_presentation_modes(&self, device: &Arc<Device>)
        -> Vec<PresentationMode> {
            let mut results = Vec::new();

            let presents = unsafe {
                self.instance.surface_loader
                    .get_physical_device_surface_present_modes(
                        device.p_device.raw(), self.surface).unwrap()
            };

            for present in presents {
                let mode: PresentationMode = present.into();
                results.push(mode);
            }

            results
        }

    pub fn get_capabilities(&self, device: &Arc<Device>) -> SurfaceCapabilities {
        let capabilities = unsafe {
            self.instance.surface_loader
                .get_physical_device_surface_capabilities(
                    device.p_device.raw(), self.surface).unwrap()
        };

        SurfaceCapabilities {
            min_image_count: capabilities.min_image_count as usize,
            max_image_count: capabilities.max_image_count as usize,
            width: capabilities.current_extent.width,
            height: capabilities.current_extent.height,
        }
    }

    pub fn dpi(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        let dim = self.window.inner_size();
        
        dim.into()
    }

    pub fn get_extent(&self, device: &Arc<Device>) -> (u32, u32) {
        let caps = self.get_capabilities(device);
        let dim = self.get_dimensions();

        let extent = match caps.width {
            std::u32::MAX => {
                dim
            }
            _ => (caps.width, caps.height)
        };

        extent
    }
}

impl Wrap<vk::SurfaceKHR> for Surface {
    fn raw(&self) -> vk::SurfaceKHR {
        self.surface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        trace!("Drop surface");
        unsafe {
            self.instance.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
