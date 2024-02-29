use ash::vk;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Undefined,
    R4g4UnormPack8,
    R4g4b4a4UnormPack16,
    B4g4r4a4UnormPack16,
    R5g6b5UnormPack16,
    B5g6r5UnormPack16,
    R5g5b5a1UnormPack16,
    B5g5r5a1UnormPack16,
    A1r5g5b5UnormPack16,
    R8Unorm,
    R8Snorm,
    R8Uscaled,
    R8Sscaled,
    R8Uint,
    R8Sint,
    R8Srgb,
    R8g8Unorm,
    R8g8Snorm,
    R8g8Uscaled,
    R8g8Sscaled,
    R8g8Uint,
    R8g8Sint,
    R8g8Srgb,
    R8g8b8Unorm,
    R8g8b8Snorm,
    R8g8b8Uscaled,
    R8g8b8Sscaled,
    R8g8b8Uint,
    R8g8b8Sint,
    R8g8b8Srgb,
    B8g8r8Unorm,
    B8g8r8Snorm,
    B8g8r8Uscaled,
    B8g8r8Sscaled,
    B8g8r8Uint,
    B8g8r8Sint,
    B8g8r8Srgb,
    R8g8b8a8Unorm,
    R8g8b8a8Snorm,
    R8g8b8a8Uscaled,
    R8g8b8a8Sscaled,
    R8g8b8a8Uint,
    R8g8b8a8Sint,
    R8g8b8a8Srgb,
    B8g8r8a8Unorm,
    B8g8r8a8Snorm,
    B8g8r8a8Uscaled,
    B8g8r8a8Sscaled,
    B8g8r8a8Uint,
    B8g8r8a8Sint,
    B8g8r8a8Srgb,
    A8b8g8r8UnormPack32,
    A8b8g8r8SnormPack32,
    A8b8g8r8UscaledPack32,
    A8b8g8r8SscaledPack32,
    A8b8g8r8UintPack32,
    A8b8g8r8SintPack32,
    A8b8g8r8SrgbPack32,
    A2r10g10b10UnormPack32,
    A2r10g10b10SnormPack32,
    A2r10g10b10UscaledPack32,
    A2r10g10b10SscaledPack32,
    A2r10g10b10UintPack32,
    A2r10g10b10SintPack32,
    A2b10g10r10UnormPack32,
    A2b10g10r10SnormPack32,
    A2b10g10r10UscaledPack32,
    A2b10g10r10SscaledPack32,
    A2b10g10r10UintPack32,
    A2b10g10r10SintPack32,
    R16Unorm,
    R16Snorm,
    R16Uscaled,
    R16Sscaled,
    R16Uint,
    R16Sint,
    R16Sfloat,
    R16g16Unorm,
    R16g16Snorm,
    R16g16Uscaled,
    R16g16Sscaled,
    R16g16Uint,
    R16g16Sint,
    R16g16Sfloat,
    R16g16b16Unorm,
    R16g16b16Snorm,
    R16g16b16Uscaled,
    R16g16b16Sscaled,
    R16g16b16Uint,
    R16g16b16Sint,
    R16g16b16Sfloat,
    R16g16b16a16Unorm,
    R16g16b16a16Snorm,
    R16g16b16a16Uscaled,
    R16g16b16a16Sscaled,
    R16g16b16a16Uint,
    R16g16b16a16Sint,
    R16g16b16a16Sfloat,
    R32Uint,
    R32Sint,
    R32Sfloat,
    R32g32Uint,
    R32g32Sint,
    R32g32Sfloat,
    R32g32b32Uint,
    R32g32b32Sint,
    R32g32b32Sfloat,
    R32g32b32a32Uint,
    R32g32b32a32Sint,
    R32g32b32a32Sfloat,
    R64Uint,
    R64Sint,
    R64Sfloat,
    R64g64Uint,
    R64g64Sint,
    R64g64Sfloat,
    R64g64b64Uint,
    R64g64b64Sint,
    R64g64b64Sfloat,
    R64g64b64a64Uint,
    R64g64b64a64Sint,
    R64g64b64a64Sfloat,
    B10g11r11UfloatPack32,
    E5b9g9r9UfloatPack32,
    D16Unorm,
    X8D24UnormPack32,
    D32Sfloat,
    S8Uint,
    D16UnormS8Uint,
    D24UnormS8Uint,
    D32SfloatS8Uint,
    Bc1RgbUnormBlock,
    Bc1RgbSrgbBlock,
    Bc1RgbaUnormBlock,
    Bc1RgbaSrgbBlock,
    Bc2UnormBlock,
    Bc2SrgbBlock,
    Bc3UnormBlock,
    Bc3SrgbBlock,
    Bc4UnormBlock,
    Bc4SnormBlock,
    Bc5UnormBlock,
    Bc5SnormBlock,
    Bc6hUfloatBlock,
    Bc6hSfloatBlock,
    Bc7UnormBlock,
    Bc7SrgbBlock,
    Etc2R8g8b8UnormBlock,
    Etc2R8g8b8SrgbBlock,
    Etc2R8g8b8a1UnormBlock,
    Etc2R8g8b8a1SrgbBlock,
    Etc2R8g8b8a8UnormBlock,
    Etc2R8g8b8a8SrgbBlock,
    EacR11UnormBlock,
    EacR11SnormBlock,
    EacR11g11UnormBlock,
    EacR11g11SnormBlock,
    Astc4x4UnormBlock,
    Astc4x4SrgbBlock,
    Astc5x4UnormBlock,
    Astc5x4SrgbBlock,
    Astc5x5UnormBlock,
    Astc5x5SrgbBlock,
    Astc6x5UnormBlock,
    Astc6x5SrgbBlock,
    Astc6x6UnormBlock,
    Astc6x6SrgbBlock,
    Astc8x5UnormBlock,
    Astc8x5SrgbBlock,
    Astc8x6UnormBlock,
    Astc8x6SrgbBlock,
    Astc8x8UnormBlock,
    Astc8x8SrgbBlock,
    Astc10x5UnormBlock,
    Astc10x5SrgbBlock,
    Astc10x6UnormBlock,
    Astc10x6SrgbBlock,
    Astc10x8UnormBlock,
    Astc10x8SrgbBlock,
    Astc10x10UnormBlock,
    Astc10x10SrgbBlock,
    Astc12x10UnormBlock,
    Astc12x10SrgbBlock,
    Astc12x12UnormBlock,
    Astc12x12SrgbBlock,
}

