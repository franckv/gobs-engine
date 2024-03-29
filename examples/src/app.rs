use std::{collections::VecDeque, sync::Arc};

use slotmap::Key as _;

use gobs::{
    core::{entity::camera::Camera, Transform},
    game::input::{Input, Key},
    render::{
        context::Context,
        geometry::VertexFlag,
        graph::{FrameGraph, RenderError},
        material::{Material, MaterialProperty},
        pass::PassType,
        renderable::Renderable,
        BlendMode,
    },
    scene::{
        graph::{node::NodeValue, scenegraph::SceneGraph},
        scene::Scene,
    },
    ui::UIRenderer,
};

use crate::CameraController;

pub struct SampleApp {
    pub process_updates: bool,
    pub draw_ui: bool,
    pub draw_bounds: bool,
    pub draw_wire: bool,
    pub show_camera: bool,
    pub show_light: bool,
    pub show_models: bool,
}

impl SampleApp {
    pub fn new() -> Self {
        log::info!("Create");

        Self {
            process_updates: false,
            draw_ui: false,
            draw_bounds: false,
            draw_wire: false,
            show_camera: true,
            show_light: true,
            show_models: true,
        }
    }

    pub fn ortho_camera(ctx: &Context) -> Camera {
        let extent = ctx.surface.get_extent(ctx.device.clone());

        Camera::ortho(extent.width as f32, extent.height as f32, 0.1, 100., 0., 0.)
    }

    pub fn perspective_camera(ctx: &Context) -> Camera {
        let extent = ctx.surface.get_extent(ctx.device.clone());

        Camera::perspective(
            extent.width as f32 / extent.height as f32,
            (60. as f32).to_radians(),
            0.1,
            100.,
            0.,
            0.,
        )
    }

    pub fn controller() -> CameraController {
        CameraController::new(3., 0.4)
    }

