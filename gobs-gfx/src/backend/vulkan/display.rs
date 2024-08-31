use std::sync::Arc;

use anyhow::{bail, Result};
use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_vulkan as vk;

use crate::backend::vulkan::{
    device::VkDevice, image::VkImage, instance::VkInstance, renderer::VkRenderer,
};
use crate::{Display, Image};

pub struct VkDisplay {
    pub(crate) surface: Option<Arc<vk::surface::Surface>>,
    pub(crate) swapchain: Option<vk::swapchain::SwapChain>,
    pub(crate) swapchain_images: Vec<VkImage>,
    pub(crate) swapchain_idx: usize,
    pub(crate) swapchain_semaphores: Vec<vk::sync::Semaphore>,
    pub(crate) render_semaphores: Vec<vk::sync::Semaphore>,
}

impl Display<VkRenderer> for VkDisplay {
    fn new(instance: Arc<VkInstance>, window: Option<Window>) -> Result<Self>
    where
        Self: Sized,
    {
        let surface = match window {
            Some(window) => Some(Arc::new(vk::surface::Surface::new(
                instance.instance.clone(),
                window,
            )?)),
            None => None,
        };

        Ok(Self {
            surface,
            swapchain: None,
            swapchain_images: Vec::new(),
            swapchain_idx: 0,
            swapchain_semaphores: Vec::new(),
            render_semaphores: Vec::new(),
        })
    }

    fn init(&mut self, device: &VkDevice, frames_in_flight: usize) {
        if let Some(surface) = &self.surface {
            let swapchain = Self::create_swapchain(surface.clone(), device.device.clone());
            self.swapchain_images = swapchain
                .create_images(&device.device)
                .into_iter()
                .map(|image| VkImage::from_raw(image))
                .collect();
            self.swapchain = Some(swapchain);

            for _ in 0..frames_in_flight {
                self.swapchain_semaphores
                    .push(vk::sync::Semaphore::new(device.device.clone(), "Swapchain"));
                self.render_semaphores
                    .push(vk::sync::Semaphore::new(device.device.clone(), "Render"));
            }
        }
    }

    fn get_extent(&self, device: &VkDevice) -> ImageExtent2D {
        match &self.surface {
            Some(surface) => surface.get_extent(&device.device),
            None => ImageExtent2D::new(0, 0),
        }
    }

    fn get_render_target(&mut self) -> Option<&mut VkImage> {
        if self.swapchain.is_some() {
            Some(&mut self.swapchain_images[self.swapchain_idx])
        } else {
            None
        }
    }

    fn acquire(&mut self, frame: usize) -> Result<()> {
        if let Some(ref mut swapchain) = &mut self.swapchain {
            tracing::trace!("Acquire with semaphore {}", frame);
            let semaphore = &self.swapchain_semaphores[frame];
            let Ok(image_index) = swapchain.acquire_image(semaphore) else {
                bail!("Fail to acquire swapchain");
            };

            self.swapchain_idx = image_index;

            self.swapchain_images[image_index as usize].invalidate();
        }

        Ok(())
    }

    fn present(&mut self, device: &VkDevice, frame: usize) -> Result<()> {
        if let Some(ref mut swapchain) = &mut self.swapchain {
            swapchain.present(
                self.swapchain_idx,
                &device.queue,
                &self.render_semaphores[frame],
            )?;
        }

        Ok(())
    }

    fn resize(&mut self, device: &VkDevice) {
        if let Some(swapchain) = &self.swapchain {
            if let Some(surface) = &self.surface {
                let extent = surface.get_extent(&device.device);
                if extent.width == 0 || extent.height == 0 {
                    return;
                }

                let swapchain = vk::swapchain::SwapChain::new(
                    device.device.clone(),
                    surface.clone(),
                    swapchain.format,
                    swapchain.present,
                    swapchain.image_count,
                    Some(&swapchain),
                );
                self.swapchain_images = swapchain
                    .create_images(&device.device)
                    .into_iter()
                    .map(|image| VkImage::from_raw(image))
                    .collect();
                self.swapchain = Some(swapchain);
            }
        }
    }

    fn request_redraw(&self) {
        match &self.surface {
            None => (),
            Some(surface) => {
                surface.window.request_redraw();
            }
        }
    }

    fn is_minimized(&self) -> bool {
        if let Some(surface) = &self.surface {
            surface.is_minimized()
        } else {
            false
        }
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

        let caps = surface.get_capabilities(&device);

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let formats = surface.get_available_format(&device.p_device);

        let format = *formats
            .iter()
            .find(|f| {
                f.format == ImageFormat::B8g8r8a8Unorm
                    && f.color_space == vk::image::ColorSpace::SrgbNonlinear
            })
            .unwrap();

        tracing::info!("Swapchain format: {:?}", format);

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