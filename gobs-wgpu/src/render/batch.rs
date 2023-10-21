use crate::{
    model::{CameraResource, LightResource, Model, Texture},
    shader::{Shader, ShaderDraw},
};

use crate::render::Gfx;
use crate::render::RenderError;

pub struct BatchItem<'a> {
    shader: &'a Shader,
    model: &'a Model,
    instances_buffer: Option<&'a wgpu::Buffer>,
    instances_count: usize,
}

pub struct BatchBuilder<'a> {
    gfx: &'a Gfx,
    depth_texture: Option<&'a Texture>,
    camera_resource: Option<&'a CameraResource>,
    light_resource: Option<&'a LightResource>,
    items: Vec<BatchItem<'a>>,
}

impl<'a> BatchBuilder<'a> {
    pub fn new(gfx: &'a Gfx) -> Self {
        BatchBuilder {
            gfx,
            depth_texture: None,
            camera_resource: None,
            light_resource: None,
            items: Vec::new(),
        }
    }

    pub fn depth_texture(mut self, depth_texture: &'a Texture) -> Self {
        self.depth_texture = Some(depth_texture);

        self
    }

    pub fn camera_resource(mut self, camera_resource: &'a CameraResource) -> Self {
        self.camera_resource = Some(camera_resource);

        self
    }

    pub fn light_resource(mut self, light_resource: &'a LightResource) -> Self {
        self.light_resource = Some(light_resource);

        self
    }

    pub fn draw(mut self, model: &'a Model, shader: &'a Shader) -> Self {
        let item = BatchItem {
            model,
            shader,
            instances_buffer: None,
            instances_count: 0,
        };

        self.items.push(item);

        self
    }

    pub fn draw_indexed(
        mut self,
        model: &'a Model,
        shader: &'a Shader,
        instances_buffer: &'a wgpu::Buffer,
        instances_count: usize,
    ) -> Self {
        let item = BatchItem {
            model,
            shader,
            instances_buffer: Some(instances_buffer),
            instances_count,
        };

        self.items.push(item);

        self
    }

    pub fn finish(self) -> Batch<'a> {
        Batch {
            gfx: self.gfx,
            depth_texture: self.depth_texture.unwrap(),
            camera_resource: self.camera_resource.unwrap(),
            light_resource: self.light_resource.unwrap(),
            items: self.items,
        }
    }
}

pub struct Batch<'a> {
    gfx: &'a Gfx,
    depth_texture: &'a Texture,
    camera_resource: &'a CameraResource,
    light_resource: &'a LightResource,
    items: Vec<BatchItem<'a>>,
}

impl<'a> Batch<'a> {
    pub fn begin(gfx: &'a Gfx) -> BatchBuilder<'a> {
        BatchBuilder::new(gfx)
    }

    pub fn render(&self) -> Result<(), RenderError> {
        let texture = match self.gfx.display.texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost) => return Err(RenderError::Lost),
            Err(wgpu::SurfaceError::Outdated) => return Err(RenderError::Outdated),
            Err(_) => return Err(RenderError::Error),
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.gfx
                .device()
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for item in &self.items {
                match item.shader {
                    Shader::Phong(shader) => {
                        shader.draw_instanced(
                            &mut render_pass,
                            item.model,
                            self.camera_resource,
                            self.light_resource,
                            item.instances_buffer.unwrap(),
                            item.instances_count as _,
                        );
                    }
                    Shader::Solid(shader) => shader.draw_instanced(
                        &mut render_pass,
                        item.model,
                        self.camera_resource,
                        self.light_resource,
                        item.instances_buffer.unwrap(),
                        item.instances_count as _,
                    ),
                }
            }
        };

        self.gfx.queue().submit(std::iter::once(encoder.finish()));

        texture.present();

        Ok(())
    }
}
