use std::ptr;
use std::sync::Arc;

use ash::vk;

use log::trace;

use crate::device::Device;
use crate::Wrap;

pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
        };

        let semaphore = unsafe {
            device.raw().create_semaphore(&semaphore_info, None).unwrap()
        };

        Semaphore {
            device,
            semaphore,
        }
    }
}

impl Wrap<vk::Semaphore> for Semaphore {
    fn raw(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        trace!("Drop semaphore");
        unsafe {
            self.device.raw().destroy_semaphore(self.semaphore, None);
        }
    }
}
