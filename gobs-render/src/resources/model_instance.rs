use wgpu::util::DeviceExt;

use crate::{
    context::Gfx,
    model::{InstanceData, InstanceFlag},
};

pub struct ModelInstance {
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: usize,
}

impl ModelInstance {
    pub fn new(gfx: &Gfx, instance_data: &Vec<InstanceData>, flags: InstanceFlag) -> Self {
        let bytes = instance_data
            .iter()
            .map(|d| d.raw(flags))
            .flat_map(|s| s)
            .collect::<Vec<u8>>();

        let instance_buffer = gfx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytes.as_slice(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        ModelInstance {
            instance_buffer,
            instance_count: instance_data.len(),
        }
    }

    pub fn update(&mut self, gfx: &Gfx, instance_data: &Vec<InstanceData>, flags: InstanceFlag) {
        let bytes = instance_data
            .iter()
            .map(|d| d.raw(flags))
            .flat_map(|s| s)
            .collect::<Vec<u8>>();

        gfx.queue()
            .write_buffer(&self.instance_buffer, 0, bytes.as_slice())
    }
}
