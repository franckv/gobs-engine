use std::sync::Arc;

use anyhow::Result;
use winit::window::Window;

use gobs_vulkan as vk;

use crate::Instance;
use crate::backend::vulkan::renderer::VkRenderer;

pub struct VkInstance {
    pub(crate) instance: Arc<vk::instance::Instance>,
}

impl Instance<VkRenderer> for VkInstance {
    fn new(name: &str, window: Option<&Window>, validation: bool) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            instance: vk::instance::Instance::new(name, 1, window, validation)?,
        }))
    }
}
