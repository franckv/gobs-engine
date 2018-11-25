use std::ffi::CStr;

use ash::vk;
use ash::version::InstanceV1_0;

use backend::instance::Instance;
use backend::queue::QueueFamily;
use backend::Wrap;

#[derive(Debug)]
pub enum PhysicalDeviceType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

pub struct PhysicalDevice {
    p_device: vk::PhysicalDevice,
    pub name: String,
    pub gpu_type: PhysicalDeviceType,
    pub queue_families: Vec<QueueFamily>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub(crate) fn new(instance: &Instance, p_device: vk::PhysicalDevice) -> Self {
        let props = instance.instance.get_physical_device_properties(p_device);

        let name = unsafe {
            CStr::from_ptr(props.device_name.as_ptr()).to_str().unwrap()
        };

        let gpu_type = match props.device_type {
            vk::types::PhysicalDeviceType::Other =>
                PhysicalDeviceType::Other,
            vk::types::PhysicalDeviceType::IntegratedGpu =>
                PhysicalDeviceType::IntegratedGpu,
            vk::types::PhysicalDeviceType::DiscreteGpu =>
                PhysicalDeviceType::DiscreteGpu,
            vk::types::PhysicalDeviceType::VirtualGpu =>
                PhysicalDeviceType::VirtualGpu,
            vk::types::PhysicalDeviceType::Cpu =>
                PhysicalDeviceType::Cpu,
        };

        let memory_properties =
            instance.instance.get_physical_device_memory_properties(p_device);

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

        if let Ok(devices) = instance.instance.enumerate_physical_devices() {
            for device in devices {
                result.push(PhysicalDevice::new(instance, device));
            }
        };

        result
    }

    fn get_queue_families(p_device: &vk::PhysicalDevice,
                          instance: &Instance) -> Vec<QueueFamily> {
        let family_properties =
            instance.instance.get_physical_device_queue_family_properties(
                *p_device);

        let mut results = Vec::new();

        for (idx, family_property) in family_properties.iter().enumerate() {
            let family = QueueFamily {
                index: idx as u32,
                size: family_property.queue_count,
                graphics_bit: family_property.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT),
                compute_bits: family_property.queue_flags.subset(vk::QUEUE_COMPUTE_BIT),
                transfer_bits: family_property.queue_flags.subset(vk::QUEUE_TRANSFER_BIT),
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
