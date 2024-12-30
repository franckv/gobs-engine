use futures::try_join;
use glam::{Quat, Vec3};

use gobs::{
    core::{Color, SamplerFilter, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    gfx::Device,
    render::{Context, FrameGraph, Model, PassType, RenderError},
    resource::{
        entity::{camera::Camera, light::Light},
        geometry::Shapes,
        material::{Texture, TextureType},
    },
    scene::{components::NodeValue, scene::Scene},
    ui::UIRenderer,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    graph: FrameGraph,
    ui: UIRenderer,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let extent = ctx.extent();

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

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, camera_position, light, light_position);

        App {
            common,
            camera_controller,
            graph,
            ui,
            scene,
        }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;

            self.scene.update_light(|transform, _| {
                let translation =
                    Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
                        * transform.translation();

                transform.set_translation(translation);
            });
        }

        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta);
        });

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
            Some(&mut self.camera_controller),
        );
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    fn close(&mut self, ctx: &Context) {
        tracing::info!("Closing");

        ctx.device.wait();

        tracing::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        self.load_scene(ctx).await;
    }

    async fn load_scene(&mut self, ctx: &Context) {
        tracing::info!("Load scene");

        let material = self.common.normal_mapping_material(ctx, &self.graph);

        let diffuse_texture = Texture::pack(
            examples::ATLAS,
            3,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );

        let normal_texture = Texture::pack(
            examples::ATLAS_N,
            examples::ATLAS_COLS,
            TextureType::Normal,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );

        let (diffuse_texture, normal_texture) = try_join!(diffuse_texture, normal_texture).unwrap();

        let material_instance = material.instantiate(vec![diffuse_texture, normal_texture]);

        let wall = Model::builder("wall")
            .mesh(
                Shapes::cubemap(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[2],
                    1.,
                    ctx.vertex_padding,
                ),
                Some(material_instance.clone()),
            )
            .build();

        let floor = Model::builder("floor")
            .mesh(
                Shapes::cubemap(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[3, 3, 3, 3, 4, 1],
                    1.,
                    ctx.vertex_padding,
                ),
                Some(material_instance.clone()),
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

        for c in examples::MAP.chars() {
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

    tracing::info!("Engine start");

    Application::<App>::new("Maze", examples::WIDTH, examples::HEIGHT).run();
}
