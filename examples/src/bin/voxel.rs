use std::{marker::PhantomData, sync::Arc};

use gobs::{
    core::{Color, Input, Key, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{
        BoundingBox, GfxContext, MaterialInstance, Model, RenderBatch, RenderError, RenderFlags,
        RenderType, Renderable, Shapes,
    },
    resource::{ResourceError, ResourceHandle, ResourceManager, camera::Camera, light::Light},
    scene::voxel::{map::VoxelTree, node::VoxelNode64},
};

use examples::InputManager;

struct VoxelData;

type Chunk = VoxelTree<VoxelData, VoxelNode64<VoxelData>>;

struct World {
    cube: Arc<Model>,
    camera: Camera,
    camera_transform: Transform,
    light: Light,
    voxels: Chunk,
    meshes: Vec<Transform>,
}

impl World {
    pub async fn new<Context: GobsContext>(ctx: &mut Context) -> Self {
        let cube = Self::create_model(ctx).await;

        let extent = ctx.extent();

        let yawn: f32 = 45.;
        let pitch: f32 = -25.;

        let camera = Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            yawn.to_radians(),
            pitch.to_radians(),
        );

        let light = Light::new(Color::WHITE);

        Self {
            cube,
            camera,
            camera_transform: Transform::from_translation([-6., 9., 36.].into()),
            light,
            voxels: Chunk::new(3),
            meshes: Vec::new(),
        }
    }

    async fn create_color_material<Context: GobsContext>(
        ctx: &mut Context,
    ) -> ResourceHandle<MaterialInstance> {
        ctx.load_material("materials.ron").await;

        ctx.new_material("color.light")
            .from_base("color.light")
            .build()
    }

    async fn create_model<Context: GobsContext>(ctx: &mut Context) -> Arc<Model> {
        let material = Self::create_color_material(ctx).await;

        let mesh = ctx
            .new_mesh("cube")
            .with_geometry(Shapes::cube(&[Color::RED], 1.))
            .for_material(material)
            .build();

        ctx.new_model("cube")
            .with_mesh(mesh)
            .with_material(material)
            .build()
    }
}

impl Renderable for World {
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        batch: &mut RenderBatch,
        _transform: Option<Transform>,
        bounding_box: Option<BoundingBox>,
        render_flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        for mesh in &self.meshes {
            batch.add_model(
                ctx,
                resource_manager,
                self.cube.clone(),
                *mesh,
                bounding_box,
                render_flags,
            )?;
        }

        let light_transform = Transform::from_translation(-self.camera.dir());

        batch.add_camera_data(
            &self.camera,
            self.camera_transform,
            &self.light,
            light_transform,
        );

        Ok(())
    }
}

struct App<Context: GobsContext> {
    world: World,
    input: InputManager,
    marker: PhantomData<Context>,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        Ok(App {
            world: World::new(ctx).await,
            input: InputManager::new(),
            marker: PhantomData,
        })
    }

    async fn start(&mut self, _ctx: &mut Context) {
        for z in 0..32 {
            for x in 0..32 {
                self.world.voxels.insert(VoxelData, x, 0, z);
            }
        }

        self.world.voxels.insert(VoxelData, 7, 2, 13);
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, _ctx: &mut Context, delta: f32) {
        self.input.controller.update_camera(
            &mut self.world.camera,
            &mut self.world.camera_transform,
            delta,
        );

        if self.world.voxels.is_dirty() {
            self.world.meshes.clear();
            self.world.voxels.visit(true, &mut |pos, _| {
                let pos_f = [pos[0] as f32, pos[1] as f32, pos[2] as f32];

                self.world
                    .meshes
                    .push(Transform::from_translation(pos_f.into()));
            });
        }
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        ctx.render()?
            .draw_bounds(self.input.draw_bounds)
            .draw_wire(self.input.draw_wire)
            .with_renderable(&self.world, RenderType::Scene)?
            .build()
    }

    fn input(&mut self, _ctx: &mut Context, input: Input) {
        self.input.input(input, false);

        if let Input::KeyPressed(key) = input {
            match key {
                Key::I => {
                    tracing::info!(target: logger::APP, "Camera: {} ({:?})", self.world.camera, self.world.camera_transform)
                }
                Key::Backspace => {}
                _ => (),
            }
        }
    }

    fn resize(&mut self, _ctx: &mut Context, width: u32, height: u32) {
        self.world.camera.resize(width, height);
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Cube", examples::WIDTH, examples::HEIGHT).run();
}
