use std::collections::HashMap;

use gobs_render_graph::RenderPassType;
use gobs_render_hal::{GfxContext, UniformData as _};
use gobs_resource::{
    ResourceError, ResourceManager,
    load::{self, AssetType},
};
use serde::{Deserialize, Serialize};

use crate::{
    Pipeline,
    data::{RenderFlags, SceneDataLayout, SceneDataProp},
    pass::{
        PassData, compute::ComputePassData, material::MaterialPassData, present::PresentPassData,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PassConfig {
    compute: HashMap<String, ComputePassConfig>,
    material: HashMap<String, MaterialPassConfig>,
    present: HashMap<String, PresentPassConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComputePassConfig {
    pub pipeline: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialPassConfig {
    pub pipeline: Option<String>,
    #[serde(default)]
    pub scene_layout: Vec<SceneDataProp>,
    #[serde(default)]
    pub flags: RenderFlags,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PresentPassConfig {
    pub target: String,
}

impl PassConfig {
    fn load_with_data(data: &str) -> Result<Self, ResourceError> {
        let options = ron::options::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        options.from_str(data).map_err(|e| {
            tracing::error!("{}", e);
            ResourceError::InvalidData
        })
    }

    pub fn load_passes(filename: &str) -> Result<PassConfig, ResourceError> {
        let data = load::load_string_sync(filename, AssetType::RESOURCES)?;

        Self::load_with_data(&data)
    }

    pub fn load_pass_data(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        config: &PassConfig,
        pass_config_name: &str,
        pass_type: RenderPassType,
    ) -> Option<PassData> {
        match pass_type {
            RenderPassType::Compute => Self::load_compute_pass_data(
                ctx,
                resource_manager,
                &config.compute,
                pass_config_name,
            ),
            RenderPassType::Material => Self::load_material_pass_data(
                ctx,
                resource_manager,
                &config.material,
                pass_config_name,
            ),
            RenderPassType::Present => {
                Self::load_present_pass_data(&config.present, pass_config_name)
            }
        }
    }

    fn load_compute_pass_data(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        config: &HashMap<String, ComputePassConfig>,
        pass_config_name: &str,
    ) -> Option<PassData> {
        let pass_config = config.get(pass_config_name)?;

        let pipeline = resource_manager
            .get_by_name::<Pipeline>(&pass_config.pipeline)
            .and_then(|handle| resource_manager.get_data(ctx, &handle).ok())
            .map(|data| data.data.pipeline)
            .expect("No pipeline for compute pass");

        Some(PassData::Compute(ComputePassData::new(pipeline)))
    }

    fn load_material_pass_data(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        config: &HashMap<String, MaterialPassConfig>,
        pass_config_name: &str,
    ) -> Option<PassData> {
        let pass_config = config.get(pass_config_name)?;

        let pipeline = pass_config.pipeline.as_ref().and_then(|pipeline| {
            resource_manager
                .get_by_name::<Pipeline>(pipeline)
                .and_then(|handle| resource_manager.get_data(ctx, &handle).ok())
                .map(|data| data.data.pipeline)
        });

        let render_flags = pass_config.flags;

        let mut scene_layout = SceneDataLayout::new();
        for prop in &pass_config.scene_layout {
            scene_layout = scene_layout.prop(*prop);
        }

        Some(PassData::Material(MaterialPassData::new(
            ctx,
            pass_config_name,
            pipeline,
            render_flags,
            scene_layout,
        )))
    }

    fn load_present_pass_data(
        config: &HashMap<String, PresentPassConfig>,
        pass_config_name: &str,
    ) -> Option<PassData> {
        let pass_config = config.get(pass_config_name)?;

        Some(PassData::Present(PresentPassData::new(&pass_config.target)))
    }
}

#[cfg(test)]
mod tests {
    use gobs_core::{ConfigWriter as _, GobsConfig};
    use gobs_render_graph::RenderPassType;
    use gobs_render_hal::{RenderHalConfig, create_hal};
    use gobs_resource::ResourceManager;
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::pass::{PassData, pass_loader::PassConfig};

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

        let mut ctx = create_hal("test", None, config, false);
        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight());

        let data = include_str!("../../../examples/resources/pass_config.ron");

        let pass_config = PassConfig::load_with_data(data).unwrap();

        let pass_data = PassConfig::load_pass_data(
            ctx.as_mut(),
            &mut resource_manager,
            &pass_config,
            "ui",
            RenderPassType::Material,
        )
        .unwrap();

        assert!(matches!(pass_data, PassData::Material(_)));
    }
}
