use gobs_resource::load::LoadingError;
use gobs_vulkan::error::VulkanError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GfxError {
    #[error("failed to get DS pool")]
    DsPoolCreation,
    #[error("failed to create device")]
    DeviceCreate,
    #[error("vulkan error")]
    VulkanError(#[from] VulkanError),
    #[error("loading error")]
    LoadingError(#[from] LoadingError),
}
