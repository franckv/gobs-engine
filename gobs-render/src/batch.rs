use std::{collections::hash_map::Entry, sync::Arc};

use ahash::HashMap;

use gobs_core::{ImageExtent2D, Transform, logger};
use gobs_render_graph::{GfxContext, MaterialRenderData, RenderFlags, RenderObject, SceneData};
use gobs_render_hal::VertexData;
use gobs_resource::{ResourceError, ResourceHandle, ResourceManager, camera::Camera, light::Light};

use crate::{
    BoundingBox, Material, MaterialInstance, Mesh, Pipeline, RenderMeshBuilder, RenderModelBuilder,
    ShapeBuilder, Texture, material_system::MaterialSystem, model::Model,
};

pub struct RenderBatch {
    pub render_list: Vec<RenderObject>,
    pub(crate) recording: bool,
    pub(crate) camera: Camera,
    pub(crate) camera_transform: Transform,
    pub(crate) lights: Vec<(Light, Transform)>,
    pub(crate) extent: ImageExtent2D,
    generate_bounds: bool,
    bounding_geometry: Option<ShapeBuilder>,
    material_cache: HashMap<ResourceHandle<MaterialInstance>, MaterialRenderData>,
}

impl RenderBatch {
    pub fn new() -> Self {
        tracing::debug!(target: logger::RENDER, ">>> Prepare render batch");

        Self {
            render_list: Vec::new(),
            recording: true,
            camera: Camera::default(),
            camera_transform: Transform::default(),
            lights: vec![],
            extent: ImageExtent2D::default(),
            generate_bounds: false,
            bounding_geometry: None,
            material_cache: HashMap::default(),
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn reset(&mut self) {
        self.render_list.clear();
        self.bounding_geometry = None;
        self.material_cache.clear();
    }

    pub fn generate_bounds(&mut self, generate_bounds: bool) {
        self.generate_bounds = generate_bounds;
    }

    fn get_material(
        &mut self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        material_instance_handle: &Option<ResourceHandle<MaterialInstance>>,
    ) -> Result<MaterialRenderData, ResourceError> {
        if let Some(material_instance_handle) = material_instance_handle {
            match self.material_cache.entry(*material_instance_handle) {
                Entry::Occupied(e) => Ok(e.get().clone()),
                Entry::Vacant(e) => {
                    let material = MaterialSystem::get_material_data(
                        ctx.hal_mut(),
                        resource_manager,
                        *material_instance_handle,
                    )?;

                    Ok(e.insert(material).clone())
                }
            }
        } else {
            Ok(MaterialRenderData::default())
        }
    }

    pub fn add_model(
        &mut self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        transform: Transform,
        bounding_box: Option<BoundingBox>,
        flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        tracing::debug!(target: logger::RENDER, "Add model: {} to render list", model.name());

        if let Some(bounding_box) = bounding_box
            && self.generate_bounds
        {
            self.add_bounds(bounding_box);
        }

        for (mesh, material_instance_handle) in &model.meshes {
            let material = self.get_material(ctx, resource_manager, material_instance_handle)?;

            let render_flags = flags.union(material.material_render_flags);

            if material.pipeline.is_none() {
                tracing::debug!("No material for model {}", model.name());
            }

            tracing::debug!(target: logger::RENDER, "Add mesh: {} to render list [{:?}]", model.name(), render_flags);

            let (vertex_buffer, index_buffer, index_len, vertex_attribute, layer) = {
                let mesh_data = resource_manager.get_data(ctx.hal_mut(), mesh)?;

                (
                    mesh_data.data.vertex_view,
                    mesh_data.data.index_view,
                    mesh_data.data.index_len,
                    mesh_data.properties.vertex_attributes,
                    mesh_data.properties.layer,
                )
            };

            let render_object = RenderObject {
                model: model.name.clone(),
                transform,
                vertex_buffer,
                index_buffer,
                index_len,
                vertex_attribute,
                material,
                render_flags,
                layer,
            };

            self.render_list.push(render_object);
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn add_bounds(&mut self, bounding_box: BoundingBox) {
        tracing::debug!(target: logger::RENDER, "Add bounding box");

        let (left, bottom, back) = bounding_box.bottom_left().into();
        let (right, top, front) = bounding_box.top_right().into();

        let v = [
            [left, top, front],
            [right, top, front],
            [left, bottom, front],
            [right, bottom, front],
            [left, top, back],
            [right, top, back],
            [left, bottom, back],
            [right, bottom, back],
        ];

        const VI: [u32; 36] = [
            2, 3, 1, 2, 1, 0, // F
            7, 6, 4, 7, 4, 5, // B
            6, 2, 0, 6, 0, 4, // L
            3, 7, 5, 3, 5, 1, // R
            0, 1, 5, 0, 5, 4, // U
            6, 7, 3, 6, 3, 2, // D
        ];

        tracing::trace!(target: logger::RENDER, "Bounding box mesh={:?}", &v);

        let builder = match self.bounding_geometry.take() {
            Some(builder) => builder,
            None => ShapeBuilder::new("bounding").geometry_only(),
        };

        let vertices: Vec<_> = v
            .iter()
            .map(|&pos| VertexData::builder().position(pos.into()).build())
            .collect();

        self.bounding_geometry = Some(builder.add_vertices(&vertices, &VI));
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn sort(&mut self) {
        self.render_list.sort_unstable();
    }

    #[cfg(debug_assertions)]
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn validate(&mut self, ctx: &mut GfxContext) {
        for obj in &self.render_list {
            if let Some(pipeline) = obj.material.pipeline {
                let descriptors = ctx.hal().get_pipeline_descriptor_types(pipeline);
                for descriptor_type in descriptors {
                    let descriptor_layout = ctx
                        .hal()
                        .get_pipeline_descriptor_layout(pipeline, &descriptor_type);
                    tracing::trace!(target: logger::RENDER, "Render object: {}, descriptor layout: {:#?}", &obj.model, descriptor_layout);
                }

                let vertex_attributes = ctx.hal().get_pipeline_vertex_attributes(pipeline);
                debug_assert_eq!(
                    vertex_attributes, obj.vertex_attribute,
                    "Invalid vertex layout for {}",
                    obj.model
                );
            }
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn finish(&mut self, ctx: &mut GfxContext, resource_manager: &mut ResourceManager) {
        let bb = self.bounding_geometry.take();

        if let Some(bb) = bb {
            let bb = bb.build();

            let mesh = RenderMeshBuilder::new(resource_manager, "box")
                .with_geometry(bb)
                .transient(true)
                .build();

            let model = RenderModelBuilder::new(resource_manager, "box")
                .with_mesh(mesh)
                .build();

            self.add_model(
                ctx,
                resource_manager,
                model,
                Transform::IDENTITY,
                None,
                RenderFlags::BOUNDS,
            )
            .expect("Add bounding box");
        } else {
            tracing::debug!(target: logger::RENDER, "No bounding box");
        }

        #[cfg(debug_assertions)]
        self.validate(ctx);

        self.sort();

        self.recording = false;

        tracing::debug!(target: logger::RENDER, "Flush resource loaders");
        resource_manager.flush::<Texture>();
        resource_manager.flush::<Mesh>();
        resource_manager.flush::<Pipeline>();
        resource_manager.flush::<Material>();
        resource_manager.flush::<MaterialInstance>();

        tracing::debug!(target: logger::RENDER, "<<< Finish render batch");
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use gobs_render_hal::RenderHalConfig;
    use tracing::Level;
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{Color, ConfigWriter as _, GobsConfig, Transform, logger, utils::timer::Timer};
    use gobs_render_graph::{GfxContext, RenderFlags};
    use gobs_resource::ResourceManager;

    use crate::{
        Mesh, MeshLoader, RenderBatch, RenderConfig, RenderMeshBuilder, RenderModelBuilder, Shapes,
    };

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

        let span = tracing::trace_span!(target: logger::PROFILE, "sort").entered();

        let mut config = GobsConfig::default();
        config.register::<RenderConfig>();
        config.register::<RenderHalConfig>();

        let mut ctx = GfxContext::new("test", None, config, false);
        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight());

        let mesh_loader = MeshLoader::new(&mut ctx);
        resource_manager.register_resource::<Mesh>(mesh_loader);

        let mesh = RenderMeshBuilder::new(&mut resource_manager, "triangle")
            .with_geometry(Shapes::triangle(
                &[Color::RED, Color::GREEN, Color::BLUE],
                1.,
            ))
            .build();
        let triangle = RenderModelBuilder::new(&mut resource_manager, "triangle")
            .with_mesh(mesh)
            .build();

        let mut batch = RenderBatch::new();

        let mut timer = Timer::new();

        for _ in 0..30000 {
            let _ = batch.add_model(
                &mut ctx,
                &mut resource_manager,
                triangle.clone(),
                Transform::IDENTITY,
                None,
                RenderFlags::empty(),
            );
        }

        batch.sort();

        span.exit();

        tracing::trace!(target: logger::PROFILE, "sort: {}", 1000. * timer.delta());
    }
}
