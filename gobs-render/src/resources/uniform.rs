use gobs_gfx::{Buffer, BufferUsage, Renderer};

use crate::context::Context;

pub struct UniformBuffer<R: Renderer> {
    pub buffer: R::Buffer,
}

impl<R: Renderer> UniformBuffer<R> {
    pub fn new(ctx: &Context<R>, size: usize) -> Self {
        let buffer = R::Buffer::new("uniform", size, BufferUsage::Uniform, &ctx.device);

        UniformBuffer { buffer }
    }

    pub fn update(&mut self, uniform_data: &[u8]) {
        self.buffer.copy(uniform_data, 0);
    }
}
