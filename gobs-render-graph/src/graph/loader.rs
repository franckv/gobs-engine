use std::collections::HashMap;

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{ImageLayout, ImageUsage};
use gobs_resource::load::{self, AssetType};
use serde::{Deserialize, Serialize};

use gobs_render_low::{
    GfxContext, ObjectDataLayout, ObjectDataProp, SceneDataLayout, SceneDataProp,
};

use crate::pass::{AttachmentAccess, AttachmentType, RenderPassType, material::MaterialPass};

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
    pub fn load_pass(ctx: &GfxContext, filename: &str, passname: &str) -> Option<MaterialPass> {
        let data = load::load_string_sync(filename, AssetType::RESOURCES).ok()?;

        Self::load_pass_with_data(ctx, &data, passname)
    }

    fn load_pass_with_data(ctx: &GfxContext, data: &str, passname: &str) -> Option<MaterialPass> {
        let graph: GraphConfig = ron::from_str(data).ok()?;

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
            object_layout.build(),
            scene_layout.build(),
            pass.render_transparent,
            pass.render_opaque,
        );

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

        Some(material_pass)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_render_low::GfxContext;

    use crate::{
        GraphConfig,
        graph::loader::{AttachmentInfo, RenderPassConfig},
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
        let graph_config = include_str!("../../../examples/resources/graph.ron");

        let _pass = GraphConfig::load_pass_with_data(&ctx, graph_config, "bounds").unwrap();
    }

    #[test]
    fn test_serialize() {
        setup();

        let pass_name = "bounds".to_string();

        let mut graph = GraphConfig {
            schedule: vec![pass_name.clone()],
            passes: HashMap::new(),
            attachments: HashMap::new(),
        };

        let mut pass = RenderPassConfig {
            ty: RenderPassType::Material,
            attachments: HashMap::new(),
            object_layout: Vec::new(),
            scene_layout: Vec::new(),
            render_transparent: true,
            render_opaque: true,
        };

        let attach = AttachmentInfo::ColorAttachment {
            access: AttachmentAccess::ReadWrite,
            clear: true,
        };

        pass.attachments.insert("draw".to_string(), attach);

        graph.passes.insert(pass_name, pass);

        let ron = ron::ser::to_string_pretty(&graph, ron::ser::PrettyConfig::default()).unwrap();

        tracing::info!("Load data: {}", ron);
    }

    #[test]
    fn test_deserialize() {
        setup();

        let graph_config = include_str!("../../../examples/resources/graph.ron");
        let _graph: GraphConfig = ron::from_str(graph_config).unwrap();
    }
}
