use std::sync::Arc;

use gobs_core::memory::allocator::Allocable;
use gobs_gfx::{Buffer, BufferUsage, GfxBuffer, GfxDevice};
use gobs_resource::entity::uniform::UniformLayout;

pub struct UniformBuffer {
    pub layout: Arc<UniformLayout>,
    pub buffer: GfxBuffer,
}

impl UniformBuffer {
    pub fn new(device: &GfxDevice, layout: Arc<UniformLayout>) -> Self {
        let buffer = GfxBuffer::new("uniform", layout.size(), BufferUsage::Uniform, device);

        UniformBuffer { layout, buffer }
    }

    pub fn update(&mut self, uniform_data: &[u8]) {
        self.buffer.copy(uniform_data, 0);
    }
}

impl Allocable<GfxDevice, Arc<UniformLayout>> for UniformBuffer {
    fn family(&self) -> Arc<UniformLayout> {
        self.layout.clone()
    }

    fn size(&self) -> usize {
        1
    }

    fn allocate(device: &GfxDevice, _name: &str, _size: usize, layout: Arc<UniformLayout>) -> Self {
        Self::new(device, layout)
    }
}
