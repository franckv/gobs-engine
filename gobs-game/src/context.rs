use winit::window::Window;

use gobs_render::{GfxContext, RenderError, Texture, TextureLoader};
use gobs_resource::manager::ResourceManager;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
}

pub struct GameContext {
    pub app_info: AppInfo,
    pub gfx: GfxContext,
    pub resource_manager: ResourceManager,
}

impl GameContext {
    pub fn new(name: &str, window: Option<Window>, validation: bool) -> Result<Self, RenderError> {
        let gfx = GfxContext::new(name, window, validation)?;
        let mut resource_manager = ResourceManager::default();

        let texture_loader = TextureLoader::new(gfx.device.clone());
        resource_manager.register_loader::<Texture>(texture_loader);

        Ok(Self {
            app_info: AppInfo {
                name: name.to_string(),
            },
            gfx,
            resource_manager,
        })
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop context");
    }
}
