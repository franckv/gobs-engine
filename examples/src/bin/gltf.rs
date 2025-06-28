use glam::{Quat, Vec3};

use gobs::{
    assets::gltf_load,
    core::{Color, Input},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render_graph::RenderError,
    resource::{entity::light::Light, load},
    scene::{graph::scenegraph::SceneGraph, scene::Scene},
    ui::UIRenderer,
};

use examples::{CameraController, SampleApp};

struct App {
    common: SampleApp,
    camera_controller: CameraController,
    ui: UIRenderer,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let camera = SampleApp::perspective_camera(ctx);
        let camera_position = Vec3::new(10., 5., 10.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let ui = UIRenderer::new(
            &ctx.renderer.gfx,
            &mut ctx.resource_manager,
            ctx.renderer.ui_pass(),
        )?;
        let scene = Scene::new(
            &ctx.renderer.gfx,
            camera,
            camera_position,
            light,
            light_position,
        );

        Ok(App {
            common,
            camera_controller,
            ui,
            scene,
        })
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx);
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;

            self.scene.update_light(|transform, _| {
                let translation =
                    Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians())
                        * transform.translation();

                transform.set_translation(translation);

                true
            });
        }

        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta)
        });

        self.scene.update(&ctx.renderer.gfx, delta);

        self.common
            .update_ui(ctx, &mut self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render(ctx, Some(&mut self.scene), Some(&mut self.ui))
    }

    fn input(&mut self, ctx: &mut GameContext, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.scene,
            &mut self.ui,
            Some(&mut self.camera_controller),
        );
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: "app", "Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &mut GameContext) {
        tracing::info!(target: "app", "Load scene 0");
        let graph = self.load_scene(ctx);
        self.scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph.root, &graph)
            .unwrap();
    }

    fn load_scene(&self, ctx: &mut GameContext) -> SceneGraph {
        let file_name = load::get_asset_dir("house2.glb", load::AssetType::MODEL).unwrap();

        let pass = ctx.renderer.ui_pass();
        let mut gltf_loader =
            gltf_load::GLTFLoader::new(&mut ctx.renderer.gfx, &mut ctx.resource_manager, pass)
                .unwrap();

        gltf_loader
            .load(&mut ctx.resource_manager, file_name)
            .expect("Load gltf");

        gltf_loader.scene
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: "app", "Engine start");

    Application::<App>::new("glTF", examples::WIDTH, examples::HEIGHT).run();
}
