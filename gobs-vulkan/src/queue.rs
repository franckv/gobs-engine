use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::Wrap;

#[derive(Clone)]
#[allow(unused)]
pub struct QueueFamily {
    pub(crate) index: u32,
    pub(crate) size: u32,
    pub(crate) graphics_bit: bool,
    pub(crate) compute_bits: bool,
    pub(crate) transfer_bits: bool,
}

/// Queue of commands to be executed on device
pub struct Queue {
    pub device: Arc<Device>,
    pub(crate) queue: vk::Queue,
    pub family: QueueFamily,
}

impl Queue {
    pub fn new(device: Arc<Device>, family: QueueFamily) -> Arc<Self> {
        let queue = unsafe {
            tracing::debug!("Create queue");
            device.raw().get_device_queue(family.index, 0)
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
