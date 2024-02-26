mod instance;
mod material;
mod texture;

pub use instance::{MaterialInstance, MaterialInstanceId};
pub use material::{Material, MaterialBuilder, MaterialId, MaterialProperty};
pub use texture::{Texture, TextureType};
