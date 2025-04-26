pub mod gltf_load;
pub mod manager;

use thiserror::Error;

use gobs_render::RenderError;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("GLTF error")]
    GLTFError(#[from] gltf::Error),
    #[error("render error")]
    RenderError(#[from] RenderError),
}
