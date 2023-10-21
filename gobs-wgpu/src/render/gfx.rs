use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::model::CameraResource;
use crate::model::InstanceRaw;
use crate::model::LightResource;
use crate::model::ModelVertex;
use crate::model::{Model, Texture};
use crate::render::Display;
use crate::shader::{DrawPhong, DrawSolid};
use crate::shader::{PhongShader, SolidShader};

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

pub struct Gfx {
    display: Display,
    device: wgpu::Device,
    queue: wgpu::Queue,
    clear_color: wgpu::Color,
}

impl Gfx {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn format(&self) -> &wgpu::TextureFormat {
        &self.display.format()
    }

    pub fn width(&self) -> u32 {
        self.display.width()
    }

    pub fn height(&self) -> u32 {
        self.display.height()
    }

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let display = Display::new(surface, config, &device);

        let clear_color = wgpu::Color::BLACK;

        Gfx {
            display,
            device,
            queue,
            clear_color,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.display.resize(&self.device, width, height);
        }
    }

    pub fn render(
        &self,
        depth_texture: &Texture,
        camera_resource: &CameraResource,
        light_resource: &LightResource,
        light_model: &Model,
        solid_shader: &SolidShader,
        phong_shader: &PhongShader,
        models: &Vec<Model>,
        instance_buffers: &Vec<wgpu::Buffer>,
        instance_count: &Vec<usize>
    ) -> Result<(), RenderError> {
        let texture = match self.display.texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost) => return Err(RenderError::Lost),
            Err(wgpu::SurfaceError::Outdated) => return Err(RenderError::Outdated),
            Err(_) => return Err(RenderError::Error),
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for i in 0..models.len() {
                render_pass.draw_phong(
                    phong_shader,
                    &models[i],
                    camera_resource,
                    light_resource,
                    &instance_buffers[i],
                    instance_count[i] as _,
                );
            }

            render_pass.draw_solid(solid_shader, light_model, camera_resource, light_resource);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        texture.present();

        Ok(())
    }

    pub fn create_camera_resource(&self, layout: &wgpu::BindGroupLayout) -> CameraResource {
        CameraResource::new(&self.device, layout)
    }

    pub fn create_light_resource(&self, layout: &wgpu::BindGroupLayout) -> LightResource {
        LightResource::new(&self.device, layout)
    }

    pub fn create_bind_group_layout(
        &self,
        layout: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(layout)
    }

    pub fn create_vertex_buffer(&self, vertices: &Vec<ModelVertex>) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    pub fn create_index_buffer(&self, indices: &Vec<u32>) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            })
    }

    pub fn create_instance_buffer(&self, instance_data: &Vec<InstanceRaw>) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn update_instance_buffer(
        &self,
        instance_buffer: &wgpu::Buffer,
        instance_data: &Vec<InstanceRaw>,
    ) {
        self.queue
            .write_buffer(instance_buffer, 0, bytemuck::cast_slice(instance_data))
    }
}
