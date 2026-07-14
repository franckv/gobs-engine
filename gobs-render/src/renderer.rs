use gobs_core::{ImageExtent2D, logger};
use gobs_render_graph::{FrameData, FrameGraph, GfxContext, PassType, RenderError};
use gobs_resource::ResourceManager;

use crate::RenderBatch;

#[derive(Debug)]
pub struct RendererOptions {
    pub graph: String,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            graph: "scene".to_string(),
        }
    }
}

pub struct Renderer {
    pub graph: FrameGraph,
    pub gfx: GfxContext,
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
}

impl Renderer {
    pub fn new(
        mut gfx: GfxContext,
        options: &RendererOptions,
        resource_manager: &mut ResourceManager,
    ) -> Self {
        let graph = FrameGraph::load(&mut gfx, resource_manager, &options.graph).unwrap();

        let frames_in_flight = gfx.frames_in_flight;

        let frames = (0..frames_in_flight)
            .map(|id| FrameData::new(&mut gfx, id, frames_in_flight))
            .collect();

        Self {
            graph,
            gfx,
            frames,
            frame_number: 0,
        }
    }

    pub fn extent(&self) -> ImageExtent2D {
        self.gfx.extent()
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        self.graph.resize(&mut self.gfx);
    }

    pub fn update(&mut self, delta: f32) {
        self.graph.update(&self.gfx, delta);
    }

    pub fn enable_pass(&mut self, pass: PassType, enabled: bool) {
        self.graph.enable_pass(pass, enabled);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn prepare(
        &mut self,
        resource_manager: &mut ResourceManager,
        draw_cmd: &mut dyn FnMut(
            &mut GfxContext,
            &mut RenderBatch,
            &mut ResourceManager,
        ) -> Result<(), RenderError>,
    ) -> Result<RenderBatch, RenderError> {
        tracing::debug!(target: logger::RENDER, "Prepare render batch");

        let mut batch = RenderBatch::new(&self.gfx);

        draw_cmd(&mut self.gfx, &mut batch, resource_manager)?;

        batch.finish(&mut self.gfx, resource_manager);

        Ok(batch)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw(&mut self, batch: &mut RenderBatch) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Begin render batch");

        self.frame_number += 1;

        tracing::debug!(target: logger::RENDER, "Begin new frame {}", self.frame_number);

        let frame = &mut self.frames[self.frame_number % self.gfx.frames_in_flight];

        frame.reset(self.frame_number);
        self.gfx.new_frame(self.frame_number);

        self.graph.begin(&mut self.gfx, frame)?;

        self.graph.render(
            &mut self.gfx,
            frame,
            &batch.render_list,
            &batch.scene_data(),
        )?;

        self.graph.end(&mut self.gfx, frame)?;

        Ok(())
    }

    pub fn frame(&self) -> &FrameData {
        &self.frames[self.frame_number % self.gfx.frames_in_flight]
    }

    pub fn frame_number(&self) -> usize {
        self.frame_number
    }

    pub fn wait(&mut self) {
        self.gfx.hal.wait();
    }
}
