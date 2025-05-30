use thiserror::Error;

use gobs_gfx::GfxError;
use gobs_resource::resource::ResourceError;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("swapchain lost")]
    Lost,
    #[error("swapchain updated")]
    Outdated,
    #[error("pass not found")]
    PassNotFound,
    #[error("gfx error")]
    Gfx(#[from] GfxError),
    #[error("resource error")]
    ResourceError(#[from] ResourceError),
}
