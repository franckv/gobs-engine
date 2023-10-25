use crate::{model::Model, shader::ShaderType};

pub struct ModelInstance {
    pub model: Model,
    pub shader: ShaderType,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub instance_count: usize,
}
