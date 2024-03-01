use glam::Quat;

use gobs::{
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    render::{context::Context, geometry::Model, graph::RenderError},
    scene::{graph::scenegraph::NodeValue, shape::Shapes},
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
        let material = self.common.color_material(ctx);
        let material_instance = material.instantiate(vec![]);

        let transparent_material = self.common.color_material_transparent(ctx);
        let transparent_material_instance = transparent_material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE, 1.),
                material_instance,
            )
            .build();

        let square = Model::builder("square")
            .mesh(
                Shapes::quad(Color::new(1., 1., 1., 0.5)),
                //Shapes::quad(Color::new(1., 1., 1., 0.5)),
                transparent_material_instance,
            )
            .build();

        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());
        self.common.scene.graph.insert(
            self.common.scene.graph.root,
            NodeValue::Model(triangle),
            transform,
        );

        let transform = Transform::new(
            [0., 0., 0.5].into(),
            Quat::IDENTITY,
            [300., 300., 1.].into(),
        );

        self.common.scene.graph.insert(
            self.common.scene.graph.root,
            NodeValue::Model(square),
            transform,
        );
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("Transparent", 1920, 1080).run::<App>();
}
