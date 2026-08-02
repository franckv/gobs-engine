use std::marker::PhantomData;

use glam::Quat;
use pollster::FutureExt;

use gobs::{
    core::{Color, Config, Input, Transform, logger},
    game::{AppError, GameContext, GobsContext, GobsGame},
    render::{RenderConfig, RenderError, RenderType, Shapes},
    scene::{components::NodeValue, scene::Scene},
};

struct App<Context: GobsContext> {
    scene: Scene,
    context: PhantomData<Context>,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_ortho_camera([0., 0., 1.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            context: PhantomData,
        })
    }

    fn update(&mut self, _ctx: &mut Context, delta: f32) {
        self.scene.update(delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, _ctx: &mut Context, _input: Input) {}

    fn resize(&mut self, _ctx: &mut Context, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut Context) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        true
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    async fn init(&mut self, ctx: &mut Context) {
        ctx.load_material("materials.ron").await;

        let material = ctx.new_material("color").from_base("color").build();

        let mesh = ctx
            .new_mesh()
            .with_geometry(Shapes::triangle(
                &[Color::RED, Color::GREEN, Color::BLUE],
                1.,
            ))
            .for_material(material)
            .build();

        let triangle = ctx
            .new_model("triangle")
            .with_mesh(mesh)
            .with_material(material)
            .build();

        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(triangle), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    let mut config = Config::default();
    config.register::<RenderConfig>();
    config.set_string(RenderConfig::GraphName, "headless");

    let mut ctx = GameContext::new("Triangle", config, None, true);

    let future = async {
        let mut app = App::create(&mut ctx).await.unwrap();
        app.start(&mut ctx).await;

        app
    };

    let mut app = future.block_on();

    app.update(&mut ctx, 0.);

    app.resize(&mut ctx, 1920, 1080);

    app.render(&mut ctx).unwrap();

    app.close(&mut ctx);

    // app.common.screenshot(&mut ctx);
}
