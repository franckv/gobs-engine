use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub colour: [f32; 3],
    pub _padding2: u32,
}

pub struct LightResource {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup
}

impl LightResource {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            colour: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ],
            label: None
        });

        LightResource {
            uniform,
            buffer,
            bind_group
        }
    }

    pub fn update(&mut self, position: [f32; 3], colour: [f32; 3]) {
        self.uniform.position = position;
        self.uniform.colour = colour;
    }
}
