use egui::{ColorImage, TextureHandle};
use egui_extras::{Column, TableBody, TableBuilder};
use glam::Vec3;

use gobs::{
    core::Transform,
    game::context::GameContext,
    render::{Material, MaterialInstance, Mesh, RenderBatch, Texture, TextureLoader},
    render_graph::{FrameGraph, Pipeline},
    render_low::FrameData,
    resource::{
        manager::ResourceManager,
        resource::{ResourceHandle, ResourceProperties, ResourceType},
    },
    scene::{
        components::{NodeId, NodeValue},
        graph::{node::Node, scenegraph::SceneGraph},
        scene::Scene,
    },
};

pub struct Ui {
    pub show_camera: bool,
    pub show_light: bool,
    pub show_models: bool,
    pub show_resources: bool,
    pub show_batch: bool,
    pub show_texture: bool,
    pub ui_hovered: bool,
    pub texture_view: Option<TextureHandle>,
    pub selected_node: NodeId,
    pub selected_texture: Option<ResourceHandle<Texture>>,
    pub selected_mesh: (
        Option<ResourceHandle<Mesh>>,
        Option<ResourceHandle<MaterialInstance>>,
    ),
}

impl Ui {
    pub fn new() -> Self {
        Self {
            show_camera: true,
            show_light: true,
            show_models: true,
            show_resources: false,
            show_batch: false,
            show_texture: false,
            ui_hovered: false,
            texture_view: None,
            selected_node: NodeId::default(),
            selected_texture: None,
            selected_mesh: (None, None),
        }
    }

    pub fn draw(
        &mut self,
        ectx: &egui::Context,
        ctx: &mut GameContext,
        scene: &mut Scene,
        delta: f32,
    ) {
        ectx.style_mut(|s| {
            for (_, id) in s.text_styles.iter_mut() {
                id.size = 14.;
            }
        });

        egui::SidePanel::left("left").show(ectx, |ui| {
            ui.heading(&ctx.app_info.name);

            self.show_resources(ectx, ui, &ctx.resource_manager);

            self.show_batch(ectx, ui, &ctx.renderer.batch);

            self.draw_general(
                ui,
                scene,
                ctx.renderer.frame_number,
                (1. / delta).round() as u32,
            );

            self.show_texture(ectx, &mut ctx.resource_manager);

            ui.separator();

            self.draw_frame(ui, &ctx.renderer.graph, ctx.renderer.frame());

            ui.separator();

            self.draw_graph(ui, &mut scene.graph);

            ui.separator();

            self.draw_camera(ui, scene);

            ui.separator();
        });

        if self.selected_node != NodeId::default() {
            egui::SidePanel::right("right").show(ectx, |ui| {
                self.draw_properties(ui, &mut scene.graph, &ctx.resource_manager);
                ui.separator();
            });
        }

        self.ui_hovered = ectx.wants_pointer_input();
    }

