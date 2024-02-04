use examples::CameraController;
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
        graph::{FrameGraph, RenderError},
        pass::PassType,
    },
    scene::{
        graph::scenegraph::{Node, NodeValue},
        import::gltf,
        scene::Scene,
    },
};

const ASSET_DIR: &str = "examples/assets";

struct App {
    graph: FrameGraph,
    scene: Scene,
    camera_controller: CameraController,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        log::info!("Create");

        let graph = FrameGraph::new(ctx);

        let camera = Camera::perspective(
            Vec3::splat(0.),
            graph.draw_extent.width as f32 / graph.draw_extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        );

        let light = Light::new(Vec3::new(-3., 0., 5.), Color::WHITE);

        let scene = Scene::new(ctx, camera, light);

        let camera_controller = CameraController::new(3., 0.4);

        App {
            graph,
            scene,
            camera_controller,
        }
    }

    fn start(&mut self, ctx: &Context) {
        log::trace!("Start");

        self.load_scene(ctx);
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        log::trace!("Update");

        let angular_speed = 40.;

        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        let old_position = self.scene.light.position;
        let position =
            Quat::from_axis_angle(Vec3::Y, (angular_speed * delta).to_radians()) * old_position;
        self.scene.light.update(position);

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
        self.scene.resize(width, height);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    #[allow(unused)]
    fn load_scene(&mut self, ctx: &Context) {
        let models = gltf::load_gltf(ctx, &format!("{}/basicmesh.glb", ASSET_DIR));

        let i_max = 3;
        let j_max = 3;
        let x_range = (-5., 5.);
        let y_range = (-3., 3.);
        let scale = 0.7;

        let model = models[2].clone();

        for i in 0..=i_max {
            for j in 0..=j_max {
                let x = x_range.0 + (i as f32) * (x_range.1 - x_range.0) / (i_max as f32);
                let y = y_range.0 + (j as f32) * (y_range.1 - y_range.0) / (j_max as f32);
                let transform = Transform::new(
                    [x, y, -7.].into(),
                    Quat::IDENTITY,
                    Vec3::new(scale, -scale, scale),
                );
                let node = Node::new(NodeValue::Model(model.clone()), transform);
                self.scene.graph.insert(self.scene.graph.root, node);
            }
        }
    }

    #[allow(unused)]
    fn load_scene2(&mut self, ctx: &Context) {
        let models = gltf::load_gltf(ctx, &format!("{}/basicmesh.glb", ASSET_DIR));

        let scale = 1.;

        let model = models[2].clone();

        let transform = Transform::new(
            [0., 0., -5.].into(),
            Quat::IDENTITY,
            Vec3::new(scale, -scale, scale),
        );
        let node = Node::new(NodeValue::Model(model.clone()), transform);
        self.scene.graph.insert(self.scene.graph.root, node);
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
