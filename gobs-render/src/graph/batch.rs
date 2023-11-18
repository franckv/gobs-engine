use std::sync::Arc;

use gobs_core as core;

use core::entity::camera::Camera;
use core::entity::instance::InstanceData;
use core::entity::light::Light;

use crate::model::Model;

pub struct BatchItem<'a> {
    pub(crate) model: Arc<Model>,
    pub(crate) instances: Option<&'a Vec<InstanceData>>,
}

pub struct BatchBuilder<'a> {
    camera: Option<&'a Camera>,
    light: Option<&'a Light>,
    items: Vec<BatchItem<'a>>,
}

impl<'a> BatchBuilder<'a> {
    pub fn new() -> Self {
        BatchBuilder {
            camera: None,
            light: None,
            items: Vec::new(),
        }
    }

    pub fn camera(mut self, camera: &'a Camera) -> Self {
        self.camera = Some(camera);

        self
    }

    pub fn light(mut self, light: &'a Light) -> Self {
        self.light = Some(light);

        self
    }

    pub fn draw_indexed(mut self, model: Arc<Model>, instances: &'a Vec<InstanceData>) -> Self {
        let item = BatchItem {
            model,
            instances: Some(instances),
        };

        self.items.push(item);

        self
    }

    pub fn finish(self) -> Batch<'a> {
        Batch {
            camera: self.camera.unwrap(),
            light: self.light.unwrap(),
            items: self.items,
        }
    }
}

impl<'a> Default for BatchBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Batch<'a> {
    pub(crate) camera: &'a Camera,
    pub(crate) light: &'a Light,
    pub(crate) items: Vec<BatchItem<'a>>,
}
