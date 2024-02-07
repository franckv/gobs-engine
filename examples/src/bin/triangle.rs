use glam::Quat;

use gobs::{
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    material::ColorMaterial,
    render::{context::Context, graph::RenderError},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        model::Model,
        shape::Shapes,
    },
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let light = Light::new((0., 0., 10.), Color::WHITE);

        let common = SampleApp::create(ctx, SampleApp::ortho_camera(ctx), light);

        App { common }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.common.update(ctx, delta);
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common.render(ctx)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(ctx, input);
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        self.common.resize(ctx, width, height);
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        self.common.close(ctx);
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let material = ColorMaterial::new(ctx);
        let material_instance = ColorMaterial::instanciate(material);

        let triangle = Model::new(
            ctx,
            "triangle",
            &[Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE)],
            &[material_instance],
        );
        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());
        let node = Node::new(NodeValue::Model(triangle), transform);
        self.common
            .scene
            .graph
            .insert(self.common.scene.graph.root, node);
    }
}
fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
