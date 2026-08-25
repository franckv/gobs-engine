use std::fmt::Debug;

use gobs_render_hal::create_hal;
use winit::window::Window;

use gobs_assets::gltf_load;
use gobs_core::{GobsConfig, ImageExtent2D, Input, logger};
use gobs_egui::UIRenderer;
use gobs_render::{
    GfxContext, Material, MaterialInstance, MaterialInstanceLoader, MaterialLoader,
    MaterialsConfig, Mesh, MeshLoader, Pipeline, PipelineLoader, RenderBuilder, RenderError,
    RenderHAL, RenderMaterialBuilder, RenderMeshBuilder, RenderModelBuilder, RenderTextureBuilder,
    Renderer, Texture, TextureLoader,
};
use gobs_resource::{ResourceHandle, ResourceManager, load};
use gobs_scene::{SceneBuilder, graph::scenegraph::SceneGraph};

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: String,
}

#[allow(async_fn_in_trait)]
pub trait GobsContext {
    fn new(name: &str, config: GobsConfig, window: Option<Window>, validation: bool) -> Self;
    fn resize(&mut self);
    fn pre_update(&mut self, delta: f32);
    fn post_update(&mut self, delta: f32);
    fn close(&mut self);
    fn render(&mut self) -> Result<RenderBuilder<'_>, RenderError>;
    fn input(&mut self, input: Input);
    fn frame_number(&self) -> usize;

    fn is_minimized(&self) -> bool;
    fn request_redraw(&mut self);
    fn lock_mouse(&mut self, lock: bool);
    fn extent(&self) -> ImageExtent2D;
    fn gfx(&self) -> &dyn RenderHAL;
    fn gfx_mut(&mut self) -> &mut dyn RenderHAL;

    fn draw_ui<F>(&mut self, delta: f32, callback: F)
    where
        F: FnMut(&mut egui::Ui, &AppInfo, &mut ResourceManager, &mut Renderer);

    fn config(&self) -> GobsConfig;

    fn new_scene(&self) -> SceneBuilder<'_>;

    fn new_model<'a>(&'a mut self, name: &'a str) -> RenderModelBuilder<'a>;
    fn new_material<'a>(&'a mut self, name: &'a str) -> RenderMaterialBuilder<'a>;
    fn new_mesh<'a>(&'a mut self, name: &'a str) -> RenderMeshBuilder<'a>;
    fn new_texture<'a>(&'a mut self, name: &'a str) -> RenderTextureBuilder<'a>;
    fn free_material(&mut self, material: ResourceHandle<MaterialInstance>, delete_textures: bool);
    fn free_mesh(&mut self, mesh: ResourceHandle<Mesh>);
    fn free_texture(&mut self, mesh: ResourceHandle<Texture>);

    async fn load_material(&mut self, filename: &str);
    fn load_gltf(&mut self, filename: &str) -> SceneGraph;
}

pub struct GameContext {
    app_info: AppInfo,
    config: GobsConfig,
    resource_manager: ResourceManager,
    renderer: Renderer,
    ui: UIRenderer,
}

