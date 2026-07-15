mod bounds;
#[allow(clippy::module_inception)]
mod mesh;
mod mesh_geometry;
mod mesh_loader;
mod shape;

pub use bounds::{Bounded, BoundingBox};
pub use mesh::{Mesh, MeshData, MeshPath, MeshPrimitiveType, MeshProperties};
pub use mesh_geometry::{MeshBuilder, MeshGeometry};
pub use mesh_loader::MeshLoader;
pub use shape::Shapes;
