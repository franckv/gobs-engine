use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use backend::device::Device;
use backend::Wrap;

#[derive(Clone)]
pub struct QueueFamily {
    pub(crate) index: u32,
    pub(crate) size: u32,
    pub(crate) graphics_bit: bool,
    pub(crate) compute_bits: bool,
    pub(crate) transfer_bits: bool,
}

pub struct Queue {
    device: Arc<Device>,
    pub(crate) queue: vk::Queue,
}

impl Queue {
    pub fn new(device: Arc<Device>) -> Self {
        let queue = unsafe {
            debug!("Create queue");
            device.raw().get_device_queue(device.queue_family.index, 0)
        };

        Queue {
            device,
            queue,
        }
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
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
