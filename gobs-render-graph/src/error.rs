use thiserror::Error;

use gobs_gfx::GfxError;

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
}
