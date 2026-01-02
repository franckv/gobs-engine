use thiserror::Error;

use crate::job::RenderJobError;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("swapchain lost")]
    Lost,
    #[error("swapchain updated")]
    Outdated,
    #[error("pass not found")]
    PassNotFound,
    #[error("render job error")]
    RenderJob(#[from] RenderJobError),
    #[error("invalid data")]
    InvalidData,
}
