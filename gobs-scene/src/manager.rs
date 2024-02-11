use std::{collections::HashMap, sync::Arc};

use gobs_render::{
    context::Context,
    geometry::{Model, ModelId},
    pass::{PassId, RenderPass},
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

impl ResourceManager<(ModelId, PassId), Arc<ModelResource>> {
    pub fn add(
        &mut self,
        ctx: &Context,
        id: ModelId,
        pass: Arc<dyn RenderPass>,
        model: Arc<Model>,
    ) {
        let key = (id, pass.id());
        self.resources
            .entry(key)
            .or_insert_with(|| ModelResource::new(ctx, model, pass));
    }

    pub fn get(&self, id: ModelId, pass_id: PassId) -> Arc<ModelResource> {
        let key = (id, pass_id);

        self.resources.get(&key).expect("Missing model").clone()
    }
}
