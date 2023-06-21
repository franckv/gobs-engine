use std::ffi::CStr;

use ash::vk;
use ash::version::InstanceV1_0;

use crate::instance::Instance;
use crate::queue::QueueFamily;
use crate::Wrap;

#[derive(Debug)]
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
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub(crate) fn new(instance: &Instance, p_device: vk::PhysicalDevice) -> Self {
        let props = unsafe {
            instance.instance.get_physical_device_properties(p_device)
        };

        let name = unsafe {
            CStr::from_ptr(props.device_name.as_ptr()).to_str().unwrap()
        };

        let gpu_type = match props.device_type {
            vk::PhysicalDeviceType::OTHER =>
                PhysicalDeviceType::Other,
            vk::PhysicalDeviceType::INTEGRATED_GPU =>
                PhysicalDeviceType::IntegratedGpu,
            vk::PhysicalDeviceType::DISCRETE_GPU =>
                PhysicalDeviceType::DiscreteGpu,
            vk::PhysicalDeviceType::VIRTUAL_GPU =>
                PhysicalDeviceType::VirtualGpu,
            vk::PhysicalDeviceType::CPU =>
                PhysicalDeviceType::Cpu,
            _ => panic!("Invalid device type")
        };

        let memory_properties = unsafe {
            instance.instance.get_physical_device_memory_properties(p_device)
        };

        PhysicalDevice {
            name: String::from(name),
            gpu_type,
            queue_families: Self::get_queue_families(&p_device, &instance),
            p_device,
            memory_properties,
        }
    }

    pub fn enumerate(instance: &Instance) -> Vec<PhysicalDevice> {
        let mut result = Vec::new();

        if let Ok(devices) = unsafe {
            instance.instance.enumerate_physical_devices()
        } {
            for device in devices {
                result.push(PhysicalDevice::new(instance, device));
            }
        }

        result
    }

    fn get_queue_families(p_device: &vk::PhysicalDevice,
                          instance: &Instance) -> Vec<QueueFamily> {
        let family_properties = unsafe {
            instance.instance.get_physical_device_queue_family_properties(
                *p_device)
        };

        let mut results = Vec::new();

        for (idx, family_property) in family_properties.iter().enumerate() {
            let family = QueueFamily {
                index: idx as u32,
                size: family_property.queue_count,
                graphics_bit: family_property.queue_flags.contains(vk::QueueFlags::GRAPHICS),
                compute_bits: family_property.queue_flags.contains(vk::QueueFlags::COMPUTE),
                transfer_bits: family_property.queue_flags.contains(vk::QueueFlags::TRANSFER),
            };

            results.push(family);
        }

        results
    }

    pub(crate) fn find_memory_type(&self, memory_req: &vk::MemoryRequirements,
                                   flags: vk::MemoryPropertyFlags) -> u32 {
        let idx = self.memory_properties.memory_types.iter().enumerate()
            .position(|(idx, memory_type)| {
                memory_type.property_flags & flags == flags &&
                    memory_req.memory_type_bits & (1 << idx) != 0
            });

        idx.unwrap() as u32
    }
}

impl Wrap<vk::PhysicalDevice> for PhysicalDevice {
    fn raw(&self) -> vk::PhysicalDevice {
        self.p_device
    }
}
