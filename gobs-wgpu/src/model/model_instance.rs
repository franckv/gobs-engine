use std::sync::Arc;

use crate::model::Model;

pub struct ModelInstance {
    pub model: Arc<Model>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub instance_count: usize,
}
