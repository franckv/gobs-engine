use std::ffi::CStr;

use ash::vk;

use crate::instance::Instance;
use crate::queue::QueueFamily;
use crate::Wrap;

#[derive(Debug, PartialEq)]
pub enum PhysicalDeviceType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

/// A physical graphic card
pub struct PhysicalDevice {
    p_device: vk::PhysicalDevice,
    pub name: String,
    pub gpu_type: PhysicalDeviceType,
    pub queue_families: Vec<QueueFamily>,
    pub(crate) props: vk::PhysicalDeviceProperties,
    pub(crate) mem_props: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub(crate) fn new(instance: &Instance, p_device: vk::PhysicalDevice) -> Self {
        let props = unsafe { instance.instance.get_physical_device_properties(p_device) };
        let mem_props = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(p_device)
        };

        let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()).to_str().unwrap() };

        let gpu_type = match props.device_type {
            vk::PhysicalDeviceType::OTHER => PhysicalDeviceType::Other,
            vk::PhysicalDeviceType::INTEGRATED_GPU => PhysicalDeviceType::IntegratedGpu,
            vk::PhysicalDeviceType::DISCRETE_GPU => PhysicalDeviceType::DiscreteGpu,
            vk::PhysicalDeviceType::VIRTUAL_GPU => PhysicalDeviceType::VirtualGpu,
            vk::PhysicalDeviceType::CPU => PhysicalDeviceType::Cpu,
            _ => panic!("Invalid device type"),
        };

        PhysicalDevice {
            name: String::from(name),
            gpu_type,
            queue_families: Self::get_queue_families(&p_device, &instance),
            p_device,
            props,
            mem_props,
        }
    }

    pub fn enumerate(instance: &Instance) -> Vec<PhysicalDevice> {
        let mut result = Vec::new();

        if let Ok(devices) = unsafe { instance.instance.enumerate_physical_devices() } {
            for device in devices {
                result.push(PhysicalDevice::new(instance, device));
            }
        }

        result
    }

    fn get_queue_families(p_device: &vk::PhysicalDevice, instance: &Instance) -> Vec<QueueFamily> {
        let family_properties = unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties(*p_device)
        };

        let mut results = Vec::new();

        for (idx, family_property) in family_properties.iter().enumerate() {
            let family = QueueFamily {
                index: idx as u32,
                size: family_property.queue_count,
                graphics_bit: family_property
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS),
                compute_bits: family_property
                    .queue_flags
                    .contains(vk::QueueFlags::COMPUTE),
                transfer_bits: family_property
                    .queue_flags
                    .contains(vk::QueueFlags::TRANSFER),
            };

            tracing::debug!(target: "init", "Queue family: {:?}", &family);

            results.push(family);
        }

        results
    }
}

impl Wrap<vk::PhysicalDevice> for PhysicalDevice {
    fn raw(&self) -> vk::PhysicalDevice {
        self.p_device
    }
}
