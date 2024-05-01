use glam::{Quat, Vec3};

use gobs::{
    assets::gltf,
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::{Input, Key},
    },
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        renderable::Renderable,
    },
    scene::{
        graph::node::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
    },
    ui::UIRenderer,
    utils::load,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    graph: FrameGraph,
    ui: UIRenderer,
    scene: Scene,
    scenes: Vec<NodeId>,
    current_scene: usize,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let camera = SampleApp::perspective_camera(ctx);
        let camera_position = Vec3::splat(0.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

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
            scenes: vec![],
            current_scene: 0,
        }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
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
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta, |ui| {
                ui.collapsing("GLTF", |ui| {
                    ui.label(format!("Current scene: {}", self.current_scene));
                });
            });
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
            &mut self.scene,
            &mut self.ui,
            &mut self.camera_controller,
        );

        match input {
            Input::KeyPressed(key) => match key {
                Key::N => {
                    self.scene.graph.toggle(self.scenes[self.current_scene]);
                    self.current_scene = (self.current_scene + 1) % self.scenes.len();
                    self.scene.graph.toggle(self.scenes[self.current_scene]);
                }
                _ => (),
            },
            _ => (),
        }
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
    fn init(&mut self, ctx: &Context) {
        log::info!("Load scene 0");
        let graph0 = self.load_scene(ctx);
        let node0 = self
            .scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph0.root, &graph0)
            .unwrap();
        self.scenes.push(node0);

        log::info!("Load scene 1");
        let graph1 = self.load_scene2(ctx);
        let node1 = self
            .scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph1.root, &graph1)
            .unwrap();
        self.scene.graph.toggle(node1);
        self.scenes.push(node1);

        log::info!("Load scene 2");
        let graph2 = self.load_scene3(ctx);
        let node2 = self
            .scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph2.root, &graph2)
            .unwrap();
        self.scene.graph.toggle(node2);
        self.scenes.push(node2);
    }

    fn load_scene(&self, ctx: &Context) -> SceneGraph {
        let mut scene = SceneGraph::new();

        let file_name = load::get_asset_dir("basicmesh.glb", load::AssetType::MODEL).unwrap();

        let mut gltf_loader =
            gltf::GLTFLoader::new(ctx, self.graph.pass_by_type(PassType::Forward).unwrap());

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
                    self.scene.graph.root,
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

        let mut gltf_loader =
            gltf::GLTFLoader::new(ctx, self.graph.pass_by_type(PassType::Forward).unwrap());

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
                    self.scene.graph.root,
                    NodeValue::Model(model.clone()),
                    transform,
                );
            }
        }

        scene
    }

    fn load_scene3(&self, ctx: &Context) -> SceneGraph {
        let file_name = load::get_asset_dir("house2.glb", load::AssetType::MODEL).unwrap();

        let mut gltf_loader =
            gltf::GLTFLoader::new(ctx, self.graph.pass_by_type(PassType::Forward).unwrap());

        gltf_loader.load(ctx, file_name);

        gltf_loader.scene
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::<App>::new("glTF", 1920, 1080).run();
}
