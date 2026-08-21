use gobs_core::{ConfigDefault, ConfigWriter as _, GobsConfig};

pub enum RenderConfig {
    GraphFileName,
    GraphName,
    PipelineFileName,
    FramesInFlight,
    LoadGraph,
    TextureArraySize,
}

impl AsRef<str> for RenderConfig {
    fn as_ref(&self) -> &str {
        match self {
            RenderConfig::GraphFileName => "config.render.graph.filename",
            RenderConfig::GraphName => "config.render.graph.name",
            RenderConfig::PipelineFileName => "config.render.pipeline.filename",
            RenderConfig::FramesInFlight => "config.render.frames_in_flight",
            RenderConfig::LoadGraph => "config.render.graph.load",
            RenderConfig::TextureArraySize => "config.render.textures.array_size",
        }
    }
}

impl ConfigDefault for RenderConfig {
    fn register_defaults(config: &mut GobsConfig) {
        config.set_string(RenderConfig::GraphFileName, "graph.ron");
        config.set_string(RenderConfig::GraphName, "scene");
        config.set_string(RenderConfig::PipelineFileName, "pipelines.ron");
        config.set_int(RenderConfig::FramesInFlight, 2);
        config.set_bool(RenderConfig::LoadGraph, true);
        config.set_int(RenderConfig::TextureArraySize, 256);
    }
}
