use std::sync::Arc;

use crate::context::Gfx;
use crate::graph::batch::Batch;
use crate::graph::pass::RenderPass;
use crate::model::{Texture, TextureType};
use crate::resources::ResourceManager;
use crate::shader::Shader;

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Error,
}

pub struct RenderGraph {
    name: String,
    resource_manager: ResourceManager,
    passes: Vec<RenderPass>,
    depth_texture: Texture,
}

impl RenderGraph {
    pub fn new(name: &str, gfx: &Gfx, shaders: &[Arc<Shader>]) -> Self {
        let depth_texture = Texture::new(
            gfx,
            "depth_texture",
            TextureType::DEPTH,
            gfx.width(),
            gfx.height(),
            &[],
        );

        let mut passes = vec![RenderPass::new("Forward Pass", true)];
        for shader in shaders {
            passes.push(RenderPass::with_shader(
                "Shader pass",
                shader.clone(),
                false,
            ));
        }

        RenderGraph {
            name: name.to_string(),
            resource_manager: ResourceManager::new(),
            passes,
            depth_texture,
        }
    }

    fn update_depth(&mut self, gfx: &Gfx) {
        let (width, height) = (gfx.width(), gfx.height());

        if self.depth_texture.width != width || self.depth_texture.height != height {
            self.depth_texture =
                Texture::new(gfx, "depth_texture", TextureType::DEPTH, width, height, &[]);
        }
    }

    pub fn execute(&mut self, gfx: &Gfx, batch: Batch<'_>) -> Result<(), RenderError> {
        let surface_texture = match gfx.display.read().unwrap().texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost) => return Err(RenderError::Lost),
            Err(wgpu::SurfaceError::Outdated) => return Err(RenderError::Outdated),
            Err(_) => return Err(RenderError::Error),
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.update_depth(gfx);

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&self.name),
            });

        for pass in &self.passes {
            pass.render(
                gfx,
                &mut encoder,
                &mut self.resource_manager,
                &surface_view,
                &self.depth_texture.view,
                &batch,
            );
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));

        surface_texture.present();

        Ok(())
    }
}
