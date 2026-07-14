use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform, logger};
use gobs_render_graph::{
    BoundingBox, GfxContext, MaterialInstance, MeshBuilder, MeshGeometry, PipelineProperties,
    RenderFlags, RenderObject, RenderPass, Shapes,
};
use gobs_render_hal::{BindResource, Handle, SceneData};
use gobs_resource::{
    ResourceError, ResourceHandle, ResourceLifetime, ResourceManager, camera::Camera, light::Light,
};

use crate::model::Model;

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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn reset(&mut self) {
        self.render_list.clear();
        self.bounding_geometry = None;
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn add_model(
        &mut self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        transform: Transform,
        flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        tracing::debug!(target: logger::RENDER, "Add model: {} to render list", model.name());

        for (mesh, material_instance_handle) in &model.meshes {
            let mut render_flags = flags;

            let pipeline = Self::get_pipeline(
                ctx,
                resource_manager,
                material_instance_handle,
                &mut render_flags,
            )?;

            if pipeline.is_none() {
                tracing::debug!("No material for model {}", model.name());
            }

            let (vertex_buffer, index_buffer, index_len, vertex_attribute, layer) = {
                let mesh_data = resource_manager.get_data(&mut ctx.hal, mesh)?;

                (
                    mesh_data.data.vertex_view,
                    mesh_data.data.index_view,
                    mesh_data.data.index_len,
                    mesh_data.properties.vertex_attributes,
                    mesh_data.properties.layer,
                )
            };

            let (material_data, material_textures) =
                Self::get_material_data(ctx, resource_manager, material_instance_handle)?;

            let render_object = RenderObject {
                model: model.name().to_string(),
                transform,
                pipeline,
                vertex_buffer,
                index_buffer,
                index_len,
                vertex_attribute,
                layer,
                material_data,
                material_textures,
                render_flags,
            };

            self.render_list.push(render_object);
        }

        Ok(())
    }

    fn get_material_data(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        material_instance_handle: &Option<ResourceHandle<MaterialInstance>>,
    ) -> Result<(Option<BindResource>, Option<BindResource>), ResourceError> {
        if let Some(material_instance_handle) = material_instance_handle {
            let (material_buffer, material, textures) = {
                let resource_data =
                    resource_manager.get_data(&mut ctx.hal, material_instance_handle)?;

                (
                    resource_data.data.material_buffer,
                    resource_data.properties.material,
                    resource_data.properties.textures.clone(),
                )
            };

            let material_properties = &resource_manager.get(&material).properties;

            let material_data_layout = material_properties.material_data_layout.bindings_layout();

            let texture_data_layout = &material_properties.texture_data_layout.bindings_layout();

            let material_data = material_buffer.map(|material_buffer| {
                BindResource::new(material_data_layout.clone(), vec![material_buffer])
            });

            let material_textures = {
                if textures.is_empty() {
                    None
                } else {
                    let mut texture_handles = vec![];

                    for texture in textures {
                        let tex_data = resource_manager.get_data(&mut ctx.hal, &texture)?;
                        texture_handles.push(tex_data.data.image);
                        texture_handles.push(tex_data.data.sampler);
                    }

                    Some(BindResource::new(
                        texture_data_layout.clone(),
                        texture_handles,
                    ))
                }
            };

            Ok((material_data, material_textures))
        } else {
            Ok((None, None))
        }
    }

    fn get_pipeline(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        material_instance_handle: &Option<ResourceHandle<MaterialInstance>>,
        render_flags: &mut RenderFlags,
    ) -> Result<Option<Handle>, ResourceError> {
        let material_handle = match material_instance_handle {
            Some(material_instance_handle) => {
                let material_instance = resource_manager.get(material_instance_handle);
                let material = material_instance.properties.material;

                Some(material)
            }
            None => None,
        };

        if let Some(material_handle) = material_handle {
            let material = resource_manager.get(&material_handle);
            if material.properties.blending_enabled {
                *render_flags |= RenderFlags::TRANSPARENT;
            }

            let material_data = resource_manager.get_data(&mut ctx.hal, &material_handle)?;

            let pipeline_handle = material_data.data.pipeline;

            let pipeline_data = resource_manager.get_data(&mut ctx.hal, &pipeline_handle)?;
            let pipeline_properties = pipeline_data.properties;

            if let PipelineProperties::Graphics(properties) = pipeline_properties {
                tracing::trace!("Using pipeline {:?}", properties);
                Ok(Some(pipeline_data.data.pipeline))
            } else {
                Err(ResourceError::InvalidData)
            }
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn validate(&mut self, ctx: &mut GfxContext) {
        for obj in &self.render_list {
            if let Some(pipeline) = obj.pipeline {
                let descriptors = ctx.hal.get_pipeline_descriptor_types(pipeline);
                for descriptor_type in descriptors {
                    let descriptor_layout = ctx
                        .hal
                        .get_pipeline_descriptor_layout(pipeline, &descriptor_type);
                    tracing::trace!(target: logger::RENDER, "Render object: {}, descriptor layout: {:#?}", &obj.model, descriptor_layout);
                }

                let vertex_attributes = ctx.hal.get_pipeline_vertex_attributes(pipeline);
                debug_assert_eq!(
                    vertex_attributes, obj.vertex_attribute,
                    "Invalid vertex layout for {}",
                    &obj.model
                );
            }
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn finish(&mut self, ctx: &mut GfxContext, resource_manager: &mut ResourceManager) {
        let bb = self.bounding_geometry.take();

        if let Some(bb) = bb {
            let model = Model::builder("box")
                .mesh(
                    bb.build(),
                    None,
                    ctx.world_vertex_attributes,
                    resource_manager,
                    ResourceLifetime::Transient,
                )
                .build();

            self.add_model(
                ctx,
                resource_manager,
                model,
                Transform::IDENTITY,
                RenderFlags::empty(),
            )
            .expect("Add bounding box");
        }

        #[cfg(debug_assertions)]
        self.validate(ctx);

        self.sort();
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{Color, Transform, logger, utils::timer::Timer};
    use gobs_render_graph::{GfxContext, Mesh, MeshLoader, RenderFlags, Shapes};
    use gobs_resource::{ResourceLifetime, ResourceManager};

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

        let span = tracing::trace_span!(target: logger::PROFILE, "sort").entered();

        let mut ctx = GfxContext::new("test", None, false).unwrap();
        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        let mesh_loader = MeshLoader::new(&mut ctx);
        resource_manager.register_resource::<Mesh>(mesh_loader);

        let triangle = Model::builder("triangle")
            .mesh(
                Shapes::triangle(
                    &[Color::RED, Color::GREEN, Color::BLUE],
                    1.,
                    ctx.vertex_padding,
                ),
                None,
                ctx.world_vertex_attributes,
                &mut resource_manager,
                ResourceLifetime::Static,
            )
            .build();

        let mut batch = RenderBatch::new(&ctx);

        let mut timer = Timer::new();

        for _ in 0..30000 {
            let _ = batch.add_model(
                &mut ctx,
                &mut resource_manager,
                triangle.clone(),
                Transform::IDENTITY,
                RenderFlags::empty(),
            );
        }

        batch.sort();

        span.exit();

        tracing::trace!(target: logger::PROFILE, "sort: {}", 1000. * timer.delta());
    }
}
