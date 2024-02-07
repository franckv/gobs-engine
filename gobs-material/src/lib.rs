mod instance;
mod material;
pub mod texture;
pub mod vertex;

pub use instance::MaterialInstance;
pub use material::color_mat::ColorMaterial;
pub use material::normal_mat::NormalMaterial;
pub use material::texture_mat::TextureMaterial;
pub use material::{Material, MaterialId};
