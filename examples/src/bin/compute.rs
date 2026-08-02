use std::marker::PhantomData;

use gobs::{
    core::{Input, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderConfig, RenderError},
};

struct App<Context: GobsContext> {
    context: PhantomData<Context>,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(_ctx: &mut Context) -> Result<Self, AppError> {
        Ok(App {
            context: PhantomData,
        })
    }

    fn update(&mut self, _ctx: &mut Context, _delta: f32) {}

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?.build()
    }

    fn input(&mut self, _ctx: &mut Context, _input: Input) {}

    fn resize(&mut self, _ctx: &mut Context, _width: u32, _height: u32) {}

    async fn start(&mut self, _ctx: &mut Context) {}

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        true
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Compute", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "compute"))
        .run();
}
