use gobs_core::logger;
use gobs_gfx::{Command, CommandQueueType, GfxCommand};

use crate::{GfxContext, RenderStats};

pub struct FrameData {
    pub id: usize,
    pub frame_number: usize,
    pub frames_in_flight: usize,
    pub stats: RenderStats,
    pub command: GfxCommand,
    //TODO: pub query_pool: QueryPool,
}

impl FrameData {
    pub fn new(ctx: &GfxContext, id: usize, frames_in_flight: usize) -> Self {
        let command = GfxCommand::new(&ctx.device, "Frame", CommandQueueType::Graphics);

        //TODO: let query_pool = QueryPool::new(ctx.device.clone(), QueryType::Timestamp, 2);

        FrameData {
            id,
            frame_number: 0,
            frames_in_flight,
            stats: RenderStats::default(),
            command,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn reset(&mut self, frame_number: usize) {
        tracing::debug!(target: logger::RENDER, "Begin new frame: {} ({}/{})", frame_number, self.id, self.frames_in_flight);

        self.frame_number = frame_number;
        self.stats.reset();

        self.command.reset();
    }
}
