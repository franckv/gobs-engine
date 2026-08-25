use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("swapchain lost")]
    Lost,
    #[error("swapchain updated")]
    Outdated,
    #[error("pass not found")]
    PassNotFound,
    #[error("invalid data")]
    InvalidData,
}
