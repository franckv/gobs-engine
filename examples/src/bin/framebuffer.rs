use glam::Quat;

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderError, RenderType, Shapes},
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

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
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

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    async fn init(&mut self, ctx: &mut Context) {
        let extent = ctx.extent();
        let (width, height) = (extent.width, extent.height);

        let framebuffer = Self::generate_framebuffer(width, height);

        ctx.load_material("materials.ron").await;

        let diffuse_texture = ctx
            .new_texture("Framebuffer")
            .diffuse_colors(examples::DIFFUSE_FORMAT, &framebuffer, extent)
            .build();

        let material = ctx
            .new_material("Framebuffer")
            .from_base("texture")
            .with_textures(&[diffuse_texture])
            .build();

        let mesh = ctx
            .new_mesh("rect")
            .with_geometry(Shapes::square(&[Color::WHITE]))
            .for_material(material)
            .build();

        let rect = ctx
            .new_model("rect")
            .with_mesh(mesh)
            .with_material(material)
            .build();

        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [width as f32, height as f32, 1.].into(),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(rect), transform);
    }

    fn generate_framebuffer(width: u32, height: u32) -> Vec<Color> {
        let mut buffer = Vec::new();

        let border = 50;

        for i in 0..height {
            for j in 0..width {
                if i < border || i >= height - border || j < border || j >= width - border {
                    buffer.push(Color::BLUE);
                } else {
                    buffer.push(Color::RED);
                }
            }
        }
        buffer
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Framebuffer", examples::WIDTH, examples::HEIGHT).run();
}
