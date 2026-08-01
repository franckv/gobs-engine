use glam::{Quat, Vec3};

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

use examples::{InputManager, Ui};

struct App {
    scene: Scene,
    ui: Ui,
    input: InputManager,
}

impl GobsGame<GameContext> for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., -25., [0., 1., 0.])
            .with_light(Color::WHITE, [-2., 2.5, 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.input.process_updates {
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
            self.input
                .controller
                .update_camera(camera, transform, delta)
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        if self.input.draw_ui {
            self.ui.draw(ctx, &mut self.scene, delta);
        }
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, _ctx: &mut GameContext, input: Input) {
        self.input.input(input, false);
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let material = ctx.resource_manager.get_by_name("depth").unwrap();

        let material_instance_properties = MaterialInstanceProperties::new("depth", material);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1.),
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

    Application::<App, GameContext>::new("Depth test", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "simple"))
        .run();
}
