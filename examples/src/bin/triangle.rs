use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform},
    game::{
        AppError,
        app::{Application, Run},
    },
    gfx::Device,
    render::{Context, FrameGraph, Model, PassType, RenderError},
    resource::{entity::light::Light, geometry::Shapes},
    scene::{components::NodeValue, scene::Scene},
    ui::UIRenderer,
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    graph: FrameGraph,
    ui: UIRenderer,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &Context) -> Result<Self, AppError> {
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let graph = FrameGraph::default(ctx)?;
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui)?)?;
        let scene = Scene::new(camera, camera_position, light, light_position);

        Ok(App {
            common,
            graph,
            ui,
            scene,
        })
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
            None,
        );
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
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

    Application::<App>::new("Triangle", examples::WIDTH, examples::HEIGHT).run();
}
