use gobs::{
    core::{Input, logger},
    game::{AppError, Application, GameContext, GobsGame},
    render::{RenderConfig, RenderError},
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl GobsGame<GameContext> for App {
    async fn create(_ctx: &mut GameContext) -> Result<Self, AppError> {
        let common = SampleApp::new();

        Ok(App { common })
    }

    fn update(&mut self, _ctx: &mut GameContext, _delta: f32) {}

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common.render(ctx, None, None)
    }

    fn input(&mut self, _ctx: &mut GameContext, _input: Input) {}

    fn resize(&mut self, _ctx: &mut GameContext, _width: u32, _height: u32) {}

    async fn start(&mut self, _ctx: &mut GameContext) {}

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.common.should_update()
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App, GameContext>::new("Compute", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "compute"))
        .run();
}
