use std;
use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::{debug, Wrap};

pub struct Fence {
    device: Arc<Device>,
    fence: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<Device>, signaled: bool, label: &str) -> Self {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            Default::default()
        };

        let fence_info = vk::FenceCreateInfo::builder().flags(flags);

        let fence = unsafe { device.raw().create_fence(&fence_info, None).unwrap() };

        let fence_label = format!("[Fence] {}", label);

        debug::add_label(
            device.clone(),
            &fence_label,
            vk::ObjectType::FENCE,
            vk::Handle::as_raw(fence),
        );

        Fence { device, fence }
    }

    pub fn reset(&self) {
        let fences = [self.fence];

        unsafe {
            self.device
                .raw()
                .reset_fences(&fences)
                .expect("Device lost")
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.device
                .raw()
                .wait_for_fences(&[self.fence], true, 5_000_000_000)
                .expect("Fence timeout");
        }
    }

    pub fn wait_and_reset(&self) {
        unsafe {
            self.device
                .raw()
                .wait_for_fences(&[self.fence], true, 5_000_000_000)
                .expect("Fence timeout");
            self.device
                .raw()
                .reset_fences(&[self.fence])
                .expect("Device lost");
        }
    }

    pub fn signaled(&self) -> bool {
        unsafe {
            self.device
                .raw()
                .get_fence_status(self.fence)
                .expect("Device lost")
        }
    }
}

impl Wrap<vk::Fence> for Fence {
    fn raw(&self) -> vk::Fence {
        self.fence
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        log::debug!("Drop fence");
        unsafe {
            self.device.raw().destroy_fence(self.fence, None);
        }
    }
}
