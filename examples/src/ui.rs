use glam::Vec3;

use gobs::{
    game::context::GameContext,
    render::{Material, Mesh, Texture},
    render_graph::{FrameGraph, Pipeline},
    render_low::FrameData,
    resource::{
        manager::ResourceManager,
        resource::{ResourceProperties, ResourceType},
    },
    scene::{
        components::{NodeId, NodeValue},
        graph::scenegraph::SceneGraph,
        scene::Scene,
    },
};

pub struct Ui {
    pub show_camera: bool,
    pub show_light: bool,
    pub show_models: bool,
    pub show_resources: bool,
    pub ui_hovered: bool,
    pub selected_node: NodeId,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            show_camera: true,
            show_light: true,
            show_models: true,
            show_resources: false,
            ui_hovered: false,
            selected_node: NodeId::default(),
        }
    }

    pub fn draw(
        &mut self,
        ectx: &egui::Context,
        ctx: &mut GameContext,
        scene: &mut Scene,
        delta: f32,
    ) {
        egui::SidePanel::left("left").show(ectx, |ui| {
            ectx.style_mut(|s| {
                for (_, id) in s.text_styles.iter_mut() {
                    id.size = 16.;
                }
            });

            ui.heading(&ctx.app_info.name);

            self.show_resources(ectx, ui, &mut ctx.resource_manager);

            self.draw_general(ui, scene, (1. / delta).round() as u32);

            ui.separator();

            self.draw_frame(ui, &ctx.renderer.graph, ctx.renderer.frame());

            ui.separator();

            self.draw_graph(ui, &mut scene.graph);

            ui.separator();

            self.draw_camera(ui, scene);

            ui.separator();

            self.draw_properties(ui, &mut scene.graph);

            ui.separator();
        });

        self.ui_hovered = ectx.wants_pointer_input();
    }

    pub fn show_resources(
        &mut self,
        ectx: &egui::Context,
        ui: &mut egui::Ui,
        resource_manager: &mut ResourceManager,
    ) {
        if ui.button("Show resources").clicked() {
            self.show_resources = true;
        }

        let mut show_resources = self.show_resources;

        egui::Window::new("Resources")
            .open(&mut show_resources)
            .show(ectx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_resource::<Texture>(ui, "Textures", resource_manager);
                    self.show_resource::<Pipeline>(ui, "Pipelines", resource_manager);
                    self.show_resource::<Material>(ui, "Materials", resource_manager);
                    self.show_resource::<Mesh>(ui, "Meshes", resource_manager);
                });
            });

        self.show_resources = show_resources;
    }

    fn show_resource<R: ResourceType + 'static>(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        resource_manager: &mut ResourceManager,
    ) {
        egui::CollapsingHeader::new(label)
            .default_open(true)
            .show(ui, |ui| {
                for resource in resource_manager.values::<R>() {
                    ui.label(format!(" {:?}", &resource.properties.name()));
                }
            });
    }

    pub fn draw_general(&mut self, ui: &mut egui::Ui, scene: &mut Scene, fps: u32) {
        egui::CollapsingHeader::new("Settings")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("FPS: {fps}"));
                ui.horizontal(|ui| {
                    ui.label("Screen");
                    ui.add(egui::Button::new(format!("{}", scene.width)));
                    ui.add(egui::Button::new(format!("{}", scene.height)));
                });

                let (camera_transform, camera) = scene.camera();

                let (mut x, mut y) = (0., 0.);
                ui.input(|input| {
                    if let Some(pos) = input.pointer.latest_pos() {
                        x = pos.x;
                        y = pos.y;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Mouse");
                    ui.add(egui::Button::new(format!("{x}")));
                    ui.add(egui::Button::new(format!("{y}")));
                });

                let pos = Vec3::new(x, y, 0.);
                let ndc = camera.screen_to_ndc(pos, scene.width, scene.height);
                ui.horizontal(|ui| {
                    ui.label("NDC");
                    ui.add(egui::Button::new(format!("{:.2}", ndc.x)));
                    ui.add(egui::Button::new(format!("{:.2}", ndc.y)));
                    ui.add(egui::Button::new(format!("{:.2}", ndc.z)));
                });

                let pos_world =
                    camera.screen_to_world(pos, camera_transform, scene.width, scene.height);
                ui.horizontal(|ui| {
                    ui.label("World");
                    ui.add(egui::Button::new(format!("{:.2}", pos_world.x)));
                    ui.add(egui::Button::new(format!("{:.2}", pos_world.y)));
                    ui.add(egui::Button::new(format!("{:.2}", pos_world.z)));
                    ui.add(egui::Button::new(format!("{:.2}", pos_world.w)));
                });
            });
    }

    pub fn draw_graph(&mut self, ui: &mut egui::Ui, graph: &mut SceneGraph) {
        let old_selected = self.selected_node;
        egui::CollapsingHeader::new("Graph")
            .default_open(true)
            .show(ui, |ui| {
                egui::CollapsingHeader::new("scene")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            self.draw_node(ui, graph, graph.root);
                        });
                    });
            });

        if self.selected_node != old_selected {
            graph.set_selected(old_selected, false);
            graph.set_selected(self.selected_node, true);
        }
    }

    fn draw_node(&mut self, ui: &mut egui::Ui, graph: &SceneGraph, node_key: NodeId) {
        let node = graph.get(node_key).unwrap();
        let node_name = match &node.base.value {
            NodeValue::None => "None",
            NodeValue::Model(_model) => "Model",
            NodeValue::Camera(_camera) => "Camera",
            NodeValue::Light(_light) => "Light",
        };
        let has_children = !node.base.children.is_empty();

        if has_children {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(node.base.id),
                true,
            )
            .show_header(ui, |ui| {
                ui.selectable_value(&mut self.selected_node, node.base.id, node_name.to_string());
            })
            .body(|ui| {
                for child in &graph.get(node_key).unwrap().base.children {
                    self.draw_node(ui, graph, *child);
                }
            });
        } else {
            ui.selectable_value(&mut self.selected_node, node.base.id, node_name.to_string());
        }
    }

    pub fn draw_camera(&mut self, ui: &mut egui::Ui, scene: &Scene) {
        let (camera_transform, camera) = scene.camera();

        let mut translation = camera_transform.translation();
        let mut yaw = camera.yaw.to_degrees();
        let mut pitch = camera.pitch.to_degrees();

        egui::CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Position");
                ui.horizontal(|ui| {
                    ui.label("x: ");
                    ui.add(egui::DragValue::new(&mut translation.x));
                    ui.label("y: ");
                    ui.add(egui::DragValue::new(&mut translation.y));
                    ui.label("z: ");
                    ui.add(egui::DragValue::new(&mut translation.z));
                });
                ui.label("Orientation");
                ui.horizontal(|ui| {
                    ui.label("pitch: ");
                    ui.add(egui::DragValue::new(&mut pitch));
                    ui.label("yaw: ");
                    ui.add(egui::DragValue::new(&mut yaw));
                });
            });
    }

    pub fn draw_properties(&mut self, ui: &mut egui::Ui, graph: &mut SceneGraph) {
        let speed = 0.05;

        graph.update(self.selected_node, |node| {
            let node_name = match &node.base.value {
                NodeValue::None => "None",
                NodeValue::Model(_model) => "Model",
                NodeValue::Camera(_camera) => "Camera",
                NodeValue::Light(_light) => "Light",
            };

            egui::CollapsingHeader::new("Properties")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(node_name.to_string());

                    egui::CollapsingHeader::new("Local")
                        .default_open(true)
                        .show(ui, |ui| {
                            node.update_transform(|transform| {
                                ui.horizontal(|ui| {
                                    let mut translation = transform.translation();
                                    ui.label("Translation");
                                    ui.label("x: ");
                                    ui.add(egui::DragValue::new(&mut translation.x).speed(speed));
                                    ui.label("y: ");
                                    ui.add(egui::DragValue::new(&mut translation.y).speed(speed));
                                    ui.label("z: ");
                                    ui.add(egui::DragValue::new(&mut translation.z).speed(speed));
                                    transform.set_translation(translation);
                                });

                                ui.horizontal(|ui| {
                                    let mut rotation = transform.rotation();
                                    ui.label("Rotation     ");
                                    ui.label("x: ");
                                    ui.add(egui::DragValue::new(&mut rotation.x).speed(speed));
                                    ui.label("y: ");
                                    ui.add(egui::DragValue::new(&mut rotation.y).speed(speed));
                                    ui.label("z: ");
                                    ui.add(egui::DragValue::new(&mut rotation.z).speed(speed));
                                    ui.label("w: ");
                                    ui.add(egui::DragValue::new(&mut rotation.w).speed(speed));
                                    transform.set_rotation(rotation);
                                });

                                ui.horizontal(|ui| {
                                    let mut scaling = transform.scaling();
                                    ui.label("Scaling        ");
                                    ui.label("x: ");
                                    ui.add(egui::DragValue::new(&mut scaling.x).speed(speed));
                                    ui.label("y: ");
                                    ui.add(egui::DragValue::new(&mut scaling.y).speed(speed));
                                    ui.label("z: ");
                                    ui.add(egui::DragValue::new(&mut scaling.z).speed(speed));
                                    transform.set_scaling(scaling);
                                });

                                true
                            });

                            ui.label("");
                        });

                    egui::CollapsingHeader::new("Global")
                        .default_open(true)
                        .show(ui, |ui| {
                            let global = node.global_transform();
                            ui.horizontal(|ui| {
                                let mut translation = global.translation();
                                ui.label("Translation");
                                ui.label("x: ");
                                ui.add(egui::DragValue::new(&mut translation.x).speed(speed));
                                ui.label("y: ");
                                ui.add(egui::DragValue::new(&mut translation.y).speed(speed));
                                ui.label("z: ");
                                ui.add(egui::DragValue::new(&mut translation.z).speed(speed));
                            });
                            ui.horizontal(|ui| {
                                let mut rotation = global.rotation();
                                ui.label("Rotation     ");
                                ui.label("x: ");
                                ui.add(egui::DragValue::new(&mut rotation.x).speed(speed));
                                ui.label("y: ");
                                ui.add(egui::DragValue::new(&mut rotation.y).speed(speed));
                                ui.label("z: ");
                                ui.add(egui::DragValue::new(&mut rotation.z).speed(speed));
                                ui.label("w: ");
                                ui.add(egui::DragValue::new(&mut rotation.w).speed(speed));
                            });
                            ui.horizontal(|ui| {
                                let mut scaling = global.scaling();
                                ui.label("Scaling        ");
                                ui.label("x: ");
                                ui.add(egui::DragValue::new(&mut scaling.x).speed(speed));
                                ui.label("y: ");
                                ui.add(egui::DragValue::new(&mut scaling.y).speed(speed));
                                ui.label("z: ");
                                ui.add(egui::DragValue::new(&mut scaling.z).speed(speed));
                            });

                            ui.label("");
                        });

                    egui::CollapsingHeader::new("AABB")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("x min: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.x_min)
                                        .speed(speed),
                                );
                                ui.label("y min: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.y_min)
                                        .speed(speed),
                                );
                                ui.label("z min: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.z_min)
                                        .speed(speed),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("x max: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.x_max)
                                        .speed(speed),
                                );
                                ui.label("y max: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.y_max)
                                        .speed(speed),
                                );
                                ui.label("z max: ");
                                ui.add(
                                    egui::DragValue::new(&mut node.bounding.bounding_box.z_max)
                                        .speed(speed),
                                );
                            });
                            ui.label("");
                        });

                    ui.checkbox(&mut node.base.enabled, "enabled");
                    ui.checkbox(&mut node.base.selected, "selected");
                });

            true
        });
    }

    fn draw_frame(&self, ui: &mut egui::Ui, graph: &FrameGraph, frame: &FrameData) {
        egui::CollapsingHeader::new("Stats")
            .default_open(true)
            .show(ui, |ui| {
                egui::CollapsingHeader::new("Global")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "Prepare time: {:.2} ms",
                            1000. * frame.stats.cpu_prepare_time
                        ));
                        ui.label(format!("Objects: {}", frame.stats.objects));

                        let mut pipeline_binds = 0;
                        let mut resource_binds = 0;
                        let mut draws = 0;
                        let mut cpu_draw_time = 0.;
                        for pass in &graph.passes {
                            if let Some(stats) = frame.stats.pass(pass.id()) {
                                pipeline_binds += stats.pipeline_binds;
                                resource_binds += stats.resource_binds;
                                draws += stats.draws;
                                cpu_draw_time += stats.cpu_draw_time;
                            }
                        }
                        ui.label(format!("Pipeline binds: {}", pipeline_binds));
                        ui.label(format!("Resource binds: {}", resource_binds));
                        ui.label(format!("Draws: {}", draws));
                        ui.label(format!("CPU time: {:.2} ms", 1000. * cpu_draw_time));
                    });

                for pass in &graph.passes {
                    if let Some(stats) = frame.stats.pass(pass.id()) {
                        egui::CollapsingHeader::new(format!("Pass: {}", pass.name()))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(format!("Pipeline binds: {}", stats.pipeline_binds));
                                ui.label(format!("Resource binds: {}", stats.resource_binds));
                                ui.label(format!("Draws: {}", stats.draws));
                                ui.label(format!("Indices: {}", stats.indices));
                                ui.label(format!(
                                    "CPU time: {:.2} ms",
                                    1000. * stats.cpu_draw_time
                                ));
                            });
                    }
                }
            });
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
