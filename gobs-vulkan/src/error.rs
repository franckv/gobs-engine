use std::{ffi::NulError, io};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error("failed to create surface")]
    SurfaceCreateError,
    #[error("failed to create instance")]
    InstanceCreateError,
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("null error")]
    NULError(#[from] NulError),
    #[error("vk result")]
    VkResult(#[from] ash::vk::Result),
}
