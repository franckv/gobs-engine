use gobs_core::{Config, ConfigDefault, ImageFormat};

pub enum GltfConfig {
    TextureFormat,
}

impl AsRef<str> for GltfConfig {
    fn as_ref(&self) -> &str {
        match self {
            GltfConfig::TextureFormat => "config.gltf.texture.format",
        }
    }
}

impl ConfigDefault for GltfConfig {
    fn register_defaults(config: &mut Config) {
        config.set_image_format(GltfConfig::TextureFormat, ImageFormat::R8g8b8a8Srgb);
    }
}
