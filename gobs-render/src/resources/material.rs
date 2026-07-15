#[allow(clippy::module_inception)]
mod material;
mod material_config;
mod material_loader;

pub use material::{Material, MaterialData, MaterialProperties};
pub use material_config::MaterialsConfig;
pub use material_loader::MaterialLoader;
