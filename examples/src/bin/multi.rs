use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    gfx::Device,
    render::{FrameGraph, Model, PassType, RenderError, Texture, TextureProperties, TextureType},
    resource::{
        entity::{camera::Camera, light::Light},
        geometry::Shapes,
    },
    scene::{components::NodeValue, scene::Scene},
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
    async fn create(ctx: &GameContext) -> Result<Self, AppError> {
        let extent = ctx.gfx.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            0.,
            0.,
        );
        let camera_position = Vec3::new(0., 0., 3.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let graph = FrameGraph::default(&ctx.gfx)?;
        let ui = UIRenderer::new(&ctx.gfx, graph.pass_by_type(PassType::Ui)?)?;
        let scene = Scene::new(camera, camera_position, light, light_position);

        Ok(App {
            common,
            camera_controller,
            graph,
            ui,
            scene,
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        self.scene.update_camera(|transform, camera| {
            self.camera_controller
                .update_camera(camera, transform, delta);
        });

        self.graph.update(&ctx.gfx, delta);
        self.scene.update(&ctx.gfx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &GameContext, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
            Some(&mut self.camera_controller),
        );
    }

    fn resize(&mut self, ctx: &mut GameContext, width: u32, height: u32) {
        self.graph.resize(&mut ctx.gfx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn close(&mut self, ctx: &GameContext) {
        tracing::info!("Closing");

        ctx.gfx.device.wait();

        tracing::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &mut GameContext) {
        let color_material = self.common.color_material(&ctx.gfx, &self.graph);
        let color_material_instance = color_material.instantiate(vec![]);

        let diffuse_material = self.common.normal_mapping_material(&ctx.gfx, &self.graph);

        let properties = TextureProperties::with_file("Wall Diffuse", examples::WALL_TEXTURE);
        let diffuse_texture = ctx.resource_manager.add::<Texture>(properties);

        let mut properties = TextureProperties::with_file("Wall Normal", examples::WALL_TEXTURE_N);
        properties.format.ty = TextureType::Normal;
        let normal_texture = ctx.resource_manager.add::<Texture>(properties);

        let diffuse_material_instance =
            diffuse_material.instantiate(vec![diffuse_texture, normal_texture]);

        let model = Model::builder("multi")
            .mesh(
                Shapes::triangle(
                    Color::RED,
                    Color::GREEN,
                    Color::BLUE,
                    1.5,
                    ctx.gfx.vertex_padding,
                ),
                Some(color_material_instance),
            )
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1., ctx.gfx.vertex_padding),
                Some(diffuse_material_instance),
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

    tracing::info!("Engine start");

    Application::<App>::new("Multi", examples::WIDTH, examples::HEIGHT).run();
}
