use std::collections::VecDeque;

use gpu_allocator::vulkan::AllocatorVisualizer;
use slotmap::Key as _;

use gobs::{
    core::Transform,
    render::{context::Context, graph::FrameGraph},
    resource::entity::camera::Camera,
    scene::{components::NodeValue, graph::scenegraph::SceneGraph, scene::Scene},
};

pub struct Ui {
    pub show_camera: bool,
    pub show_light: bool,
    pub show_models: bool,
    pub memory_visualiser: AllocatorVisualizer,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            show_camera: true,
            show_light: true,
            show_models: true,
            memory_visualiser: AllocatorVisualizer::new(),
        }
    }

    pub fn draw(
        &mut self,
        ctx: &Context,
        ectx: &egui::Context,
        graph: &FrameGraph,
        scene: &Scene,
        camera: &Camera,
        camera_transform: &Transform,
    ) {
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
            });
    }

    pub fn show_fps(&self, ui: &mut egui::Ui, fps: u32) {
        ui.label(format!("FPS: {}", fps));
    }

    pub fn show_stats(&self, ui: &mut egui::Ui, header: &str, graph: &FrameGraph) {
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

    pub fn show_camera(&self, ui: &mut egui::Ui, camera: &Camera, camera_transform: &Transform) {
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

    pub fn show_memory(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.collapsing("Memory", |ui| {
            self.memory_visualiser
                .render_breakdown_ui(ui, &ctx.device.allocator.allocator.lock().unwrap());
        });
        ui.collapsing("Memory blocks", |ui| {
            self.memory_visualiser
                .render_memory_block_ui(ui, &ctx.device.allocator.allocator.lock().unwrap());
        });
    }

    pub fn show_scene(&mut self, ui: &mut egui::Ui, graph: &SceneGraph) {
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
                    match node.base.value {
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
                            &node.base.value,
                            node.base.enabled,
                        ),
                        |ui| {
                            let pad = pad + 10;
                            ui.label(format!("{:>pad$}{:?}", "", node.bounding.bounding_box));
                            ui.label(format!("G: {:>pad$}{:?}", "", node.global_transform()));
                            ui.label(format!("P: {:>pad$}{:?}", "", node.parent_transform));
                            ui.label(format!("T: {:>pad$}{:?}", "", node.transform()));
                        },
                    );

                    for child in graph.get(node_key).unwrap().base.children.iter().rev() {
                        nodes.push_front((d + 1, *child));
                    }
                }
            });
        });
    }
}
