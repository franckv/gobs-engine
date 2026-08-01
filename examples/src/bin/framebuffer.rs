use glam::Quat;

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialInstanceProperties, MaterialsConfig, Model, RenderError, RenderType, Shapes,
        TextureProperties,
    },
    resource::ResourceLifetime,
    scene::{components::NodeValue, scene::Scene},
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
            .with_ortho_camera([0., 0., 1.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
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

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        let extent = ctx.renderer.extent();
        let (width, height) = (extent.width, extent.height);

        let framebuffer = Self::generate_framebuffer(width, height);

        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let material = ctx.resource_manager.get_by_name("texture").unwrap();

        let properties = TextureProperties::with_colors(
            "Framebuffer",
            examples::DIFFUSE_FORMAT,
            framebuffer,
            extent,
        );

        let texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let material_instance_properties =
            MaterialInstanceProperties::new("texture", material).textures(&[texture]);
        let material_instance = ctx.resource_manager.add(
            material_instance_properties,
            ResourceLifetime::Static,
            false,
        );

        let rect = Model::builder("rect")
            .mesh(
                Shapes::quad(&[Color::WHITE]),
                Some(material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
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

    Application::<App, GameContext>::new("Framebuffer", examples::WIDTH, examples::HEIGHT).run();
}
