use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render::{MaterialInstance, MaterialsConfig, Model, RenderError},
    resource::{entity::light::Light, geometry::Shapes, resource::ResourceLifetime},
    scene::{components::NodeValue, scene::Scene},
    ui::UIRenderer,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    ui: UIRenderer,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let ui = UIRenderer::new(&ctx.renderer.gfx, &mut ctx.resource_manager, true)?;
        let scene = Scene::new(
            &ctx.renderer.gfx,
            camera,
            camera_position,
            light,
            light_position,
        );

        Ok(App {
            common,
            camera_controller,
            ui,
            scene,
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta)
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        self.common
            .update_ui(ctx, &mut self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render(ctx, Some(&mut self.scene), Some(&mut self.ui))
    }

    fn input(&mut self, ctx: &mut GameContext, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.scene,
            &mut self.ui,
            Some(&mut self.camera_controller),
        );
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
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
        let material_instance = MaterialInstance::new(material, vec![]);

        let transparent_material = ctx
            .resource_manager
            .get_by_name("color.transparent")
            .unwrap();
        let transparent_material_instance = MaterialInstance::new(transparent_material, vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    Color::RED,
                    Color::GREEN,
                    Color::BLUE,
                    1.,
                    ctx.renderer.gfx.vertex_padding,
                ),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build(&mut ctx.resource_manager);

        let square = Model::builder("square")
            .mesh(
                Shapes::quad(Color::new(1., 1., 1., 0.5), ctx.renderer.gfx.vertex_padding),
                Some(transparent_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build(&mut ctx.resource_manager);

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

    Application::<App>::new("Transparent", examples::WIDTH, examples::HEIGHT).run();
}
