use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use gobs::{
    core::{Color, Input, Key, Transform, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    render::{
        BoundingBox, GfxContext, MaterialInstance, Model, RenderBatch, RenderError, RenderFlags,
        RenderType, Renderable, Shapes,
    },
    resource::{ResourceError, ResourceHandle, ResourceManager, camera::Camera, light::Light},
    scene::voxel::{chunk::Chunks, ray::RayCast as _},
};

use examples::InputManager;

struct VoxelData;

struct World<Context: GobsContext> {
    camera: Camera,
    camera_transform: Transform,
    light: Light,
    voxels: Chunks<VoxelData>,
    material: ResourceHandle<MaterialInstance>,
    meshes: HashMap<[i32; 3], Arc<Model>>,
    selection: Arc<Model>,
    marker: PhantomData<Context>,
}

impl<Context: GobsContext> World<Context> {
    pub async fn new(ctx: &mut Context) -> Self {
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

        let material = Self::create_color_material(ctx).await;

        let selection = Self::create_selection_model(ctx);

        Self {
            camera,
            camera_transform: Transform::from_translation([-17., 27., 48.].into()),
            light,
            voxels: Chunks::new(3),
            material,
            meshes: HashMap::new(),
            selection,
            marker: PhantomData,
        }
    }

    async fn create_color_material(ctx: &mut Context) -> ResourceHandle<MaterialInstance> {
        ctx.load_material("materials.ron").await;

        ctx.new_material("color.light")
            .from_base("color.light")
            .build()
    }

    fn create_selection_model(ctx: &mut Context) -> Arc<Model> {
        let material = ctx
            .new_material("color.light")
            .from_base("color.light")
            .build();

        let mesh = ctx
            .new_mesh("selection")
            .with_geometry(Shapes::cube(&[Color::YELLOW], 1.01))
            .for_material(material)
            .build();

        ctx.new_model("selection")
            .with_mesh(mesh)
            .with_material(material)
            .build()
    }
}

impl<Context: GobsContext> Renderable for World<Context> {
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        batch: &mut RenderBatch,
        _transform: Option<Transform>,
        _bounding_box: Option<BoundingBox>,
        render_flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        for mesh in self.meshes.values() {
            batch.add_model(
                ctx,
                resource_manager,
                mesh.clone(),
                Transform::default(),
                Some(mesh.bounding_box),
                render_flags,
            )?;
        }

        let origin = self.camera_transform.translation();
        let dir = self.camera.dir();

        if let Some(hit) = self.voxels.raycast(origin.into(), dir.into(), 20.) {
            let transform = Transform::from_translation(
                [hit.pos[0] as f32, hit.pos[1] as f32, hit.pos[2] as f32].into(),
            );

            batch.add_model(
                ctx,
                resource_manager,
                self.selection.clone(),
                transform,
                None,
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
    world: World<Context>,
    input: InputManager,
    fps: f32,
}

impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        Ok(App {
            world: World::new(ctx).await,
            input: InputManager::new(),
            fps: 0.,
        })
    }

    async fn start(&mut self, _ctx: &mut Context) {
        self.load_plane();
        self.load_sphere();
    }

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        self.input.process_updates
    }

    fn update(&mut self, ctx: &mut Context, delta: f32) {
        self.input.controller.update_camera(
            &mut self.world.camera,
            &mut self.world.camera_transform,
            delta,
        );

        ctx.lock_mouse(self.input.controller.lock_mouse());

        self.fps = 1. / delta;

        let chunk_positions: Vec<[i32; 3]> = self.world.voxels.chunks().collect();

        let mut count = 0;
        for pos in &chunk_positions {
            if let Some(chunk) = self.world.voxels.get_mut(*pos)
                && chunk.is_dirty()
            {
                let geometry = chunk.meshify();

                let mesh = ctx
                    .new_mesh("voxel")
                    .with_geometry(geometry)
                    .for_material(self.world.material)
                    .build();

                let model = ctx
                    .new_model("voxel")
                    .with_mesh(mesh)
                    .with_material(self.world.material)
                    .build();

                self.world.meshes.insert(*pos, model);
                count += 1;
            }
        }

        if count > 0 {
            tracing::info!(target: logger::APP, "{} meshes added", count);
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
                Key::F => {
                    tracing::info!(target: logger::APP, "FPS: {}", self.fps);
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

#[allow(unused)]
impl<Context: GobsContext> App<Context> {
    pub fn load_plane(&mut self) {
        let chunk = self.world.voxels.get_or_create([0, 0, 0]);
        for z in 0..32 {
            for x in 0..32 {
                chunk.insert(VoxelData, x, 0, z);
            }
        }
    }

    pub fn load_sphere(&mut self) {
        let radius: i32 = 8;
        let diameter = 2 * radius;

        let chunk = self.world.voxels.get_or_create([0, 0, 0]);

        for z in 0..diameter {
            for y in 0..diameter {
                for x in 0..diameter {
                    let d = (x - radius).pow(2) + (y - radius).pow(2) + (z - radius).pow(2);
                    if d.isqrt() < radius {
                        chunk.insert(
                            VoxelData,
                            (x + radius) as u32,
                            (y + 5) as u32,
                            (z + radius) as u32,
                        );
                    }
                }
            }
        }
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Cube", examples::WIDTH, examples::HEIGHT).run();
}
