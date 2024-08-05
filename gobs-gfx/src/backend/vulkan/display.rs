use std::sync::Arc;

use anyhow::{bail, Result};
use winit::window::Window;

use gobs_vulkan as vk;

use crate::backend::vulkan::{VkImage, VkInstance};
use crate::{Display, Image};

use super::device::VkDevice;

pub struct VkDisplay {
    pub(crate) surface: Arc<vk::surface::Surface>,
    pub(crate) swapchain: Option<vk::swapchain::SwapChain>,
    pub(crate) swapchain_images: Vec<VkImage>,
    pub(crate) swapchain_idx: usize,
    pub(crate) swapchain_semaphores: Vec<vk::sync::Semaphore>,
    pub(crate) render_semaphores: Vec<vk::sync::Semaphore>,
}

impl Display for VkDisplay {
    fn new(instance: Arc<VkInstance>, window: Option<Window>) -> Result<Arc<Self>>
    where
        Self: Sized,
    {
        let surface = Arc::new(vk::surface::Surface::new(
            instance.instance.clone(),
            window.unwrap(),
        )?);

        Ok(Arc::new(Self {
            surface,
            swapchain: None,
            swapchain_images: Vec::new(),
            swapchain_idx: 0,
            swapchain_semaphores: Vec::new(),
            render_semaphores: Vec::new(),
        }))
    }

    fn init(&mut self, device: &VkDevice) {
        let swapchain = Self::create_swapchain(self.surface.clone(), device.device.clone());
        self.swapchain_images = swapchain
            .create_images()
            .into_iter()
            .map(|image| VkImage::from_raw(image))
            .collect();
        self.swapchain = Some(swapchain);

        let n_images = self.swapchain_images.len();

        for _ in 0..n_images {
            self.swapchain_semaphores
                .push(vk::sync::Semaphore::new(device.device.clone(), "Swapchain"));
            self.render_semaphores
                .push(vk::sync::Semaphore::new(device.device.clone(), "Render"));
        }
    }

    fn get_extent(&self) -> vk::image::ImageExtent2D {
            self.surface.get_dimensions()
    }

    fn acquire(&mut self) -> Result<()> {
        let semaphore = self.swapchain_semaphores[self.swapchain_idx];
        let Ok(image_index) = self.swapchain.acquire_image(semaphore) else {
            bail!("Fail to acquire swapchain");
        };

        self.swapchain_idx = image_index;

        self.swapchain_images[image_index as usize].invalidate();

        Ok(())
    }
}

impl VkDisplay {
    fn create_swapchain(
        surface: Arc<vk::surface::Surface>,
        device: Arc<vk::device::Device>,
    ) -> vk::swapchain::SwapChain {
        let presents = surface.get_available_presentation_modes(device.clone());

        let present = *presents
            .iter()
            .find(|&&p| p == vk::swapchain::PresentationMode::Fifo)
            .unwrap();

        let caps = surface.get_capabilities(device.clone());

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let formats = surface.get_available_format(&device.p_device);

        let format = *formats
            .iter()
            .find(|f| {
                f.format == vk::image::ImageFormat::B8g8r8a8Unorm
                    && f.color_space == vk::image::ColorSpace::SrgbNonlinear
            })
            .unwrap();

        log::info!("Swapchain format: {:?}", format);

        vk::swapchain::SwapChain::new(
            device.clone(),
            surface.clone(),
            format,
            present,
            image_count,
            None,
        )
    }
}
