use crate::model::Model;
use crate::resources::{CameraResource, LightResource};

pub struct BatchItem<'a> {
    pub(crate) model: &'a Model,
    pub(crate) instances_buffer: Option<&'a wgpu::Buffer>,
    pub(crate) instances_count: usize,
}

pub struct BatchBuilder<'a> {
    camera_resource: Option<&'a CameraResource>,
    light_resource: Option<&'a LightResource>,
    items: Vec<BatchItem<'a>>,
}

impl<'a> BatchBuilder<'a> {
    pub fn new() -> Self {
        BatchBuilder {
            camera_resource: None,
            light_resource: None,
            items: Vec::new(),
        }
    }

    pub fn camera_resource(mut self, camera_resource: &'a CameraResource) -> Self {
        self.camera_resource = Some(camera_resource);

        self
    }

    pub fn light_resource(mut self, light_resource: &'a LightResource) -> Self {
        self.light_resource = Some(light_resource);

        self
    }

    pub fn draw(mut self, model: &'a Model) -> Self {
        let item = BatchItem {
            model,
            instances_buffer: None,
            instances_count: 0,
        };

        self.items.push(item);

        self
    }

    pub fn draw_indexed(
        mut self,
        model: &'a Model,
        instances_buffer: &'a wgpu::Buffer,
        instances_count: usize,
    ) -> Self {
        let item = BatchItem {
            model,
            instances_buffer: Some(instances_buffer),
            instances_count,
        };

        self.items.push(item);

        self
    }

    pub fn finish(self) -> Batch<'a> {
        Batch {
            camera_resource: self.camera_resource.unwrap(),
            light_resource: self.light_resource.unwrap(),
            items: self.items,
        }
    }
}

pub struct Batch<'a> {
    pub(crate) camera_resource: &'a CameraResource,
    pub(crate) light_resource: &'a LightResource,
    pub(crate) items: Vec<BatchItem<'a>>,
}

impl<'a> Batch<'a> {
    pub fn begin() -> BatchBuilder<'a> {
        BatchBuilder::new()
    }
}