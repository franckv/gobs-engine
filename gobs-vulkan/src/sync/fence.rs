use std;
use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::device::Device;
use crate::Wrap;

pub struct Fence {
    device: Arc<Device>,
    fence: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<Device>, signaled: bool) -> Self {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags,
        };

        let fence = unsafe {
            device.raw().create_fence(&fence_info, None).unwrap()
        };

        Fence {
            device,
            fence,
        }
    }

    pub fn reset(&self) {
        let fences = [self.fence];

        unsafe {
            self.device.raw().reset_fences(&fences).unwrap()
        }
    }

    pub fn wait(&self) {
        let fences = [self.fence];

        unsafe {
            self.device.raw().wait_for_fences(&fences, true,
                                              std::u64::MAX).unwrap()
        }
    }

    pub fn wait_and_reset(&self) {
        let fences = [self.fence];

        unsafe {
            self.device.raw().wait_for_fences(&fences, true,
                                              std::u64::MAX).unwrap();
            self.device.raw().reset_fences(&fences).unwrap()
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
        trace!("Drop fence");
        unsafe {
            self.device.raw().destroy_fence(self.fence, None);
        }
    }
}
