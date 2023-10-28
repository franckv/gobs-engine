use std::sync::Arc;

use crate::{model::Model, shader::Shader};

pub struct ModelInstance {
    pub model: Model,
    pub shader: Arc<Shader>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub instance_count: usize,
}
