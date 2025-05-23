use gobs_gfx::{Buffer, BufferUsage, GfxBuffer};

use crate::GfxContext;

pub struct UniformBuffer {
    pub buffer: GfxBuffer,
}

impl UniformBuffer {
    pub fn new(ctx: &GfxContext, size: usize) -> Self {
        let buffer = GfxBuffer::new("uniform", size, BufferUsage::Uniform, &ctx.device);

        UniformBuffer { buffer }
    }

    pub fn update(&mut self, uniform_data: &[u8]) {
        self.buffer.copy(uniform_data, 0);
    }
}
