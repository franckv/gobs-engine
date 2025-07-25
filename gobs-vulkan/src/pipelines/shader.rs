use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use ash::vk;

use gobs_core::logger;

use crate::Wrap;
use crate::device::Device;
use crate::error::VulkanError;

pub enum ShaderType {
    Compute,
    Vertex,
    Fragment,
}

pub struct Shader {
    device: Arc<Device>,
    shader: vk::ShaderModule,
    pub ty: ShaderType,
}

impl Shader {
    pub fn from_file<P>(
        filename: P,
        device: Arc<Device>,
        ty: ShaderType,
    ) -> Result<Self, VulkanError>
    where
        P: AsRef<Path> + Debug,
    {
        let file = File::open(&filename)?;
        let reader = BufReader::new(file);

        let data: Vec<u8> = reader.bytes().filter_map(|b| b.ok()).collect();

        let shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            code_size: data.len(),
            p_code: data.as_ptr() as *const u32,
            _marker: std::marker::PhantomData,
        };

        let shader = unsafe {
            device
                .raw()
                .create_shader_module(&shader_info, None)
                .unwrap()
        };

        Ok(Shader { device, shader, ty })
    }
}

impl Wrap<vk::ShaderModule> for Shader {
    fn raw(&self) -> vk::ShaderModule {
        self.shader
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop shader");
        unsafe {
            self.device.raw().destroy_shader_module(self.shader, None);
        }
    }
}
