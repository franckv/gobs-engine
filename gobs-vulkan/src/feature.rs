use ash::vk;

use crate::{Wrap, instance::Instance, physical::PhysicalDevice};

#[derive(Default)]
pub struct Features {
    pub fill_mode_non_solid: bool,
    pub buffer_device_address: bool,
    pub descriptor_indexing: bool,
    pub dynamic_rendering: bool,
    pub synchronization2: bool,
}

impl Features {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn from_device(instance: &Instance, p_device: &PhysicalDevice) -> Self {
        let mut features11 = vk::PhysicalDeviceVulkan11Features::default();
        let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut features11)
            .push_next(&mut features12)
            .push_next(&mut features13);

        unsafe {
            instance
                .instance
                .get_physical_device_features2(p_device.raw(), &mut features);
        };

        let features10 = features.features;

        tracing::debug!(
            "Features: {:?},{:?},{:?},{:?}",
            features10,
            features11,
            features12,
            features13
        );

        Self {
            fill_mode_non_solid: features10.fill_mode_non_solid == 1,
            buffer_device_address: features12.buffer_device_address == 1,
            descriptor_indexing: features12.descriptor_indexing == 1,
            dynamic_rendering: features13.dynamic_rendering == 1,
            synchronization2: features13.synchronization2 == 1,
        }
    }

    pub fn check_features(&self, expected: &Self) -> bool {
        (!expected.fill_mode_non_solid || self.fill_mode_non_solid)
            && (!expected.buffer_device_address || self.buffer_device_address)
            && (!expected.descriptor_indexing || self.descriptor_indexing)
            && (!expected.dynamic_rendering || self.dynamic_rendering)
            && (!expected.synchronization2 || self.synchronization2)
    }

    pub fn features10(&self) -> vk::PhysicalDeviceFeatures {
        vk::PhysicalDeviceFeatures::default().fill_mode_non_solid(self.fill_mode_non_solid)
    }

    pub fn features12(&self) -> vk::PhysicalDeviceVulkan12Features {
        vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(self.buffer_device_address)
            .descriptor_indexing(self.descriptor_indexing)
    }

    pub fn features13(&self) -> vk::PhysicalDeviceVulkan13Features {
        vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(self.dynamic_rendering)
            .synchronization2(self.synchronization2)
    }

    pub fn fill_mode_non_solid(mut self) -> Self {
        self.fill_mode_non_solid = true;

        self
    }

    pub fn buffer_device_address(mut self) -> Self {
        self.buffer_device_address = true;

        self
    }

    pub fn descriptor_indexing(mut self) -> Self {
        self.descriptor_indexing = true;

        self
    }

    pub fn dynamic_rendering(mut self) -> Self {
        self.dynamic_rendering = true;

        self
    }

    pub fn synchronization2(mut self) -> Self {
        self.synchronization2 = true;

        self
    }
}
