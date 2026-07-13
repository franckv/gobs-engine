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
        const DeviceFault = 1 << 6;
        const ScalarBlockLayout = 1 << 7;
    }
}

#[derive(Debug, Default)]
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
        let mut fault_features = vk::PhysicalDeviceFaultFeaturesEXT::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut features11)
            .push_next(&mut features12)
            .push_next(&mut features13)
            .push_next(&mut fault_features);

        unsafe {
            instance
                .instance
                .get_physical_device_features2(p_device.raw(), &mut features);
        };

        let features10 = features.features;

        tracing::debug!(target: logger::INIT,
            "Device features: {:#?},{:#?},{:#?},{:#?},{:#?}",
            features10,
            features11,
            features12,
            features13,
            fault_features,
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
        enabled_features.set(
            Feature::ScalarBlockLayout,
            features12.scalar_block_layout == 1,
        );
        enabled_features.set(Feature::DynamicRendering, features13.dynamic_rendering == 1);
        enabled_features.set(Feature::Synchronization2, features13.synchronization2 == 1);
        enabled_features.set(Feature::DeviceFault, fault_features.device_fault == 1);

        Self { enabled_features }
    }

    pub fn check_features(&self, expected: &Self) -> bool {
        self.enabled_features.contains(expected.enabled_features)
    }

    pub fn features10(&self) -> vk::PhysicalDeviceFeatures {
        vk::PhysicalDeviceFeatures::default()
            .fill_mode_non_solid(self.enabled_features.contains(Feature::FillModeNonSolid))
    }

    pub fn features11(&'_ self) -> vk::PhysicalDeviceVulkan11Features<'_> {
        vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(
            self.enabled_features
                .contains(Feature::ShaderDrawParameters),
        )
    }

    pub fn features12(&'_ self) -> vk::PhysicalDeviceVulkan12Features<'_> {
        vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(self.enabled_features.contains(Feature::BufferDeviceAddress))
            .descriptor_indexing(self.enabled_features.contains(Feature::DescriptorIndexing))
            .scalar_block_layout(self.enabled_features.contains(Feature::ScalarBlockLayout))
    }

    pub fn features13(&'_ self) -> vk::PhysicalDeviceVulkan13Features<'_> {
        vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(self.enabled_features.contains(Feature::DynamicRendering))
            .synchronization2(self.enabled_features.contains(Feature::Synchronization2))
    }

    pub fn fault_features(&'_ self) -> vk::PhysicalDeviceFaultFeaturesEXT<'_> {
        vk::PhysicalDeviceFaultFeaturesEXT::default()
            .device_fault(self.enabled_features.contains(Feature::DeviceFault))
    }

    pub fn fill_mode_non_solid(mut self) -> Self {
        self.enabled_features.set(Feature::FillModeNonSolid, true);

        self
    }

    pub fn shader_draw_parameters(mut self) -> Self {
        self.enabled_features
            .set(Feature::ShaderDrawParameters, true);

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

    pub fn device_fault(mut self) -> Self {
        self.enabled_features.set(Feature::DeviceFault, true);

        self
    }

    pub fn scalar_block_layout(mut self) -> Self {
        self.enabled_features.set(Feature::ScalarBlockLayout, true);

        self
    }
}
