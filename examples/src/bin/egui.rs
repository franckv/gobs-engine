use glam::{Quat, Vec3};
use gobs_egui as gui;
use gobs_game as game;
use gobs_scene as scene;

use game::{
    app::{Application, Run},
    input::Input,
};
use gui::UIRenderer;
use scene::light::Light;
use scene::scene::Scene;
use scene::Gfx;
use scene::{camera::Camera, RenderError};

struct App {
    scene: Scene,
    ui: UIRenderer,
}

impl Run for App {
    async fn create(gfx: &mut Gfx) -> Self {
        let (width, height) = (gfx.width() as f32, gfx.height() as f32);

        let camera = Camera::ortho(
            (width / 2., height / 2., -1.),
            width,
            height,
            0.1,
            10.,
            (90. as f32).to_radians(),
            (0. as f32).to_radians(),
            -Vec3::Y,
        );

        let light = Light::new((0., 0., 10.), (1., 1., 1.));

        let scene = Scene::new(gfx, camera, light).await;

        let ui = UIRenderer::new(width, height, scene.ui_shader.clone());

        App { scene, ui }
    }

    fn update(&mut self, _delta: f32, gfx: &mut Gfx) {
        let models = self.ui.update(gfx, |ctx| {
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ctx.inspection_ui(ui);
                ctx.settings_ui(ui);
                ctx.memory_ui(ui);
            });
        });

        self.scene.nodes.clear();

        models.into_iter().for_each(|m| {
            self.scene.add_node([0., 0., 0.].into(), Quat::IDENTITY, m);
        });

        self.scene.update(gfx);
    }

    fn render(&mut self, gfx: &mut Gfx) -> Result<(), RenderError> {
        self.scene.render(gfx)
    }

    fn input(&mut self, _gfx: &mut Gfx, input: Input) {
        self.ui.input(input);
    }

    fn resize(&mut self, width: u32, height: u32, gfx: &mut Gfx) {
        self.scene.resize(gfx, width, height);
        self.scene.camera.position = [width as f32 / 2., height as f32 / 2., -1.].into();
        self.ui.resize(width, height);
    }
}

fn main() {
    examples::init_logger();

    Application::new().run::<App>();
}
