use glam::Quat;

use gobs::{
    core::{Color, ConfigWriter as _, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderConfig, RenderError, RenderType, Shapes},
    scene::{components::NodeValue, scene::Scene},
};

use examples::{InputManager, Ui};

struct App<Context>
where
    Context: GobsContext,
{
    scene: Scene,
    ui: Ui<Context>,
    input: InputManager,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_ortho_camera([0., 0., 1.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
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

    async fn start(&mut self, ctx: &mut Context) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    async fn init(&mut self, ctx: &mut Context) {
        ctx.load_material("materials.ron").await;

        let material = ctx.new_material("color").from_base("color").build();

        let mesh = ctx
            .new_mesh("hex")
            .with_geometry(Shapes::hexagon(&[
                Color::WHITE,
                Color::RED,
                Color::GREEN,
                Color::BLUE,
                Color::RED,
                Color::GREEN,
                Color::BLUE,
            ]))
            .for_material(material)
            .build();

        let model = ctx
            .new_model("hex")
            .with_mesh(mesh)
            .with_material(material)
            .build();

        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(model), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Hex", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "simple"))
        .run();
}
