use std::collections::HashMap;

use gobs_core::{ConfigReader as _, GobsConfig, ImageExtent2D, logger};
use gobs_render_graph::{FrameData, FrameGraph, PassId, RenderError};
use gobs_render_hal::GfxContext;
use gobs_render_material::PipelinesConfig;
use gobs_resource::ResourceManager;

use crate::{
    RenderBatch, RenderConfig,
    pass::{
        PassData, compute::ComputePass, material::MaterialPass, pass_loader::PassConfig,
        present::PresentPass,
    },
};

struct FpsTimer {
    fps: u32,
    fps_time: f32,
    fps_frames: u32,
}

impl FpsTimer {
    pub fn new() -> Self {
        Self {
            fps: 0,
            fps_time: 0.,
            fps_frames: 0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.fps_time += delta;
        self.fps_frames += 1;
        if self.fps_time >= 1. {
            self.fps = (self.fps_frames as f32 / self.fps_time).round() as u32;
            self.fps_time = 0.;
            self.fps_frames = 0;
        }
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }
}

pub struct Renderer {
    pub graph: FrameGraph,
    pub gfx: Box<GfxContext>,
    pub frames: Vec<FrameData>,
    pub frame_number: usize,
    passes: HashMap<PassId, PassData>,
    fps_timer: FpsTimer,
}

impl Renderer {
    pub fn new(
        mut gfx: Box<GfxContext>,
        config: GobsConfig,
        resource_manager: &mut ResourceManager,
    ) -> Self {
        let mut passes = HashMap::new();

        let graph = if config.get_bool(RenderConfig::LoadGraph) {
            PipelinesConfig::load_resources(
                gfx.as_ref(),
                &config.get_string(RenderConfig::PipelineFileName),
                resource_manager,
            )
            .expect("Load pipelines");

            let passes_config =
                PassConfig::load_passes(&config.get_string(RenderConfig::PassConfigFileName))
                    .expect("Load passes config");

            FrameGraph::load(
                gfx.as_mut(),
                &config.get_string(RenderConfig::GraphFileName),
                &config.get_string(RenderConfig::GraphName),
                |ctx, pass_metadata, ty| {
                    if let Some(pass_data) = PassConfig::load_pass_data(
                        ctx,
                        resource_manager,
                        &passes_config,
                        &pass_metadata.config,
                        ty,
                    ) {
                        passes.insert(pass_metadata.id, pass_data);
                    } else {
                        tracing::error!(target: logger::RESOURCES, "Pass {} with type {:?} with no config", pass_metadata.name(), ty);
                    }
                },
            )
            .unwrap()
        } else {
            FrameGraph::default()
        };

        let frames_in_flight = gfx.frames_in_flight();

        let frames = (0..frames_in_flight)
            .map(|id| {
                let stats = gfx.create_gpu_stats();
                FrameData::new(gfx.as_mut(), id, frames_in_flight, stats)
            })
            .collect();

        Self {
            graph,
            gfx,
            frames,
            frame_number: 0,
            passes,
            fps_timer: FpsTimer::new(),
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.fps_timer.update(delta);
    }

    pub fn fps(&self) -> u32 {
        self.fps_timer.fps()
    }

    pub fn extent(&self) -> ImageExtent2D {
        self.gfx.get_extent()
    }

    pub fn resize(&mut self) {
        self.gfx.wait();
        self.gfx.resize();
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

        let frame_id = self.begin_frame()?;

        let frame = &mut self.frames[frame_id];

        self.graph.run(
            self.gfx.as_mut(),
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

        self.end_frame()?;

        Ok(())
    }

    fn begin_frame(&mut self) -> Result<usize, RenderError> {
        tracing::debug!(target: logger::SYNC, "Begin new frame {}", self.frame_number);
        tracing::debug!(target: logger::RENDER, "Begin new frame {}", self.frame_number);

        let frame_id = self.gfx.frame_id(self.frame_number);
        let frame = &mut self.frames[frame_id];

        frame.wait(self.frame_number);

        self.gfx.new_frame(self.frame_number);

        let cmd = &mut frame.command;

        if self.gfx.acquire(frame.id).is_err() {
            return Err(RenderError::Outdated);
        }

        cmd.reset();

        cmd.begin(frame.frame_number);

        cmd.begin_label(&format!("Frame {}", frame.frame_number));

        if frame.frame_number > frame.frames_in_flight {
            let stats = self.gfx.get_gpu_stats_ms(frame.stats);
            tracing::debug!(target: logger::STATS, "Gpu time={}ms", stats);
        }
        tracing::debug!(target: logger::STATS, "FPS={}", self.fps_timer.fps());

        cmd.reset_gpu_stats(self.gfx.as_mut(), frame.stats);
        cmd.begin_gpu_stats(self.gfx.as_mut(), frame.stats);

        Ok(frame_id)
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        let frame_id = self.gfx.frame_id(self.frame_number);
        let frame = &mut self.frames[frame_id];

        let cmd = &mut frame.command;

        cmd.end_gpu_stats(self.gfx.as_mut(), frame.stats);

        cmd.end_label();

        cmd.end();

        cmd.submit_graphics(self.gfx.as_ref(), frame_id);

        let Ok(_) = self.gfx.present() else {
            tracing::debug!(target: logger::SYNC, "Exit frame: outdated");
            return Err(RenderError::Outdated);
        };

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
        self.gfx.wait();
    }
}
