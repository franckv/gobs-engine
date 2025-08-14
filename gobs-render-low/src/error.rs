use thiserror::Error;

use gobs_gfx::GfxError;

use crate::job::RenderJobError;

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
    #[error("render job error")]
    RenderJob(#[from] RenderJobError),
    #[error("invalid data")]
    InvalidData,
}
