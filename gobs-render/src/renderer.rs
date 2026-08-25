use gobs_core::{ConfigReader as _, GobsConfig, ImageExtent2D, logger};
use gobs_render_graph::{FrameData, FrameGraph, GfxContext, RenderError, RenderPassType};
use gobs_resource::ResourceManager;

use crate::{Pipeline, PipelinesConfig, RenderBatch, RenderConfig};

pub struct Renderer {
    pub graph: FrameGraph,
    pub gfx: GfxContext,
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
}

impl Renderer {
    pub fn new(
        mut gfx: GfxContext,
        config: GobsConfig,
        resource_manager: &mut ResourceManager,
    ) -> Self {
        let graph = if config.get_bool(RenderConfig::LoadGraph) {
            PipelinesConfig::load_resources(
                &gfx,
                &config.get_string(RenderConfig::PipelineFileName),
                resource_manager,
            )
            .expect("Load pipelines");

            FrameGraph::load(
                &mut gfx,
                &config.get_string(RenderConfig::GraphFileName),
                &config.get_string(RenderConfig::GraphName),
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

    pub fn resize(&mut self) {
        self.graph.resize(&mut self.gfx);
    }

    pub fn update(&mut self, delta: f32) {
        self.graph.update(&self.gfx, delta);
    }

    pub fn enable_pass(&mut self, name: &str, enabled: bool) {
        self.graph.enable_pass(name, enabled);
    }

    pub fn get_batch(&self) -> RenderBatch {
        RenderBatch::new()
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

        self.graph.run(
            &mut self.gfx,
            frame,
            |ctx, frame, resource_manager, pass| match pass.metadata().ty {
                RenderPassType::Compute | RenderPassType::Material | RenderPassType::Present => {
                    pass.render(
                        ctx,
                        frame,
                        resource_manager,
                        &batch.render_list,
                        &batch.scene_data(),
                    )
                }
            },
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
