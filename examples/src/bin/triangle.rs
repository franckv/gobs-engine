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
    material::Material,
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        pass::PassType,
    },
    scene::{
        graph::scenegraph::{Node, NodeValue},
        model::Model,
        scene::Scene,
        shape::Shapes,
    },
};

use examples::CameraController;

struct App {
    camera_controller: CameraController,
    graph: FrameGraph,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let graph = FrameGraph::new(ctx);

        let camera = Camera::ortho(
            (0., 0., 10.),
            graph.draw_extent.width as f32,
            graph.draw_extent.height as f32,
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        );

        let light = Light::new((0., 0., 10.), Color::WHITE);

        let scene = Scene::new(ctx, camera, light);

        let camera_controller = CameraController::new(3., 0.4);

        App {
            camera_controller,
            graph,
            scene,
        }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        self.scene.update(ctx);
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        log::trace!("Render frame {}", self.graph.frame_number);

        self.graph.begin(ctx)?;

        self.graph
            .render(ctx, &|pass_type, _, cmd| match pass_type {
                PassType::Compute => {
                    cmd.dispatch(
                        self.graph.draw_extent.width / 16 + 1,
                        self.graph.draw_extent.height / 16 + 1,
                        1,
                    );
                }
                PassType::Forward => {
                    self.scene.draw(ctx, cmd);
                }
            })?;

        self.graph.end(ctx)?;

        log::trace!("End render");

        Ok(())
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        log::trace!("Input");

        examples::default_input(
            ctx,
            &self.scene,
            &mut self.graph,
            &mut self.camera_controller,
            input,
        );
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        log::trace!("Resize");

        self.graph.resize(ctx);
        self.scene.resize(width, height)
    }

    fn start(&mut self, ctx: &Context) {
        let triangle = Model::new(
            ctx,
            "triangle",
            &[Shapes::triangle(
                [1., 0., 0., 1.],
                [0., 1., 0., 1.],
                [0., 0., 1., 1.],
            )],
            &[Material::default(ctx)],
        );
        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [300., -300., 1.].into(),
        );
        let node = Node::new(NodeValue::Model(triangle), transform);
        self.scene.graph.insert(self.scene.graph.root, node);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
