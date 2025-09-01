use glam::{Quat, Vec3};
use pollster::FutureExt;

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, GameContext, GameOptions, Run},
    render::{BuiltinGraphs, Model, RenderError},
    render_resources::{MaterialInstanceProperties, MaterialsConfig},
    resource::{entity::light::Light, geometry::Shapes, resource::ResourceLifetime},
    scene::{components::NodeValue, scene::Scene},
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let scene = Scene::new(
            &ctx.renderer.gfx,
            camera,
            camera_position,
            light,
            light_position,
        );

        Ok(App { common, scene })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.scene.update(&ctx.renderer.gfx, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common.render(ctx, Some(&mut self.scene), None)
    }

    fn input(&mut self, _ctx: &mut GameContext, _input: Input) {}

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.common.should_update()
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources(
            &ctx.renderer.gfx,
            "materials.ron",
            &mut ctx.resource_manager,
        )
        .await;

        let material = ctx.resource_manager.get_by_name("color").unwrap();
        let material_instance_properties = MaterialInstanceProperties::new("color", material);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    &[Color::RED, Color::GREEN, Color::BLUE],
                    1.,
                    ctx.renderer.gfx.vertex_padding,
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

    tracing::info!(target: logger::APP, "Engine start");

    let mut options = GameOptions::default();
    options.renderer.graph = BuiltinGraphs::Headless;

    let mut ctx = GameContext::new("Triangle", &options, None, true).unwrap();

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

    app.common.screenshot(&mut ctx);
}
