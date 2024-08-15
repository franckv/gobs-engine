use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    gfx::Device,
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        renderable::Renderable,
        Model,
    },
    resource::entity::{camera::Camera, light::Light},
    scene::{components::NodeValue, scene::Scene, shape::Shapes},
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
    async fn create(ctx: &Context) -> Self {
        let extent = ctx.extent();

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            (-25. as f32).to_radians(),
        );
        let camera_position = Vec3::new(0., 1., 0.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(-2., 2.5, 10.);

        let common = SampleApp::new();

        let camera_controller = SampleApp::controller();

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, camera_position, light, light_position);

        App {
            common,
            camera_controller,
            graph,
            ui,
            scene,
        }
    }

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx).await;
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
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

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.scene,
            &mut self.ui,
            &mut self.camera_controller,
        );
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.scene.resize(width, height);
        self.ui.resize(width, height);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        let material = self.common.depth_material(ctx, &self.graph);

        let material_instance =
            //NormalMaterial::instanciate(material, diffuse_texture, normal_texture);
            material.instantiate(vec![]);

        let cube = Model::builder("cube")
            .mesh(Shapes::cubemap(1, 1, &[1], 1.), Some(material_instance))
            .build();

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}
fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::<App>::new("Depth test", examples::WIDTH, examples::HEIGHT).run();
}