impl GobsContext for GameContext {
    fn new(name: &str, config: GobsConfig, window: Option<Window>, validation: bool) -> Self {
        let mut gfx = create_hal(name, window, config.clone(), validation);
        let mut resource_manager = ResourceManager::new(gfx.frames_in_flight());

        let texture_loader = TextureLoader::new(gfx.as_mut());
        resource_manager.register_resource::<Texture>(texture_loader);

        let mesh_loader = MeshLoader::new(gfx.as_mut());
        resource_manager.register_resource::<Mesh>(mesh_loader);

        let pipeline_loader = PipelineLoader::new();
        resource_manager.register_resource::<Pipeline>(pipeline_loader);

        let material_loader = MaterialLoader::new();
        resource_manager.register_resource::<Material>(material_loader);

        let material_instance_loader = MaterialInstanceLoader::new();
        resource_manager.register_resource::<MaterialInstance>(material_instance_loader);

        let ui = UIRenderer::new(gfx.as_ref(), config.clone(), &mut resource_manager);

        let renderer = Renderer::new(gfx, config.clone(), &mut resource_manager);

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
    fn pre_update(&mut self, _delta: f32) {
        self.resource_manager
            .update::<Texture>(self.renderer.gfx.as_mut());
        self.resource_manager
            .update::<Mesh>(self.renderer.gfx.as_mut());
        self.resource_manager
            .update::<Pipeline>(self.renderer.gfx.as_mut());
        self.resource_manager
            .update::<Material>(self.renderer.gfx.as_mut());
        self.resource_manager
            .update::<MaterialInstance>(self.renderer.gfx.as_mut());
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn post_update(&mut self, _delta: f32) {
        self.ui
            .update(self.renderer.gfx.as_mut(), &mut self.resource_manager);
    }

    fn close(&mut self) {
        self.renderer.wait();
    }

    fn render(&mut self) -> Result<RenderBuilder<'_>, RenderError> {
        RenderBuilder::new(&mut self.renderer, &mut self.resource_manager)
            .with_renderable(&self.ui, gobs_render::RenderType::Ui)
    }

    fn input(&mut self, input: Input) {
        self.ui.input(input);
    }

    fn frame_number(&self) -> usize {
        self.renderer.frame_number()
    }

    fn is_minimized(&self) -> bool {
        self.renderer.gfx.is_minimized()
    }

    fn request_redraw(&mut self) {
        self.renderer.gfx.request_redraw();
    }

    fn lock_mouse(&mut self, lock: bool) {
        self.renderer.gfx.lock_mouse(lock);
    }

    fn extent(&self) -> ImageExtent2D {
        self.renderer.extent()
    }

    fn gfx(&self) -> &GfxContext<'_> {
        self.renderer.gfx.as_ref()
    }

    fn gfx_mut(&mut self) -> &mut GfxContext<'_> {
        self.renderer.gfx.as_mut()
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

    fn config(&self) -> GobsConfig {
        self.config.clone()
    }

    fn new_scene(&self) -> SceneBuilder<'_> {
        SceneBuilder::new(self.renderer.gfx.as_ref())
    }

    fn new_model<'a>(&'a mut self, name: &'a str) -> RenderModelBuilder<'a> {
        RenderModelBuilder::new(&mut self.resource_manager, name)
    }

    fn new_material<'a>(&'a mut self, name: &'a str) -> RenderMaterialBuilder<'a> {
        RenderMaterialBuilder::new(&mut self.resource_manager, name)
    }

    fn new_mesh<'a>(&'a mut self, name: &'a str) -> RenderMeshBuilder<'a> {
        RenderMeshBuilder::new(&mut self.resource_manager, name)
    }

    fn new_texture<'a>(&'a mut self, name: &'a str) -> RenderTextureBuilder<'a> {
        RenderTextureBuilder::new(&mut self.resource_manager, name)
    }

    fn free_material(&mut self, material: ResourceHandle<MaterialInstance>, delete_textures: bool) {
        if delete_textures {
            let textures = self
                .resource_manager
                .get(&material)
                .properties
                .textures
                .clone();

            for texture in textures {
                self.resource_manager.schedule_removal(&texture);
            }
        }

        self.resource_manager.schedule_removal(&material);
    }

    fn free_mesh(&mut self, mesh: ResourceHandle<Mesh>) {
        self.resource_manager.schedule_removal(&mesh);
    }

    fn free_texture(&mut self, texture: ResourceHandle<Texture>) {
        self.resource_manager.schedule_removal(&texture);
    }

    async fn load_material(&mut self, filename: &str) {
        MaterialsConfig::load_resources(self.config.clone(), filename, &mut self.resource_manager)
            .await;
    }

    fn load_gltf(&mut self, filename: &str) -> SceneGraph {
        let filename = load::get_asset_dir(filename, load::AssetType::MODEL).unwrap();
        let mut gltf_loader =
            gltf_load::GLTFLoader::new(self.config.clone(), &mut self.resource_manager).unwrap();

        gltf_loader
            .load(self.config(), &mut self.resource_manager, filename)
            .expect("Load gltf");

        gltf_loader.scene
    }
}

impl Drop for GameContext {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop context");
    }
}
