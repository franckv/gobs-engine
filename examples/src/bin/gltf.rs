use glam::{Quat, Vec3};

use gobs::{
    assets::gltf_load,
    core::{Color, Input, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{RenderError, RenderType},
    resource::load,
    scene::{graph::scenegraph::SceneGraph, scene::Scene},
};

use examples::{InputManager, Ui};

struct App {
    scene: Scene,
    ui: Ui,
    input: InputManager,
}

impl GobsGame<GameContext> for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., 0., [10., 5., 10.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx);
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.input.process_updates {
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
            self.input
                .controller
                .update_camera(camera, transform, delta)
        });

        if self.input.draw_ui {
            self.ui.draw(ctx, &mut self.scene, delta);
        }

        self.scene.update(&ctx.renderer.gfx, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, _ctx: &mut GameContext, input: Input) {
        self.input.input(input, false);
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Load scene");
        let graph = self.load_scene(ctx);
        self.scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph.root, &graph)
            .unwrap();
    }

    fn load_scene(&self, ctx: &mut GameContext) -> SceneGraph {
        let file_name = load::get_asset_dir(examples::GLTF_MODEL, load::AssetType::MODEL).unwrap();

        let mut gltf_loader = gltf_load::GLTFLoader::new(&mut ctx.resource_manager).unwrap();

        gltf_loader
            .load(ctx.config(), &mut ctx.resource_manager, file_name)
            .expect("Load gltf");

        gltf_loader.scene
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App, GameContext>::new("glTF", examples::WIDTH, examples::HEIGHT).run();
}
