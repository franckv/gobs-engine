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
        renderable::Renderable,
        ImageExtent2D,
    },
    scene::scene::Scene,
    ui::UIRenderer,
};

use crate::CameraController;

pub struct SampleApp {
    camera_controller: CameraController,
    pub graph: FrameGraph,
    pub ui: UIRenderer,
    pub scene: Scene,
    pub process_updates: bool,
    pub draw_ui: bool,
    pub draw_wire: bool,
    pub fps: u32,
}

impl SampleApp {
    pub fn create(ctx: &Context, camera: Camera, light: Light) -> Self {
        log::info!("Create");

        let graph = FrameGraph::new(ctx);

        let ui = UIRenderer::new(ctx, graph.passes["ui"].clone());

        let scene = Scene::new(camera, light);

        let camera_controller = CameraController::new(3., 0.1);

        Self {
            camera_controller,
            graph,
            ui,
            scene,
            process_updates: true,
            draw_ui: true,
            draw_wire: false,
            fps: 0,
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

    pub fn color_material(&self, ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx, self.graph.passes["forward"].clone())
    }

    pub fn texture_material(&self, ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(ctx, self.graph.passes["forward"].clone())
    }

    pub fn normal_mapping_material(&self, ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh_n.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(ctx, self.graph.passes["forward"].clone())
    }

    pub fn depth_material(&self, ctx: &Context) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "depth.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx, self.graph.passes["forward"].clone())
    }

    pub fn start(&mut self, _ctx: &Context) {}

    pub fn update(&mut self, ctx: &Context, delta: f32) {
        self.camera_controller
            .update_camera(&mut self.scene.camera, delta);

        if self.graph.frame_number % ctx.stats_refresh == 0 {
            self.fps = (1. / delta).round() as u32;
        }

        self.scene.update(ctx, delta);
        if self.draw_ui {
            self.ui
                .update(ctx, self.graph.passes["ui"].clone(), |ectx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(ectx, |ui| {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                            ui.heading(&ctx.app_name);
                            ui.separator();
                            Self::show_fps(ui, self.fps);
                            Self::show_stats(ui, "Render Stats", &self.graph);
                            Self::show_camera(ui, &self.scene.camera);
                            Self::show_memory(ui, ctx);
                        });
                });
        }
    }

    fn show_fps(ui: &mut egui::Ui, fps: u32) {
        ui.label(format!("FPS: {}", fps));
    }

    fn show_stats(ui: &mut egui::Ui, header: &str, graph: &FrameGraph) {
        let stats = graph.render_stats();
        ui.collapsing(header, |ui| {
            for (pass_id, pass_stats) in &stats.pass_stats {
                let pass = graph.pass(*pass_id);
                ui.label(format!("Pass: {}", pass.name()));
                ui.label(format!("  Vertices: {}", pass_stats.vertices));
                ui.label(format!("  Indices: {}", pass_stats.indices));
                ui.label(format!("  Models: {}", pass_stats.models));
                ui.label(format!("  Instances: {}", pass_stats.instances));
                ui.label(format!("  Textures: {}", pass_stats.textures));
            }
            ui.label("Performance");
            ui.label(format!("  Draws: {}", stats.draws));
            ui.label(format!("  Binds: {}", stats.binds));
            ui.label(format!(
                "  CPU draw time: {:.2}ms",
                1000. * stats.cpu_draw_time
            ));
            ui.label(format!("  GPU time: {:.2}ms", 1000. * stats.gpu_draw_time));
            ui.label(format!("  Update time: {:.2}ms", 1000. * stats.update_time));
        });
    }

    fn show_camera(ui: &mut egui::Ui, camera: &Camera) {
        ui.collapsing("Camera", |ui| {
            ui.label(format!(
                "  Position: [{:.2}, {:.2}, {:.2}]",
                camera.position.x, camera.position.y, camera.position.z
            ));
            let dir = camera.dir();
            ui.label(format!(
                "  Direction: [{:.2}, {:.2}, {:.2}]",
                dir.x, dir.y, dir.z
            ));
            ui.label(format!("  Yaw: {:.1}°", camera.yaw.to_degrees()));
            ui.label(format!("  Pitch: {:.1}°", camera.pitch.to_degrees()));
        });
    }

    fn show_memory(ui: &mut egui::Ui, ctx: &Context) {
        ui.collapsing("Memory", |ui| {
            ui.label(format!("{:?}", ctx.allocator.allocator.lock().unwrap()));
        });
    }

    pub fn render(
        &mut self,
        ctx: &gobs::render::context::Context,
    ) -> Result<(), gobs::render::graph::RenderError> {
        log::trace!("Render frame {}", self.graph.frame_number);

        self.graph.begin(ctx)?;

        self.graph.render(ctx, &mut |pass, batch| match pass.ty() {
            PassType::Compute => {}
            PassType::Forward => {
                self.scene.draw(ctx, pass, batch);
            }
            PassType::Wire => {
                if self.draw_wire {
                    self.scene.draw(ctx, pass, batch);
                }
            }
            PassType::Ui => {
                if self.draw_ui {
                    self.ui.draw(ctx, pass, batch);
                }
            }
        })?;

        self.graph.end(ctx)?;

        log::trace!("End render");

        Ok(())
    }

    pub fn input(&mut self, ctx: &Context, input: Input) {
        log::trace!("Input");

        self.ui.input(input);

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => self.graph.render_scaling = (self.graph.render_scaling + 0.1).min(1.),
                Key::A => self.graph.render_scaling = (self.graph.render_scaling - 0.1).max(0.1),
                Key::L => log::info!("{:?}", ctx.allocator.allocator.lock().unwrap()),
                Key::P => self.process_updates = !self.process_updates,
                Key::W => self.draw_wire = !self.draw_wire,
                Key::U => self.draw_ui = !self.draw_ui,
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
