use glam::Quat;

use gobs::{
    core::Transform,
    game::{
        app::{Application, Run},
        input::Input,
    },
    material::{texture::Texture, Material},
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
        let common = SampleApp::create_ortho(ctx);

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

    fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        self.common.close(ctx);
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let material = Material::default(ctx);
        let texture = Texture::default(ctx);
        let material_instance = material.instanciate(texture);

        let triangle = Model::new(
            ctx,
            "triangle",
            &[Shapes::triangle(
                [1., 0., 0., 1.],
                [0., 1., 0., 1.],
                [0., 0., 1., 1.],
            )],
            &[material_instance],
        );
        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [300., -300., 1.].into(),
        );
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
