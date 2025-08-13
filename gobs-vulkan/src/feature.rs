use ash::vk;
use bitflags::bitflags;

use gobs_core::logger;

use crate::{Wrap, instance::Instance, physical::PhysicalDevice};

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Feature: u32 {
        const FillModeNonSolid = 1;
        const BufferDeviceAddress = 1 << 1;
        const DescriptorIndexing = 1 << 2;
        const DynamicRendering = 1 << 3;
        const Synchronization2 = 1 << 4;
        const ShaderDrawParameters = 1 << 5;
    }
}

#[derive(Default)]
pub struct Features {
    enabled_features: Feature,
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

        tracing::debug!(target: logger::RENDER,
            "Features: {:?},{:?},{:?},{:?}",
            features10,
            features11,
            features12,
            features13
        );

        let mut enabled_features = Feature::empty();
        enabled_features.set(
            Feature::FillModeNonSolid,
            features10.fill_mode_non_solid == 1,
        );
        enabled_features.set(
            Feature::ShaderDrawParameters,
            features11.shader_draw_parameters == 1,
        );
        enabled_features.set(
            Feature::BufferDeviceAddress,
            features12.buffer_device_address == 1,
        );
        enabled_features.set(
            Feature::DescriptorIndexing,
            features12.descriptor_indexing == 1,
        );
        enabled_features.set(Feature::DynamicRendering, features13.dynamic_rendering == 1);
        enabled_features.set(Feature::Synchronization2, features13.synchronization2 == 1);

        Self { enabled_features }
    }

    pub fn check_features(&self, expected: &Self) -> bool {
        self.enabled_features.contains(expected.enabled_features)
    }

    pub fn features10(&self) -> vk::PhysicalDeviceFeatures {
        vk::PhysicalDeviceFeatures::default()
            .fill_mode_non_solid(self.enabled_features.contains(Feature::FillModeNonSolid))
    }

    pub fn features11(&self) -> vk::PhysicalDeviceVulkan11Features {
        vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(
            self.enabled_features
                .contains(Feature::ShaderDrawParameters),
        )
    }

    pub fn features12(&self) -> vk::PhysicalDeviceVulkan12Features {
        vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(self.enabled_features.contains(Feature::BufferDeviceAddress))
            .descriptor_indexing(self.enabled_features.contains(Feature::DescriptorIndexing))
    }

    pub fn features13(&self) -> vk::PhysicalDeviceVulkan13Features {
        vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(self.enabled_features.contains(Feature::DynamicRendering))
            .synchronization2(self.enabled_features.contains(Feature::Synchronization2))
    }

    pub fn fill_mode_non_solid(mut self) -> Self {
        self.enabled_features.set(Feature::FillModeNonSolid, true);

        self
    }

    pub fn buffer_device_address(mut self) -> Self {
        self.enabled_features
            .set(Feature::BufferDeviceAddress, true);

        self
    }

    pub fn descriptor_indexing(mut self) -> Self {
        self.enabled_features.set(Feature::DescriptorIndexing, true);

        self
    }

    pub fn dynamic_rendering(mut self) -> Self {
        self.enabled_features.set(Feature::DynamicRendering, true);

        self
    }

    pub fn synchronization2(mut self) -> Self {
        self.enabled_features.set(Feature::Synchronization2, true);

        self
    }
}
