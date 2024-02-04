use glam::{Quat, Vec3};

use gobs::render::context::Context;
use gobs::render::graph::RenderError;
use gobs::render::SamplerFilter;
use gobs::scene::shape::Shapes;
use gobs::{
    core::Transform,
    game::{
        app::{Application, Run},
        input::Input,
    },
    material::{texture::Texture, Material},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        model::Model,
    },
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let common = SampleApp::create_perspective(ctx);

        App { common }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        let angular_speed = 40.;

        let position = Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
            * self.common.scene.light.position;

        self.common.scene.light.update(position);

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
        let material = Material::default(ctx);
        let texture = Texture::with_file(ctx, examples::WALL_TEXTURE, SamplerFilter::FilterLinear)
            .await
            .unwrap();
        let material_instance = material.instanciate(texture);

        let cube = Model::new(
            ctx,
            "cube",
            &[Shapes::cube(1, 1, &[1])],
            &[material_instance],
        );

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, [1., -1., 1.].into());
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
