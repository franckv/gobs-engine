use std::fmt::Debug;
use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::Wrap;

#[derive(Clone)]
pub struct QueueFamily {
    pub index: u32,
    pub size: u32,
    pub graphics_bit: bool,
    pub compute_bits: bool,
    pub transfer_bits: bool,
}

impl Debug for QueueFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = vec![];

        if self.graphics_bit {
            flags.push("Graphics");
        }
        if self.compute_bits {
            flags.push("Compute");
        }
        if self.transfer_bits {
            flags.push("Transfer");
        }

        f.debug_struct("QueueFamily")
            .field("index", &self.index)
            .field("size", &self.size)
            .field("flags", &flags.join(" | "))
            .finish()
    }
}

/// Queue of commands to be executed on device
pub struct Queue {
    pub device: Arc<Device>,
    pub(crate) queue: vk::Queue,
    pub family: QueueFamily,
}

impl Queue {
    pub fn new(device: Arc<Device>, family: QueueFamily, index: u32) -> Arc<Self> {
        let queue = unsafe {
            tracing::debug!("Create queue");
            device.raw().get_device_queue(family.index, index)
        };

        Arc::new(Queue {
            device,
            queue,
            family,
        })
    }

    pub fn wait(&self) {
        self.device.wait();
    }
}

impl Wrap<vk::Queue> for Queue {
    fn raw(&self) -> vk::Queue {
        self.queue
    }
}
