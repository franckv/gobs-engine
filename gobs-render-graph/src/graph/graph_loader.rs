use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use gobs_core::{ImageExtent2D, ImageFormat, logger};
use gobs_render_hal::{ImageLayout, ImageUsage};
use gobs_resource::{
    ResourceError,
    load::{self, AssetType},
};

use crate::{
    FrameGraph, GfxContext, PassMetaData,
    pass::{Attachment, AttachmentAccess, AttachmentType, RenderPassType},
};

// TODO: store in config file
const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphConfig {
    graphes: HashMap<String, Vec<String>>,
    passes: HashMap<String, RenderPassConfig>,
    attachments: HashMap<String, ImageAttachmentInfo>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenderPassConfig {
    pub ty: RenderPassType,
    pub config: String,
    #[serde(default)]
    attachments: HashMap<String, AttachmentInfo>,
    #[serde(default = "default_true")]
    enabled: bool,
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

    pub fn load_graph<F>(
        ctx: &mut GfxContext,
        filename: &str,
        name: &str,
        pass_config: F,
    ) -> Result<FrameGraph, ResourceError>
    where
        F: FnMut(&mut GfxContext, &PassMetaData, RenderPassType),
    {
        let data = load::load_string_sync(filename, AssetType::RESOURCES)?;

        Self::load_graph_with_data(ctx, &data, name, pass_config)
    }

    pub fn load_graph_with_data<F>(
        ctx: &mut GfxContext,
        data: &str,
        name: &str,
        mut pass_config: F,
    ) -> Result<FrameGraph, ResourceError>
    where
        F: FnMut(&mut GfxContext, &PassMetaData, RenderPassType),
    {
        let graph_config = Self::load_with_data(data)?;

        let mut graph = FrameGraph::new();

        // TODO: only register attachments used by passes
        for (attach_name, attach_config) in &graph_config.attachments {
            let attachment =
                Self::load_attachment(ctx, attach_config).ok_or(ResourceError::InvalidData)?;

            graph.register_attachment(ctx, attach_name, attachment);
        }

        tracing::debug!(target: logger::INIT, "Load graph: {}", "scene");

        for passname in &graph_config.graphes[name] {
            tracing::debug!(target: logger::INIT, "Load pass: {}", passname);

            let pass = Self::load_pass(ctx, &graph_config, passname, &mut pass_config)
                .unwrap_or_else(|| panic!("Failed to load pass {}", passname));

            let enabled = graph_config.passes.get(passname).is_some_and(|p| p.enabled);

            graph.register_pass(pass, enabled);
        }

        Ok(graph)
    }

    pub fn load_pass<F>(
        ctx: &mut GfxContext,
        graph: &GraphConfig,
        passname: &str,
        mut pass_config: F,
    ) -> Option<PassMetaData>
    where
        F: FnMut(&mut GfxContext, &PassMetaData, RenderPassType),
    {
        tracing::info!(target: logger::INIT, "Load pass: {}", passname);

        let pass = graph.passes.get(passname)?;

        let mut metadata = PassMetaData::new(passname, &pass.config);

        for (attach_name, attach_config) in &pass.attachments {
            let attachment = Self::load_attachment_usage(ctx, graph, attach_name, attach_config)?;
            metadata.add_attachment(attach_name, attachment);
        }

        pass_config(ctx, &metadata, pass.ty);

        Some(metadata)
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

    use gobs_core::{ConfigWriter as _, GobsConfig};
    use gobs_render_hal::RenderHalConfig;
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::{
        GfxContext, GraphConfig,
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

        let mut config = GobsConfig::default();
        config.register::<RenderHalConfig>();

        let mut ctx = GfxContext::new("test", None, config, false);

        let data = include_str!("../../../examples/resources/graph.ron");

        let graph = GraphConfig::load_with_data(data).unwrap();
        tracing::info!("Graph: {:?}", graph.graphes["scene"]);

        let graph = GraphConfig::load_graph_with_data(&mut ctx, data, "ui", |_, _, _| {}).unwrap();

        for pass in graph.passes {
            tracing::info!("Load pass: {}", &pass.pass.name);
        }
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_load_pass() {
        setup();

        let mut config = GobsConfig::default();
        config.register::<RenderHalConfig>();

        let mut ctx = GfxContext::new("test", None, config, false);

        let data = include_str!("../../../examples/resources/graph.ron");

        let graph_config = GraphConfig::load_with_data(data).unwrap();

        let _pass =
            GraphConfig::load_pass(&mut ctx, &graph_config, "forward", |_, _, _| {}).unwrap();
    }

    #[test]
    fn test_serialize() {
        setup();

        let pass_name = "bounds".to_string();

        let graph = GraphConfig {
            graphes: HashMap::from([("scene".to_string(), vec![pass_name.clone()])]),
            passes: HashMap::from([(
                pass_name.clone(),
                RenderPassConfig {
                    ty: RenderPassType::Material,
                    config: pass_name,
                    attachments: HashMap::from([(
                        "draw".to_string(),
                        AttachmentInfo::ColorAttachment {
                            access: AttachmentAccess::ReadWrite,
                            clear: true,
                        },
                    )]),
                    enabled: true,
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
