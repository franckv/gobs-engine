use glam::{Quat, Vec3};
use pollster::FutureExt;

use gobs::{
    core::{Color, Input, Transform},
    game::{AppError, app::Run, context::GameContext},
    gfx::Device,
    render::{FrameGraph, Model, RenderError},
    resource::{entity::light::Light, geometry::Shapes, resource::ResourceLifetime},
    scene::{components::NodeValue, scene::Scene},
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    graph: FrameGraph,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let graph = FrameGraph::headless(&ctx.gfx)?;
        let scene = Scene::new(camera, camera_position, light, light_position);

        Ok(App {
            common,
            graph,
            scene,
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.graph.update(&ctx.gfx, delta);
        self.scene.update(&ctx.gfx, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render_noui(ctx, &mut self.graph, &mut self.scene)
    }

    fn input(&mut self, _ctx: &GameContext, _input: Input) {}

    fn resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {
        self.graph.resize(&mut ctx.gfx);
        self.scene.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &GameContext) {
        tracing::info!(target: "app", "Closing");

        ctx.gfx.device.wait();

        tracing::info!(target: "app", "Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &mut GameContext) {
        let material = self
            .common
            .color_material(&ctx.gfx, &mut ctx.resource_manager, &self.graph);
        let material_instance = material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    Color::RED,
                    Color::GREEN,
                    Color::BLUE,
                    1.,
                    ctx.gfx.vertex_padding,
                ),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
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

    tracing::info!(target: "app", "Engine start");

    let mut ctx = GameContext::new("Triangle", None, true).unwrap();

    let future = async {
        let mut app = App::create(&mut ctx).await.unwrap();
        app.start(&mut ctx).await;

        app
    };

    let mut app = future.block_on();

    app.update(&mut ctx, 0.);

    app.resize(&mut ctx, 1920, 1080);

    app.render(&mut ctx).unwrap();

    app.close(&ctx);

    app.common.screenshot(&ctx.gfx, &mut app.graph);
}
