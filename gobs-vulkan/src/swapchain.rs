use std;
use std::sync::Arc;

use ash::khr::swapchain;
use ash::vk;

use crate::device::Device;
use crate::image::{Image, ImageUsage};
use crate::queue::Queue;
use crate::surface::{Surface, SurfaceFormat};
use crate::sync::Semaphore;
use crate::Wrap;

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
            vk::PresentModeKHR::FIFO => PresentationMode::Fifo,
            vk::PresentModeKHR::FIFO_RELAXED => PresentationMode::FifoRelaxed,
            vk::PresentModeKHR::MAILBOX => PresentationMode::Mailbox,
            vk::PresentModeKHR::IMMEDIATE => PresentationMode::Immediate,
            _ => panic!("Invalid present mode"),
        }
    }
}

impl Into<vk::PresentModeKHR> for PresentationMode {
    fn into(self) -> vk::PresentModeKHR {
        match self {
            PresentationMode::Fifo => vk::PresentModeKHR::FIFO,
            PresentationMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
            PresentationMode::Mailbox => vk::PresentModeKHR::MAILBOX,
            PresentationMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
        }
    }
}

/// Set of images that can be presented on a surface
pub struct SwapChain {
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
    pub format: SurfaceFormat,
    pub present: PresentationMode,
    pub image_count: usize,
    loader: swapchain::Device,
    swapchain: vk::SwapchainKHR,
}

impl SwapChain {
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        format: SurfaceFormat,
        present: PresentationMode,
        image_count: usize,
        old_swapchain: Option<&SwapChain>,
    ) -> Self {
        //TODO: let extent = surface.get_extent(device.clone());
        let extent = surface.get_dimensions();

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw())
            .min_image_count(image_count as u32)
            .image_format(format.format.into())
            .image_color_space(format.color_space.into())
            .image_extent(extent.into())
            .image_usage(ImageUsage::Swapchain.into())
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present.into())
            .clipped(true)
            .old_swapchain(match old_swapchain {
                Some(old_swapchain) => old_swapchain.swapchain,
                None => vk::SwapchainKHR::null(),
            })
            .image_array_layers(1)
            .queue_family_indices(&[]);

        let loader = swapchain::Device::new(&device.instance().raw(), device.raw());

        let swapchain = unsafe { loader.create_swapchain(&swapchain_info, None).unwrap() };

        SwapChain {
            device,
            surface,
            format,
            present,
            image_count,
            loader,
            swapchain,
        }
    }

    pub fn create_images(&self) -> Vec<Image> {
        //TODO: let extent = self.surface.get_extent(self.device.clone());
        let extent = self.surface.get_dimensions();

        unsafe {
            let vk_images = self.loader.get_swapchain_images(self.swapchain).unwrap();

            vk_images
                .iter()
                .map(|&image| {
                    Image::with_raw(
                        "swapchain",
                        self.device.clone(),
                        image,
                        self.format.format,
                        ImageUsage::Swapchain,
                        extent,
                    )
                })
                .collect()
        }
    }

    pub fn acquire_image(&mut self, signal: &Semaphore) -> Result<usize, ()> {
        unsafe {
            match self.loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                signal.raw(),
                vk::Fence::null(),
            ) {
                Ok((idx, _)) => Ok(idx as usize),
                Err(vk::Result::SUBOPTIMAL_KHR) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(()),
                _ => panic!("Unable to acquire swapchain"),
            }
        }
    }

    pub fn present(&mut self, index: usize, queue: &Queue, wait: &Semaphore) -> Result<(), ()> {
        let wait_semaphore = wait.raw();
        let image_indice = index as u32;
        let swapchains = self.swapchain;

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(std::slice::from_ref(&wait_semaphore))
            .image_indices(std::slice::from_ref(&image_indice))
            .swapchains(std::slice::from_ref(&swapchains));

        unsafe {
            match self.loader.queue_present(queue.queue, &present_info) {
                Ok(_) => Ok(()),
                Err(vk::Result::SUBOPTIMAL_KHR) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(()),
                _ => panic!("Unable to present swapchain"),
            }
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        log::debug!("Drop swapchain");
        self.cleanup();
    }
}
