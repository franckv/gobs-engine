use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{ImageLayout, ImageUsage};
use gobs_render_low::{
    GfxContext, ObjectDataLayout, ObjectDataProp, SceneDataLayout, SceneDataProp,
};
use gobs_resource::{
    load::{self, AssetType},
    manager::ResourceManager,
    resource::ResourceError,
};

use crate::{
    PassType, Pipeline,
    pass::{AttachmentAccess, AttachmentType, RenderPass, RenderPassType, material::MaterialPass},
};

// TODO: store in config file
const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphConfig {
    schedule: Vec<String>,
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
    object_layout: Vec<ObjectDataProp>,
    #[serde(default)]
    scene_layout: Vec<SceneDataProp>,
    #[serde(default)]
    render_transparent: bool,
    #[serde(default)]
    render_opaque: bool,
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
}

#[derive(Debug, Deserialize, Serialize)]
struct ImageAttachmentInfo {
    usage: ImageUsage,
    format: ImageFormat,
    layout: ImageLayout,
}

impl GraphConfig {
    pub fn load(filename: &str) -> Result<Self, ResourceError> {
        let data = load::load_string_sync(filename, AssetType::RESOURCES)?;

        Self::load_with_data(&data)
    }

    fn load_with_data(data: &str) -> Result<Self, ResourceError> {
        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        options
            .from_str(data)
            .map_err(|_| ResourceError::InvalidData)
    }

    pub fn load_graph(
        ctx: &GfxContext,
        filename: &str,
        resource_manager: &mut ResourceManager,
    ) -> Result<Vec<Arc<dyn RenderPass>>, ResourceError> {
        let graph = Self::load(filename)?;

        let passes = graph
            .schedule
            .iter()
            .filter_map(|passname| Self::load_pass(ctx, &graph, passname, resource_manager))
            .collect();

        Ok(passes)
    }

    pub fn load_pass(
        ctx: &GfxContext,
        graph: &GraphConfig,
        passname: &str,
        resource_manager: &mut ResourceManager,
    ) -> Option<Arc<dyn RenderPass>> {
        let pass = graph.passes.get(passname)?;

        let mut scene_layout = SceneDataLayout::builder();
        for prop in &pass.scene_layout {
            scene_layout = scene_layout.prop(*prop);
        }

        let mut object_layout = ObjectDataLayout::builder();
        for prop in &pass.object_layout {
            object_layout = object_layout.prop(*prop);
        }

        let default_extent = ctx.extent();
        let default_extent = ImageExtent2D::new(
            default_extent.width.max(FRAME_WIDTH),
            default_extent.height.max(FRAME_HEIGHT),
        );

        let mut material_pass = MaterialPass::new(
            ctx,
            passname,
            pass.tag,
            object_layout.build(),
            scene_layout.build(),
            pass.render_transparent,
            pass.render_opaque,
        );

        if let Some(pipeline) = &pass.pipeline {
            let pipeline_handle = resource_manager.get_by_name::<Pipeline>(pipeline)?;
            let pipeline = resource_manager.get_data(&pipeline_handle, ()).ok()?;

            material_pass.set_fixed_pipeline(pipeline.pipeline.clone());
        }

        for (attach_name, attach_config) in &pass.attachments {
            let image_info = graph.attachments.get(attach_name)?;

            // TODO: add image extent
            match attach_config {
                AttachmentInfo::ColorAttachment { access, clear } => {
                    material_pass
                        .add_attachment(attach_name, AttachmentType::Color, *access)
                        .with_usage(ImageUsage::Color)
                        .with_format(image_info.format)
                        .with_clear(*clear)
                        .with_extent(default_extent)
                        .with_layout(ImageLayout::Color);
                }
                AttachmentInfo::DepthAttachment { access, clear } => {
                    material_pass
                        .add_attachment(attach_name, AttachmentType::Depth, *access)
                        .with_usage(ImageUsage::Depth)
                        .with_format(image_info.format)
                        .with_clear(*clear)
                        .with_extent(default_extent)
                        .with_layout(ImageLayout::Depth);
                }
            }
        }

        Some(Arc::new(material_pass))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gobs_resource::manager::ResourceManager;
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_render_low::GfxContext;

    use crate::{
        GraphConfig, PassType,
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
    fn test_load() {
        setup();

        let ctx = GfxContext::new("test", None, true).unwrap();

        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight);

        let data = include_str!("../../../examples/resources/graph.ron");

        let graph_config = GraphConfig::load_with_data(data).unwrap();

        let _pass =
            GraphConfig::load_pass(&ctx, &graph_config, "forward", &mut resource_manager).unwrap();
    }

    #[test]
    fn test_serialize() {
        setup();

        let pass_name = "bounds".to_string();

        let graph = GraphConfig {
            schedule: vec![pass_name.clone()],
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
                    object_layout: Vec::new(),
                    scene_layout: Vec::new(),
                    render_transparent: true,
                    render_opaque: true,
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
