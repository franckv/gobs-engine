use std::ptr;
use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::Wrap;

pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .raw()
                .create_semaphore(&semaphore_info, None)
                .unwrap()
        };

        Semaphore { device, semaphore }
    }
}

impl Wrap<vk::Semaphore> for Semaphore {
    fn raw(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        log::info!("Drop semaphore");
        unsafe {
            self.device.raw().destroy_semaphore(self.semaphore, None);
        }
    }
}
