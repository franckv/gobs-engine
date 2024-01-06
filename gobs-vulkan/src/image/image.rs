use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::image::ImageFormat;
use crate::memory::Memory;
use crate::Wrap;

#[derive(Copy, Clone)]
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

#[derive(Clone, Copy, Debug)]
pub struct ImageExtent2D {
    pub width: u32,
    pub height: u32,
}

impl ImageExtent2D {
    pub fn new(width: u32, height: u32) -> Self {
        ImageExtent2D { width, height }
    }
}

impl Into<vk::Extent2D> for ImageExtent2D {
    fn into(self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }
}

impl From<(u32, u32)> for ImageExtent2D {
    fn from(value: (u32, u32)) -> Self {
        ImageExtent2D::new(value.0, value.1)
    }
}

/// Image buffer allocated in memory
pub struct Image {
    device: Arc<Device>,
    image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub usage: ImageUsage,
    pub extent: ImageExtent2D,
    memory: Option<Memory>,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self {
        let image = Self::create_image(&device, extent, format, usage);

        let memory = Memory::with_image(device.clone(), image);

        let image_view = Self::create_image_view(&device, image, format, usage);

        Image {
            device,
            image,
            image_view,
            usage,
            extent,
            memory: Some(memory), // swapchain images don't need manual memory allocation
        }
    }

    pub(crate) fn with_raw(
        device: Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self {
        let image_view = Self::create_image_view(&device, image, format, usage);

        Image {
            device,
            image,
            image_view,
            usage,
            extent,
            memory: None,
        }
    }

    fn create_image(
        device: &Arc<Device>,
        extent: ImageExtent2D,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::Image {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::builder()
                    .width(extent.width)
                    .height(extent.height)
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(1)
            .format(format.into())
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .queue_family_indices(&[])
            .build();

        unsafe { device.raw().create_image(&image_info, None).unwrap() }
    }

    pub(crate) fn create_image_view(
        device: &Arc<Device>,
        image: vk::Image,
        format: ImageFormat,
        usage: ImageUsage,
    ) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format.into())
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(usage.into())
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .components(
                vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::R)
                    .g(vk::ComponentSwizzle::G)
                    .b(vk::ComponentSwizzle::B)
                    .a(vk::ComponentSwizzle::A)
                    .build(),
            )
            .build();

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
        log::info!("Drop image");
        unsafe {
            self.device.raw().destroy_image_view(self.image_view, None);
            if self.memory.is_some() {
                self.device.raw().destroy_image(self.image, None);
            }
        }
    }
}
