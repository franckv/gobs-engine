use gobs::{
    core::{Color, ConfigWriter as _, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    graphics::Shapes,
    render::{RenderConfig, RenderError, RenderType},
    scene::{components::NodeValue, scene::Scene},
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
        self.input.update(delta);

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
        let cube_size = 600.;

        ctx.load_material("materials.ron").await;

        let checker_material = ctx.new_material("gamma").from_base("gamma").build();

        let checker_mesh = ctx
            .new_mesh("checker")
            .with_geometry(Shapes::rect(
                &[],
                cube_size / 2.,
                -cube_size / 2.,
                -cube_size,
                0.,
            ))
            .for_material(checker_material)
            .build();

        let checker = ctx
            .new_model("checker")
            .with_mesh(checker_mesh)
            .with_material(checker_material)
            .build();

        let grey_material = ctx.new_material("grey").from_base("color").build();

        let grey_mesh = ctx
            .new_mesh("greybox")
            .with_geometry(Shapes::rect(
                &[Color::new(0.5, 0.5, 0.5, 1.0)],
                cube_size / 2.,
                -cube_size / 2.,
                0.,
                cube_size,
            ))
            .for_material(grey_material)
            .build();

        let greybox = ctx
            .new_model("grey")
            .with_mesh(grey_mesh)
            .with_material(grey_material)
            .build();

        self.scene.graph.insert(
            self.scene.graph.root,
            NodeValue::Model(checker),
            Transform::IDENTITY,
        );

        self.scene.graph.insert(
            self.scene.graph.root,
            NodeValue::Model(greybox),
            Transform::IDENTITY,
        );
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Triangle", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| config.set_string(RenderConfig::GraphName, "simple"))
        .run();
}
