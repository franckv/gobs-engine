use futures::try_join;
use glam::{Quat, Vec3};

use gobs::{
    core::{
        entity::{camera::Camera, light::Light},
        Color, Transform,
    },
    game::{
        app::{Application, Run},
        input::Input,
    },
    render::{
        context::Context,
        geometry::Model,
        graph::{FrameGraph, RenderError},
        material::{Texture, TextureType},
        pass::PassType,
        renderable::Renderable,
        SamplerFilter,
    },
    scene::{graph::scenegraph::NodeValue, scene::Scene, shape::Shapes},
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
            (0., 25., 25.),
            extent.width as f32 / extent.height as f32,
            (45. as f32).to_radians(),
            0.1,
            150.,
            (0. as f32).to_radians(),
            (-50. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((0., 40., -40.), Color::WHITE);

        let common = SampleApp::new();

        let camera_controller = CameraController::new(3., 0.1);

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, light);

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
        let angular_speed = 10.;

        let position = Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
            * self.scene.light.position;
        self.scene.light.update(position);

        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, |_| {});
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.ui,
            &mut self.camera_controller,
        );
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        self.load_scene(ctx).await;
    }

    async fn load_scene(&mut self, ctx: &Context) {
        log::info!("Load scene");

        let material = self.common.normal_mapping_material(ctx, &self.graph);

        let diffuse_texture = Texture::pack(
            ctx,
            examples::ATLAS,
            3,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );

        let normal_texture = Texture::pack(
            ctx,
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
                Shapes::cube(examples::ATLAS_COLS, examples::ATLAS_ROWS, &[2], 1.),
                material_instance.clone(),
            )
            .build();

        let floor = Model::builder("floor")
            .mesh(
                Shapes::cube(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[3, 3, 3, 3, 4, 1],
                    1.,
                ),
                material_instance.clone(),
            )
            .build();

        let offset = 16.;

        let (mut i, mut j) = (0., 0.);

        let rotation = Quat::from_axis_angle(Vec3::Z, 0.);

        for c in examples::MAP.chars() {
            match c {
                'w' => {
                    i += examples::TILE_SIZE;
                    let mut position = Vec3 {
                        x: i - offset,
                        y: 0.,
                        z: j - offset,
                    };

                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene.graph.insert(
                        self.scene.graph.root,
                        NodeValue::Model(wall.clone()),
                        transform,
                    );

                    position.y = -examples::TILE_SIZE;
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene.graph.insert(
                        self.scene.graph.root,
                        NodeValue::Model(floor.clone()),
                        transform,
                    );
                }
                '@' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene.graph.insert(
                        self.scene.graph.root,
                        NodeValue::Model(floor.clone()),
                        transform,
                    );
                }
                '.' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    self.scene.graph.insert(
                        self.scene.graph.root,
                        NodeValue::Model(floor.clone()),
                        transform,
                    );
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

    log::info!("Engine start");

    Application::new("Maze", 1920, 1080).run::<App>();
}
