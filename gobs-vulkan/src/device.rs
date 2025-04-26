use std::fmt::Debug;
use std::sync::Arc;

use ash::{
    ext::debug_utils,
    khr::{push_descriptor, swapchain},
    vk::{self, FormatFeatureFlags},
};

use gobs_core::ImageFormat;

use crate::{
    Wrap,
    error::VulkanError,
    feature::Features,
    images::{ImageUsage, VkFormat},
    instance::Instance,
    physical::PhysicalDevice,
    queue::{Queue, QueueFamily},
    surface::Surface,
};

/// Logical device
pub struct Device {
    pub instance: Arc<Instance>,
    device: ash::Device,
    pub p_device: PhysicalDevice,
    pub(crate) debug_utils_device: debug_utils::Device,
    pub(crate) push_descriptor_device: push_descriptor::Device,
    pub features: Features,
    pub graphics_family: QueueFamily,
    pub transfer_family: QueueFamily,
}

impl Device {
    pub fn new(
        instance: Arc<Instance>,
        p_device: PhysicalDevice,
        surface: Option<&Surface>,
    ) -> Result<Arc<Self>, VulkanError> {
        let (graphics_family, transfer_family) = p_device.find_family(surface);
        tracing::debug!(target: "init", "Using queue families Graphics={:?}, Transfer={:?}", &graphics_family, &transfer_family);

        let priorities = if transfer_family.index != graphics_family.index {
            vec![1.0]
        } else {
            vec![1.0, 1.0]
        };

        let mut queues = vec![];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_family.index)
            .queue_priorities(&priorities);
        queues.push(queue_info);

        if transfer_family.index != graphics_family.index {
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(transfer_family.index)
                .queue_priorities(&priorities);
            queues.push(queue_info);
        }

        let extensions = [swapchain::NAME.as_ptr(), push_descriptor::NAME.as_ptr()];

        let features = Features::from_device(&instance, &p_device);
        let features10 = features.features10();
        let mut features12 = features.features12();
        let mut features13 = features.features13();

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queues)
            .enabled_extension_names(&extensions)
            .enabled_features(&features10)
            .push_next(&mut features12)
            .push_next(&mut features13);

        let device: ash::Device = unsafe {
            tracing::debug!("Create device");
            instance
                .instance
                .create_device(p_device.raw(), &device_info, None)?
        };

        let debug_utils_device = debug_utils::Device::new(&instance.instance, &device);

        let push_descriptor_device = push_descriptor::Device::new(&instance.instance, &device);

        Ok(Arc::new(Device {
            instance,
            device,
            p_device,
            debug_utils_device,
            push_descriptor_device,
            features,
            graphics_family,
            transfer_family,
        }))
    }

    pub fn graphics_queue(self: Arc<Self>) -> Arc<Queue> {
        Queue::new(self.clone(), self.graphics_family, 0)
    }

    pub fn transfer_queue(self: Arc<Self>) -> Arc<Queue> {
        let transfer_queue_index = if self.transfer_family.index != self.graphics_family.index {
            0
        } else {
            1
        };

        Queue::new(self.clone(), self.transfer_family, transfer_queue_index)
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

    pub fn support_blit(&self, format: ImageFormat, usage: ImageUsage, src: bool) -> bool {
        let format_properties = unsafe {
            self.instance.raw().get_physical_device_format_properties(
                self.p_device.raw(),
                VkFormat::from(format).into(),
            )
        };

        let tiling: vk::ImageTiling = usage.into();
        let flag = if src {
            FormatFeatureFlags::BLIT_SRC
        } else {
            FormatFeatureFlags::BLIT_DST
        };

        if tiling == vk::ImageTiling::LINEAR {
            format_properties.linear_tiling_features.contains(flag)
        } else {
            format_properties.optimal_tiling_features.contains(flag)
        }
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
        tracing::debug!(target: "memory", "Drop device");
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
