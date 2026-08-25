use std::collections::HashMap;

use gobs_core::{ConfigReader as _, GobsConfig, ImageExtent2D, logger};
use gobs_render_graph::{
    FrameData, FrameGraph, GfxContext, PassId, PassMetaData, RenderError, RenderPassConfig,
    RenderPassType, SceneDataLayout,
};
use gobs_render_hal::{AlignMode, RenderHalConfig, UniformData as _};
use gobs_resource::ResourceManager;

use crate::{
    Pipeline, PipelinesConfig, RenderBatch, RenderConfig,
    pass::{
        PassData,
        compute::{ComputePass, ComputePassData},
        material::{MaterialPass, MaterialPassData},
        present::{PresentPass, PresentPassData},
    },
};

pub struct Renderer {
    pub graph: FrameGraph,
    pub gfx: GfxContext,
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
    passes: HashMap<PassId, PassData>,
}

impl Renderer {
    pub fn new(
        mut gfx: GfxContext,
        config: GobsConfig,
        resource_manager: &mut ResourceManager,
    ) -> Self {
        let mut passes = HashMap::new();
        let frames_in_flight = config.get_int(RenderHalConfig::FramesInFlight) as usize;

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
                |ctx, pass_metadata, pass_config| {
                    let pass_data = Self::build_pass_data(
                        ctx,
                        resource_manager,
                        pass_metadata,
                        pass_config,
                        frames_in_flight,
                    );

                    passes.insert(pass_metadata.id, pass_data);
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
            passes,
        }
    }

    fn build_pass_data(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        pass_metadata: &PassMetaData,
        pass_config: &RenderPassConfig,
        frames_in_flight: usize,
    ) -> PassData {
        let pipeline = pass_config.pipeline.as_ref().and_then(|pipeline| {
            let pipeline_handle = resource_manager.get_by_name::<Pipeline>(pipeline)?;
            let pipeline_data = resource_manager
                .get_data(ctx.hal_mut(), &pipeline_handle)
                .ok()?;
            Some(pipeline_data.data.pipeline)
        });

        let render_flags = pass_config.flags;

        match pass_config.ty {
            RenderPassType::Compute => {
                let pass_data =
                    ComputePassData::new(pipeline.expect("Compute pass with no pipeline"));

                PassData::Compute(pass_data)
            }
            RenderPassType::Material => {
                let mut scene_layout = SceneDataLayout::new(AlignMode::Std140);
                for prop in &pass_config.scene_layout {
                    scene_layout = scene_layout.prop(*prop);
                }

                let pass_data = MaterialPassData::new(
                    ctx,
                    pass_metadata,
                    pipeline,
                    render_flags,
                    scene_layout,
                    frames_in_flight,
                );

                PassData::Material(pass_data)
            }
            RenderPassType::Present => {
                let target = pass_config.target.as_ref().expect("Invalid present target");

                let pass_data = PresentPassData::new(target);

                PassData::Present(pass_data)
            }
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
            |ctx, frame, resource_manager, pass| {
                if let Some(pass_data) = self.passes.get_mut(&pass.id) {
                    match pass_data {
                        PassData::Material(pass_data) => MaterialPass::render(
                            ctx,
                            pass_data,
                            pass,
                            frame,
                            resource_manager,
                            &batch.render_list,
                            &batch.scene_data(),
                        ),
                        PassData::Present(pass_data) => PresentPass::render(
                            ctx,
                            pass_data,
                            frame,
                            resource_manager,
                            ),
                        PassData::Compute(pass_data) => ComputePass::render(
                            ctx,
                            pass_data,
                            pass,
                            frame,
                            resource_manager,
                        )
                    }
                } else {
                    tracing::error!(target: logger::RENDER, "Invoke unregisted pass: {}", pass.name());

                    Ok(())
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
