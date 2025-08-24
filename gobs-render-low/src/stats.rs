use std::collections::HashMap;

use uuid::Uuid;

use gobs_core::utils::timer::Timer;

#[derive(Clone, Debug, Default)]
pub struct PassStats {
    pub draws: u32,
    pub indices: u32,
    pub pipeline_binds: u32,
    pub resource_binds: u32,
    pub cpu_draw_time: f32,
}

impl PassStats {
    pub fn reset(&mut self) {
        self.draws = 0;
        self.indices = 0;
        self.pipeline_binds = 0;
        self.resource_binds = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    timer: Timer,
    pass_stats: HashMap<Uuid, PassStats>,
    pub objects: u32,
    pub cpu_prepare_begin_time: f32,
    pub cpu_prepare_draw_time: f32,
    pub cpu_prepare_end_time: f32,
}

impl RenderStats {
    pub fn reset(&mut self) {
        self.pass_stats.clear();
        self.timer.reset();
        self.objects = 0;
    }

    pub fn prepare_begin(&mut self) {
        self.cpu_prepare_begin_time = self.timer.delta();
    }

    pub fn prepare_draw(&mut self) {
        self.cpu_prepare_draw_time = self.timer.delta();
    }

    pub fn prepare_end(&mut self) {
        self.cpu_prepare_end_time = self.timer.delta();
    }

    pub fn pass(&self, id: Uuid) -> Option<&PassStats> {
        self.pass_stats.get(&id)
    }

    pub fn objects(&mut self, len: u32) {
        self.objects += len;
    }

    pub fn draw(&mut self, id: Uuid, indices: u32) {
        let pass = self.pass_stats.entry(id).or_default();

        pass.draws += 1;
        pass.indices += indices;
    }

    pub fn bind_pipeline(&mut self, id: Uuid) {
        let pass = self.pass_stats.entry(id).or_default();

        pass.pipeline_binds += 1;
    }

    pub fn bind_resource(&mut self, id: Uuid) {
        let pass = self.pass_stats.entry(id).or_default();

        pass.resource_binds += 1;
    }

    pub fn finish(&mut self, id: Uuid) {
        let pass = self.pass_stats.entry(id).or_default();

        pass.cpu_draw_time = self.timer.delta();
    }
}
