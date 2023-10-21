use log::*;
use wgpu::util::DeviceExt;

use crate::{render::Gfx, shader_data::LightUniform};

pub struct LightResource {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl LightResource {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            colour: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        info!("Create Light bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        LightResource {
            uniform,
            buffer,
            bind_group,
        }
    }

    //pub fn update(&mut self, gfx: &Gfx, light: &Light) {
    pub fn update(&mut self, gfx: &Gfx, position: [f32; 3], colour: [f32; 3]) {
        self.uniform.position = position;
        self.uniform.colour = colour;

        gfx.queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
