use std::fmt::Debug;
use std::sync::Arc;

use ash::vk;

use gobs_core::{ImageExtent2D, ImageFormat};

use crate::alloc::Allocator;
use crate::device::Device;
use crate::memory::Memory;
use crate::{debug, Wrap};

use super::format::VkFormat;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImageLayout {
    Undefined,
    General,
    TransferSrc,
    TransferDst,
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
            ImageLayout::TransferSrc => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDst => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::Shader => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::Depth => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            ImageLayout::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::Present => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ImageUsage {
    Swapchain,
    Texture,
    Color,
    Depth,
}

impl Into<vk::ImageUsageFlags> for ImageUsage {
    fn into(self) -> vk::ImageUsageFlags {
        match self {
            ImageUsage::Swapchain => {
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT
            }
            ImageUsage::Texture => vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ImageUsage::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            ImageUsage::Color => {
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::STORAGE
            }
        }
    }
}

impl Into<vk::ImageAspectFlags> for ImageUsage {
    fn into(self) -> vk::ImageAspectFlags {
        match self {
            ImageUsage::Swapchain => vk::ImageAspectFlags::COLOR,
            ImageUsage::Texture => vk::ImageAspectFlags::COLOR,
            ImageUsage::Color => vk::ImageAspectFlags::COLOR,
            ImageUsage::Depth => vk::ImageAspectFlags::DEPTH,
        }
    }
}

/// Image buffer allocated in memory
pub struct Image {
    pub label: String,
    device: Arc<Device>,
    image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub format: ImageFormat,
    pub usage: ImageUsage,
    pub layout: ImageLayout,
    pub extent: ImageExtent2D,
    memory: Option<Memory>,
}

impl Image {
    pub fn new(
        label: &str,
        device: Arc<Device>,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
        allocator: Arc<Allocator>,
    ) -> Self {
        let image_label = format!("[Image] {}", label);

        let image = Self::create_image(&device, extent, format, usage);

        debug::add_label(device.clone(), &image_label, image);

        let memory = allocator.allocate_image(image, &image_label);

        let image_view = Self::create_image_view(device.clone(), image, format, usage);

        let view_label = format!("[Image View] {}", label);

        debug::add_label(device.clone(), &view_label, image_view);

        let layout = ImageLayout::Undefined;

        Image {
            label: image_label,
            device,
            image,
            image_view,
            format,
            usage,
            layout,
            extent,
            memory: Some(memory), // swapchain images don't need manual memory allocation
        }
    }

    pub(crate) fn with_raw(
        label: &str,
        device: Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self {
        let image_label = format!("[Image] {}", label);

        debug::add_label(device.clone(), &image_label, image);

        let image_view = Self::create_image_view(device.clone(), image, format, usage);

        let view_label = format!("[Image View] {}", label);
        debug::add_label(device.clone(), &view_label, image_view);

        let layout = ImageLayout::Undefined;

        Image {
            label: image_label,
            device,
            image,
            image_view,
            format,
            usage,
            layout,
            extent,
            memory: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.layout = ImageLayout::Undefined;
    }

    fn create_image(
        device: &Arc<Device>,
        extent: ImageExtent2D,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::Image {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::default()
                    .width(extent.width)
                    .height(extent.height)
                    .depth(1),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(VkFormat::from(format).into())
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        unsafe { device.raw().create_image(&image_info, None).unwrap() }
    }

    pub(crate) fn create_image_view(
        device: Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(VkFormat::from(format).into())
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(usage.into())
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe { device.raw().create_image_view(&view_info, None).unwrap() }
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image {}", self.label)
    }
}

impl Wrap<vk::Image> for Image {
    fn raw(&self) -> vk::Image {
        self.image
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        tracing::debug!("Drop image {}", self.label);
        unsafe {
            self.device.raw().destroy_image_view(self.image_view, None);
            if self.memory.is_some() {
                tracing::debug!("Destroy image {}", self.label);
                self.device.raw().destroy_image(self.image, None);
            }
        }
    }
}
