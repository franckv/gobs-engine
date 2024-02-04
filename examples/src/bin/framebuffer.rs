use glam::Quat;

use gobs::{
    core::{Color, Transform},
    game::{
        app::{Application, Run},
        input::Input,
    },
    material::{texture::Texture, Material},
    render::{context::Context, graph::RenderError, SamplerFilter},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        model::Model,
        shape::Shapes,
    },
};

use examples::SampleApp;

struct App {
    common: SampleApp,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let common = SampleApp::create_ortho(ctx);

        App { common }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.common.update(ctx, delta);
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        self.common.render(ctx)
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.common.input(ctx, input);
    }

    fn resize(&mut self, ctx: &Context, width: u32, height: u32) {
        self.common.resize(ctx, width, height);
    }

    fn start(&mut self, ctx: &Context) {
        self.init(ctx);
    }

    fn close(&mut self, ctx: &gobs::render::context::Context) {
        self.common.close(ctx);
    }
}

impl App {
    fn init(&mut self, ctx: &Context) {
        let extent = self.common.graph.draw_extent;
        let (width, height) = (
            self.common.graph.draw_extent.width,
            self.common.graph.draw_extent.height,
        );

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
        let node: Node = Node::new(NodeValue::Model(rect), transform);
        self.common
            .scene
            .graph
            .insert(self.common.scene.graph.root, node);
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

    Application::new("examples", 1600, 900).run::<App>();
}