    pub fn color_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn color_material_transparent(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .blend_mode(BlendMode::Alpha)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn texture_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn texture_material_transparent(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .blend_mode(BlendMode::Alpha)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn normal_mapping_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder("mesh.vert.spv", "mesh_n.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn depth_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder("color.vert.spv", "depth.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx, graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn update_ui<F>(
        &mut self,
        ctx: &Context,
        graph: &FrameGraph,
        scene: &Scene,
        ui: &mut UIRenderer,
        delta: f32,
        mut f: F,
    ) where
        F: FnMut(&mut egui::Ui),
    {
        if self.draw_ui {
            let (camera_transform, camera) = scene.camera();
            ui.update(
                ctx,
                graph.pass_by_type(PassType::Ui).unwrap(),
                delta,
                |ectx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(ectx, |ui| {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                            ectx.style_mut(|s| {
                                for (_, id) in s.text_styles.iter_mut() {
                                    id.size = 16.;
                                }
                            });
                            ui.heading(&ctx.app_name);
                            ui.separator();
                            self.show_fps(ui, graph.render_stats().fps);
                            self.show_stats(ui, "Render Stats", graph);
                            self.show_camera(ui, camera, &camera_transform);
                            self.show_memory(ui, ctx);
                            self.show_scene(ui, &scene.graph);
                            f(ui);
                        });
                },
            );
        }
    }

    fn show_fps(&self, ui: &mut egui::Ui, fps: u32) {
        ui.label(format!("FPS: {}", fps));
    }

    fn show_stats(&self, ui: &mut egui::Ui, header: &str, graph: &FrameGraph) {
        let stats = graph.render_stats();
        ui.collapsing(header, |ui| {
            for pass in &graph.passes {
                ui.collapsing(format!("Pass: {}", pass.name()), |ui| {
                    if let Some(pass_stats) = stats.pass_stats.get(&pass.id()) {
                        ui.label(format!("  Vertices: {}", pass_stats.vertices));
                        ui.label(format!("  Indices: {}", pass_stats.indices));
                        ui.label(format!("  Models: {}", pass_stats.models));
                        ui.label(format!("  Instances: {}", pass_stats.instances));
                        ui.label(format!("  Textures: {}", pass_stats.textures));
                    }
                });
            }
            ui.label("Performance");
            ui.label(format!("  Draws: {}", stats.draws));
            ui.label(format!("  Binds: {}", stats.binds));
            ui.label(format!(
                "  CPU draw time: {:.2}ms",
                1000. * stats.cpu_draw_time
            ));
            ui.collapsing("Details", |ui| {
                ui.label(format!("  Update: {:.2}ms", 1000. * stats.cpu_draw_update));
                ui.label(format!("  Push: {:.2}ms", 1000. * stats.cpu_draw_push));
                ui.label(format!("  Bind: {:.2}ms", 1000. * stats.cpu_draw_bind));
                ui.label(format!("  Submit: {:.2}ms", 1000. * stats.cpu_draw_submit));
            });
            ui.label(format!("  GPU time: {:.2}ms", 1000. * stats.gpu_draw_time));
            ui.label(format!("  Update time: {:.2}ms", 1000. * stats.update_time));
        });
    }

    fn show_camera(&self, ui: &mut egui::Ui, camera: &Camera, camera_transform: &Transform) {
        ui.collapsing("Camera", |ui| {
            let translation = camera_transform.translation();
            ui.label(format!(
                "  Position: [{:.2}, {:.2}, {:.2}]",
                translation.x, translation.y, translation.z
            ));
            let dir = camera.dir();
            ui.label(format!(
                "  Direction: [{:.2}, {:.2}, {:.2}]",
                dir.x, dir.y, dir.z
            ));
            ui.label(format!("  Yaw: {:.1}°", camera.yaw.to_degrees()));
            ui.label(format!("  Pitch: {:.1}°", camera.pitch.to_degrees()));
            ui.label(format!("  Transform: {:?}", camera_transform));
        });
    }

    fn show_memory(&self, ui: &mut egui::Ui, ctx: &Context) {
        ui.collapsing("Memory", |ui| {
            ui.label(format!("{:?}", ctx.allocator.allocator.lock().unwrap()));
        });
    }

    fn show_scene(&mut self, ui: &mut egui::Ui, graph: &SceneGraph) {
        let mut nodes = VecDeque::from([(0, graph.root)]);

        ui.collapsing("Scene", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_camera, "Camera");
                ui.checkbox(&mut self.show_light, "Light");
                ui.checkbox(&mut self.show_models, "Models");
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                while !nodes.is_empty() {
                    let (d, node_key) = nodes.pop_front().unwrap();
                    let node = graph.get(node_key).unwrap();
                    match node.value {
                        NodeValue::Model(_) => {
                            if !self.show_models {
                                continue;
                            }
                        }
                        NodeValue::Camera(_) => {
                            if !self.show_camera {
                                continue;
                            }
                        }
                        NodeValue::Light(_) => {
                            if !self.show_light {
                                continue;
                            }
                        }
                        _ => (),
                    }

                    let pad = 5 * d;
                    ui.collapsing(
                        format!(
                            "{:>pad$}[{:?}] Node: {:?} ({:?})",
                            "",
                            node_key.data(),
                            &node.value,
                            node.enabled,
                        ),
                        |ui| {
                            let pad = pad + 10;
                            ui.label(format!("{:>pad$}{:?}", "", node.bounding_box));
                            ui.label(format!("G: {:>pad$}{:?}", "", node.global_transform()));
                            ui.label(format!("P: {:>pad$}{:?}", "", node.parent_transform));
                            ui.label(format!("T: {:>pad$}{:?}", "", node.transform()));
                        },
                    );

                    for child in graph.get(node_key).unwrap().children.iter().rev() {
                        nodes.push_front((d + 1, *child));
                    }
                }
            });
        });
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        graph: &mut FrameGraph,
        scene: &mut Scene,
        ui: &mut UIRenderer,
    ) -> Result<(), RenderError> {
        log::trace!("Render frame {}", ctx.frame_number);

        graph.begin(ctx)?;

        graph.render(ctx, &mut |pass, batch| match pass.ty() {
            PassType::Compute => {}
            PassType::Depth | PassType::Forward => {
                scene.draw(ctx, pass, batch);
            }
            PassType::Wire => {
                if self.draw_wire {
                    scene.draw(ctx, pass, batch);
                }
            }
            PassType::Bounds => {
                if self.draw_bounds {
                    scene.draw_bounds(ctx, pass, batch);
                }
            }
            PassType::Ui => {
                if self.draw_ui {
                    ui.draw(ctx, pass, batch);
                }
            }
        })?;

        graph.end(ctx)?;

        log::trace!("End render");

        Ok(())
    }

    pub fn input(
        &mut self,
        ctx: &Context,
        input: Input,
        graph: &mut FrameGraph,
        scene: &mut Scene,
        ui: &mut UIRenderer,
        camera_controller: &mut CameraController,
    ) {
        log::trace!("Input");

        ui.input(input);

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => graph.render_scaling = (graph.render_scaling + 0.1).min(1.),
                Key::A => graph.render_scaling = (graph.render_scaling - 0.1).max(0.1),
                Key::L => log::info!("{:?}", ctx.allocator.allocator.lock().unwrap()),
                Key::P => self.process_updates = !self.process_updates,
                Key::W => self.draw_wire = !self.draw_wire,
                Key::B => self.draw_bounds = !self.draw_bounds,
                Key::U => self.draw_ui = !self.draw_ui,
                Key::Equals => scene.update_camera(|_, camera| {
                    camera.pitch = 0.;
                    camera.yaw = 0.;
                }),
                _ => camera_controller.key_pressed(key),
            },
            Input::KeyReleased(key) => camera_controller.key_released(key),
            Input::MousePressed => camera_controller.mouse_pressed(),
            Input::MouseReleased => camera_controller.mouse_released(),
            Input::MouseWheel(delta) => camera_controller.mouse_scroll(delta),
            Input::MouseMotion(dx, dy) => camera_controller.mouse_drag(dx, dy),
            _ => (),
        }
    }
}
