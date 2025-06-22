use gobs::{
    game::context::AppInfo,
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
    pub ui_hovered: bool,
    pub selected_node: NodeId,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            show_camera: true,
            show_light: true,
            show_models: true,
            ui_hovered: false,
            selected_node: NodeId::default(),
        }
    }

    pub fn draw(
        &mut self,
        app_info: &AppInfo,
        ectx: &egui::Context,
        scene: &mut Scene,
        delta: f32,
    ) {
        egui::SidePanel::left("left").show(ectx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ectx.style_mut(|s| {
                for (_, id) in s.text_styles.iter_mut() {
                    id.size = 16.;
                }
            });

            ui.heading(&app_info.name);

            self.draw_fps(ui, (1. / delta).round() as u32);

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

    pub fn draw_fps(&mut self, ui: &mut egui::Ui, fps: u32) {
        ui.strong(format!("FPS: {}", fps));
    }

    pub fn draw_graph(&mut self, ui: &mut egui::Ui, graph: &mut SceneGraph) {
        let old_selected = self.selected_node;

        ui.strong("Graph");
        egui::CollapsingHeader::new("scene")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_node(ui, graph, graph.root);
                });
            });

        if self.selected_node != old_selected {
            if let Some(node) = graph.get_mut(old_selected) {
                node.base.selected = false;
            }
            if let Some(node) = graph.get_mut(self.selected_node) {
                node.base.selected = true;
            }
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

        ui.strong("Camera");
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

            ui.strong("Properties");
            ui.label(node_name.to_string());
            ui.horizontal(|ui| {
                node.update_transform(|transform| {
                    let mut translation = transform.translation();
                    ui.label("x: ");
                    ui.add(egui::DragValue::new(&mut translation.x).speed(speed));
                    ui.label("y: ");
                    ui.add(egui::DragValue::new(&mut translation.y).speed(speed));
                    ui.label("z: ");
                    ui.add(egui::DragValue::new(&mut translation.z).speed(speed));
                    transform.set_translation(translation);
                });
            });
            ui.checkbox(&mut node.base.enabled, "enabled");
            ui.checkbox(&mut node.base.selected, "selected");
        });
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
