use gobs_core::ImageExtent2D;
use gobs_gfx::Device;
use gobs_render_graph::{FrameData, FrameGraph, GfxContext, PassType, RenderError, RenderPass};

use crate::RenderBatch;

pub struct Renderer {
    pub graph: FrameGraph,
    pub batch: RenderBatch,
    pub gfx: GfxContext,
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
}

impl Renderer {
    pub fn new(gfx: GfxContext) -> Self {
        let frames = (0..gfx.frames_in_flight)
            .map(|id| FrameData::new(&gfx, id))
            .collect();

        Self {
            graph: FrameGraph::default(&gfx).unwrap(),
            batch: RenderBatch::new(&gfx),
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

    fn new_frame(&mut self) {
        self.frame_number += 1;
        let frame_id = self.frame_number % self.gfx.frames_in_flight;

        tracing::debug!(target: "render", "Begin new frame: {} ({}/{})", self.frame_number, frame_id, self.gfx.frames_in_flight);

        let frame = &mut self.frames[frame_id];
        assert_eq!(frame.id, frame_id);

        frame.frame_number = self.frame_number;

        tracing::debug!(target: "sync", "Wait for frame: {} ({}/{})", self.frame_number, frame_id, self.gfx.frames_in_flight);

        frame.reset();
    }

    pub fn draw(
        &mut self,
        draw_cmd: &mut dyn FnMut(RenderPass, &mut RenderBatch) -> Result<(), RenderError>,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: "render", "Begin render batch");

        self.new_frame();

        let frame = &self.frames[self.frame_number % self.gfx.frames_in_flight];

        {
            //TODO: let mut buf = [0 as u64; 2];
            //frame.query_pool.get_query_pool_results(0, &mut buf);

            //self.batch.render_stats.gpu_draw_time =
            //    ((buf[1] - buf[0]) as f32 * frame.query_pool.period) / 1_000_000_000.;
        }

        self.graph.begin(&mut self.gfx, frame).unwrap();

        self.batch.reset();

        for pass in &self.graph.passes {
            draw_cmd(pass.clone(), &mut self.batch)?;
        }

        self.batch.finish();

        self.graph
            .render(&mut self.gfx, frame, &self.batch.render_list, &|pass| {
                self.batch.scene_data(pass)
            })
            .unwrap();

        self.graph.end(&mut self.gfx, frame).unwrap();

        Ok(())
    }

    pub fn frame_number(&self) -> usize {
        self.frame_number
    }

    pub fn wait(&self) {
        self.gfx.device.wait();
    }

    pub fn forward_pass(&self) -> RenderPass {
        self.graph.pass_by_type(PassType::Forward).unwrap()
    }

    pub fn ui_pass(&self) -> RenderPass {
        self.graph.pass_by_type(PassType::Forward).unwrap()
    }
}