    pub fn show_resources(
        &mut self,
        ectx: &egui::Context,
        ui: &mut egui::Ui,
        resource_manager: &ResourceManager,
    ) {
        if ui.button("Show resources").clicked() {
            self.show_resources = true;
        }

        let mut show_resources = self.show_resources;

        egui::Window::new("Resources")
            .open(&mut show_resources)
            .show(ectx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .auto_shrink(false)
                        .columns(Column::auto(), 4)
                        .column(Column::remainder())
                        .header(10., |mut headers| {
                            headers.col(|ui| {
                                ui.heading("resource type");
                            });
                            headers.col(|ui| {
                                ui.heading("resource name");
                            });
                            headers.col(|ui| {
                                ui.heading("resource id");
                            });
                            headers.col(|ui| {
                                ui.heading("lifetime");
                            });
                            headers.col(|ui| {
                                ui.heading("life");
                            });
                        })
                        .body(|mut body| {
                            self.show_resource::<Texture>(&mut body, "Textures", resource_manager);
                            self.show_resource::<Pipeline>(
                                &mut body,
                                "Pipelines",
                                resource_manager,
                            );
                            self.show_resource::<Material>(
                                &mut body,
                                "Materials",
                                resource_manager,
                            );
                            self.show_resource::<MaterialInstance>(
                                &mut body,
                                "Material instances",
                                resource_manager,
                            );
                            self.show_resource::<Mesh>(&mut body, "Meshes", resource_manager);
                        });
                });
            });

        self.show_resources = show_resources;
    }

    pub fn show_batch(&mut self, ectx: &egui::Context, ui: &mut egui::Ui, batch: &RenderBatch) {
        if ui.button("Show batch").clicked() {
            self.show_batch = true;
        }

        let mut show_batch = self.show_batch;

        egui::Window::new("Batch")
            .open(&mut show_batch)
            .show(ectx, |ui| {
                ui.label(format!("Count: {}", batch.render_list.len()));

                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .auto_shrink(false)
                    .columns(Column::auto(), 5)
                    .column(Column::remainder())
                    .header(10., |mut headers| {
                        headers.col(|ui| {
                            ui.heading("model id");
                        });
                        headers.col(|ui| {
                            ui.heading("transparent");
                        });
                        headers.col(|ui| {
                            ui.heading("pipeline");
                        });
                        headers.col(|ui| {
                            ui.heading("material");
                        });
                        headers.col(|ui| {
                            ui.heading("pass id");
                        });
                        headers.col(|ui| {
                            ui.heading("layer");
                        });
                    })
                    .body(|mut body| {
                        for object in &batch.render_list {
                            body.row(10., |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.model_id));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.is_transparent()));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.pipeline_id()));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.material_instance_id));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.pass_id));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:?}", object.layer));
                                });
                            });
                        }
                    });
            });

        self.show_batch = show_batch;
    }

    fn show_resource<R: ResourceType + 'static>(
        &mut self,
        body: &mut TableBody,
        label: &str,
        resource_manager: &ResourceManager,
    ) {
        for resource in resource_manager.values::<R>() {
            body.row(10., |mut row| {
                row.col(|ui| {
                    ui.label(format!("{:?}", label));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", resource.properties.name()));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", resource.handle.id));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", resource.lifetime));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", resource.life));
                });
            });
        }
    }

    pub fn draw_general(&mut self, ui: &mut egui::Ui, scene: &mut Scene, frame: usize, fps: u32) {
        egui::CollapsingHeader::new("Settings")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("Frame: {frame}"));
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
            .default_open(false)
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
            self.selected_mesh = (None, None);
            self.selected_texture = None;
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

    const SPEED: f64 = 0.05;

    fn draw_transform(&mut self, ui: &mut egui::Ui, transform: &mut Transform, update: bool) {
        ui.horizontal(|ui| {
            let mut translation = transform.translation();
            ui.label("Translation");
            ui.label("x: ");
            ui.add(egui::DragValue::new(&mut translation.x).speed(Self::SPEED));
            ui.label("y: ");
            ui.add(egui::DragValue::new(&mut translation.y).speed(Self::SPEED));
            ui.label("z: ");
            ui.add(egui::DragValue::new(&mut translation.z).speed(Self::SPEED));
            if update {
                transform.set_translation(translation);
            }
        });

        ui.horizontal(|ui| {
            let mut rotation = transform.rotation();
            ui.label("Rotation     ");
            ui.label("x: ");
            ui.add(egui::DragValue::new(&mut rotation.x).speed(Self::SPEED));
            ui.label("y: ");
            ui.add(egui::DragValue::new(&mut rotation.y).speed(Self::SPEED));
            ui.label("z: ");
            ui.add(egui::DragValue::new(&mut rotation.z).speed(Self::SPEED));
            ui.label("w: ");
            ui.add(egui::DragValue::new(&mut rotation.w).speed(Self::SPEED));
            if update {
                transform.set_rotation(rotation);
            }
        });

        ui.horizontal(|ui| {
            let mut scaling = transform.scaling();
            ui.label("Scaling        ");
            ui.label("x: ");
            ui.add(egui::DragValue::new(&mut scaling.x).speed(Self::SPEED));
            ui.label("y: ");
            ui.add(egui::DragValue::new(&mut scaling.y).speed(Self::SPEED));
            ui.label("z: ");
            ui.add(egui::DragValue::new(&mut scaling.z).speed(Self::SPEED));
            if update {
                transform.set_scaling(scaling);
            }
        });
    }

    fn draw_local_properties(&mut self, ui: &mut egui::Ui, node: &mut Node) {
        egui::CollapsingHeader::new("Local")
            .default_open(true)
            .show(ui, |ui| {
                node.update_transform(|transform| {
                    self.draw_transform(ui, transform, true);

                    true
                });

                ui.label("");
            });
    }

    fn draw_global_properties(&mut self, ui: &mut egui::Ui, node: &mut Node) {
        egui::CollapsingHeader::new("Global")
            .default_open(true)
            .show(ui, |ui| {
                let global = node.global_transform();
                self.draw_transform(ui, &mut global.clone(), false);

                ui.label("");
            });
    }

    fn draw_aabb(&mut self, ui: &mut egui::Ui, node: &mut Node) {
        egui::CollapsingHeader::new("AABB")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("x min: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.x_min)
                            .speed(Self::SPEED),
                    );
                    ui.label("y min: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.y_min)
                            .speed(Self::SPEED),
                    );
                    ui.label("z min: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.z_min)
                            .speed(Self::SPEED),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("x max: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.x_max)
                            .speed(Self::SPEED),
                    );
                    ui.label("y max: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.y_max)
                            .speed(Self::SPEED),
                    );
                    ui.label("z max: ");
                    ui.add(
                        egui::DragValue::new(&mut node.bounding.bounding_box.z_max)
                            .speed(Self::SPEED),
                    );
                });
                ui.label("");
            });
    }

    fn draw_mesh(&mut self, ui: &mut egui::Ui, resource_manager: &ResourceManager) {
        let (mesh, _) = &self.selected_mesh;

        if let Some(mesh) = mesh {
            let mesh_props = &resource_manager.get(mesh).properties;
            egui::CollapsingHeader::new("Mesh")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Name: {}", mesh_props.name()));
                    ui.label(format!("Id: {}", mesh.id));
                });
        }
    }

    fn show_texture(&mut self, ectx: &egui::Context, resource_manager: &mut ResourceManager) {
        let mut show_texture = self.show_texture;

        egui::Window::new("Texture")
            .open(&mut show_texture)
            .show(ectx, |ui| {
                if let Some(texture) = self.selected_texture {
                    let texture_properties =
                        resource_manager.get_data(&texture, ()).unwrap().properties;

                    if self.texture_view.is_none() {
                        let mut format = texture_properties.format.clone();
                        let size = format.extent.into();
                        TextureLoader::get_bytes(&texture_properties.path, &mut format, |data| {
                            let img = ColorImage::from_rgba_unmultiplied(size, data);
                            let texture =
                                ectx.load_texture("texture-view", img, Default::default());
                            self.texture_view = Some(texture);
                        });
                    };
                    let texture = self.texture_view.as_ref().unwrap();
                    ui.image((texture.id(), texture.size_vec2()));
                }
            });

        self.show_texture = show_texture;
    }

    fn draw_material(&mut self, ui: &mut egui::Ui, resource_manager: &ResourceManager) {
        let (_, material) = &self.selected_mesh;
        if let Some(material) = material {
            let mat_instance_props = &resource_manager.get(material).properties;
            let mat_props = &resource_manager
                .get(&mat_instance_props.material)
                .properties;

            egui::CollapsingHeader::new("Material")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Material instance: {}", mat_instance_props.name(),));
                    ui.label(format!("  Id: {}", material.id));
                    ui.separator();
                    ui.label(format!("Material: {}", mat_props.name(),));
                    ui.label(format!("  Id: {}", mat_instance_props.material.id));
                    ui.separator();
                    ui.label(format!("Transparent: {}", mat_props.blending_enabled));
                    ui.separator();
                    if !mat_instance_props.textures.is_empty() {
                        ui.label("Textures:");
                    }
                    for texture in &mat_instance_props.textures {
                        let texture_props = &resource_manager.get(texture).properties;

                        ui.label(format!("  Name: {}", texture_props.name(),));
                        ui.label(format!("  Id: {}", texture.id));
                        if ui.button("").clicked() {
                            self.show_texture = !self.show_texture;
                            self.selected_texture = Some(*texture);
                            self.texture_view = None;
                        }
                    }
                });
        }
    }

    fn draw_model(
        &mut self,
        ui: &mut egui::Ui,
        node: &mut Node,
        resource_manager: &ResourceManager,
    ) {
        if let NodeValue::Model(model) = &node.base.value {
            egui::CollapsingHeader::new("Model")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Name: {}", model.name()));
                    ui.label(format!("Id: {}", model.id));
                    ui.label(format!("# Meshes: {}", model.meshes.len()));
                    for (mesh, material) in &model.meshes {
                        ui.selectable_value(
                            &mut self.selected_mesh,
                            (Some(*mesh), *material),
                            mesh.id.to_string(),
                        );
                    }
                    ui.label("");

                    if self.selected_mesh.0.is_some() {
                        self.draw_mesh(ui, resource_manager);
                        ui.label("");
                        self.draw_material(ui, resource_manager);
                        ui.label("");
                    }
                });
        }
    }

    fn draw_properties(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut SceneGraph,
        resource_manager: &ResourceManager,
    ) {
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
                    ui.label(format!("Type: {}", node_name));

                    self.draw_local_properties(ui, node);
                    self.draw_global_properties(ui, node);
                    self.draw_aabb(ui, node);
                    self.draw_model(ui, node, resource_manager);

                    ui.checkbox(&mut node.base.enabled, "enabled");
                    ui.checkbox(&mut node.base.selected, "selected");

                    if !node.base.selected {
                        self.selected_node = NodeId::default();
                    }
                });

            true
        });
    }

    fn draw_frame(&self, ui: &mut egui::Ui, graph: &FrameGraph, frame: &FrameData) {
        egui::CollapsingHeader::new("Stats")
            .default_open(false)
            .show(ui, |ui| {
                egui::CollapsingHeader::new("Global")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "Prepare time: {:.2} ms",
                            1000. * frame.stats.cpu_prepare_begin_time
                        ));
                        ui.label(format!(
                            "Prepare time: {:.2} ms",
                            1000. * frame.stats.cpu_prepare_end_time
                        ));
                        ui.label(format!("Objects: {}", frame.stats.objects));

                        let mut pipeline_binds = 0;
                        let mut material_resource_binds = 0;
                        let mut scene_resource_binds = 0;
                        let mut index_resource_binds = 0;
                        let mut attach_resource_binds = 0;
                        let mut draws = 0;
                        let mut cpu_draw_time = 0.;
                        let mut cpu_prepare_draw_time = 0.;
                        for pass in &graph.passes {
                            if let Some(stats) = frame.stats.pass(pass.id()) {
                                pipeline_binds += stats.pipeline_binds;
                                material_resource_binds += stats.material_resource_binds;
                                scene_resource_binds += stats.scene_resource_binds;
                                index_resource_binds += stats.index_resource_binds;
                                attach_resource_binds += stats.attach_resource_binds;
                                draws += stats.draws;
                                cpu_draw_time += stats.cpu_draw_time;
                                cpu_prepare_draw_time += stats.cpu_prepare_draw_time;
                            }
                        }
                        ui.label(format!("Pipeline binds: {}", pipeline_binds));
                        ui.label(format!(
                            "Resource binds (material): {}",
                            material_resource_binds
                        ));
                        ui.label(format!("Resource binds (scene): {}", scene_resource_binds));
                        ui.label(format!("Resource binds (index): {}", index_resource_binds));
                        ui.label(format!(
                            "Resource binds (attach): {}",
                            attach_resource_binds
                        ));
                        ui.label(format!("Draws: {}", draws));
                        ui.label(format!(
                            "Prepare time: {:.2} ms",
                            1000. * cpu_prepare_draw_time
                        ));
                        ui.label(format!("CPU time: {:.2} ms", 1000. * cpu_draw_time));
                    });

                for pass in &graph.passes {
                    if let Some(stats) = frame.stats.pass(pass.id()) {
                        egui::CollapsingHeader::new(format!("Pass: {}", pass.name()))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(format!("Pipeline binds: {}", stats.pipeline_binds));
                                ui.label(format!(
                                    "Resource binds (material): {}",
                                    stats.material_resource_binds
                                ));
                                ui.label(format!(
                                    "Resource binds (scene): {}",
                                    stats.scene_resource_binds
                                ));
                                ui.label(format!(
                                    "Resource binds (index): {}",
                                    stats.index_resource_binds
                                ));
                                ui.label(format!(
                                    "Resource binds (attach): {}",
                                    stats.attach_resource_binds
                                ));
                                ui.label(format!("Draws: {}", stats.draws));
                                ui.label(format!("Indices: {}", stats.indices));
                                ui.label(format!(
                                    "Prepare time: {:.2} ms",
                                    1000. * stats.cpu_prepare_draw_time
                                ));
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
