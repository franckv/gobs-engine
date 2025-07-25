use winit::window::Window;

use gobs_render::{
    GfxContext, Material, MaterialLoader, Mesh, MeshLoader, RenderError, Renderer, Texture,
    TextureLoader,
};
use gobs_render_graph::{Pipeline, PipelineLoader};
use gobs_resource::manager::ResourceManager;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
}

pub struct GameContext {
    pub app_info: AppInfo,
    pub resource_manager: ResourceManager,
    pub renderer: Renderer,
}

impl GameContext {
    pub fn new(name: &str, window: Option<Window>, validation: bool) -> Result<Self, RenderError> {
        let gfx = GfxContext::new(name, window, validation)?;
        let mut resource_manager = ResourceManager::new(gfx.frames_in_flight);

        let texture_loader = TextureLoader::new(gfx.device.clone());
        resource_manager.register_resource::<Texture>(texture_loader);

        let mesh_loader = MeshLoader::new(gfx.device.clone());
        resource_manager.register_resource::<Mesh>(mesh_loader);

        let pipeline_loader = PipelineLoader::new(gfx.device.clone());
        resource_manager.register_resource::<Pipeline>(pipeline_loader);

        let material_loader = MaterialLoader::new();
        resource_manager.register_resource::<Material>(material_loader);

        let renderer = Renderer::new(gfx, &mut resource_manager);

        Ok(Self {
            app_info: AppInfo {
                name: name.to_string(),
            },
            resource_manager,
            renderer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn update(&mut self, delta: f32) {
        self.renderer.update(delta);
        self.resource_manager.update::<Mesh>();
        self.resource_manager.update::<Texture>();
        self.resource_manager.update::<Pipeline>();
    }

    pub fn close(&mut self) {
        self.renderer.wait();
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop context");
    }
}
