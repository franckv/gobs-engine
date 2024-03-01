use glam::{Quat, Vec3};

use gobs::{
    assets::gltf,
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::{Input, Key},
    },
    render::{context::Context, graph::RenderError, pass::PassType},
    scene::graph::scenegraph::{NodeValue, SceneGraph},
    utils::load,
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    scenes: Vec<SceneGraph>,
    current_scene: usize,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let light = Light::new((0., 0., 10.), Color::WHITE);

        let common = SampleApp::create(ctx, SampleApp::perspective_camera(ctx), light);

        App {
            common,
            scenes: vec![],
            current_scene: 0,
        }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 40.;
            let position = Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
                * self.common.scene.light.position;
            self.common.scene.light.update(position);
        }

        self.common.update(ctx, delta);
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common.render(ctx)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(ctx, input);

        match input {
            Input::KeyPressed(key) => match key {
                Key::N => {
                    self.current_scene = (self.current_scene + 1) % self.scenes.len();
                    self.common.scene.graph = self.scenes[self.current_scene].clone();
                }
                _ => (),
            },
            _ => (),
        }
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
        self.scenes.push(self.load_scene(ctx));
        self.scenes.push(self.load_scene2(ctx));
        self.scenes.push(self.load_scene3(ctx));

        self.common.scene.graph = self.scenes[self.current_scene].clone();
    }

    fn load_scene(&self, ctx: &Context) -> SceneGraph {
        let mut scene = SceneGraph::new();

        let file_name = load::get_asset_dir("basicmesh.glb", load::AssetType::MODEL).unwrap();

        let mut gltf_loader = gltf::GLTFLoader::new(
            ctx,
            self.common.graph.pass_by_type(PassType::Forward).unwrap(),
        );

        gltf_loader.load(ctx, file_name);

        let i_max = 3;
        let j_max = 3;
        let x_range = (-5., 5.);
        let y_range = (-3., 3.);
        let scale = 0.7;

        let model = gltf_loader.models[2].clone();

        for i in 0..=i_max {
            for j in 0..=j_max {
                let x = x_range.0 + (i as f32) * (x_range.1 - x_range.0) / (i_max as f32);
                let y = y_range.0 + (j as f32) * (y_range.1 - y_range.0) / (j_max as f32);
                let transform = Transform::new(
                    [x, y, -7.].into(),
                    Quat::IDENTITY,
                    Vec3::new(scale, scale, scale),
                );
                scene.insert(
                    self.common.scene.graph.root,
                    NodeValue::Model(model.clone()),
                    transform,
                );
            }
        }

        scene
    }

    fn load_scene2(&self, ctx: &Context) -> SceneGraph {
        let mut scene = SceneGraph::new();

        let file_name = load::get_asset_dir("Cube.gltf", load::AssetType::MODEL).unwrap();

        let mut gltf_loader = gltf::GLTFLoader::new(
            ctx,
            self.common.graph.pass_by_type(PassType::Forward).unwrap(),
        );

        gltf_loader.load(ctx, file_name);

        let i_max = 3;
        let j_max = 3;
        let x_range = (-5., 5.);
        let y_range = (-3., 3.);
        let scale = 0.7;

        let model = gltf_loader.models[0].clone();

        for i in 0..=i_max {
            for j in 0..=j_max {
                let x = x_range.0 + (i as f32) * (x_range.1 - x_range.0) / (i_max as f32);
                let y = y_range.0 + (j as f32) * (y_range.1 - y_range.0) / (j_max as f32);
                let transform = Transform::new(
                    [x, y, -7.].into(),
                    Quat::IDENTITY,
                    Vec3::new(scale, scale, scale),
                );
                scene.insert(
                    self.common.scene.graph.root,
                    NodeValue::Model(model.clone()),
                    transform,
                );
            }
        }

        scene
    }

    fn load_scene3(&self, ctx: &Context) -> SceneGraph {
        let file_name = load::get_asset_dir("house2.glb", load::AssetType::MODEL).unwrap();

        let mut gltf_loader = gltf::GLTFLoader::new(
            ctx,
            self.common.graph.pass_by_type(PassType::Forward).unwrap(),
        );

        gltf_loader.load(ctx, file_name);

        gltf_loader.scene
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("glTF", 1920, 1080).run::<App>();
}
