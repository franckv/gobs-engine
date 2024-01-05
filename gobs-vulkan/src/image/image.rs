use std::ptr;
use std::sync::Arc;

use ash::vk;

use log::trace;

use crate::device::Device;
use crate::image::ImageFormat;
use crate::memory::Memory;
use crate::Wrap;

#[derive(Copy, Clone)]
pub enum ImageLayout {
    Undefined,
    General,
    Transfer,
    Shader,
    Depth,
    Color,
    Present,
}

impl Into<vk::ImageLayout> for ImageLayout {
    fn into(self) -> vk::ImageLayout {
        match self {
            ImageLayout::Undefined => vk::ImageLayout::UNDEFINED,
            ImageLayout::General => vk::ImageLayout::GENERAL,
            ImageLayout::Transfer => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::Shader => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::Depth => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::Present => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ImageUsage {
    Swapchain,
    Texture,
    Depth,
}

impl Into<vk::ImageUsageFlags> for ImageUsage {
    fn into(self) -> vk::ImageUsageFlags {
        match self {
            ImageUsage::Swapchain => vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ImageUsage::Texture => vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ImageUsage::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        }
    }
}

impl Into<vk::ImageAspectFlags> for ImageUsage {
    fn into(self) -> vk::ImageAspectFlags {
        match self {
            ImageUsage::Swapchain => vk::ImageAspectFlags::COLOR,
            ImageUsage::Texture => vk::ImageAspectFlags::COLOR,
            ImageUsage::Depth => vk::ImageAspectFlags::DEPTH,
        }
    }
}

/// Image buffer allocated in memory
pub struct Image {
    device: Arc<Device>,
    image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub usage: ImageUsage,
    pub width: u32,
    pub height: u32,
    memory: Option<Memory>,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        format: ImageFormat,
        usage: ImageUsage,
        width: u32,
        height: u32,
    ) -> Self {
        let image = Self::create_image(&device, width, height, format, usage);

        let memory = Memory::with_image(device.clone(), image);

        let image_view = Self::create_image_view(&device, image, format, usage);

        Image {
            device,
            image,
            image_view,
            usage,
            width,
            height,
            memory: Some(memory), // swapchain images don't need manual memory allocation
        }
    }

    pub(crate) fn with_raw(
        device: Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
        width: u32,
        height: u32,
    ) -> Self {
        let image_view = Self::create_image_view(&device, image, format, usage);

        Image {
            device,
            image,
            image_view,
            usage,
            width,
            height,
            memory: None,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn create_image(
        device: &Arc<Device>,
        width: u32,
        height: u32,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::Image {
        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format: format.into(),
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: usage.into(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples: vk::SampleCountFlags::TYPE_1,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        unsafe { device.raw().create_image(&image_info, None).unwrap() }
    }

    pub(crate) fn create_image_view(
        device: &Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: format.into(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: usage.into(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
        };

        unsafe { device.raw().create_image_view(&view_info, None).unwrap() }
    }
}

impl Wrap<vk::Image> for Image {
    fn raw(&self) -> vk::Image {
        self.image
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        trace!("Drop image");
        unsafe {
            self.device.raw().destroy_image_view(self.image_view, None);
            if self.memory.is_some() {
                self.device.raw().destroy_image(self.image, None);
            }
        }
    }
}
