use wgpu::util::DeviceExt;

use gobs_core as core;

use core::entity::uniform::UniformData;

use crate::context::Gfx;

pub struct UniformResource {
    pub data: UniformData,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UniformResource {
    pub fn new(gfx: &Gfx, data: UniformData) -> Self {
        let layout = gfx
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let buffer = gfx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} uniform buffer", &data.name)),
                contents: bytemuck::cast_slice(&data.raw()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&format!("{} bind group", &data.name)),
        });

        UniformResource {
            data,
            buffer,
            bind_group,
        }
    }

    pub fn update(&mut self, gfx: &Gfx) {
        gfx.queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data.raw()));
    }
}
