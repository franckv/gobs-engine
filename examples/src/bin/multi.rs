use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GameOptions, Run},
    render::{Model, RenderError},
    render_resources::{
        MaterialInstanceProperties, MaterialsConfig, TextureProperties, TextureType,
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
            0.,
        );
        let camera_position = Vec3::new(0., 0., 3.);

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

        let color_material = ctx.resource_manager.get_by_name("color").unwrap();

        let color_instance_properties =
            MaterialInstanceProperties::new("color instance", color_material);

        let color_material_instance =
            ctx.resource_manager
                .add(color_instance_properties, ResourceLifetime::Static, false);

        let properties = TextureProperties::with_file("Wall Diffuse", examples::WALL_TEXTURE);
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let mut properties = TextureProperties::with_file("Wall Normal", examples::WALL_TEXTURE_N);
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let diffuse_material = ctx.resource_manager.get_by_name("normal").unwrap();

        let diffuse_instance_properties =
            MaterialInstanceProperties::new("diffuse instance", diffuse_material)
                .textures(&[diffuse_texture, normal_texture]);

        let diffuse_material_instance =
            ctx.resource_manager
                .add(diffuse_instance_properties, ResourceLifetime::Static, false);

        let model = Model::builder("multi")
            .mesh(
                Shapes::triangle(
                    &[Color::RED, Color::GREEN, Color::BLUE],
                    1.5,
                    ctx.renderer.gfx.vertex_padding,
                ),
                Some(color_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1., ctx.renderer.gfx.vertex_padding),
                Some(diffuse_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let transform = Transform::new([0., 0., 0.].into(), Quat::IDENTITY, Vec3::ONE);
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(model), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App>::new(
        "Multi",
        GameOptions::default(),
        examples::WIDTH,
        examples::HEIGHT,
    )
    .run();
}
