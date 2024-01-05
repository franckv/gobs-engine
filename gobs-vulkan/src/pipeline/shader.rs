use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use ash::vk;

use log::trace;

use crate::device::Device;
use crate::Wrap;

pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct Shader {
    device: Arc<Device>,
    shader: vk::ShaderModule,
    pub ty: ShaderType,
}

impl Shader {
    pub fn from_file(filename: &str, device: Arc<Device>, ty: ShaderType) -> Self {
        let file = File::open(Path::new(filename)).unwrap();

        let data: Vec<u8> = file.bytes().filter_map(|b| b.ok()).collect();

        let shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            code_size: data.len(),
            p_code: data.as_ptr() as *const u32,
        };

        let shader = unsafe {
            device
                .raw()
                .create_shader_module(&shader_info, None)
                .unwrap()
        };

        Shader { device, shader, ty }
    }
}

impl Wrap<vk::ShaderModule> for Shader {
    fn raw(&self) -> vk::ShaderModule {
        self.shader
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        trace!("Drop shader");
        unsafe {
            self.device.raw().destroy_shader_module(self.shader, None);
        }
    }
}
