#[allow(clippy::module_inception)]
mod mesh;
mod mesh_loader;

pub use mesh::{Mesh, MeshData, MeshPath, MeshPrimitiveType, MeshProperties};
pub use mesh_loader::MeshLoader;
