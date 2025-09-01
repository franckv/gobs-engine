use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use gobs_core::ImageFormat;
use gobs_gfx::{
    BindingGroupType, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace, PolygonMode,
};
use gobs_render_low::{GfxContext, ObjectDataLayout, ObjectDataProp, UniformData};
use gobs_resource::{
    geometry::VertexAttribute,
    load::{self, AssetType},
    manager::ResourceManager,
    resource::{ResourceError, ResourceLifetime},
};

use crate::resources::{Pipeline, PipelineProperties};

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelinesConfig {
    pipelines: HashMap<String, PipelineConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PipelineConfig {
    vertex_shader: Option<ShaderConfig>,
    fragment_shader: Option<ShaderConfig>,
    #[serde(default)]
    object_layout: Vec<ObjectDataProp>,
    vertex_attributes: VertexAttribute,
    #[serde(default)]
    bindings: Vec<BindingConfig>,
    polygon_mode: PolygonMode,
    attachments: AttachmentFormat,
    depth_test: DepthConfig,
    cull_mode: CullMode,
    front_face: FrontFace,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShaderConfig {
    file: String,
    entry: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BindingConfig {
    group: BindingGroupType,
    stage: DescriptorStage,
    descriptor_type: DescriptorType,
}

#[derive(Debug, Serialize, Deserialize)]
struct AttachmentFormat {
    color_format: Option<ImageFormat>,
    depth_format: Option<ImageFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DepthConfig {
    enable: bool,
    #[serde(default)]
    write_enable: bool,
    #[serde(default)]
    compare: CompareOp,
}

impl PipelinesConfig {
    pub fn load_resources(
        ctx: &GfxContext,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) -> Result<(), ResourceError> {
        let data = load::load_string_sync(filename, AssetType::RESOURCES)?;

        Self::load_resources_with_data(ctx, &data, resource_manager)
    }

    fn load_resources_with_data(
        ctx: &GfxContext,
        data: &str,
        resource_manager: &mut ResourceManager,
    ) -> Result<(), ResourceError> {
        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let config: PipelinesConfig = options
            .from_str(data)
            .map_err(|_| ResourceError::InvalidData)?;

        for pipeline_name in config.pipelines.keys() {
            let pipeline = Self::load_pipeline(ctx, &config, pipeline_name);
            if let Some(pipeline) = pipeline {
                resource_manager.add::<Pipeline>(pipeline, ResourceLifetime::Static, true);
            }
        }

        Ok(())
    }

    fn load_pipeline(
        ctx: &GfxContext,
        config: &PipelinesConfig,
        name: &str,
    ) -> Option<PipelineProperties> {
        let pipeline = config.pipelines.get(name)?;

        let mut object_layout = ObjectDataLayout::default();
        for prop in &pipeline.object_layout {
            object_layout = object_layout.prop(*prop);
        }

        let mut props = PipelineProperties::graphics(name)
            .push_constants(object_layout.uniform_layout().size())
            .pool_size(ctx.frames_in_flight)
            .vertex_attributes(pipeline.vertex_attributes)
            .polygon_mode(pipeline.polygon_mode)
            .cull_mode(pipeline.cull_mode)
            .front_face(pipeline.front_face);

        let mut last_group = BindingGroupType::None;
        for binding in &pipeline.bindings {
            if binding.group != last_group {
                props = props.binding_group(binding.stage, binding.group);
                last_group = binding.group;
            }
            props = props.binding(binding.descriptor_type);
        }

        if let Some(format) = pipeline.attachments.color_format {
            props = props.color_format(format);
        }
        if let Some(format) = pipeline.attachments.depth_format {
            props = props.depth_format(format);
        }

        if let Some(shader) = &pipeline.vertex_shader {
            props = props
                .vertex_shader(&shader.file)
                .vertex_entry(&shader.entry);
        }

        if let Some(shader) = &pipeline.fragment_shader {
            props = props
                .fragment_shader(&shader.file)
                .fragment_entry(&shader.entry);
        }

        if pipeline.depth_test.enable {
            props = props.depth_test_enable(
                pipeline.depth_test.write_enable,
                pipeline.depth_test.compare,
            );
        } else {
            props = props.depth_test_disable();
        }

        Some(PipelineProperties::Graphics(props))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gobs_resource::{geometry::VertexAttribute, manager::ResourceManager};
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::ImageFormat;
    use gobs_gfx::{CompareOp, CullMode, FrontFace, PolygonMode};
    use gobs_render_low::GfxContext;

    use crate::resources::{
        PipelinesConfig,
        pipeline::pipeline_config::{AttachmentFormat, DepthConfig, PipelineConfig},
    };

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_load_resources() {
        setup();

        let ctx = GfxContext::new("test", None, false).unwrap();

        let data = include_str!("../../../../examples/resources/pipelines.ron");

        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        PipelinesConfig::load_resources_with_data(&ctx, data, &mut resource_manager).unwrap();
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_load_pipeline() {
        setup();

        let ctx = GfxContext::new("test", None, false).unwrap();

        let data = include_str!("../../../../examples/resources/pipelines.ron");

        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        let pipeline_config: PipelinesConfig = options.from_str(data).unwrap();

        let _pipeline =
            PipelinesConfig::load_pipeline(&ctx, &pipeline_config, "wireframe").unwrap();
    }

    #[test]
    fn test_serialize() {
        setup();

        let config = PipelinesConfig {
            pipelines: HashMap::from([(
                "wireframe".to_string(),
                PipelineConfig {
                    vertex_shader: None,
                    fragment_shader: None,
                    object_layout: Vec::new(),
                    vertex_attributes: VertexAttribute::POSITION,
                    bindings: Vec::new(),
                    polygon_mode: PolygonMode::Fill,
                    attachments: AttachmentFormat {
                        color_format: Some(ImageFormat::R16g16b16a16Sfloat),
                        depth_format: None,
                    },
                    depth_test: DepthConfig {
                        enable: true,
                        write_enable: true,
                        compare: CompareOp::Less,
                    },
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::CCW,
                },
            )]),
        };

        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        let ron = options
            .to_string_pretty(&config, ron::ser::PrettyConfig::default())
            .unwrap();

        tracing::info!("Load data: {}", ron);
    }
}
