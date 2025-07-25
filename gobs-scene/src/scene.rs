use glam::{Vec3, Vec4};

use gobs_core::Transform;
use gobs_render::{GfxContext, RenderBatch, Renderable};
use gobs_render_graph::{PassType, RenderPass};
use gobs_resource::entity::{camera::Camera, light::Light};
use gobs_resource::manager::ResourceManager;
use gobs_resource::resource::ResourceError;

use crate::components::{NodeId, NodeValue};
use crate::graph::scenegraph::SceneGraph;

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: NodeId,
    pub light: NodeId,
    pub width: f32,
    pub height: f32,
}

impl Scene {
    pub fn new(
        ctx: &GfxContext,
        camera: Camera,
        camera_position: Vec3,
        light: Light,
        light_position: Vec3,
    ) -> Self {
        let mut graph = SceneGraph::new();

        let (width, height): (f32, f32) = ctx.extent().into();

        let camera = graph
            .insert(
                graph.root,
                NodeValue::Camera(camera),
                Transform::from_translation(camera_position),
            )
            .expect("Cannot insert camera");

        let light = graph
            .insert(
                graph.root,
                NodeValue::Light(light),
                Transform::from_translation(light_position),
            )
            .expect("Cannot insert light");

        Scene {
            graph,
            camera,
            light,
            width,
            height,
        }
    }

    pub fn update(&mut self, _ctx: &GfxContext, _delta: f32) {
        self.graph.update_nodes();
    }

    pub fn camera(&self) -> (Transform, &Camera) {
        if let Some(node) = self.graph.get(self.camera) {
            if let NodeValue::Camera(camera) = &node.base.value {
                return (*node.global_transform(), camera);
            }
        }

        unreachable!()
    }

    pub fn update_camera<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform, &mut Camera) -> bool,
    {
        self.graph.update(self.camera, |node| {
            if let NodeValue::Camera(ref mut camera) = node.base.value {
                f(&mut node.transform, camera)
            } else {
                false
            }
        });
    }

    pub fn light(&self) -> (Transform, &Light) {
        if let Some(node) = self.graph.get(self.light) {
            if let NodeValue::Light(light) = &node.base.value {
                return (*node.global_transform(), light);
            }
        }

        unreachable!();
    }

    pub fn update_light<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform, &mut Light) -> bool,
    {
        self.graph.update(self.light, |node| {
            if let NodeValue::Light(ref mut light) = node.base.value {
                f(&mut node.transform, light)
            } else {
                false
            }
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;

        self.update_camera(|_, camera| {
            camera.resize(width, height);
            false
        });
    }

    pub fn select_dir(&self, x: f32, y: f32) -> Vec4 {
        let (camera_transform, camera) = self.camera();

        // screen space
        let pos = Vec3::new(x, y, 0.);

        camera.screen_to_world(pos, camera_transform, self.width, self.height)
    }

    pub fn select_node(&self, _x: f32, _y: f32) -> Option<NodeId> {
        None
    }
}

impl Renderable for Scene {
    fn draw(
        &self,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        _transform: Option<Transform>,
    ) -> Result<(), ResourceError> {
        self.graph.visit(self.graph.root, &mut |node| {
            match pass.ty() {
                PassType::Bounds => {
                    if let NodeValue::Model(_) = node.base.value {
                        batch.add_bounds(node.bounding.bounding_box, pass.clone())?;
                    }
                }
                PassType::Depth | PassType::Forward | PassType::Wire => {
                    if let NodeValue::Model(model) = &node.base.value {
                        model.draw(
                            resource_manager,
                            pass.clone(),
                            batch,
                            Some(*node.global_transform()),
                        )?;
                    }
                }
                PassType::Select => {
                    if node.base.selected {
                        if let NodeValue::Model(model) = &node.base.value {
                            model.draw(
                                resource_manager,
                                pass.clone(),
                                batch,
                                Some(*node.global_transform()),
                            )?;
                        }
                    }
                }
                _ => {}
            }

            Ok(())
        })?;

        let (light_transform, light) = self.light();
        let (camera_transform, camera) = self.camera();
        batch.add_camera_data(camera, camera_transform, light, light_transform);

        Ok(())
    }
}
