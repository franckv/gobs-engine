use glam::Vec3;

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

        let mut tracer = Tracer::new(gfx, width, height, N_RAYS, Self::background_color).await;

        tracer.add_model(Sphere::new(
            Vec3::new(0., -500.5, -10.),
            500.,
            Color::GREY,
            0.1,
        ));
        tracer.add_model(Sphere::new(Vec3::new(1., 0.5, 2.), 0.5, Color::BLACK, 0.8));
        tracer.add_model(Sphere::new(Vec3::new(-1., 0., 1.5), 0.5, Color::GREEN, 0.4));

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
