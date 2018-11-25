use std;
use std::ptr;
use std::sync::Arc;

use ash::vk;

use backend::device::Device;
use backend::image::{Image, ImageUsage};
use backend::queue::Queue;
use backend::surface::{Surface, SurfaceFormat};
use backend::sync::Semaphore;
use backend::Wrap;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PresentationMode {
    Fifo,
    FifoRelaxed,
    Mailbox,
    Immediate,
}

impl From<vk::PresentModeKHR> for PresentationMode {
    fn from(present: vk::PresentModeKHR) -> PresentationMode {
        match present {
            vk::PresentModeKHR::Fifo => PresentationMode::Fifo,
            vk::PresentModeKHR::FifoRelaxed => PresentationMode::FifoRelaxed,
            vk::PresentModeKHR::Mailbox => PresentationMode::Mailbox,
            vk::PresentModeKHR::Immediate => PresentationMode::Immediate,
        }
    }
}

impl Into<vk::PresentModeKHR> for PresentationMode {
    fn into(self) -> vk::PresentModeKHR {
        match self {
            PresentationMode::Fifo => vk::PresentModeKHR::Fifo,
            PresentationMode::FifoRelaxed => vk::PresentModeKHR::FifoRelaxed,
            PresentationMode::Mailbox => vk::PresentModeKHR::Mailbox,
            PresentationMode::Immediate => vk::PresentModeKHR::Immediate
        }
    }
}

pub struct SwapChain {
    device: Arc<Device>,
    surface: Arc<Surface>,
    format: SurfaceFormat,
    swapchain: vk::SwapchainKHR
}

impl SwapChain {
    pub fn new(device: Arc<Device>,
               surface: Arc<Surface>,
               format: SurfaceFormat,
               present: PresentationMode,
               image_count: usize,
               old_swapchain: Option<&SwapChain>) -> Self  {

        let extent = surface.get_extent(&device);

        let swapchain_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SwapchainCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: surface.raw(),
            min_image_count: image_count as u32,
            image_format: format.format.into(),
            image_color_space: format.color_space.into(),
            image_extent: vk::Extent2D {
                width: extent.0,
                height: extent.1,
            },
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            image_sharing_mode: vk::SharingMode::Exclusive,
            pre_transform: vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode: present.into(),
            clipped: 1,
            old_swapchain: match old_swapchain {
                Some(old_swapchain) => old_swapchain.swapchain,
                None => vk::SwapchainKHR::null(),
            },
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };

        let swapchain = unsafe {
            device.swapchain_loader.create_swapchain_khr(&swapchain_info,
                                                         None).unwrap()
        };

        SwapChain {
            device,
            surface,
            format,
            swapchain
        }
    }

    pub fn create_images(&self) -> Vec<Image> {
        let extent = self.surface.get_extent(&self.device);

        let vk_images = self.device.swapchain_loader.get_swapchain_images_khr(
            self.swapchain).unwrap();

        vk_images.iter().map(|&image| {
            Image::with_raw(self.device.clone(),
                            image,
                            self.format.format,
                            ImageUsage::Swapchain,
                            extent.0,
                            extent.1)
        }).collect()
    }

    pub fn acquire_image(&mut self, signal: &Semaphore) -> Result<usize, ()> {
        unsafe {
            match self.device.swapchain_loader.acquire_next_image_khr(self.swapchain,
                                                     std::u64::MAX,
                                                     signal.raw(),
                                                     vk::Fence::null()) {
                Ok(idx) => {
                    Ok(idx as usize)
                }
                Err(vk::Result::SuboptimalKhr) | Err(vk::Result::ErrorOutOfDateKhr) => {
                    Err(())
                }
                _ => panic!("Unable to acquire swapchain")
            }
        }
    }

    pub fn present(&mut self, index: usize, queue: &Queue, wait: &Semaphore) -> Result<(), ()> {
        let wait_semaphores = [wait.raw()];

        let indices = [index as u32];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: &self.swapchain,
            p_image_indices: indices.as_ptr(),
            p_results: ptr::null_mut(),
        };

        unsafe {
            match self.device.swapchain_loader.queue_present_khr(queue.queue, &present_info) {
                Ok(_) => Ok(()),
                Err(vk::Result::SuboptimalKhr) | Err(vk::Result::ErrorOutOfDateKhr) => {
                    Err(())
                }
                _ => panic!("Unable to present swapchain")
            }
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            self.device.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
        }
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        trace!("Drop swapchain");
        self.cleanup();
    }
}
