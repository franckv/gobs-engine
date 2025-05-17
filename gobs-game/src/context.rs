use winit::window::Window;

use gobs_render::{GfxContext, Mesh, MeshLoader, RenderError, Texture, TextureLoader};
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
        let mut resource_manager = ResourceManager::new(gfx.frames_in_flight);

        let texture_loader = TextureLoader::new(gfx.device.clone());
        resource_manager.register_resource::<Texture>(texture_loader);

        let mesh_loader = MeshLoader::new(gfx.device.clone());
        resource_manager.register_resource::<Mesh>(mesh_loader);

        Ok(Self {
            app_info: AppInfo {
                name: name.to_string(),
            },
            gfx,
            resource_manager,
        })
    }

    pub fn update(&mut self) {
        self.resource_manager.update::<Mesh>();
        self.resource_manager.update::<Texture>();
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop context");
    }
}
