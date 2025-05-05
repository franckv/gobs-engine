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
            (-25_f32).to_radians(),
        );
        let camera_position = Vec3::new(0., 1., 0.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(-2., 2.5, 10.);

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

    async fn start(&mut self, ctx: &mut GameContext) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        if self.common.process_updates {
            let angular_speed = 10.;

            self.scene
                .graph
                .visit_update(self.scene.graph.root, &mut |node| {
                    if let NodeValue::Model(_) = node.base.value {
                        node.update_transform(|transform| {
                            transform.rotate(Quat::from_axis_angle(
                                Vec3::Y,
                                (angular_speed * delta).to_radians(),
                            ));
                        });
                    }
                });
        }

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

    fn close(&mut self, ctx: &GameContext) {
        tracing::info!("Closing");

        ctx.gfx.device.wait();

        tracing::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &GameContext) {
        let material = self.common.depth_material(&ctx.gfx, &self.graph);

        let material_instance =
            //NormalMaterial::instanciate(material, diffuse_texture, normal_texture);
            material.instantiate(vec![]);

        let cube = Model::builder("cube")
            .mesh(
                Shapes::cubemap(1, 1, &[1], 1., ctx.gfx.vertex_padding),
                Some(material_instance),
            )
            .build();

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}
fn main() {
    examples::init_logger();

    tracing::info!("Engine start");

    Application::<App>::new("Depth test", examples::WIDTH, examples::HEIGHT).run();
}
