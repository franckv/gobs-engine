use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderError, RenderType},
    scene::scene::Scene,
};

use examples::{InputManager, Ui};

struct App<Context: GobsContext> {
    scene: Scene,
    ui: Ui<Context>,
    input: InputManager,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
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

    async fn start(&mut self, ctx: &mut Context) {
        self.init(ctx);
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
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

        self.scene.update(delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, ctx: &mut Context, input: Input) {
        self.input.input(ctx, input, self.ui.ui_hovered);
    }

    fn resize(&mut self, _ctx: &mut Context, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    fn init(&mut self, ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Load scene");
        let graph = ctx.load_gltf(examples::GLTF_MODEL);

        self.scene
            .graph
            .insert_subgraph(self.scene.graph.root, graph.root, &graph)
            .unwrap();
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("glTF", examples::WIDTH, examples::HEIGHT).run();
}
