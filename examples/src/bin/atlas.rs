use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render::{
        MaterialInstance, MaterialsConfig, Model, RenderError, TextureProperties, TextureType,
    },
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
                            true
                        });
                    }
                    false
                });
        }

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

        let material = ctx.resource_manager.get_by_name("normal").unwrap();

        let properties =
            TextureProperties::with_atlas("Atlas Diffuse", examples::ATLAS, examples::ATLAS_COLS);
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static);

        let mut properties =
            TextureProperties::with_atlas("Atlas Normal", examples::ATLAS_N, examples::ATLAS_COLS);
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static);

        let material_instance =
            MaterialInstance::new(material, vec![diffuse_texture, normal_texture]);

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[3, 3, 3, 3, 4, 1],
                    1.,
                    ctx.renderer.gfx.vertex_padding,
                ),
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

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App>::new("Atlas", examples::WIDTH, examples::HEIGHT).run();
}
