use glam::Quat;

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderConfig, RenderError, RenderType,
        Shapes,
    },
    resource::ResourceLifetime,
    scene::{components::NodeValue, scene::Scene},
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    scene: Scene,
}

impl GobsGame<GameContext> for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_ortho_camera([0., 0., 1.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        Ok(App {
            common,
            camera_controller,
            scene,
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta)
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        self.common.update_ui(ctx, &mut self.scene, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.common.draw_bounds)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, ctx: &mut GameContext, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.scene,
            Some(&mut self.camera_controller),
        );
    }

    fn resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
        ctx.ui.resize(width, height);
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
        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let material = ctx.resource_manager.get_by_name("color").unwrap();
        let material_instance_properties = MaterialInstanceProperties::new("color", material);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let transparent_material = ctx
            .resource_manager
            .get_by_name("color.transparent")
            .unwrap();
        let transparent_instance_properties =
            MaterialInstanceProperties::new("transparent", transparent_material);
        let transparent_material_instance = ctx.resource_manager.add(
            transparent_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(&[Color::RED, Color::GREEN, Color::BLUE], 1.),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let square = Model::builder("square")
            .mesh(
                Shapes::quad(&[Color::new(1., 1., 1., 0.5)]),
                Some(transparent_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(triangle), transform);

        let transform = Transform::new(
            [0., 0., 0.5].into(),
            Quat::IDENTITY,
            [300., 300., 1.].into(),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(square), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App, GameContext>::new("Transparent", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "simple"))
        .run();
}
