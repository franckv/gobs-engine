use gobs_core::logger;
use gobs_render_graph::RenderError;
use gobs_resource::ResourceManager;

use crate::{RenderBatch, Renderable, Renderer, data::RenderFlags};

#[derive(Clone, Copy, Debug)]
pub enum RenderType {
    Scene,
    Ui,
}

impl From<RenderType> for RenderFlags {
    fn from(ty: RenderType) -> Self {
        match ty {
            RenderType::Scene => RenderFlags::ENTITY,
            RenderType::Ui => RenderFlags::UI,
        }
    }
}

pub struct RenderBuilder<'a> {
    batch: RenderBatch,
    renderer: &'a mut Renderer,
    resource_manager: &'a mut ResourceManager,
}

impl<'a> RenderBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer, resource_manager: &'a mut ResourceManager) -> Self {
        tracing::trace!(target: logger::RENDER, "Render frame {}", renderer.frame_number());

        Self {
            batch: RenderBatch::new(),
            renderer,
            resource_manager,
        }
    }

    pub fn with_renderable<T: Renderable>(
        mut self,
        renderable: &T,
        ty: RenderType,
    ) -> Result<Self, RenderError> {
        tracing::debug!("Draw renderable: {:?}", ty);
        renderable
            .draw(
                &mut self.renderer.gfx,
                self.resource_manager,
                &mut self.batch,
                None,
                None,
                ty.into(),
            )
            .map_err(|_| RenderError::InvalidData)?;

        Ok(self)
    }

    pub fn draw_bounds(mut self, draw_bounds: bool) -> Self {
        self.renderer.enable_pass("bounds", draw_bounds);
        self.batch.generate_bounds(draw_bounds);

        self
    }

    pub fn draw_wire(self, draw_wire: bool) -> Self {
        self.renderer.enable_pass("wire", draw_wire);

        self
    }

    pub fn build(mut self) -> Result<(), RenderError> {
        self.batch
            .finish(&mut self.renderer.gfx, self.resource_manager);

        self.renderer.submit(&mut self.batch)?;

        tracing::trace!(target: logger::RENDER, "End render");

        Ok(())
    }
}
