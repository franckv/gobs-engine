use gobs_render_graph::{FrameGraph, GfxContext, RenderPass};

use crate::RenderBatch;

pub struct Renderer {
    pub graph: FrameGraph,
    pub batch: RenderBatch,
}

impl Renderer {
    pub fn new(ctx: &GfxContext) -> Self {
        Self {
            graph: FrameGraph::default(ctx).unwrap(),
            batch: RenderBatch::new(),
        }
    }

    pub fn resize(&mut self, ctx: &mut GfxContext, _width: u32, _height: u32) {
        self.graph.resize(ctx);
    }

    pub fn update(&mut self, ctx: &GfxContext, delta: f32) {
        self.graph.update(ctx, delta);
    }

    pub fn begin(&mut self, ctx: &mut GfxContext) {
        tracing::debug!(target: "render", "Begin render batch");

        self.batch.reset();

        self.graph.begin(ctx).unwrap();
    }

    pub fn draw(&mut self, draw_cmd: &mut dyn FnMut(RenderPass, &mut RenderBatch)) {
        for pass in &self.graph.passes {
            draw_cmd(pass.clone(), &mut self.batch);
        }

        self.batch.finish();
    }

    pub fn end(&mut self, ctx: &mut GfxContext) {
        let passes = self.graph.passes.clone();
        for pass in passes {
            self.graph
                .render(
                    ctx,
                    pass.clone(),
                    &self.batch.render_list,
                    self.batch.scene_data(pass.id()),
                )
                .unwrap();
        }

        self.graph.end(ctx).unwrap();
    }

    pub fn frame_number(&self) -> usize {
        self.graph.frame_number
    }
}
