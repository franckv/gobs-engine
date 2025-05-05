use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    gfx::Device,
    render::{FrameGraph, Model, PassType, RenderError},
    resource::{entity::light::Light, geometry::Shapes},
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
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

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
        self.common.render(
            &mut ctx.gfx,
            &mut ctx.resource_manager,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
        )
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
        self.init(ctx);
    }

    fn close(&mut self, ctx: &GameContext) {
        tracing::info!("Closing");

        ctx.gfx.device.wait();

        tracing::info!("Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &GameContext) {
        let material = self.common.color_material(&ctx.gfx, &self.graph);
        let material_instance = material.instantiate(vec![]);

        let transparent_material = self
            .common
            .color_material_transparent(&ctx.gfx, &self.graph);
        let transparent_material_instance = transparent_material.instantiate(vec![]);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    Color::RED,
                    Color::GREEN,
                    Color::BLUE,
                    1.,
                    ctx.gfx.vertex_padding,
                ),
                Some(material_instance),
            )
            .build();

        let square = Model::builder("square")
            .mesh(
                Shapes::quad(Color::new(1., 1., 1., 0.5), ctx.gfx.vertex_padding),
                //Shapes::quad(Color::new(1., 1., 1., 0.5)),
                Some(transparent_material_instance),
            )
            .build();

        let transform =
            Transform::new([0., 0., 0.].into(), Quat::IDENTITY, [300., 300., 1.].into());
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(triangle), transform);

        let transform = Transform::new(
            [0., 0., 0.5].into(),
            Quat::IDENTITY,
            [300., 300., 1.].into(),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(square), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!("Engine start");

    Application::<App>::new("Transparent", examples::WIDTH, examples::HEIGHT).run();
}
