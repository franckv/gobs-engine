use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4]
}

pub struct CameraResource {
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup
}

impl CameraResource {
    pub fn new(
        device: &wgpu::Device, 
        layout: &wgpu::BindGroupLayout) -> Self {
        let uniform = CameraUniform {
            view_position: [0.; 4],
            view_proj: [[0.; 4]; 4]
        };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
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
            label: Some("camera_bind_group")
        });

        CameraResource {
            uniform,
            buffer, 
            bind_group
        }
    }

    pub fn update(&mut self, view_position: [f32; 4], view_proj: [[f32; 4]; 4]) {
        self.uniform.view_position = view_position;
        self.uniform.view_proj = view_proj;
    }
}