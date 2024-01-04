use std::sync::Arc;

use ash::vk;
use ash::extensions::khr::Swapchain;

use log::{debug, trace};

use crate::instance::Instance;
use crate::physical::PhysicalDevice;
use crate::queue::QueueFamily;
use crate::Wrap;

/// Logical device
pub struct Device {
    instance: Arc<Instance>,
    device: ash::Device,
    pub(crate) p_device: PhysicalDevice,
}

impl Device {
    pub fn new(instance: Arc<Instance>, p_device: PhysicalDevice,
               queue_family: &QueueFamily) -> Arc<Self> {
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family.index)
                .queue_priorities(&priorities);

        let extensions = [Swapchain::name().as_ptr()];

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions);

        let device: ash::Device = unsafe {
            debug!("Create device");
            instance.instance.create_device(p_device.raw(),
                                            &device_info,
                                            None)
                .unwrap()
        };

        Arc::new(Device {
            instance,
            device,
            p_device,
        })
    }

    pub(crate) fn instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    pub fn wait(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap()
        };
    }

    pub(crate) fn raw(&self) -> &ash::Device {
        &self.device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        trace!("Drop device");
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
