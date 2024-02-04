use glam::{Quat, Vec3};

use gobs::{
    core::Transform,
    game::{
        app::{Application, Run},
        input::Input,
    },
    render::{context::Context, graph::RenderError},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        import::gltf,
    },
    utils::load,
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
        self.init(ctx);
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        let angular_speed = 40.;

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
    fn init(&mut self, ctx: &Context) {
        self.load_scene(ctx);
    }

    #[allow(unused)]
    fn load_scene(&mut self, ctx: &Context) {
        let file_name = load::get_asset_dir("basicmesh.glb", load::AssetType::MODEL).unwrap();

        let models = gltf::load_gltf(ctx, file_name);

        let i_max = 3;
        let j_max = 3;
        let x_range = (-5., 5.);
        let y_range = (-3., 3.);
        let scale = 0.7;

        let model = models[2].clone();

        for i in 0..=i_max {
            for j in 0..=j_max {
                let x = x_range.0 + (i as f32) * (x_range.1 - x_range.0) / (i_max as f32);
                let y = y_range.0 + (j as f32) * (y_range.1 - y_range.0) / (j_max as f32);
                let transform = Transform::new(
                    [x, y, -7.].into(),
                    Quat::IDENTITY,
                    Vec3::new(scale, -scale, scale),
                );
                let node = Node::new(NodeValue::Model(model.clone()), transform);
                self.common
                    .scene
                    .graph
                    .insert(self.common.scene.graph.root, node);
            }
        }
    }

    #[allow(unused)]
    fn load_scene2(&mut self, ctx: &Context) {
        let file_name = load::get_asset_dir("basicmesh.glb", load::AssetType::MODEL).unwrap();

        let models = gltf::load_gltf(ctx, file_name);

        let scale = 1.;

        let model = models[2].clone();

        let transform = Transform::new(
            [0., 0., -5.].into(),
            Quat::IDENTITY,
            Vec3::new(scale, -scale, scale),
        );
        let node = Node::new(NodeValue::Model(model.clone()), transform);
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
