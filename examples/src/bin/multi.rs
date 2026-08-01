use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsGame, context::GobsContext as _},
    render::{
        MaterialDataPropData, MaterialInstanceProperties, MaterialsConfig, Model, RenderError,
        RenderType, Shapes, TextureProperties, TextureType,
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
            .with_perspective_camera(0., 0., [0., 0., 3.])
            .with_light(Color::WHITE, [0., 0., 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
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

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        self.input.process_updates
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        MaterialsConfig::load_resources("materials.ron", &mut ctx.resource_manager).await;

        let color_material = ctx.resource_manager.get_by_name("color.material").unwrap();

        let color_instance_properties =
            MaterialInstanceProperties::new("color instance", color_material)
                .prop(MaterialDataPropData::DiffuseColor(Color::RED.into()))
                .prop(MaterialDataPropData::EmissionColor(
                    Color::new(0., 0.1, 0., 1.).into(),
                ));

        let color_material_instance =
            ctx.resource_manager
                .add(color_instance_properties, ResourceLifetime::Static, false);

        let properties = TextureProperties::with_file(
            "Wall Diffuse",
            examples::DIFFUSE_FORMAT,
            examples::WALL_TEXTURE,
        );
        let diffuse_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let mut properties = TextureProperties::with_file(
            "Wall Normal",
            examples::NORMAL_FORMAT,
            examples::WALL_TEXTURE_N,
        );
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx
            .resource_manager
            .add(properties, ResourceLifetime::Static, false);

        let diffuse_material = ctx.resource_manager.get_by_name("normal").unwrap();

        let diffuse_instance_properties =
            MaterialInstanceProperties::new("diffuse instance", diffuse_material)
                .textures(&[diffuse_texture, normal_texture]);

        let diffuse_material_instance =
            ctx.resource_manager
                .add(diffuse_instance_properties, ResourceLifetime::Static, false);

        let model = Model::builder("multi")
            .mesh(
                Shapes::triangle(&[Color::RED, Color::GREEN, Color::BLUE], 1.5),
                Some(color_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1.),
                Some(diffuse_material_instance),
                &mut ctx.resource_manager,
                ResourceLifetime::Static,
            )
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

    Application::<App, GameContext>::new("Multi", examples::WIDTH, examples::HEIGHT).run();
}
