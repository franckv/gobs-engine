use gobs_core::{ConfigDefault, ConfigWriter as _, GobsConfig, ImageFormat};

pub enum RenderHalConfig {
    FramesInFlight,
    TextureArraySize,
    MaterialArraySize,
    MaterialDataSize,
    InstanceArraySize,
    InstanceDataSize,
    SwapchainFormat,
}

impl AsRef<str> for RenderHalConfig {
    fn as_ref(&self) -> &str {
        match self {
            RenderHalConfig::FramesInFlight => "config.render.hal.frames_in_flight",
            RenderHalConfig::TextureArraySize => "config.render.hal.textures.array_size",
            RenderHalConfig::MaterialArraySize => "config.render.hal.materials.array_size",
            RenderHalConfig::MaterialDataSize => "config.render.hal.materials.data_size",
            RenderHalConfig::InstanceArraySize => "config.render.hal.instances.array_size",
            RenderHalConfig::InstanceDataSize => "config.render.hal.instances.data_size",
            RenderHalConfig::SwapchainFormat => "config.render.hal.swapchain.format",
        }
    }
}

impl ConfigDefault for RenderHalConfig {
    fn register_defaults(config: &mut GobsConfig) {
        config.set_int(RenderHalConfig::FramesInFlight, 2);
        config.set_int(RenderHalConfig::TextureArraySize, 256);
        config.set_int(RenderHalConfig::MaterialArraySize, 256);
        config.set_int(RenderHalConfig::MaterialDataSize, 256);
        config.set_int(RenderHalConfig::InstanceArraySize, 10000);
        config.set_int(RenderHalConfig::InstanceDataSize, 256);
        config.set_image_format(RenderHalConfig::SwapchainFormat, ImageFormat::B8g8r8a8Srgb);
    }
}
