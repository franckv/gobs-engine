use winit::window::Window;

use gobs_core::{Config, logger};
use gobs_egui::UIRenderer;
use gobs_render::{
    GfxContext, Material, MaterialInstance, MaterialInstanceLoader, MaterialLoader, Mesh,
    MeshLoader, Pipeline, PipelineLoader, RenderBuilder, RenderConfig, RenderError, Renderer,
    Texture, TextureLoader,
};
use gobs_resource::ResourceManager;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
}

pub trait GobsContext {
    fn new(name: &str, config: Config, window: Option<Window>, validation: bool) -> Self;
    fn resize(&mut self);
    fn pre_update(&mut self, delta: f32);
    fn post_update(&mut self, delta: f32);
    fn close(&mut self);
    fn is_minimized(&self) -> bool;
    fn request_redraw(&mut self);
    fn render(&mut self) -> Result<RenderBuilder<'_>, RenderError>;
    fn draw_ui<F>(&mut self, delta: f32, callback: F)
    where
        F: FnMut(&mut egui::Ui, &AppInfo, &mut ResourceManager, &mut Renderer);
}

pub struct GameContext {
    pub app_info: AppInfo,
    pub config: Config,
    pub resource_manager: ResourceManager,
    pub renderer: Renderer,
    pub ui: UIRenderer,
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

        let ui = UIRenderer::new(&gfx, &mut resource_manager);

        let renderer = Renderer::new(gfx, &config, &mut resource_manager);

        Self {
            app_info: AppInfo {
                name: name.to_string(),
            },
            config,
            resource_manager,
            renderer,
            ui,
        }
    }

    fn resize(&mut self) {
        self.renderer.resize();

        let (width, height) = self.renderer.extent().into();
        self.ui.resize(width, height);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn pre_update(&mut self, delta: f32) {
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn post_update(&mut self, _delta: f32) {
        self.ui
            .update(&mut self.renderer.gfx, &mut self.resource_manager);
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

    fn render(&mut self) -> Result<RenderBuilder<'_>, RenderError> {
        RenderBuilder::new(&mut self.renderer, &mut self.resource_manager)
            .with_renderable(&self.ui, gobs_render::RenderType::Ui)
    }

    fn draw_ui<F>(&mut self, delta: f32, mut callback: F)
    where
        F: FnMut(&mut egui::Ui, &AppInfo, &mut ResourceManager, &mut Renderer),
    {
        self.ui.draw_ui(delta, |ui| {
            callback(
                ui,
                &self.app_info,
                &mut self.resource_manager,
                &mut self.renderer,
            )
        });
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop context");
    }
}
