use std::sync::Arc;

use glam::{Quat, Vec3};

use gobs::{
    core::{Color, Input, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{Mesh, Model, RenderError, Shapes},
    resource::ResourceHandle,
    scene::{
        components::{NodeId, NodeValue},
        scene::Scene,
    },
};

use examples::{InputManager, Ui};

struct App<Context: GobsContext> {
    scene: Scene,
    ui: Ui<Context>,
    input: InputManager,
    cube: Option<ResourceHandle<Mesh>>,
    n_cubes: usize,
    starting_cube: Option<NodeId>,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let scene = ctx
            .new_scene()
            .with_perspective_camera(0., -25., [-0.6, 2., 4.])
            .with_light(Color::WHITE, [-2., 2.5, 10.])
            .build();

        Ok(App {
            scene,
            ui: Ui::new(),
            input: InputManager::new(),
            cube: None,
            n_cubes: 0,
            starting_cube: None,
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

        let frame = ctx.frame_number();

        if let Some(id) = self.starting_cube
            && frame == 99
        {
            self.despawn_cube(ctx, id);

            self.starting_cube = None;
        }

        if frame > 0 && frame.is_multiple_of(99) && self.n_cubes < 6 {
            self.spawn_cube(ctx);
        }

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
            .with_renderable(&self.scene, gobs::render::RenderType::Scene)?
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
    fn new_cube(&mut self, ctx: &mut Context, name: &str) -> Arc<Model> {
        let diffuse_texture = ctx
            .new_texture(&format!("[Diffuse] {}, {}", name, self.n_cubes))
            .diffuse(examples::ATLAS[self.n_cubes], examples::DIFFUSE_FORMAT)
            .build();

        let normal_texture = ctx
            .new_texture(&format!("[Normal] {}, {}", name, self.n_cubes))
            .normal(examples::ATLAS_N[self.n_cubes], examples::NORMAL_FORMAT)
            .build();

        let material = ctx
            .new_material(name)
            .from_base("normal.array")
            .with_textures(&[diffuse_texture, normal_texture])
            .build();

        let mesh = if let Some(mesh) = self.cube {
            mesh
        } else {
            let mesh = ctx
                .new_mesh("cube")
                .with_geometry(Shapes::cube(&[Color::WHITE], 1.))
                .for_material(material)
                .build();

            self.cube = Some(mesh);

            mesh
        };

        ctx.new_model("cube")
            .with_mesh(mesh)
            .with_material(material)
            .build()
    }

    fn spawn_cube(&mut self, ctx: &mut Context) {
        let name = format!("Texture {}", self.n_cubes);

        let cube = self.new_cube(ctx, &name);

        let x = -2. + (self.n_cubes % 3) as f32 * 1.5;
        let y = -2. + (self.n_cubes / 3) as f32 * 2.;

        let transform = Transform::new(
            [x, y, -2.].into(),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::splat(1.),
        );

        self.scene
            .graph
            .insert(self.scene.graph.root, NodeValue::Model(cube), transform);

        self.n_cubes += 1;
    }

    fn despawn_cube(&mut self, ctx: &mut Context, id: NodeId) {
        let cube = self.scene.graph.remove(id);

        if let Some(cube) = cube
            && let NodeValue::Model(model) = cube.value()
        {
            for (_mesh, material) in &model.meshes {
                if let Some(material) = material {
                    ctx.free_material(*material, true);
                }
            }
        }
    }

    async fn init(&mut self, ctx: &mut Context) {
        ctx.load_material("materials.ron").await;

        let cube = self.new_cube(ctx, "start cube");

        let transform = Transform::new(
            [-0.5, -1., -2.].into(),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::splat(2.),
        );

        self.starting_cube =
            self.scene
                .graph
                .insert(self.scene.graph.root, NodeValue::Model(cube), transform);
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Cubes", examples::WIDTH, examples::HEIGHT).run();
}
