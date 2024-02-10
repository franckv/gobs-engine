use std::sync::Arc;

use glam::Vec3;

use gobs::{
    core::entity::{camera::Camera, light::Light},
    game::input::{Input, Key},
    render::{
        context::Context,
        geometry::VertexFlag,
        graph::FrameGraph,
        material::{Material, MaterialProperty},
        pass::PassType,
        ImageExtent2D,
    },
    scene::scene::Scene,
};

use crate::CameraController;

pub struct SampleApp {
    camera_controller: CameraController,
    pub graph: FrameGraph,
    pub scene: Scene,
    pub process_updates: bool,
}

impl SampleApp {
    pub fn create(ctx: &Context, camera: Camera, light: Light) -> Self {
        log::info!("Create");

        let graph = FrameGraph::new(ctx);

        let scene = Scene::new(ctx, camera, light);

        let camera_controller = CameraController::new(3., 0.1);

        Self {
            camera_controller,
            graph,
            scene,
            process_updates: true,
        }
    }

    pub fn extent(ctx: &Context) -> ImageExtent2D {
        ctx.surface.get_extent(ctx.device.clone())
    }

    pub fn ortho_camera(ctx: &Context) -> Camera {
        let extent = ctx.surface.get_extent(ctx.device.clone());

        Camera::ortho(
            (0., 0., 1.),
            extent.width as f32,
            extent.height as f32,
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        )
    }

    pub fn perspective_camera(ctx: &Context) -> Camera {
        let extent = ctx.surface.get_extent(ctx.device.clone());

        Camera::perspective(
            Vec3::splat(0.),
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            0.,
            Vec3::Y,
        )
    }

    pub fn color_material(ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx)
    }

    pub fn texture_material(ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(ctx)
    }

    pub fn normal_mapping_material(ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh_n.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(ctx)
    }

    pub fn depth_material(ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "depth.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx)
    }

    pub fn start(&mut self, _ctx: &Context) {}

    pub fn update(&mut self, ctx: &Context, delta: f32) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        self.scene.update(ctx);
    }

    pub fn render(
        &mut self,
        ctx: &gobs::render::context::Context,
    ) -> Result<(), gobs::render::graph::RenderError> {
        log::trace!("Render frame {}", self.graph.frame_number);

        self.graph.begin(ctx)?;

        self.graph
            .render(ctx, &|pass_type, _, cmd| match pass_type {
                PassType::Compute => {
                    cmd.dispatch(
                        self.graph.draw_extent.width / 16 + 1,
                        self.graph.draw_extent.height / 16 + 1,
                        1,
                    );
                }
                PassType::Forward => {
                    self.scene.draw(ctx, cmd);
                }
            })?;

        self.graph.end(ctx)?;

        log::trace!("End render");

        Ok(())
    }

    pub fn input(&mut self, ctx: &gobs::render::context::Context, input: gobs::game::input::Input) {
        log::trace!("Input");

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => self.graph.render_scaling = (self.graph.render_scaling + 0.1).min(1.),
                Key::A => self.graph.render_scaling = (self.graph.render_scaling - 0.1).max(0.1),
                Key::L => log::info!("{:?}", ctx.allocator.allocator.lock().unwrap()),
                Key::C => log::info!("{:?}", self.scene.camera),
                Key::P => self.process_updates = !self.process_updates,
                _ => self.camera_controller.key_pressed(key),
            },
            Input::KeyReleased(key) => self.camera_controller.key_released(key),
            Input::MousePressed => self.camera_controller.mouse_pressed(),
            Input::MouseReleased => self.camera_controller.mouse_released(),
            Input::MouseWheel(delta) => self.camera_controller.mouse_scroll(delta),
            Input::MouseMotion(dx, dy) => self.camera_controller.mouse_drag(dx, dy),
            _ => (),
        }
    }

    pub fn resize(&mut self, ctx: &gobs::render::context::Context, width: u32, height: u32) {
        log::trace!("Resize");

        self.graph.resize(ctx);
        self.scene.resize(width, height)
    }

    pub fn close(&mut self, ctx: &gobs::render::context::Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}
