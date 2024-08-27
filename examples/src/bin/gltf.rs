use glam::{Quat, Vec3};

use gobs::{
    assets::gltf,
    core::Color,
    game::{
        app::{Application, Run},
        input::Input,
    },
    gfx::Device,
    render::{Context, FrameGraph, PassType, RenderError, Renderable},
    resource::{entity::light::Light, load},
    scene::{graph::scenegraph::SceneGraph, scene::Scene},
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
        let camera = SampleApp::perspective_camera(ctx);
        let camera_position = Vec3::new(10., 5., 10.);

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
            &mut self.camera_controller,
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
    fn init(&mut self, ctx: &Context) {
        tracing::info!("Load scene 0");
        let graph = self.load_scene(ctx);
        self.scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph.root, &graph)
            .unwrap();
    }

    fn load_scene(&self, ctx: &Context) -> SceneGraph {
        let file_name = load::get_asset_dir("house.glb", load::AssetType::MODEL).unwrap();

        let mut gltf_loader =
            gltf::GLTFLoader::new(ctx, self.graph.pass_by_type(PassType::Forward).unwrap());

        gltf_loader.load(file_name);

        gltf_loader.scene
    }
}

fn main() {
    examples::init_logger();

    tracing::info!("Engine start");

    Application::<App>::new("glTF", examples::WIDTH, examples::HEIGHT).run();
}
