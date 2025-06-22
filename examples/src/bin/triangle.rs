use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render::{MaterialInstance, Model},
    render_graph::RenderError,
    resource::{entity::light::Light, geometry::Shapes, resource::ResourceLifetime},
    scene::{components::NodeValue, scene::Scene},
    ui::UIRenderer,
};

use examples::SampleApp;

struct App {
    common: SampleApp,
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

        let ui = UIRenderer::new(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.ui_pass(),
        )?;
        let scene = Scene::new(camera, camera_position, light, light_position);

        Ok(App { common, ui, scene })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.scene.update(&ctx.renderer.gfx, delta);

        self.common
            .update_ui(ctx, &mut self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render(ctx, Some(&mut self.scene), Some(&mut self.ui))
    }

    fn input(&mut self, ctx: &mut GameContext, input: Input) {
        self.common
            .input(ctx, input, &mut self.scene, &mut self.ui, None);
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: "app", "Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &mut GameContext) {
        let material = self.common.color_material(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.forward_pass(),
        );
        let material_instance = MaterialInstance::new(material, vec![]);

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

    Application::<App>::new("Triangle", examples::WIDTH, examples::HEIGHT).run();
}
