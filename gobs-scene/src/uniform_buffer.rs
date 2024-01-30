use std::sync::Arc;

use gobs_core::entity::uniform::UniformData;
use gobs_render::context::Context;
use gobs_vulkan::{
    alloc::Allocator,
    buffer::{Buffer, BufferUsage},
    descriptor::DescriptorSetLayout,
};

pub struct UniformBuffer {
    pub ds_layout: Arc<DescriptorSetLayout>,
    pub buffer: Buffer,
}

impl UniformBuffer {
    pub fn new(
        ctx: &Context,
        ds_layout: Arc<DescriptorSetLayout>,
        size: usize,
        allocator: Arc<Allocator>,
    ) -> Self {
        let buffer = Buffer::new(
            "uniform",
            size,
            BufferUsage::Uniform,
            ctx.device.clone(),
            allocator,
        );

        UniformBuffer { ds_layout, buffer }
    }

    pub fn update(&mut self, uniform_data: &UniformData) {
        self.buffer.copy(&uniform_data.raw(), 0);
    }
}
