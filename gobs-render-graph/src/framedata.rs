use gobs_gfx::{Command, CommandQueueType, GfxCommand};
use gobs_render_low::GfxContext;

pub struct FrameData {
    pub id: usize,
    pub frame_number: usize,
    pub command: GfxCommand,
    //TODO: pub query_pool: QueryPool,
}

impl FrameData {
    pub fn new(ctx: &GfxContext, id: usize) -> Self {
        let command = GfxCommand::new(&ctx.device, "Frame", CommandQueueType::Graphics);

        //TODO: let query_pool = QueryPool::new(ctx.device.clone(), QueryType::Timestamp, 2);

        FrameData {
            id,
            frame_number: 0,
            command,
        }
    }

    pub fn reset(&mut self) {
        self.command.reset();
    }
}
