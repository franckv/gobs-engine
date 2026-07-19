use gobs_core::{ImageExtent2D, logger};
use gobs_render_graph::{FrameData, FrameGraph, GfxContext, PassType, RenderError};
use gobs_resource::ResourceManager;

use crate::{Pipeline, PipelinesConfig, RenderBatch};

#[derive(Debug)]
pub struct RendererOptions {
    pub graph_filename: String,
    pub graph: String,
    pub pipeline_filename: String,
    pub frames_in_flight: usize,
    pub load_graph: bool,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            graph_filename: "graph.ron".to_string(),
            graph: "scene".to_string(),
            pipeline_filename: "pipelines.ron".to_string(),
            frames_in_flight: 2,
            load_graph: true,
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
        let graph = if options.load_graph {
            PipelinesConfig::load_resources(&gfx, &options.pipeline_filename, resource_manager)
                .expect("Load pipelines");

            FrameGraph::load(
                &mut gfx,
                &options.graph_filename,
                &options.graph,
                |pipeline, ctx| {
                    let pipeline_handle = resource_manager.get_by_name::<Pipeline>(pipeline)?;

                    let pipeline = resource_manager.get_data(ctx.hal_mut(), &pipeline_handle);

                    pipeline.ok().map(|data| data.data.pipeline)
                },
            )
            .unwrap()
        } else {
            FrameGraph::default()
        };

        let frames_in_flight = gfx.frames_in_flight();

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

    pub fn get_batch(&self) -> RenderBatch {
        RenderBatch::new(&self.gfx)
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
    pub fn submit(&mut self, batch: &mut RenderBatch) -> Result<(), RenderError> {
        assert!(!batch.recording, "Batch recording not finished");

        tracing::debug!(target: logger::RENDER, "Submit render batch");

        tracing::debug!(target: logger::SYNC, "Begin new frame {}", self.frame_number);
        tracing::debug!(target: logger::RENDER, "Begin new frame {}", self.frame_number);

        let frame_id = self.gfx.frame_id(self.frame_number);

        let frame = &mut self.frames[frame_id];
        frame.wait(self.frame_number);

        self.gfx.new_frame(self.frame_number);

        self.graph.begin(&mut self.gfx, frame)?;

        self.graph.render(
            &mut self.gfx,
            frame,
            &batch.render_list,
            &batch.scene_data(),
        )?;

        self.graph.end(&mut self.gfx, frame)?;

        tracing::debug!(target: logger::SYNC, "End frame {}", self.frame_number);
        tracing::debug!(target: logger::RENDER, "End frame {}", self.frame_number);

        self.frame_number += 1;

        Ok(())
    }

    pub fn frame(&self) -> &FrameData {
        let frame_id = self.gfx.frame_id(self.frame_number);
        &self.frames[frame_id]
    }

    pub fn frame_number(&self) -> usize {
        self.frame_number
    }

    pub fn wait(&mut self) {
        self.gfx.hal_mut().wait();
    }
}
