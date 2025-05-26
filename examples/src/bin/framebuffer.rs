use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render::{MaterialInstance, Model, TextureProperties},
    render_graph::RenderError,
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

        let ui = UIRenderer::new(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.ui_pass(),
        )?;
        let scene = Scene::new(camera, camera_position, light, light_position);

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
                .update_camera(camera, transform, delta);
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        self.common.update_ui(ctx, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common.render(ctx, &mut self.scene, &mut self.ui)
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
        self.init(ctx);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: "app", "Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &mut GameContext) {
        let extent = ctx.renderer.graph.draw_extent;
        let (width, height) = (extent.width, extent.height);

        let framebuffer = Self::generate_framebuffer(width, height);

        let material = self.common.texture_material(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.forward_pass(),
        );

        let properties = TextureProperties::with_colors("Framebuffer", framebuffer, extent);

        let texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static);

        let material_instance = MaterialInstance::new(material, vec![texture]);

        let rect = Model::builder("rect")
            .mesh(
                Shapes::quad(Color::WHITE, ctx.renderer.gfx.vertex_padding),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build(&mut ctx.resource_manager);

        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [width as f32, height as f32, 1.].into(),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(rect), transform);
    }

    fn generate_framebuffer(width: u32, height: u32) -> Vec<Color> {
        let mut buffer = Vec::new();

        let border = 50;

        for i in 0..height {
            for j in 0..width {
                if i < border || i >= height - border || j < border || j >= width - border {
                    buffer.push(Color::BLUE);
                } else {
                    buffer.push(Color::RED);
                }
            }
        }
        buffer
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: "app", "Engine start");

    Application::<App>::new("Framebuffer", examples::WIDTH, examples::HEIGHT).run();
}
