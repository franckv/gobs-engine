use glam::{Quat, Vec3};
use pollster::FutureExt;

use gobs::{
    core::{Color, Transform},
    game::{app::Run, input::Input},
    gfx::Device,
    render::{Context, FrameGraph, Model, RenderError},
    resource::entity::light::Light,
    scene::{components::NodeValue, scene::Scene, shape::Shapes},
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    graph: FrameGraph,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let graph = FrameGraph::headless(ctx);
        let scene = Scene::new(camera, camera_position, light, light_position);

        App {
            common,
            graph,
            scene,
        }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        self.common
            .render_noui(ctx, &mut self.graph, &mut self.scene)
    }

    fn input(&mut self, _ctx: &Context, _input: Input) {}

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        tracing::info!("Closing");

        ctx.device.wait();

        tracing::info!("Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let material = self.common.color_material(ctx, &self.graph);
        let material_instance = material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    Color::RED,
                    Color::GREEN,
                    Color::BLUE,
                    1.,
                    ctx.vertex_padding,
                ),
                Some(material_instance),
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

    tracing::info!("Engine start");

    let mut ctx = Context::new("Triangle", None, true);

    let future = async {
        let mut app = App::create(&ctx).await;
        app.start(&ctx).await;

        app
    };

    let mut app = future.block_on();

    app.update(&ctx, 0.);

    app.resize(&mut ctx, 1920, 1080);

    app.render(&mut ctx).unwrap();

    app.close(&ctx);

    app.common.screenshot(&ctx, &mut app.graph);
}
