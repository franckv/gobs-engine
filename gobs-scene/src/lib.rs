pub mod assets;
pub mod camera;
pub mod light;
pub mod model;
pub mod node;
pub mod scene;

use gobs_wgpu as render;

pub use render::render::Gfx;
pub use render::render::RenderError;
pub use render::shader::ShaderType;
