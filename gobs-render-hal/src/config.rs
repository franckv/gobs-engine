use gobs_core::{ConfigDefault, ConfigWriter as _, GobsConfig};

pub enum RenderHalConfig {
    FramesInFlight,
    TextureArraySize,
    MaterialArraySize,
    MaterialDataSize,
}

impl AsRef<str> for RenderHalConfig {
    fn as_ref(&self) -> &str {
        match self {
            RenderHalConfig::FramesInFlight => "config.render.hal.frames_in_flight",
            RenderHalConfig::TextureArraySize => "config.render.hal.textures.array_size",
            RenderHalConfig::MaterialArraySize => "config.render.hal.materials.array_size",
            RenderHalConfig::MaterialDataSize => "config.render.hal.materials.data_size",
        }
    }
}

impl ConfigDefault for RenderHalConfig {
    fn register_defaults(config: &mut GobsConfig) {
        config.set_int(RenderHalConfig::FramesInFlight, 2);
        config.set_int(RenderHalConfig::TextureArraySize, 256);
        config.set_int(RenderHalConfig::MaterialArraySize, 256);
        config.set_int(RenderHalConfig::MaterialDataSize, 256);
    }
}
