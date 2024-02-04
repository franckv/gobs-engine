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
    material::{texture::Texture, Material},
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        SamplerFilter,
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
            (0., 0., 1.),
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
        let extent = self.graph.draw_extent;
        let (width, height) = (self.graph.draw_extent.width, self.graph.draw_extent.height);

        let framebuffer = Self::generate_framebuffer(width, height);

        let material = Material::default(ctx);

        let texture = Texture::with_data(ctx, framebuffer, extent, SamplerFilter::FilterLinear);

        let material_instance = material.instanciate(texture);

        let rect = Model::new(ctx, "rect", &[Shapes::quad()], &[material_instance]);
        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [width as f32, -(height as f32), 1.].into(),
        );
        let node = Node::new(NodeValue::Model(rect), transform);
        self.scene.graph.insert(self.scene.graph.root, node);
    }

    fn close(&mut self, ctx: &gobs::render::context::Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
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

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
