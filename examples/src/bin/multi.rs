use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{MaterialDataPropData, RenderError, RenderType, Shapes},
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
            .with_perspective_camera(0., 0., [0., 0., 3.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
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

    fn input(&mut self, _ctx: &mut Context, input: Input) {
        self.input.input(input, self.ui.ui_hovered);
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

        let color_material = ctx
            .new_material("color instance")
            .from_base("color.material")
            .with_prop(MaterialDataPropData::DiffuseColor(Color::RED.into()))
            .with_prop(MaterialDataPropData::EmissionColor(
                Color::new(0., 0.1, 0., 1.).into(),
            ))
            .build();

        let diffuse_texture = ctx
            .new_texture("Wall Diffuse")
            .diffuse(examples::WALL_TEXTURE, examples::DIFFUSE_FORMAT)
            .build();
        let normal_texture = ctx
            .new_texture("Wall Normal")
            .normal(examples::WALL_TEXTURE_N, examples::NORMAL_FORMAT)
            .build();

        let diffuse_material = ctx
            .new_material("normal")
            .from_base("normal")
            .with_textures(&[diffuse_texture, normal_texture])
            .build();

        let triangle_mesh = ctx
            .new_mesh("triangle")
            .with_geometry(Shapes::triangle(
                &[Color::RED, Color::GREEN, Color::BLUE],
                1.5,
            ))
            .for_material(color_material)
            .build();

        let cube_mesh = ctx
            .new_mesh("cube")
            .with_geometry(Shapes::cubemap(1, 1, &[0], 1.))
            .for_material(diffuse_material)
            .build();

        let model = ctx
            .new_model("cube")
            .with_mesh(triangle_mesh)
            .with_material(color_material)
            .with_mesh(cube_mesh)
            .with_material(diffuse_material)
            .build();

        let transform = Transform::new([0., 0., 0.].into(), Quat::IDENTITY, Vec3::ONE);
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(model), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Multi", examples::WIDTH, examples::HEIGHT).run();
}
