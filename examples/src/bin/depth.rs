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
    resource::{
        entity::{camera::Camera, light::Light},
        geometry::Shapes,
        resource::ResourceLifetime,
    },
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
        let extent = ctx.renderer.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            0.,
            (-25_f32).to_radians(),
        );
        let camera_position = Vec3::new(0., 1., 0.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(-2., 2.5, 10.);

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

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;

            self.scene
                .graph
                .visit_update(self.scene.graph.root, &mut |node| {
                    if let NodeValue::Model(_) = node.base.value {
                        node.update_transform(|transform| {
                            transform.rotate(Quat::from_axis_angle(
                                Vec3::Y,
                                (angular_speed * delta).to_radians(),
                            ));
                        });
                    }
                });
        }

        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta);
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

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: "app", "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        let material = self.common.depth_material(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.forward_pass(),
        );

        let material_instance =
            //NormalMaterial::instanciate(material, diffuse_texture, normal_texture);
            MaterialInstance::new(material,vec![]);

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1., ctx.renderer.gfx.vertex_padding),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build(&mut ctx.resource_manager);

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: "app", "Engine start");

    Application::<App>::new("Depth test", examples::WIDTH, examples::HEIGHT).run();
}
