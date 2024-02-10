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
        graph::RenderError,
        material::{Texture, TextureType},
        SamplerFilter,
    },
    scene::{
        graph::scenegraph::{Node, NodeValue},
        shape::Shapes,
    },
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let extent = SampleApp::extent(ctx);

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

        let common = SampleApp::create(ctx, camera, light);

        App { common }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        let angular_speed = 10.;

        let position = Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
            * self.common.scene.light.position;
        self.common.scene.light.update(position);

        self.common.update(ctx, delta);
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common.render(ctx)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(ctx, input);
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        self.common.resize(ctx, width, height);
    }

    fn close(&mut self, ctx: &Context) {
        self.common.close(ctx);
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        self.load_scene(ctx).await;
    }

    async fn load_scene(&mut self, ctx: &Context) {
        log::info!("Load scene");

        let material = SampleApp::normal_mapping_material(ctx);

        let diffuse_texture = Texture::pack(
            ctx,
            examples::ATLAS,
            3,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
        );

        let normal_texture = Texture::pack(
            ctx,
            examples::ATLAS_N,
            examples::ATLAS_COLS,
            TextureType::Normal,
            SamplerFilter::FilterLinear,
        );

        let (diffuse_texture, normal_texture) = try_join!(diffuse_texture, normal_texture).unwrap();

        let material_instance = material.instantiate(vec![diffuse_texture, normal_texture]);

        let wall = Model::builder("wall")
            .mesh(
                Shapes::cube(examples::ATLAS_COLS, examples::ATLAS_ROWS, &[2]),
                0,
            )
            .material(material_instance.clone())
            .build();

        let floor = Model::builder("floor")
            .mesh(
                Shapes::cube(
                    examples::ATLAS_COLS,
                    examples::ATLAS_ROWS,
                    &[3, 3, 3, 3, 4, 1],
                ),
                0,
            )
            .material(material_instance.clone())
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
                    let node = Node::new(NodeValue::Model(wall.clone()), transform);
                    self.common
                        .scene
                        .graph
                        .insert(self.common.scene.graph.root, node);

                    position.y = -examples::TILE_SIZE;
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    let node = Node::new(NodeValue::Model(floor.clone()), transform);
                    self.common
                        .scene
                        .graph
                        .insert(self.common.scene.graph.root, node);
                }
                '@' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    let node = Node::new(NodeValue::Model(floor.clone()), transform);
                    self.common
                        .scene
                        .graph
                        .insert(self.common.scene.graph.root, node);
                }
                '.' => {
                    i += examples::TILE_SIZE;
                    let position = Vec3 {
                        x: i - offset,
                        y: -examples::TILE_SIZE,
                        z: j - offset,
                    };
                    let transform = Transform::new(position, rotation, Vec3::splat(1.));
                    let node = Node::new(NodeValue::Model(floor.clone()), transform);
                    self.common
                        .scene
                        .graph
                        .insert(self.common.scene.graph.root, node);
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

    Application::new("examples", 1600, 900).run::<App>();
}
