use thiserror::Error;

use gobs_vulkan::error::VulkanError;

#[derive(Debug, Error)]
pub enum RenderBackendError {
    #[error("vulkan error")]
    VulkanBackendError(#[from] VulkanError),
}
