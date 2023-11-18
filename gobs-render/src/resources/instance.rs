use wgpu::util::DeviceExt;

use gobs_core as core;

use core::entity::instance::{InstanceData, InstanceFlag};

use crate::context::Gfx;

pub struct InstanceBuffer {
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: usize,
}

impl InstanceBuffer {
    pub fn new(gfx: &Gfx, instance_data: &[InstanceData], flags: InstanceFlag) -> Self {
        let bytes = instance_data
            .iter()
            .flat_map(|d| d.raw(flags))
            .collect::<Vec<u8>>();

        let instance_buffer = gfx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytes.as_slice(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        InstanceBuffer {
            instance_buffer,
            instance_count: instance_data.len(),
        }
    }

    pub fn update(&mut self, gfx: &Gfx, instance_data: &[InstanceData], flags: InstanceFlag) {
        let bytes = instance_data
            .iter()
            .flat_map(|d| d.raw(flags))
            .collect::<Vec<u8>>();

        gfx.queue()
            .write_buffer(&self.instance_buffer, 0, bytes.as_slice())
    }
}
