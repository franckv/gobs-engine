use std::sync::Arc;

use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat, logger};
use gobs_vulkan::{
    Queue,
    device::Device,
    images::{ColorSpace, Image},
    instance::Instance,
    surface::Surface,
    swapchain::{PresentationMode, SwapChain},
    sync::Semaphore,
};

use crate::{
    Handle, RenderHAL,
    backend::{VulkanHAL, vulkan::ResourcesRegistry},
};

pub struct Display {
    pub(crate) surface: Option<Arc<Surface>>,
    pub(crate) swapchain: Option<SwapChain>,
    pub(crate) swapchain_images: Vec<Handle>,
    pub(crate) swapchain_idx: usize,
    pub(crate) swapchain_semaphores: Vec<Semaphore>,
    pub(crate) render_semaphores: Vec<Semaphore>,
}

impl Display {
    pub fn new(instance: Arc<Instance>, window: Option<Window>) -> Self {
        let surface =
            window.map(|window| Arc::new(Surface::new(instance.clone(), window).unwrap()));

        Self {
            surface,
            swapchain: None,
            swapchain_images: Vec::new(),
            swapchain_idx: 0,
            swapchain_semaphores: Vec::new(), // per frames in flight
            render_semaphores: Vec::new(),    // per swapchain image
        }
    }

    pub fn init(
        &mut self,
        registry: &mut ResourcesRegistry,
        device: Arc<Device>,
        frames_in_flight: usize,
    ) {
        if let Some(surface) = &self.surface {
            let swapchain = Self::create_swapchain(surface.clone(), device.clone());
            self.swapchain_images = swapchain
                .create_images()
                .into_iter()
                .map(|image| registry.images.insert(image))
                .collect();

            self.swapchain = Some(swapchain);

            for _ in 0..frames_in_flight {
                self.swapchain_semaphores
                    .push(Semaphore::new(device.clone(), "Swapchain"));
            }
            for _ in 0..self.swapchain_images.len() {
                self.render_semaphores
                    .push(Semaphore::new(device.clone(), "Render"));
            }
        }
    }

    pub fn resize(&mut self, registry: &mut ResourcesRegistry, device: Arc<Device>) {
        if let Some(swapchain) = &self.swapchain
            && let Some(surface) = &self.surface
        {
            let extent = surface.get_extent(&device);
            if extent.width == 0 || extent.height == 0 {
                return;
            }

            // FIXME: fence TIMEOUT on resize

            let swapchain = SwapChain::new(
                device.clone(),
                surface.clone(),
                swapchain.format,
                swapchain.present,
                swapchain.image_count,
                Some(swapchain),
            );

            for image in &self.swapchain_images {
                registry.images.remove(*image);
            }

            self.swapchain_images = swapchain
                .create_images()
                .into_iter()
                .map(|image| registry.images.insert(image))
                .collect();

            self.swapchain = Some(swapchain);
        }
    }

    pub(crate) fn acquire(
        &mut self,
        registry: &mut ResourcesRegistry,
        frame: usize,
    ) -> Result<(), ()> {
        if let Some(swapchain) = &mut self.swapchain {
            tracing::trace!(target: logger::SYNC, "Acquire with swapchain semaphore {}", frame);
            let semaphore = &self.swapchain_semaphores[frame];

            let image_index = swapchain.acquire_image(semaphore).map_err(|_| ())?;
            tracing::trace!(target: logger::SYNC, "Acquire image {}", image_index);

            self.swapchain_idx = image_index;
            let image_handle = self.swapchain_images[image_index];
            let swapchain_image = registry.images.get_mut(image_handle).unwrap();
            swapchain_image.invalidate();
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn present(&mut self, queue: &Queue) -> Result<(), ()> {
        if let Some(swapchain) = &mut self.swapchain {
            tracing::trace!(target: logger::SYNC, "Present with render semaphore {}", self.swapchain_idx);
            swapchain
                .present(
                    self.swapchain_idx,
                    queue,
                    &self.render_semaphores[self.swapchain_idx],
                )
                .map_err(|_| ())?;
        }

        Ok(())
    }

    pub fn get_extent(&self, device: &Device) -> ImageExtent2D {
        match &self.surface {
            Some(surface) => surface.get_extent(device),
            None => ImageExtent2D::new(0, 0),
        }
    }

    fn create_swapchain(surface: Arc<Surface>, device: Arc<Device>) -> SwapChain {
        let presents = surface.get_available_presentation_modes(device.clone());

        let present = *presents
            .iter()
            .find(|&&p| p == PresentationMode::Fifo)
            .unwrap();

        let caps = surface.get_capabilities(&device);

        tracing::debug!(target: logger::INIT,
            "image count: {}-{}",
            caps.min_image_count,
            caps.max_image_count
        );

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let formats = surface.get_available_format(&device.p_device);

        let format = *formats
            .iter()
            .find(|f| {
                f.format == ImageFormat::B8g8r8a8Unorm && f.color_space == ColorSpace::SrgbNonlinear
            })
            .unwrap();

        tracing::debug!(target: logger::INIT, "Swapchain format: {:?}", format);

        SwapChain::new(
            device.clone(),
            surface.clone(),
            format,
            present,
            image_count,
            None,
        )
    }

    pub(crate) fn get_render_target(&self) -> Handle {
        self.swapchain_images[self.swapchain_idx]
    }
}
