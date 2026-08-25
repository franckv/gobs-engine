use gobs_core::{ConfigDefault, ConfigWriter as _, GobsConfig};

pub enum RenderConfig {
    GraphFileName,
    GraphName,
    PipelineFileName,
    PassConfigFileName,
    LoadGraph,
}

impl AsRef<str> for RenderConfig {
    fn as_ref(&self) -> &str {
        match self {
            RenderConfig::GraphFileName => "config.render.graph.filename",
            RenderConfig::GraphName => "config.render.graph.name",
            RenderConfig::PipelineFileName => "config.render.pipeline.filename",
            RenderConfig::LoadGraph => "config.render.graph.load",
            RenderConfig::PassConfigFileName => "config.render.passconfig.filename",
        }
    }
}

impl ConfigDefault for RenderConfig {
    fn register_defaults(config: &mut GobsConfig) {
        config.set_string(RenderConfig::GraphFileName, "graph.ron");
        config.set_string(RenderConfig::GraphName, "scene");
        config.set_string(RenderConfig::PipelineFileName, "pipelines.ron");
        config.set_string(RenderConfig::PassConfigFileName, "pass_config.ron");
        config.set_bool(RenderConfig::LoadGraph, true);
    }
}
