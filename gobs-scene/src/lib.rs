pub mod assets;
pub mod camera;
pub mod light;
pub mod node;
pub mod scene;
pub mod shape;
pub mod transform;

use gobs_wgpu as render;

pub use render::model::{Material, MaterialBuilder, Mesh, MeshBuilder, Model, ModelBuilder};
pub use render::render::Gfx;
pub use render::render::RenderError;
