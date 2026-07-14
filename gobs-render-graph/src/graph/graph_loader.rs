use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, ImageFormat, logger};
use gobs_render_hal::{ImageLayout, ImageUsage, SceneDataLayout, SceneDataProp, UniformData as _};
use gobs_resource::{
    ResourceError, ResourceManager,
    load::{self, AssetType},
};

use crate::{
    FrameGraph, GfxContext, PassType, Pipeline, PipelineProperties, PipelinesConfig, RenderFlags,
    pass::{
        Attachment, AttachmentAccess, AttachmentType, RenderPass, RenderPassType,
        compute::ComputePass, material::MaterialPass, present::PresentPass,
    },
};

// TODO: store in config file
const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphConfig {
    pipelines: Option<String>,
    graphes: HashMap<String, Vec<String>>,
    passes: HashMap<String, RenderPassConfig>,
    attachments: HashMap<String, ImageAttachmentInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RenderPassConfig {
    ty: RenderPassType,
    tag: PassType,
    pipeline: Option<String>,
    #[serde(default)]
    attachments: HashMap<String, AttachmentInfo>,
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
    #[serde(default)]
    flags: RenderFlags,
    target: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
enum AttachmentInfo {
    ColorAttachment {
        access: AttachmentAccess,
        clear: bool,
    },
    DepthAttachment {
        access: AttachmentAccess,
        clear: bool,
    },
    StorageImage {
        access: AttachmentAccess,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct ImageAttachmentInfo {
    usage: ImageUsage,
    format: ImageFormat,
}

impl GraphConfig {
    fn load_with_data(data: &str) -> Result<Self, ResourceError> {
        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        options.from_str(data).map_err(|e| {
            tracing::error!("{}", e);
            ResourceError::InvalidData
        })
    }

    pub fn load_graph(
        ctx: &mut GfxContext,
        filename: &str,
        name: &str,
        resource_manager: &mut ResourceManager,
    ) -> Result<FrameGraph, ResourceError> {
        let data = load::load_string_sync(filename, AssetType::RESOURCES)?;

        Self::load_graph_with_data(ctx, &data, name, resource_manager, true)
    }

    pub fn load_graph_with_data(
        ctx: &mut GfxContext,
        data: &str,
        name: &str,
        resource_manager: &mut ResourceManager,
        load_pipelines: bool,
    ) -> Result<FrameGraph, ResourceError> {
        let graph_config = Self::load_with_data(data)?;

        let mut graph = FrameGraph::new();

        // TODO: only register attachments used by passes
        for (attach_name, attach_config) in &graph_config.attachments {
            let attachment =
                Self::load_attachment(ctx, attach_config).ok_or(ResourceError::InvalidData)?;

            graph.register_attachment(ctx, attach_name, attachment);
        }

        if load_pipelines && let Some(pipelines) = &graph_config.pipelines {
            PipelinesConfig::load_resources(ctx, pipelines, resource_manager)
                .expect("Load pipelines");
        }

        tracing::debug!(target: logger::INIT, "Load graph: {}", "scene");

        for pass in &graph_config.graphes[name] {
            tracing::debug!(target: logger::INIT, "Load pass: {}", pass);

            let pass = Self::load_pass(ctx, &graph_config, pass, resource_manager)
                .unwrap_or_else(|| panic!("Failed to load pass {}", pass));

            graph.register_pass(pass.clone());
        }

        Ok(graph)
    }

    pub fn load_pass(
        ctx: &mut GfxContext,
        graph: &GraphConfig,
        passname: &str,
        resource_manager: &mut ResourceManager,
    ) -> Option<Arc<dyn RenderPass>> {
        tracing::info!(target: logger::INIT, "Load pass: {}", passname);

        let pass = graph.passes.get(passname)?;

        match pass.ty {
            RenderPassType::Compute => {
                Self::load_compute_pass(ctx, passname, pass, graph, resource_manager)
            }
            RenderPassType::Material => {
                Self::load_material_pass(ctx, passname, pass, graph, resource_manager)
            }
            RenderPassType::Present => Self::load_present_pass(ctx, passname, pass),
        }
    }

    fn load_compute_pass(
        ctx: &mut GfxContext,
        passname: &str,
        pass: &RenderPassConfig,
        graph: &GraphConfig,
        resource_manager: &mut ResourceManager,
    ) -> Option<Arc<dyn RenderPass>> {
        let pipeline_handle = resource_manager.get_by_name::<Pipeline>(pass.pipeline.as_ref()?)?;
        let pipeline = resource_manager
            .get_data(&mut ctx.hal, &pipeline_handle)
            .ok()?;

        let binding_group_layout =
            if let PipelineProperties::Compute(properties) = pipeline.properties {
                &properties.binding_groups
            } else {
                return None;
            };

        let mut compute_pass = ComputePass::new(
            passname,
            pipeline.data.pipeline,
            binding_group_layout.clone(),
        );

        for (attach_name, attach_config) in &pass.attachments {
            let attachment = Self::load_attachment_usage(ctx, graph, attach_name, attach_config)?;
            compute_pass.add_attachment(attach_name, attachment);
        }

        Some(Arc::new(compute_pass))
    }

    fn load_present_pass(
        ctx: &mut GfxContext,
        passname: &str,
        pass: &RenderPassConfig,
    ) -> Option<Arc<dyn RenderPass>> {
        if let Some(target) = &pass.target {
            Some(Arc::new(PresentPass::new(ctx, passname, target)))
        } else {
            tracing::error!(target: logger::INIT, "Invalid present target");
            None
        }
    }

    fn load_material_pass(
        ctx: &mut GfxContext,
        passname: &str,
        pass: &RenderPassConfig,
        graph: &GraphConfig,
        resource_manager: &mut ResourceManager,
    ) -> Option<Arc<dyn RenderPass>> {
        let mut scene_layout = SceneDataLayout::default();
        for prop in &pass.scene_layout {
            scene_layout = scene_layout.prop(*prop);
        }

        let mut material_pass =
            MaterialPass::new(ctx, passname, pass.tag, scene_layout, pass.flags);

        if let Some(pipeline) = &pass.pipeline {
            let pipeline_handle = resource_manager.get_by_name::<Pipeline>(pipeline)?;
            let pipeline = resource_manager
                .get_data(&mut ctx.hal, &pipeline_handle)
                .ok()?;
            material_pass.set_fixed_pipeline(pipeline.data.pipeline);
        }

        for (attach_name, attach_config) in &pass.attachments {
            let attachment = Self::load_attachment_usage(ctx, graph, attach_name, attach_config)?;

            material_pass.add_attachment(attach_name, attachment);
        }

        Some(Arc::new(material_pass))
    }

    fn get_render_target_extent(ctx: &GfxContext) -> ImageExtent2D {
        let extent = ctx.extent();
        ImageExtent2D::new(
            extent.width.max(FRAME_WIDTH),
            extent.height.max(FRAME_HEIGHT),
        )
    }

    fn load_attachment(ctx: &GfxContext, attach_info: &ImageAttachmentInfo) -> Option<Attachment> {
        let default_extent = Self::get_render_target_extent(ctx);

        let mut attachment = Attachment::new(AttachmentType::Color, AttachmentAccess::ReadWrite);
        attachment
            .with_usage(attach_info.usage)
            .with_format(attach_info.format)
            .with_extent(default_extent);

        Some(attachment)
    }

    fn load_attachment_usage(
        ctx: &GfxContext,
        graph: &GraphConfig,
        attach_name: &str,
        attach_usage: &AttachmentInfo,
    ) -> Option<Attachment> {
        let image_info = graph.attachments.get(attach_name)?;

        let default_extent = Self::get_render_target_extent(ctx);

        match attach_usage {
            AttachmentInfo::ColorAttachment { access, clear } => {
                let mut attachment = Attachment::new(AttachmentType::Color, *access);
                attachment
                    .with_usage(ImageUsage::Color)
                    .with_format(image_info.format)
                    .with_clear(*clear)
                    .with_extent(default_extent)
                    .with_layout(ImageLayout::Color);

                Some(attachment)
            }
            AttachmentInfo::DepthAttachment { access, clear } => {
                let mut attachment = Attachment::new(AttachmentType::Depth, *access);
                attachment
                    .with_usage(ImageUsage::Depth)
                    .with_format(image_info.format)
                    .with_clear(*clear)
                    .with_extent(default_extent)
                    .with_layout(ImageLayout::Depth);

                Some(attachment)
            }
            AttachmentInfo::StorageImage { access } => {
                let mut attachment = Attachment::new(AttachmentType::ImageStorage, *access);
                attachment.with_layout(ImageLayout::General);

                Some(attachment)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gobs_resource::ResourceManager;
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::{
        GfxContext, GraphConfig, PassType, RenderFlags,
        graph::graph_loader::{AttachmentInfo, RenderPassConfig},
        pass::{AttachmentAccess, RenderPassType},
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
    fn test_load() {
        setup();

        let mut ctx = GfxContext::new("test", None, false).unwrap();

        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        let data = include_str!("../../../examples/resources/graph.ron");

        let graph = GraphConfig::load_with_data(data).unwrap();
        tracing::info!("Graph: {:?}", graph.graphes["scene"]);

        let graph =
            GraphConfig::load_graph_with_data(&mut ctx, data, "ui", &mut resource_manager, false)
                .unwrap();

        for pass in graph.passes {
            tracing::info!("Load pass: {}", pass.pass.name());
        }
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_load_pass() {
        setup();

        let mut ctx = GfxContext::new("test", None, false).unwrap();

        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        let data = include_str!("../../../examples/resources/graph.ron");

        let graph_config = GraphConfig::load_with_data(data).unwrap();

        let _pass =
            GraphConfig::load_pass(&mut ctx, &graph_config, "forward", &mut resource_manager)
                .unwrap();
    }

    #[test]
    fn test_serialize() {
        setup();

        let pass_name = "bounds".to_string();

        let graph = GraphConfig {
            pipelines: None,
            graphes: HashMap::from([("scene".to_string(), vec![pass_name.clone()])]),
            passes: HashMap::from([(
                pass_name,
                RenderPassConfig {
                    ty: RenderPassType::Material,
                    tag: PassType::Bounds,
                    pipeline: None,
                    attachments: HashMap::from([(
                        "draw".to_string(),
                        AttachmentInfo::ColorAttachment {
                            access: AttachmentAccess::ReadWrite,
                            clear: true,
                        },
                    )]),
                    scene_layout: Vec::new(),
                    flags: RenderFlags::ENTITY,
                    target: None,
                },
            )]),
            attachments: HashMap::new(),
        };

        let ron = ron::ser::to_string_pretty(&graph, ron::ser::PrettyConfig::default()).unwrap();

        tracing::info!("Load data: {}", ron);
    }

    #[test]
    fn test_deserialize() {
        setup();

        let _graph_config =
            GraphConfig::load_with_data(include_str!("../../../examples/resources/graph.ron"))
                .unwrap();
    }
}
