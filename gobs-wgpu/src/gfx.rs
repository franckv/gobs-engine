use winit::window::Window;

use crate::camera::CameraResource;
use crate::light::LightResource;
use crate::scene::Scene;
use crate::model::{ DrawLight, DrawModel };

pub struct Gfx {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
}

impl Gfx {
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.size
    }

    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = unsafe {instance.create_surface(&window)}.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None
            },
            None
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
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

        let clear_color = wgpu::Color::BLACK;

        Gfx {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &scene.depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });

            render_pass.set_pipeline(&scene.render_pipeline().pipeline);
            render_pass.set_bind_group(1, &scene.camera().resource.bind_group, &[]);
            render_pass.set_bind_group(2, &scene.light().resource.bind_group, &[]);
            render_pass.set_vertex_buffer(1, scene.instance_buffer().slice(..));
            render_pass.draw_model_instanced(&scene.obj_model(), 0..scene.instances().len() as _);

            render_pass.set_pipeline(&scene.light_render_pipeline().pipeline);
            render_pass.set_bind_group(0, &scene.camera().resource.bind_group, &[]);
            render_pass.set_bind_group(1, &scene.light().resource.bind_group, &[]);
            render_pass.draw_light_model(&scene.obj_model());
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn create_camera_resource(&self, layout: &wgpu::BindGroupLayout) -> CameraResource {
        CameraResource::new(&self.device, layout)
    }

    pub fn create_light_resource(&self, layout: &wgpu::BindGroupLayout) -> LightResource {
        LightResource::new(&self.device, layout)
    }
}
