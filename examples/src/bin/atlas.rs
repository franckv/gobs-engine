use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderError, RenderType, Shapes,
        TextureProperties, TextureType,
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
            .with_perspective_camera(0., -25., [0., 1., 0.])
            .with_light(Color::WHITE, [-2., 2.5, 10.])
            .build();

        let common = SampleApp::new();
        let camera_controller = SampleApp::controller();

        Ok(App {
            common,
            camera_controller,
            scene,
        })
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.common.should_update()
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

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let material = ctx.resource_manager.get_by_name("normal").unwrap();

        let properties = TextureProperties::with_atlas(
            "Atlas Diffuse",
            examples::DIFFUSE_FORMAT,
            examples::ATLAS,
            examples::ATLAS_COLS,
        );
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let mut properties = TextureProperties::with_atlas(
            "Atlas Normal",
            examples::NORMAL_FORMAT,
            examples::ATLAS_N,
            examples::ATLAS_COLS,
        );
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let material_instance_properties = MaterialInstanceProperties::new("atlas", material)
            .textures(&[diffuse_texture, normal_texture]);

        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[3, 3, 3, 3, 4, 1],
                    1.,
                ),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App, GameContext>::new("Atlas", examples::WIDTH, examples::HEIGHT).run();
}
