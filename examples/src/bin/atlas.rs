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
    material::{
        texture::{Texture, TextureType},
        NormalMaterial,
    },
    render::{context::Context, graph::RenderError, SamplerFilter},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        model::Model,
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
            Vec3::new(0., 1., 0.),
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            (-25. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((-2., 2.5, 10.), Color::WHITE);

        let common = SampleApp::create(ctx, camera, light);

        App { common }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        let angular_speed = 40.;

        let root_id = self.common.scene.graph.root;
        let root = self.common.scene.graph.get(root_id).unwrap();

        let node = root.children[0];

        let child = self.common.scene.graph.get_mut(node).unwrap();

        child.transform.rotate(Quat::from_axis_angle(
            Vec3::Y,
            (0.3 * angular_speed * delta).to_radians(),
        ));

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
        let material = NormalMaterial::new(ctx);

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

        let material_instance =
            NormalMaterial::instanciate(material, diffuse_texture, normal_texture);

        let cube = Model::new(
            ctx,
            "cube",
            &[Shapes::cube(
                examples::ATLAS_COLS,
                examples::ATLAS_ROWS,
                &[3, 3, 3, 3, 4, 1],
            )],
            &[material_instance],
        );

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        let node = Node::new(NodeValue::Model(cube), transform);
        self.common
            .scene
            .graph
            .insert(self.common.scene.graph.root, node);
    }
}
fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
