use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{RenderError, RenderType, Shapes},
    scene::{components::NodeValue, scene::Scene},
};

use examples::{InputManager, Ui};

struct App<Context: GobsContext> {
    scene: Scene,
    ui: Ui<Context>,
    input: InputManager,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., -25., [0., 1., 0.])
            .with_light(Color::WHITE, [-2., 2.5, 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
        })
    }

    async fn start(&mut self, ctx: &mut Context) {
        self.init(ctx).await;
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
        self.input.update(delta);

        if self.input.process_updates {
            let angular_speed = 10.;

            self.scene
                .graph
                .visit_update(self.scene.graph.root, &mut |node| {
                    if let NodeValue::Model(_) = node.base.value {
                        node.update_transform(|transform| {
                            transform.rotate(Quat::from_axis_angle(
                                Vec3::Y,
                                (angular_speed * delta).to_radians(),
                            ));
                            true
                        });
                    }
                    false
                });
        }

        self.scene.update_camera(|transform, camera| {
            self.input
                .controller
                .update_camera(camera, transform, delta)
        });

        if self.input.draw_ui {
            self.ui.draw(ctx, &mut self.scene, delta);
        }

        self.scene.update(delta);
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.scene, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, ctx: &mut Context, input: Input) {
        self.input.input(ctx, input, self.ui.ui_hovered);
    }

    fn resize(&mut self, _ctx: &mut Context, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    async fn init(&mut self, ctx: &mut Context) {
        ctx.load_material("materials.ron").await;

        let diffuse_texture = ctx
            .new_texture("Atlas Diffuse")
            .diffuse_atlas(
                examples::ATLAS,
                examples::DIFFUSE_FORMAT,
                examples::ATLAS_COLS,
            )
            .build();
        let normal_texture = ctx
            .new_texture("Atlas Normal")
            .normal_atlas(
                examples::ATLAS_N,
                examples::NORMAL_FORMAT,
                examples::ATLAS_COLS,
            )
            .build();

        let material = ctx
            .new_material("atlas")
            .from_base("normal")
            .with_textures(&[diffuse_texture, normal_texture])
            .build();

        let mesh = ctx
            .new_mesh("cube")
            .with_geometry(Shapes::cubemap(
                examples::ATLAS_COLS,
                examples::ATLAS_ROWS,
                &[2, 2, 2, 2, 3, 0],
                1.,
            ))
            .for_material(material)
            .build();

        let cube = ctx
            .new_model("cube")
            .with_mesh(mesh)
            .with_material(material)
            .build();

        let transform = Transform::new([0., 0., -2.].into(), Quat::IDENTITY, Vec3::splat(1.));
        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Atlas", examples::WIDTH, examples::HEIGHT).run();
}
