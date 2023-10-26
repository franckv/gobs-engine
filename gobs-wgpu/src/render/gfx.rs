use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::model::CameraResource;
use crate::model::LightResource;
use crate::render::Display;
use crate::shader_data::InstanceData;
use crate::shader_data::VertexData;

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

pub struct Gfx {
    pub(crate) display: Display,
    device: wgpu::Device,
    queue: wgpu::Queue,
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

        Gfx {
            display,
            device,
            queue,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.display.resize(&self.device, width, height);
        }
    }

    pub fn create_camera_resource(&self, layout: &wgpu::BindGroupLayout) -> CameraResource {
        CameraResource::new(&self.device, layout)
    }

    pub fn create_light_resource(&self, layout: &wgpu::BindGroupLayout) -> LightResource {
        LightResource::new(&self.device, layout)
    }

    pub fn create_bind_group_layout<'a>(
        &self,
        layout: &wgpu::BindGroupLayoutDescriptor<'a>,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(layout)
    }

    pub fn create_vertex_buffer(&self, vertex_data: &Vec<VertexData>) -> wgpu::Buffer {
        let bytes = vertex_data
            .iter()
            .map(|v| v.raw())
            .flat_map(|s| s)
            .collect::<Vec<u8>>();

        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytes.as_slice(),
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

    pub fn create_atlas_buffer(&self, atlas: &Vec<f32>) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Atlas Buffer"),
                contents: bytemuck::cast_slice(atlas),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn create_instance_buffer(&self, instance_data: &Vec<InstanceData>) -> wgpu::Buffer {
        let bytes = instance_data
            .iter()
            .map(|d| d.raw())
            .flat_map(|s| s)
            .collect::<Vec<u8>>();

        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytes.as_slice(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn update_instance_buffer(
        &self,
        instance_buffer: &wgpu::Buffer,
        instance_data: &Vec<InstanceData>,
    ) {
        let bytes = instance_data
            .iter()
            .map(|d| d.raw())
            .flat_map(|s| s)
            .collect::<Vec<u8>>();

        self.queue
            .write_buffer(instance_buffer, 0, bytes.as_slice())
    }
}
