use glam::Vec3;

use gobs_core::Transform;
use gobs_render::{RenderBatch, Renderable};
use gobs_render_graph::{GfxContext, RenderPass};
use gobs_resource::entity::{camera::Camera, light::Light};
use gobs_resource::manager::ResourceManager;

use crate::components::{NodeId, NodeValue};
use crate::graph::scenegraph::SceneGraph;

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: NodeId,
    pub light: NodeId,
}

impl Scene {
    pub fn new(camera: Camera, camera_position: Vec3, light: Light, light_position: Vec3) -> Self {
        let mut graph = SceneGraph::new();

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
        }
    }

    pub fn update(&mut self, _ctx: &GfxContext, _delta: f32) {}

    pub fn camera(&self) -> (Transform, &Camera) {
        if let Some(node) = self.graph.get(self.camera) {
            if let NodeValue::Camera(camera) = &node.base.value {
                return (node.global_transform(), camera);
            }
        }

        unreachable!()
    }

    pub fn update_camera<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform, &mut Camera),
    {
        self.graph.update(self.camera, |node| {
            if let NodeValue::Camera(ref mut camera) = node.base.value {
                f(&mut node.transform, camera);
                node.base.updated = true;
            }
        });
    }

    pub fn light(&self) -> (Transform, &Light) {
        if let Some(node) = self.graph.get(self.light) {
            if let NodeValue::Light(light) = &node.base.value {
                return (node.global_transform(), light);
            }
        }

        unreachable!();
    }

    pub fn update_light<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform, &mut Light),
    {
        self.graph.update(self.light, |node| {
            if let NodeValue::Light(ref mut light) = node.base.value {
                f(&mut node.transform, light);
                node.base.updated = true;
            }
        });
    }

    pub fn draw_bounds(
        &mut self,
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
    ) {
        self.graph.visit(self.graph.root, &mut |node| {
            if let NodeValue::Model(_) = node.base.value {
                batch.add_bounds(
                    ctx,
                    resource_manager,
                    node.bounding.bounding_box,
                    node.global_transform(),
                    pass.clone(),
                );
            }
        });

        let (light_transform, light) = self.light();
        let (camera_transform, camera) = self.camera();
        batch.add_camera_data(camera, &camera_transform, light, &light_transform, pass);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.update_camera(|_, camera| {
            camera.resize(width, height);
        });
    }
}

impl Renderable for Scene {
    fn draw(
        &self,
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        _transform: Option<Transform>,
    ) {
        self.graph.visit(self.graph.root, &mut |node| {
            if let NodeValue::Model(model) = &node.base.value {
                model.draw(
                    ctx,
                    resource_manager,
                    pass.clone(),
                    batch,
                    Some(node.global_transform()),
                );
            }
        });

        let (light_transform, light) = self.light();
        let (camera_transform, camera) = self.camera();
        batch.add_camera_data(camera, &camera_transform, light, &light_transform, pass);
    }
}
