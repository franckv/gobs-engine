use gobs_core::logger;
use gobs_render_hal::{CommandBuffer, CommandQueueType};

use crate::GfxContext;

pub struct FrameData {
    pub id: usize,
    pub frame_number: usize,
    pub frames_in_flight: usize,
    pub command: Box<dyn CommandBuffer>,
}

impl FrameData {
    pub fn new(ctx: &mut GfxContext, id: usize, frames_in_flight: usize) -> Self {
        let command = ctx
            .hal_mut()
            .create_command_buffer("Frame", CommandQueueType::Graphics);

        FrameData {
            id,
            frame_number: 0,
            frames_in_flight,
            command,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn wait(&mut self, frame_number: usize) {
        tracing::debug!(target: logger::RENDER, "Wait for frame in flight: {} / {}", self.id + 1, self.frames_in_flight);

        self.frame_number = frame_number;

        self.command.wait();
    }
}
