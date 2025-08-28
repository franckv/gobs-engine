use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform, logger};
use gobs_gfx::{BindingGroup, BindingGroupUpdates, Buffer, GfxBindingGroup};
use gobs_render_graph::RenderPass;
use gobs_render_low::{GfxContext, RenderObject, SceneData};
use gobs_resource::{
    entity::{camera::Camera, light::Light},
    geometry::{BoundingBox, MeshBuilder, MeshGeometry, Shapes},
    manager::ResourceManager,
    resource::{ResourceError, ResourceHandle, ResourceId, ResourceLifetime},
};

use crate::{MaterialInstance, model::Model};

pub struct RenderBatch {
    pub render_list: Vec<RenderObject>,
    vertex_padding: bool,
    bounding_geometry: Option<MeshBuilder>,
    bounding_pass: Option<RenderPass>,
    pub(crate) camera: Camera,
    pub(crate) camera_transform: Transform,
    pub(crate) lights: Vec<(Light, Transform)>,
    pub(crate) extent: ImageExtent2D,
}

impl RenderBatch {
    pub fn new(ctx: &GfxContext) -> Self {
        Self {
            render_list: Vec::new(),
            vertex_padding: ctx.vertex_padding,
            bounding_geometry: None,
            bounding_pass: None,
            camera: Camera::default(),
            camera_transform: Transform::default(),
            lights: vec![],
            extent: ImageExtent2D::default(),
        }
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn reset(&mut self) {
        self.render_list.clear();
        self.bounding_geometry = None;
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    fn get_bind_groups(
        material_instance: Option<&ResourceHandle<MaterialInstance>>,
        resource_manager: &mut ResourceManager,
    ) -> Vec<GfxBindingGroup> {
        let mut bind_groups = Vec::new();

        if let Some(material_instance_handle) = material_instance
            && let Ok(material_instance) =
                resource_manager.get_data_mut(material_instance_handle, ())
        {
            let bind_group = &material_instance.data.material_binding;
            if let Some(bind_group) = bind_group {
                bind_groups.push(bind_group.clone());
            }

            let bind_group = material_instance.data.texture_binding.clone();

            if let Some(bind_group) = bind_group {
                if !material_instance.data.bound {
                    material_instance.data.bound = true;

                    let textures = material_instance.properties.textures.clone();
                    let mut updater = bind_group.update();
                    for texture_handle in &textures {
                        let texture = resource_manager.get_data(texture_handle, ()).unwrap().data;
                        updater = updater
                            .bind_sampled_image(&texture.image, gobs_gfx::ImageLayout::Shader)
                            .bind_sampler(&texture.sampler);
                    }
                    updater.end();
                }

                bind_groups.push(bind_group);
            }
        }

        bind_groups
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn add_model(
        &mut self,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        transform: Transform,
        pass: RenderPass,
    ) -> Result<(), ResourceError> {
        tracing::debug!(target: logger::RENDER, "Add model: {} to pass {}", model.name(), pass.name());

        // TODO: add material data for forward pass only
        for (mesh, material_instance) in &model.meshes {
            let bind_groups = Self::get_bind_groups(material_instance.as_ref(), resource_manager);

            let material_instance_id = match material_instance {
                Some(instance) => instance.id,
                None => ResourceId::default(),
            };

            let material_handle = material_instance.as_ref().and_then(|material_instance| {
                resource_manager
                    .get_data(material_instance, ())
                    .ok()
                    .map(|material_instance_data| material_instance_data.data.material)
            });

            let vertex_attributes = match pass.vertex_attributes() {
                Some(vertex_attributes) => vertex_attributes,
                None => {
                    if let Some(material_handle) = material_handle {
                        let material = resource_manager.get(&material_handle);

                        material.properties.pipeline_properties.vertex_attributes
                    } else {
                        return Err(ResourceError::InvalidData);
                    }
                }
            };

            let (pipeline, is_transparent) = if let Some(material_handle) = material_handle {
                let material = resource_manager.get(&material_handle);
                let blending_enabled = material.properties.blending_enabled;

                let pipeline_handle = resource_manager
                    .get_data(&material_handle, ())?
                    .data
                    .pipeline;

                let pipeline_data = resource_manager.get_data(&pipeline_handle, ())?.data;

                (Some(pipeline_data.pipeline.clone()), blending_enabled)
            } else {
                tracing::debug!("No material for model {}", model.name());
                (None, false)
            };

            let mesh_data = resource_manager.get_data(mesh, vertex_attributes)?;

            self.render_list.push(RenderObject {
                model_id: model.id,
                transform,
                pass_id: pass.id(),
                pipeline,
                is_transparent,
                bind_groups,
                vertex_buffer: mesh_data.data.vertex_buffer.clone(),
                vertices_offset: mesh_data.data.vertices_offset,
                vertices_len: mesh_data.data.vertices_len,
                vertices_count: mesh_data.data.vertices_count,
                index_buffer: mesh_data.data.index_buffer.clone(),
                indices_offset: mesh_data.data.indices_offset,
                indices_len: mesh_data.data.indices_len,
                material_instance_id,
                layer: mesh_data.properties.layer,
            });
        }

        Ok(())
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn add_bounds(
        &mut self,
        bounding_box: BoundingBox,
        pass: RenderPass,
    ) -> Result<(), ResourceError> {
        let mesh = Shapes::bounding_box(bounding_box, self.vertex_padding);

        if self.bounding_geometry.is_none() {
            self.bounding_geometry = Some(MeshGeometry::builder("bounding"));
        }

        let builder = self.bounding_geometry.take();

        if let Some(builder) = builder {
            self.bounding_geometry = Some(builder.extend(mesh));
            self.bounding_pass = Some(pass);
        }

        Ok(())
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn add_camera_data(
        &mut self,
        camera: &Camera,
        camera_transform: Transform,
        light: &Light,
        light_transform: Transform,
    ) {
        self.camera = camera.clone();
        self.camera_transform = camera_transform;
        self.lights.clear();
        self.lights.push((light.clone(), light_transform));
    }

    pub fn add_extent_data(&mut self, extent: ImageExtent2D) {
        self.extent = extent;
    }

    pub fn scene_data(&'_ self) -> SceneData<'_> {
        let default_light = &self.lights.first();

        SceneData {
            camera: &self.camera,
            camera_transform: &self.camera_transform,
            light: default_light.map(|l| &l.0),
            light_transform: default_light.map(|l| &l.1),
            extent: self.extent,
        }
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    fn sort(&mut self) {
        self.render_list.sort_unstable_by(|a, b| {
            // sort order: pass, transparent, material, model
            (a.pass_id.cmp(&b.pass_id))
                .then(a.layer.cmp(&b.layer))
                .then(a.is_transparent().cmp(&b.is_transparent()))
                .then(a.pipeline_id().cmp(&b.pipeline_id()))
                .then(a.material_instance_id.cmp(&b.material_instance_id))
                .then(a.index_buffer.id().cmp(&b.index_buffer.id()))
                .then(a.indices_offset.cmp(&b.indices_offset))
        });
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn finish(&mut self, resource_manager: &mut ResourceManager) {
        let bb = self.bounding_geometry.take();

        if let Some(bb) = bb {
            let model = Model::builder("box")
                .mesh(
                    bb.build(),
                    None,
                    resource_manager,
                    ResourceLifetime::Transient,
                )
                .build();

            let pass = self.bounding_pass.take().unwrap();

            self.add_model(resource_manager, model, Transform::IDENTITY, pass)
                .expect("Add bounding box");
        }

        self.sort();
    }
}

#[cfg(test)]
mod tests {
    use gobs_core::{Color, Transform, utils::timer::Timer};
    use gobs_render_graph::GraphConfig;
    use gobs_render_low::GfxContext;
    use gobs_resource::{geometry::Shapes, manager::ResourceManager, resource::ResourceLifetime};
    use tracing::Level;
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use crate::{Model, RenderBatch};

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_sort() {
        setup();

        let span = tracing::trace_span!(target: "perf", "sort").entered();

        let ctx = GfxContext::new("test", None, false).unwrap();
        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        let graph_data = include_str!("../../examples/resources/graph.ron");
        let passes =
            GraphConfig::load_graph_with_data(&ctx, graph_data, "test", &mut resource_manager)
                .unwrap();

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    &[Color::RED, Color::GREEN, Color::BLUE],
                    1.,
                    ctx.vertex_padding,
                ),
                None,
                &mut resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let mut batch = RenderBatch::new(&ctx);

        let mut timer = Timer::new();

        for _ in 0..30000 {
            let _ = batch.add_model(
                &mut resource_manager,
                triangle.clone(),
                Transform::IDENTITY,
                passes[0].clone(),
            );
        }

        batch.sort();

        span.exit();

        tracing::trace!(target: "perf", "sort: {}", 1000. * timer.delta());
    }
}
