pub mod gltf_load;
pub mod manager;
pub mod shaders;

use thiserror::Error;

use gobs_render_graph::RenderError;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("GLTF error")]
    GLTFError(#[from] gltf::Error),
    #[error("render error")]
    RenderError(#[from] RenderError),
}
