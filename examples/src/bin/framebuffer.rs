use glam::Quat;

use gobs::{
    core::{entity::light::Light, Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    render::{
        context::Context,
        geometry::Model,
        graph::{FrameGraph, RenderError},
        material::{Texture, TextureType},
        pass::PassType,
        renderable::Renderable,
        SamplerFilter,
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
        let light = Light::new((0., 0., 10.), Color::WHITE);
        let camera = SampleApp::ortho_camera(ctx);

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

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

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

    async fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &gobs::render::context::Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let extent = self.graph.draw_extent;
        let (width, height) = (extent.width, extent.height);

        let framebuffer = Self::generate_framebuffer(width, height);

        let material = self.common.texture_material(ctx, &self.graph);

        let texture = Texture::with_colors(
            ctx,
            &framebuffer,
            extent,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
        );

        let material_instance = material.instantiate(vec![texture]);

        let rect = Model::builder("rect")
            .mesh(Shapes::quad(Color::WHITE), material_instance)
            .build();

        let transform = Transform::new(
            [0., 0., 0.].into(),
            Quat::IDENTITY,
            [width as f32, height as f32, 1.].into(),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(rect), transform);
    }

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

    Application::new("Framebuffer", 1920, 1080).run::<App>();
}
