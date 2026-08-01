use winit::window::Window;

use gobs_core::{Config, logger};
use gobs_render::{
    GfxContext, Material, MaterialInstance, MaterialInstanceLoader, MaterialLoader, Mesh,
    MeshLoader, Pipeline, PipelineLoader, RenderConfig, Renderer, Texture, TextureLoader,
};
use gobs_resource::ResourceManager;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
}

pub trait GobsContext {
    fn new(name: &str, config: Config, window: Option<Window>, validation: bool) -> Self;
    fn resize(&mut self);
    fn update(&mut self, delta: f32);
    fn close(&mut self);
    fn is_minimized(&self) -> bool;
    fn request_redraw(&mut self);
}

pub struct GameContext {
    pub app_info: AppInfo,
    pub config: Config,
    pub resource_manager: ResourceManager,
    pub renderer: Renderer,
}

impl GobsContext for GameContext {
    fn new(name: &str, config: Config, window: Option<Window>, validation: bool) -> Self {
        let frames_in_flight = config.get_int(RenderConfig::FramesInFlight) as usize;

        let mut gfx = GfxContext::new(name, window, frames_in_flight, validation);
        let mut resource_manager = ResourceManager::new(gfx.frames_in_flight());

        let texture_loader = TextureLoader::new(&mut gfx);
        resource_manager.register_resource::<Texture>(texture_loader);

        let mesh_loader = MeshLoader::new(&mut gfx);
        resource_manager.register_resource::<Mesh>(mesh_loader);

        let pipeline_loader = PipelineLoader::new();
        resource_manager.register_resource::<Pipeline>(pipeline_loader);

        let material_loader = MaterialLoader::new();
        resource_manager.register_resource::<Material>(material_loader);

        let material_instance_loader = MaterialInstanceLoader::new();
        resource_manager.register_resource::<MaterialInstance>(material_instance_loader);

        let renderer = Renderer::new(gfx, &config, &mut resource_manager);

        Self {
            app_info: AppInfo {
                name: name.to_string(),
            },
            config,
            resource_manager,
            renderer,
        }
    }

    fn resize(&mut self) {
        self.renderer.resize();
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn update(&mut self, delta: f32) {
        self.renderer.update(delta);
        self.resource_manager
            .update::<Texture>(self.renderer.gfx.hal_mut());
        self.resource_manager
            .update::<Mesh>(self.renderer.gfx.hal_mut());
        self.resource_manager
            .update::<Pipeline>(self.renderer.gfx.hal_mut());
        self.resource_manager
            .update::<Material>(self.renderer.gfx.hal_mut());
        self.resource_manager
            .update::<MaterialInstance>(self.renderer.gfx.hal_mut());
    }

    fn close(&mut self) {
        self.renderer.wait();
    }

    fn is_minimized(&self) -> bool {
        self.renderer.gfx.is_minimized()
    }

    fn request_redraw(&mut self) {
        self.renderer.gfx.request_redraw();
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop context");
    }
}
