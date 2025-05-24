use gobs_core::ImageExtent2D;
use gobs_gfx::Device;
use gobs_render_graph::{FrameGraph, GfxContext, RenderPass};

use crate::RenderBatch;

pub struct Renderer {
    pub graph: FrameGraph,
    pub batch: RenderBatch,
    pub gfx: GfxContext,
}

impl Renderer {
    pub fn new(gfx: GfxContext) -> Self {
        Self {
            graph: FrameGraph::default(&gfx).unwrap(),
            batch: RenderBatch::new(&gfx),
            gfx,
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

    pub fn draw(&mut self, draw_cmd: &mut dyn FnMut(RenderPass, &mut RenderBatch)) {
        tracing::debug!(target: "render", "Begin render batch");

        self.batch.reset();

        self.graph.begin(&mut self.gfx).unwrap();

        for pass in &self.graph.passes {
            draw_cmd(pass.clone(), &mut self.batch);
        }

        self.batch.finish();

        self.graph
            .render(&mut self.gfx, &self.batch.render_list, &|pass| {
                self.batch.scene_data(pass)
            })
            .unwrap();

        self.graph.end(&mut self.gfx).unwrap();
    }

    pub fn frame_number(&self) -> usize {
        self.graph.frame_number
    }

    pub fn wait(&self) {
        self.gfx.device.wait();
    }
}
