use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::extensions::khr::Swapchain;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;

use crate::backend::instance::Instance;
use crate::backend::physical::PhysicalDevice;
use crate::backend::queue::QueueFamily;
use crate::backend::Wrap;

/// Logical device
pub struct Device {
    instance: Arc<Instance>,
    device: ash::Device,
    pub(crate) p_device: PhysicalDevice,
    pub(crate) queue_family: QueueFamily,
}

impl Device {
    pub fn queue_family(&self) -> &QueueFamily {
        &self.queue_family
    }

    pub fn new(instance: Arc<Instance>, p_device: PhysicalDevice,
               queue_family: QueueFamily) -> Arc<Self> {
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_family.index,
            p_queue_priorities: priorities.as_ptr(),
            queue_count: priorities.len() as u32,
        };

        let extensions = [Swapchain::name().as_ptr()];

        let device_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            p_enabled_features: ptr::null(),
        };

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
            queue_family,
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
