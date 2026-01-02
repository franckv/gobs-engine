use gobs_core::{ImageExtent2D, logger};
use gobs_render_graph::{FrameData, FrameGraph, GfxContext, RenderError, RenderPass};
use gobs_resource::manager::ResourceManager;
use tracing::Level;

use crate::RenderBatch;

#[derive(Debug, Default)]
pub enum BuiltinGraphs {
    #[default]
    Scene,
    Headless,
    Ui,
}

#[derive(Debug, Default)]
pub struct RendererOptions {
    pub graph: BuiltinGraphs,
}

pub struct Renderer {
    pub graph: FrameGraph,
    pub batch: RenderBatch,
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
        let graph = match options.graph {
            BuiltinGraphs::Scene => FrameGraph::default(&mut gfx, resource_manager).unwrap(),
            BuiltinGraphs::Headless => FrameGraph::headless(&mut gfx, resource_manager).unwrap(),
            BuiltinGraphs::Ui => FrameGraph::ui(&mut gfx, resource_manager).unwrap(),
        };

        let frames_in_flight = gfx.frames_in_flight;

        let frames = (0..frames_in_flight)
            .map(|id| FrameData::new(&mut gfx, id, frames_in_flight))
            .collect();

        Self {
            graph,
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw(
        &mut self,
        resource_manager: &mut ResourceManager,
        draw_cmd: &mut dyn FnMut(
            &mut GfxContext,
            RenderPass,
            &mut RenderBatch,
            &mut ResourceManager,
        ) -> Result<(), RenderError>,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Begin render batch");

        self.frame_number += 1;

        tracing::debug!(target: logger::RENDER, "Begin new frame {}", self.frame_number);

        let frame = &mut self.frames[self.frame_number % self.gfx.frames_in_flight];

        frame.reset(self.frame_number);

        {
            //TODO: let mut buf = [0 as u64; 2];
            //frame.query_pool.get_query_pool_results(0, &mut buf);

            //self.batch.render_stats.gpu_draw_time =
            //    ((buf[1] - buf[0]) as f32 * frame.query_pool.period) / 1_000_000_000.;
        }

        self.graph.begin(&mut self.gfx, frame).unwrap();

        self.batch.reset();

        frame.stats.prepare_begin();

        for pass in &self.graph.passes {
            let span =
                tracing::span!(target: logger::PROFILE, Level::TRACE, "Pass", "{}", pass.name())
                    .entered();
            draw_cmd(
                &mut self.gfx,
                pass.clone(),
                &mut self.batch,
                resource_manager,
            )?;
            frame.stats.prepare_draw(pass.id());
            frame.stats.objects(
                self.batch
                    .render_list
                    .get(&pass.id())
                    .map(|list| list.len() as u32)
                    .unwrap_or_default(),
            );
            span.exit();
        }

        self.batch.finish(&mut self.gfx, resource_manager);

        frame.stats.prepare_end();

        self.graph
            .render(
                &mut self.gfx,
                frame,
                &self.batch.render_list,
                &self.batch.scene_data(),
            )
            .unwrap();

        self.graph.end(&mut self.gfx, frame).unwrap();

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
