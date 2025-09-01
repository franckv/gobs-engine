#[allow(clippy::module_inception)]
mod material_instance;
mod material_instance_loader;

pub use material_instance::{MaterialInstance, MaterialInstanceData, MaterialInstanceProperties};
pub use material_instance_loader::MaterialInstanceLoader;
