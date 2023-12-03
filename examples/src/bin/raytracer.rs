use glam::Vec3;

use gobs::core::entity::camera::Camera;
use gobs::core::Color;
use gobs::game::{
    app::{Application, Run},
    input::Input,
};
use gobs::ray::{Ray, Sphere, Tracer};
use gobs::scene::{Gfx, RenderError};

const N_RAYS: u32 = 20;

struct App {
    tracer: Tracer,
}

impl Run for App {
    async fn create(gfx: &Gfx) -> Self {
        let (width, height) = (gfx.width(), gfx.height());

        let camera = Camera::perspective(
            Vec3::new(0., 0.2, 0.),
            width as f32 / height as f32,
            (45. as f32).to_radians(),
            0.1,
            100.,
            (-90. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        let mut tracer =
            Tracer::new(gfx, width, height, camera, N_RAYS, Self::background_color).await;

        tracer.add_model(Sphere::new(
            Vec3::new(0., -5000.2, 0.),
            5000.,
            Color::GREY,
            0.1,
        ));
        tracer.add_model(Sphere::new(Vec3::new(0., 0.5, 1.2), 0.3, Color::BLACK, 0.8));
        tracer.add_model(Sphere::new(
            Vec3::new(-0.5, 0.2, 0.7),
            0.3,
            Color::GREEN,
            0.4,
        ));
        tracer.add_model(Sphere::new(Vec3::new(0.5, 0.2, 0.7), 0.3, Color::RED, 0.25));

        App { tracer }
    }

    fn update(&mut self, _delta: f32, gfx: &Gfx) {
        self.tracer.update(gfx);
    }

    fn render(&mut self, gfx: &Gfx) -> Result<(), RenderError> {
        self.tracer.render(gfx)
    }

    fn resize(&mut self, width: u32, height: u32, _gfx: &Gfx) {
        self.tracer.resize(width, height)
    }

    fn input(&mut self, _gfx: &Gfx, _input: Input) {}
}

impl App {
    fn background_color(ray: &Ray) -> Color {
        let dot_x = ray.direction.dot(Vec3::X);
        let dot_y = ray.direction.dot(Vec3::Y);

        Color::new(0.2 * dot_x, 0.5 + 0.5 * dot_y, 1., 1.)
    }
}

fn main() {
    examples::init_logger();

    Application::new().run::<App>();
}
