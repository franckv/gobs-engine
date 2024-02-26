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
    #[allow(unused)]
    fn init(&mut self, ctx: &Context) {
        let material = self.common.color_material(ctx);
        let material_instance = material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE, 1.),
                material_instance,
            )
            .build();

        let graph = &mut self.common.scene.graph;

        let node_value = NodeValue::Model(triangle);

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let dx = extent.width as f32 / 12.;
        let dy = extent.height as f32 / 6.;

        let node1 = graph.root;
        if let Some(node) = graph.get_mut(node1) {
            node.transform.translate([0., 2. * dy, 0.].into());
            node.transform.scale([100., 100., 1.].into());
        }

        let node2 = graph
            .insert(
                node1,
                node_value.clone(),
                Transform::translation([-2. * dx, 0., 0.].into()),
            )
            .unwrap();

        let node3 = graph
            .insert(
                node1,
                node_value.clone(),
                Transform::translation([2. * dx, 0., 0.].into()),
            )
            .unwrap();

        let node4 = graph
            .insert(
                node2,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let node5 = graph
            .insert(
                node2,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let node6 = graph
            .insert(
                node3,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let node7 = graph
            .insert(
                node3,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let node8 = graph
            .insert(
                node4,
                node_value.clone(),
                Transform::translation([-dx, -dy, 0.].into()),
            )
            .unwrap();

        let node9 = graph
            .insert(
                node4,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let node10 = graph
            .insert(
                node4,
                node_value.clone(),
                Transform::translation([dx, -dy, 0.].into()),
            )
            .unwrap();

        let node11 = graph
            .insert(
                node7,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
            )
            .unwrap();

        let node12 = graph
            .insert(
                node8,
                node_value.clone(),
                Transform::translation([0., -dy, 0.].into()),
            )
            .unwrap();
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("Scenegraph", 1920, 1080).run::<App>();
}
