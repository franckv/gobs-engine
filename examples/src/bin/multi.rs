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
    scene::{graph::scenegraph::NodeValue, shape::Shapes},
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let extent = SampleApp::extent(ctx);

        let camera = Camera::perspective(
            Vec3::new(0., 0., 3.),
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        );

        let light = Light::new((0., 0., 10.), Color::WHITE);

        let common = SampleApp::create(ctx, camera, light);

        App { common }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
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

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn close(&mut self, ctx: &Context) {
        self.common.close(ctx);
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        let color_material = self.common.color_material(ctx);
        let color_material_instance = color_material.instantiate(vec![]);

        let diffuse_material = self.common.normal_mapping_material(ctx);
        let diffuse_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
        );
        let normal_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE_N,
            TextureType::Normal,
            SamplerFilter::FilterLinear,
        );
        let (diffuse_texture, normal_texture) = try_join!(diffuse_texture, normal_texture).unwrap();
        let diffuse_material_instance =
            diffuse_material.instantiate(vec![diffuse_texture, normal_texture]);

        let model = Model::builder("multi")
            .mesh(
                Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE, 1.5),
                color_material_instance,
            )
            .mesh(Shapes::cube(1, 1, &[1], 1.), diffuse_material_instance)
            .build();

        let transform = Transform::new([0., 0., 0.].into(), Quat::IDENTITY, Vec3::ONE);
        self.common.scene.graph.insert(
            self.common.scene.graph.root,
            NodeValue::Model(model),
            transform,
        );
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("Multi", 1920, 1080).run::<App>();
}
