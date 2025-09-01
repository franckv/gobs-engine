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
        load,
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
            45_f32.to_radians(),
            0.1,
            150.,
            0_f32.to_radians(),
            (-50_f32).to_radians(),
        );
        let camera_position = Vec3::new(0., 25., 25.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 40., -40.);

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

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.common.should_update()
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.common.process_updates {
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
        self.load_scene(ctx).await;
    }

    async fn load_scene(&mut self, ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Load scene");

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
            .add(properties, ResourceLifetime::Static, false);

        let mut properties =
            TextureProperties::with_atlas("Atlas Normal", examples::ATLAS_N, examples::ATLAS_COLS);
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
                Shapes::cubemap(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[2],
                    1.,
                    ctx.renderer.gfx.vertex_padding,
                ),
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
                    ctx.renderer.gfx.vertex_padding,
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

    Application::<App>::new(
        "Maze",
        GameOptions::default(),
        examples::WIDTH,
        examples::HEIGHT,
    )
    .run();
}
