use gobs_gfx::{Buffer, BufferUsage, GfxBuffer, GfxDevice};

pub struct UniformBuffer {
    pub buffer: GfxBuffer,
}

impl UniformBuffer {
    pub fn new(device: &GfxDevice, size: usize) -> Self {
        let buffer = GfxBuffer::new("uniform", size, BufferUsage::Uniform, device);

        UniformBuffer { buffer }
    }

    pub fn update(&mut self, uniform_data: &[u8]) {
        self.buffer.copy(uniform_data, 0);
    }
}
