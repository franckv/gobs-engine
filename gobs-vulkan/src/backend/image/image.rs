use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use backend::device::Device;
use backend::image::ImageFormat;
use backend::memory::Memory;
use backend::Wrap;

#[derive(Copy, Clone)]
pub enum ImageLayout {
    Undefined,
    Transfer,
    Shader,
    Depth,
    Color
}

impl Into<vk::ImageLayout> for ImageLayout {
    fn into(self) -> vk::ImageLayout {
        match self {
            ImageLayout::Undefined => vk::ImageLayout::Undefined,
            ImageLayout::Transfer => vk::ImageLayout::TransferDstOptimal,
            ImageLayout::Shader => vk::ImageLayout::ShaderReadOnlyOptimal,
            ImageLayout::Depth => vk::ImageLayout::DepthStencilAttachmentOptimal,
            ImageLayout::Color => vk::ImageLayout::ColorAttachmentOptimal,
        }
    }
}

impl Into<vk::AccessFlags> for ImageLayout {
    fn into(self) -> vk::AccessFlags {
        match self {
            ImageLayout::Undefined => Default::default(),
            ImageLayout::Transfer => vk::ACCESS_TRANSFER_WRITE_BIT,
            ImageLayout::Shader => vk::ACCESS_SHADER_READ_BIT,
            ImageLayout::Depth => vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT |
                vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT,
            ImageLayout::Color => vk::ACCESS_COLOR_ATTACHMENT_READ_BIT |
                vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT
        }
    }
}

impl Into<vk::PipelineStageFlags> for ImageLayout {
    fn into(self) -> vk::PipelineStageFlags {
        match self {
            ImageLayout::Undefined => vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
            ImageLayout::Transfer => vk::PIPELINE_STAGE_TRANSFER_BIT,
            ImageLayout::Shader => vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
            ImageLayout::Depth => vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT,
            ImageLayout::Color => vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ImageUsage {
    Swapchain,
    Texture,
    Depth
}

impl Into<vk::ImageUsageFlags> for ImageUsage {
    fn into(self) -> vk::ImageUsageFlags {
        match self {
            ImageUsage::Swapchain => vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            ImageUsage::Texture => vk::IMAGE_USAGE_TRANSFER_DST_BIT |
                vk::IMAGE_USAGE_SAMPLED_BIT,
            ImageUsage::Depth => vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT
        }
    }
}

impl Into<vk::ImageAspectFlags> for ImageUsage {
    fn into(self) -> vk::ImageAspectFlags {
        match self {
            ImageUsage::Swapchain => vk::IMAGE_ASPECT_COLOR_BIT,
            ImageUsage::Texture => vk::IMAGE_ASPECT_COLOR_BIT,
            ImageUsage::Depth => vk::IMAGE_ASPECT_DEPTH_BIT
        }
    }
}

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
    pub fn new(device: Arc<Device>, format: ImageFormat,
               usage: ImageUsage,
               width: u32, height: u32) -> Self {
        let image = Self::create_image(&device, width, height,
                                       format, usage);

        let memory = Memory::with_image(device.clone(), image);

        let image_view = Self::create_image_view(&device, image,
                                                 format, usage);

        Image {
            device,
            image,
            image_view,
            usage,
            width,
            height,
            memory: Some(memory),
        }
    }

    pub(crate) fn with_raw(device: Arc<Device>,
                           image: vk::Image,
                           format: ImageFormat,
                           usage: ImageUsage,
                           width: u32, height: u32) -> Self {


        let image_view = Self::create_image_view(&device, image,
                                                 format, usage);

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

    fn create_image(device: &Arc<Device>, width: u32, height: u32,
                    format: ImageFormat, usage: ImageUsage) -> vk::Image {
        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::ImageCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            image_type: vk::ImageType::Type2d,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format: format.into(),
            tiling: vk::ImageTiling::Optimal,
            initial_layout: vk::ImageLayout::Undefined,
            usage: usage.into(),
            sharing_mode: vk::SharingMode::Exclusive,
            samples: vk::SAMPLE_COUNT_1_BIT,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        unsafe {
            device.raw().create_image(&image_info,
                                       None).unwrap()
        }
    }

    pub(crate) fn create_image_view(device: &Arc<Device>,
                                    image: vk::Image,
                                    format: ImageFormat,
                                    usage: ImageUsage) -> vk::ImageView {

        let view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::ImageViewCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            image,
            view_type: vk::ImageViewType::Type2d,
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

        unsafe {
            device.raw().create_image_view(&view_info,
                                            None).unwrap()
        }
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