use ash::vk;

use gobs_core::ImageFormat;

pub struct VkFormat(vk::Format);

impl From<VkFormat> for ImageFormat {
    fn from(format: VkFormat) -> ImageFormat {
        match format.0 {
            vk::Format::R8G8B8A8_UNORM => ImageFormat::R8g8b8a8Unorm,
            vk::Format::R16G16B16A16_SFLOAT => ImageFormat::R16g16b16a16Sfloat,
            vk::Format::R16G16B16A16_UNORM => ImageFormat::R16g16b16a16Unorm,
            vk::Format::B8G8R8A8_SRGB => ImageFormat::B8g8r8a8Srgb,
            vk::Format::R8G8B8A8_SRGB => ImageFormat::R8g8b8a8Srgb,
            vk::Format::B8G8R8A8_UNORM => ImageFormat::B8g8r8a8Unorm,
            vk::Format::R32G32_SFLOAT => ImageFormat::R32g32Sfloat,
            vk::Format::R32G32B32_SFLOAT => ImageFormat::R32g32b32Sfloat,
            vk::Format::R32G32B32A32_SFLOAT => ImageFormat::R32g32b32a32Sfloat,
            vk::Format::D32_SFLOAT => ImageFormat::D32Sfloat,
            vk::Format::A2B10G10R10_UNORM_PACK32 => ImageFormat::A2b10g10r10UnormPack32,
            _ => panic!("Format not implemented: {:?}", format.0),
        }
    }
}

impl From<ImageFormat> for VkFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::R8g8b8a8Unorm => VkFormat(vk::Format::R8G8B8A8_UNORM),
            ImageFormat::R16g16b16a16Sfloat => VkFormat(vk::Format::R16G16B16A16_SFLOAT),
            ImageFormat::R16g16b16a16Unorm => VkFormat(vk::Format::R16G16B16A16_UNORM),
            ImageFormat::B8g8r8a8Srgb => VkFormat(vk::Format::B8G8R8A8_SRGB),
            ImageFormat::R8g8b8a8Srgb => VkFormat(vk::Format::R8G8B8A8_SRGB),
            ImageFormat::R8g8b8Srgb => VkFormat(vk::Format::R8G8B8_SRGB),
            ImageFormat::B8g8r8a8Unorm => VkFormat(vk::Format::B8G8R8A8_UNORM),
            ImageFormat::R32g32Sfloat => VkFormat(vk::Format::R32G32_SFLOAT),
            ImageFormat::R32g32b32Sfloat => VkFormat(vk::Format::R32G32B32_SFLOAT),
            ImageFormat::R32g32b32a32Sfloat => VkFormat(vk::Format::R32G32B32A32_SFLOAT),
            ImageFormat::D32Sfloat => VkFormat(vk::Format::D32_SFLOAT),
            _ => panic!("Format not implemented: {:?}", value),
        }
    }
}

impl From<VkFormat> for vk::Format {
    fn from(value: VkFormat) -> Self {
        value.0
    }
}

impl From<vk::Format> for VkFormat {
    fn from(value: vk::Format) -> Self {
        Self(value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColorSpace {
    SrgbNonlinear,
}

impl From<vk::ColorSpaceKHR> for ColorSpace {
    fn from(color_space: vk::ColorSpaceKHR) -> ColorSpace {
        match color_space {
            _ => ColorSpace::SrgbNonlinear,
        }
    }
}

impl Into<vk::ColorSpaceKHR> for ColorSpace {
    fn into(self) -> vk::ColorSpaceKHR {
        vk::ColorSpaceKHR::SRGB_NONLINEAR
    }
}
