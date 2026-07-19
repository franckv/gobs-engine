use std::sync::Arc;

use ash::vk;

use gobs_core::logger;

use crate::device::Device;
use crate::{Wrap, debug};

pub struct TimeLineSemaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl TimeLineSemaphore {
    pub fn new(device: Arc<Device>, label: &str, value: u64) -> Self {
        let mut semaphore_type_info = vk::SemaphoreTypeCreateInfo::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(value);

        let semaphore_info = vk::SemaphoreCreateInfo::default().push_next(&mut semaphore_type_info);

        let semaphore = unsafe {
            device
                .raw()
                .create_semaphore(&semaphore_info, None)
                .unwrap()
        };

        let semaphore_label = format!("[Semaphore] {label}");

        debug::add_label(device.clone(), &semaphore_label, semaphore);

        Self { device, semaphore }
    }

    pub fn value(&self) -> u64 {
        unsafe {
            self.device
                .raw()
                .get_semaphore_counter_value(self.semaphore)
                .expect("Device lost")
        }
    }

    pub fn wait(&self, value: u64) {
        unsafe {
            self.device
                .raw()
                .wait_semaphores(
                    &vk::SemaphoreWaitInfo::default()
                        .semaphores(&[self.semaphore])
                        .values(&[value]),
                    5_000_000_000,
                )
                .expect("Device lost")
        }
    }
}

impl Wrap<vk::Semaphore> for TimeLineSemaphore {
    fn raw(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for TimeLineSemaphore {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop semaphore");
        unsafe {
            self.device.raw().destroy_semaphore(self.semaphore, None);
        }
    }
}
