#[allow(clippy::module_inception)]
mod texture;
mod texture_loader;

pub use texture::{
    Texture, TextureData, TextureFormat, TexturePath, TextureProperties, TextureType, TextureUpdate,
};
pub use texture_loader::TextureLoader;
