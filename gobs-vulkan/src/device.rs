use std::fmt::Debug;
use std::sync::Arc;

use ash::extensions::khr::Swapchain;
use ash::vk::{
    self, PhysicalDeviceFeatures, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkan13Features,
};

use crate::instance::Instance;
use crate::physical::PhysicalDevice;
use crate::queue::QueueFamily;
use crate::Wrap;

/// Logical device
pub struct Device {
    pub instance: Arc<Instance>,
    device: ash::Device,
    pub p_device: PhysicalDevice,
}

impl Device {
    pub fn new(
        instance: Arc<Instance>,
        p_device: PhysicalDevice,
        queue_family: &QueueFamily,
    ) -> Arc<Self> {
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family.index)
            .queue_priorities(&priorities);

        let extensions = [Swapchain::name().as_ptr()];

        let features10 = PhysicalDeviceFeatures::builder().fill_mode_non_solid(true);
        let mut features12 = PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .descriptor_indexing(true);
        let mut features13 = PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions)
            .enabled_features(&features10)
            .push_next(&mut features12)
            .push_next(&mut features13);

        let device: ash::Device = unsafe {
            log::debug!("Create device");
            instance
                .instance
                .create_device(p_device.raw(), &device_info, None)
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
        unsafe { self.device.device_wait_idle().expect("Wait idle") };
    }

    pub fn raw(&self) -> &ash::Device {
        &self.device
    }

    pub fn cloned(&self) -> ash::Device {
        self.device.clone()
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device").finish()
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        log::debug!("Drop device");
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
