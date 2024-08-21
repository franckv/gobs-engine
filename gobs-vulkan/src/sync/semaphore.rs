use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::{debug, Wrap};

pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>, label: &str) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .raw()
                .create_semaphore(&semaphore_info, None)
                .unwrap()
        };

        let semaphore_label = format!("[Semaphore] {}", label);

        debug::add_label(device.clone(), &semaphore_label, semaphore);

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
        tracing::debug!(target: "memory", "Drop semaphore");
        unsafe {
            self.device.raw().destroy_semaphore(self.semaphore, None);
        }
    }
}
