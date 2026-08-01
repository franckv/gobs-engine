use examples::{InputManager, Ui};
use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderError, RenderType, Shapes,
        TextureProperties, TextureType,
    },
    resource::{ResourceLifetime, load},
    scene::{components::NodeValue, scene::Scene},
};

struct App {
    scene: Scene,
    ui: Ui,
    input: InputManager,
}

impl GobsGame<GameContext> for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., -50., [0., 25., 25.])
            .with_light(Color::WHITE, [0., 40., -40.])
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.input.process_updates {
            let angular_speed = 10.;

            self.scene.update_light(|transform, _| {
                let translation =
                    Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
                        * transform.translation();

                transform.set_translation(translation);

                true
            });
        }

        self.scene.update_camera(|transform, camera| {
            self.input
                .controller
                .update_camera(camera, transform, delta)
        });

        if self.input.draw_ui {
            self.ui.draw(ctx, &mut self.scene, delta);
        }

        self.scene.update(&ctx.renderer.gfx, delta);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn input(&mut self, _ctx: &mut GameContext, input: Input) {
        self.input.input(input, false);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        self.load_scene(ctx).await;
    }

    async fn load_scene(&mut self, ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Load scene");

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

        let material_instance_properties = MaterialInstanceProperties::new("normal", material)
            .textures(&[diffuse_texture, normal_texture]);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let wall = Model::builder("wall")
            .mesh(
                Shapes::cubemap(examples::ATLAS_COLS, examples::ATLAS_ROWS, &[2], 1.),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let floor = Model::builder("floor")
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

        let offset = 16.;

        let (mut i, mut j) = (0., 0.);

        let rotation = Quat::from_axis_angle(Vec3::Z, 0.);

        let wall_node = self
            .scene
            .graph
            .insert(self.scene.graph.root, NodeValue::None, Transform::IDENTITY)
            .unwrap();
        let floor_node = self
            .scene
            .graph
            .insert(
                self.scene.graph.root,
                NodeValue::None,
                Transform::from_translation(-examples::TILE_SIZE * Vec3::Y),
            )
            .unwrap();

        let map = load::load_string(examples::MAP, load::AssetType::DATA)
            .await
            .unwrap();

        for c in map.chars() {
            match c {
                'w' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: 0.,
                        z: j - offset,
                    };

                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene
                        .graph
                        .insert(wall_node, NodeValue::Model(wall.clone()), transform);

                    self.scene
                        .graph
                        .insert(floor_node, NodeValue::Model(floor.clone()), transform);
                }
                '@' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: 0.,
                        z: j - offset,
                    };

                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene
                        .graph
                        .insert(floor_node, NodeValue::Model(floor.clone()), transform);
                }
                '.' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: 0.,
                        z: j - offset,
                    };

                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene
                        .graph
                        .insert(floor_node, NodeValue::Model(floor.clone()), transform);
                }
                '\n' => {
                    j += examples::TILE_SIZE;
                    i = 0.;
                }
                _ => (),
            }
        }
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App, GameContext>::new("Maze", examples::WIDTH, examples::HEIGHT).run();
}
