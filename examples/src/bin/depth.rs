use glam::{Quat, Vec3};

use gobs::{
    core::{
        entity::{camera::Camera, light::Light},
        Color, Transform,
    },
    game::{
        app::{Application, Run},
        input::Input,
    },
    render::{
        context::Context,
        geometry::Model,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        renderable::Renderable,
    },
    scene::{graph::scenegraph::NodeValue, scene::Scene, shape::Shapes},
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
            Vec3::new(0., 1., 0.),
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            (-25. as f32).to_radians(),
            Vec3::Y,
        );

        let light = Light::new((-2., 2.5, 10.), Color::WHITE);

        let common = SampleApp::new();

        let camera_controller = CameraController::new(3., 0.1);

        let graph = FrameGraph::default(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());
        let scene = Scene::new(camera, light);

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
        let angular_speed = 40.;

        let root_id = self.scene.graph.root;
        let root = self.scene.graph.get(root_id).unwrap();

        let node = root.children[0];

        self.scene.graph.update(node, |transform| {
            transform.rotate(Quat::from_axis_angle(
                Vec3::Y,
                (0.3 * angular_speed * delta).to_radians(),
            ));
        });

        self.camera_controller
            .update_camera(&mut self.scene.camera_mut(), delta);

        self.graph.update(ctx, delta);
        self.scene.update(ctx, delta);

        self.common
            .update_ui(ctx, &self.graph, &self.scene, &mut self.ui, |_| {});
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common
            .render(ctx, &mut self.graph, &mut self.scene, &mut self.ui)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(
            ctx,
            input,
            &mut self.graph,
            &mut self.ui,
            &mut self.camera_controller,
        );
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
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
            .mesh(Shapes::cube(1, 1, &[1], 1.), material_instance)
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

    Application::new("Depth test", 1920, 1080).run::<App>();
}