impl From<vk::Format> for ImageFormat {
    fn from(format: vk::Format) -> ImageFormat {
        match format {
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
            _ => panic!("Format not implemented: {:?}", format),
        }
    }
}

impl Into<vk::Format> for ImageFormat {
    fn into(self) -> vk::Format {
        match self {
            ImageFormat::R8g8b8a8Unorm => vk::Format::R8G8B8A8_UNORM,
            ImageFormat::R16g16b16a16Sfloat => vk::Format::R16G16B16A16_SFLOAT,
            ImageFormat::R16g16b16a16Unorm => vk::Format::R16G16B16A16_UNORM,
            ImageFormat::B8g8r8a8Srgb => vk::Format::B8G8R8A8_SRGB,
            ImageFormat::R8g8b8a8Srgb => vk::Format::R8G8B8A8_SRGB,
            ImageFormat::R8g8b8Srgb => vk::Format::R8G8B8_SRGB,
            ImageFormat::B8g8r8a8Unorm => vk::Format::B8G8R8A8_UNORM,
            ImageFormat::R32g32Sfloat => vk::Format::R32G32_SFLOAT,
            ImageFormat::R32g32b32Sfloat => vk::Format::R32G32B32_SFLOAT,
            ImageFormat::R32g32b32a32Sfloat => vk::Format::R32G32B32A32_SFLOAT,
            ImageFormat::D32Sfloat => vk::Format::D32_SFLOAT,
            _ => panic!("Format not implemented: {:?}", self),
        }
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
