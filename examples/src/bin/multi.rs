use futures::try_join;
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
        let extent = ctx.extent();

        let camera = Camera::perspective(
            Vec3::new(0., 0., 3.),
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        );

        let light = Light::new((0., 0., 10.), Color::WHITE);

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
        self.init(ctx).await;
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    async fn init(&mut self, ctx: &Context) {
        let color_material = self.common.color_material(ctx, &self.graph);
        let color_material_instance = color_material.instantiate(vec![]);

        let diffuse_material = self.common.normal_mapping_material(ctx, &self.graph);
        let diffuse_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE,
            TextureType::Diffuse,
            SamplerFilter::FilterLinear,
        );
        let normal_texture = Texture::with_file(
            ctx,
            examples::WALL_TEXTURE_N,
            TextureType::Normal,
            SamplerFilter::FilterLinear,
        );
        let (diffuse_texture, normal_texture) = try_join!(diffuse_texture, normal_texture).unwrap();
        let diffuse_material_instance =
            diffuse_material.instantiate(vec![diffuse_texture, normal_texture]);

        let model = Model::builder("multi")
            .mesh(
                Shapes::triangle(Color::RED, Color::GREEN, Color::BLUE, 1.5),
                color_material_instance,
            )
            .mesh(Shapes::cube(1, 1, &[1], 1.), diffuse_material_instance)
            .build();

        let transform = Transform::new([0., 0., 0.].into(), Quat::IDENTITY, Vec3::ONE);
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(model), transform);
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("Multi", 1920, 1080).run::<App>();
}
