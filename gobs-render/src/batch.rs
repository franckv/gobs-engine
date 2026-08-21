use std::{collections::hash_map::Entry, sync::Arc};

use ahash::HashMap;

use gobs_core::{ImageExtent2D, Transform, logger};
use gobs_render_graph::{GfxContext, RenderFlags, RenderObject, SceneData, SceneDataLayout};
use gobs_render_hal::{
    AlignMode, BindResource, BindingGroupType, DescriptorType, Handle, RenderHAL, VertexData,
};
use gobs_resource::{ResourceError, ResourceHandle, ResourceManager, camera::Camera, light::Light};

use crate::{
    BoundingBox, GraphicsPipelineProperties, Material, MaterialInstance, Mesh, Pipeline,
    PipelineProperties, RenderMeshBuilder, RenderModelBuilder, ShapeBuilder, Texture, model::Model,
};

#[derive(Clone)]
struct MaterialData {
    render_flags: RenderFlags,
    pipeline: Option<Handle>,
    pipeline_properties: Option<GraphicsPipelineProperties>,
    material_data: Option<BindResource>,
    material_textures: Option<BindResource>,
}

pub struct RenderBatch {
    pub render_list: Vec<RenderObject>,
    pub(crate) recording: bool,
    pub(crate) camera: Camera,
    pub(crate) camera_transform: Transform,
    pub(crate) lights: Vec<(Light, Transform)>,
    pub(crate) extent: ImageExtent2D,
    generate_bounds: bool,
    bounding_geometry: Option<ShapeBuilder>,
    material_cache: HashMap<ResourceHandle<MaterialInstance>, MaterialData>,
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
    ) -> Result<MaterialData, ResourceError> {
        let mut render_flags = RenderFlags::default();

        if let Some(material_instance_handle) = material_instance_handle {
            match self.material_cache.entry(*material_instance_handle) {
                Entry::Occupied(e) => Ok(e.get().clone()),
                Entry::Vacant(e) => {
                    let (pipeline, pipeline_properties) = Self::get_pipeline(
                        ctx.hal_mut(),
                        resource_manager,
                        *material_instance_handle,
                        &mut render_flags,
                    )?;

                    let (material_data, material_textures) = Self::get_material_data(
                        ctx.hal_mut(),
                        resource_manager,
                        *material_instance_handle,
                    )?;

                    let material = MaterialData {
                        render_flags,
                        pipeline: Some(pipeline),
                        pipeline_properties: Some(pipeline_properties),
                        material_data,
                        material_textures,
                    };
                    Ok(e.insert(material).clone())
                }
            }
        } else {
            Ok(MaterialData {
                render_flags,
                pipeline: None,
                pipeline_properties: None,
                material_data: None,
                material_textures: None,
            })
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

            let render_flags = flags.union(material.render_flags);

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

            let scene_layout = match material.pipeline_properties {
                Some(properties) => properties.scene_data_layout,
                None => SceneDataLayout::new(AlignMode::Std140),
            };

            let render_object = RenderObject {
                model: model.name.clone(),
                transform,
                pipeline: material.pipeline,
                vertex_buffer,
                index_buffer,
                index_len,
                vertex_attribute,
                scene_layout,
                layer,
                material_data: material.material_data,
                material_textures: material.material_textures,
                render_flags,
            };

            self.render_list.push(render_object);
        }

        Ok(())
    }

    fn get_material_data(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
    ) -> Result<(Option<BindResource>, Option<BindResource>), ResourceError> {
        let (material_buffer, material, textures) = {
            let resource_data = resource_manager.get_data(hal, &material_instance_handle)?;

            (
                resource_data.data.material_buffer,
                resource_data.properties.material,
                resource_data.properties.textures.clone(),
            )
        };

        let material_properties = &resource_manager.get(&material).properties;

        let material_data = material_buffer.map(|material_buffer| {
            let material_data_layout = material_properties
                .pipeline_properties
                .binding_groups
                .iter()
                .find(|group| group.binding_group_type == BindingGroupType::MaterialData)
                .expect("Material pipeline has no material data layout")
                .clone();

            BindResource::with_resources(material_data_layout, vec![material_buffer])
        });

        let material_textures = {
            if textures.is_empty() {
                None
            } else {
                let texture_data_layout = material_properties
                    .pipeline_properties
                    .binding_groups
                    .iter()
                    .find(|group| group.binding_group_type == BindingGroupType::MaterialTextures)
                    .expect("Material pipeline has no textures layout")
                    .clone();

                let tex_data = textures
                    .iter()
                    .map(|t| {
                        let data = resource_manager.get_data(&mut *hal, t)?;

                        Ok((data.data.image, data.data.sampler))
                    })
                    .collect::<Result<Vec<_>, ResourceError>>()?;

                let mut texture_idx = 0;
                let mut sampler_idx = 0;

                let mut resource = BindResource::new(texture_data_layout.clone());

                for (ty, _, count) in &texture_data_layout.bindings {
                    resource = resource.next();
                    match ty {
                        DescriptorType::SampledImage => {
                            let to_write = (tex_data.len() - texture_idx).min(*count as usize);

                            for i in 0..to_write {
                                resource = resource.binding(tex_data[texture_idx + i].0, i)
                            }
                            texture_idx += to_write;
                        }
                        DescriptorType::Sampler => {
                            let to_write = (tex_data.len() - sampler_idx).min(*count as usize);

                            for i in 0..to_write {
                                resource = resource.binding(tex_data[sampler_idx + i].1, i)
                            }
                            sampler_idx += to_write;
                        }
                        _ => unimplemented!(),
                    }
                }

                Some(resource)
            }
        };

        Ok((material_data, material_textures))
    }

    fn get_pipeline(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
        render_flags: &mut RenderFlags,
    ) -> Result<(Handle, GraphicsPipelineProperties), ResourceError> {
        let material_instance = resource_manager.get(&material_instance_handle);
        let material_handle = material_instance.properties.material;
        let material = resource_manager.get(&material_handle);

        if material.properties.blending_enabled {
            *render_flags |= RenderFlags::TRANSPARENT;
        } else {
            *render_flags |= RenderFlags::OPAQUE;
        }

        let material_data = resource_manager.get_data(hal, &material_handle)?;

        let pipeline_handle = material_data.data.pipeline;

        let pipeline_data = resource_manager.get_data(hal, &pipeline_handle)?;
        let pipeline_properties = pipeline_data.properties;

        if let PipelineProperties::Graphics(properties) = pipeline_properties {
            tracing::trace!(target: logger::RENDER, "Using pipeline {:?}", properties);
            Ok((pipeline_data.data.pipeline, properties.clone()))
        } else {
            Err(ResourceError::InvalidData)
        }
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
            if let Some(pipeline) = obj.pipeline {
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
    use tracing::Level;
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{Color, GobsConfig, Transform, logger, utils::timer::Timer};
    use gobs_render_graph::{GfxContext, RenderFlags};
    use gobs_resource::ResourceManager;

    use crate::{Mesh, MeshLoader, RenderBatch, RenderMeshBuilder, RenderModelBuilder, Shapes};

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

        let config = GobsConfig::default();
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
