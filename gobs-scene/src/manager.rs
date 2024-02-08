use std::{collections::HashMap, sync::Arc};

use gobs_render::{
    context::Context,
    geometry::{Model, ModelId},
};

use crate::resources::ModelResource;

pub struct ResourceManager<I, R> {
    resources: HashMap<I, R>,
}

impl<I, R> ResourceManager<I, R> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl ResourceManager<ModelId, Arc<ModelResource>> {
    pub fn add(&mut self, ctx: &Context, id: ModelId, model: Arc<Model>) {
        self.resources
            .entry(id)
            .or_insert_with(|| ModelResource::new(ctx, model));
    }

    pub fn get(&self, id: ModelId) -> Arc<ModelResource> {
        self.resources.get(&id).expect("Missing model").clone()
    }
}
