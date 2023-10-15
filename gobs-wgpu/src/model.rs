mod material;
mod mesh;
mod texture;

pub use material::Material;
pub use mesh::{Mesh, ModelVertex, Vertex};
pub use texture::Texture;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}
