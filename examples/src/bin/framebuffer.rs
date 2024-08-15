use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    gfx::{Device, SamplerFilter},
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        material::{Texture, TextureType},
        pass::PassType,
        renderable::Renderable,
        Model,
    },
    resource::entity::light::Light,
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
        let camera = SampleApp::ortho_camera(ctx);
        let camera_position = Vec3::new(0., 0., 1.);

        let light = Light::new(Color::WHITE);
        let light_position = Vec3::new(0., 0., 10.);

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

    fn update(&mut self, ctx: &Context, delta: f32) {
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
            SamplerFilter::FilterLinear,
        );

        let material_instance = material.instantiate(vec![texture]);

        let rect = Model::builder("rect")
            .mesh(Shapes::quad(Color::WHITE), Some(material_instance))
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

    Application::<App>::new("Framebuffer", examples::WIDTH, examples::HEIGHT).run();
}
