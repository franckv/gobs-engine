use std::{collections::HashMap, sync::Arc};

use egui::{
    ColorImage, Event, FullOutput, Modifiers, MouseWheelUnit, PointerButton, RawInput, Rect,
    TextureId, TouchPhase,
    epaint::{ImageDelta, Primitive},
};
use parking_lot::RwLock;
use tracing::Level;

use gobs_core::{
    GobsConfig, ImageExtent2D, ImageFormat, Input, Key, MouseButton, Transform, logger,
};
use gobs_graphics::BoundingBox;
use gobs_render::{
    GfxContext, Material, MaterialInstance, MaterialInstanceProperties, MaterialsConfig, Model,
    RenderBatch, RenderFlags, RenderMeshBuilder, RenderModelBuilder, Renderable, Texture,
    TextureProperties, TextureUpdate,
};
use gobs_resource::{
    ResourceManager, {ResourceError, ResourceHandle, ResourceLifetime},
};

const PIXEL_PER_POINT: f32 = 1.;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}

pub struct UIRenderer {
    ectx: egui::Context,
    width: f32,
    height: f32,
    material: ResourceHandle<Material>,
    font_texture: HashMap<TextureId, ResourceHandle<MaterialInstance>>,
    input: Vec<Input>,
    mouse_position: (f32, f32),
    output: RwLock<Option<FullOutput>>,
}

impl UIRenderer {
    pub fn new(
        ctx: &GfxContext,
        config: GobsConfig,
        resource_manager: &mut ResourceManager,
    ) -> Self {
        let ectx = egui::Context::default();

        let (width, height): (f32, f32) = ctx.get_extent().into();

        ectx.set_pixels_per_point(PIXEL_PER_POINT);

        MaterialsConfig::load_resources_sync(config, "ui_materials.ron", resource_manager);

        let material = resource_manager.get_by_name("ui").unwrap();

        UIRenderer {
            ectx,
            width,
            height,
            material,
            font_texture: HashMap::new(),
            input: Vec::new(),
            mouse_position: (0., 0.),
            output: RwLock::new(None),
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn draw_ui<F>(&mut self, delta: f32, callback: F)
    where
        F: FnMut(&mut egui::Ui),
    {
        let input = self.prepare_inputs(delta);
        let output = self.ectx.run_ui(input, callback);

        self.output.write().replace(output);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn update(&mut self, ctx: &mut GfxContext, resource_manager: &mut ResourceManager) {
        self.update_textures(resource_manager);
        self.cleanup_textures(ctx, resource_manager);
    }

    fn get_key(key: Key) -> egui::Key {
        match key {
            Key::N0 => egui::Key::Num0,
            Key::N1 => egui::Key::Num1,
            Key::N2 => egui::Key::Num2,
            Key::N3 => egui::Key::Num3,
            Key::N4 => egui::Key::Num4,
            Key::N5 => egui::Key::Num5,
            Key::N6 => egui::Key::Num6,
            Key::N7 => egui::Key::Num7,
            Key::N8 => egui::Key::Num8,
            Key::N9 => egui::Key::Num9,
            Key::Minus => egui::Key::Minus,
            Key::Plus => egui::Key::Plus,
            Key::Equals => egui::Key::Equals,
            Key::A => egui::Key::A,
            Key::B => egui::Key::B,
            Key::C => egui::Key::C,
            Key::D => egui::Key::D,
            Key::E => egui::Key::E,
            Key::F => egui::Key::F,
            Key::G => egui::Key::G,
            Key::H => egui::Key::H,
            Key::I => egui::Key::I,
            Key::J => egui::Key::J,
            Key::K => egui::Key::K,
            Key::L => egui::Key::L,
            Key::M => egui::Key::M,
            Key::N => egui::Key::N,
            Key::O => egui::Key::O,
            Key::P => egui::Key::P,
            Key::Q => egui::Key::Q,
            Key::R => egui::Key::R,
            Key::S => egui::Key::S,
            Key::T => egui::Key::T,
            Key::U => egui::Key::U,
            Key::V => egui::Key::V,
            Key::W => egui::Key::W,
            Key::X => egui::Key::X,
            Key::Y => egui::Key::Y,
            Key::Z => egui::Key::Z,
            Key::Backspace => egui::Key::Backspace,
            Key::Return => egui::Key::Enter,
            Key::Space => egui::Key::Space,
            Key::Tab => egui::Key::Tab,
            Key::Left => egui::Key::ArrowLeft,
            Key::Right => egui::Key::ArrowRight,
            Key::Up => egui::Key::ArrowUp,
            Key::Down => egui::Key::ArrowDown,
            Key::PageUp => egui::Key::PageUp,
            Key::PageDown => egui::Key::PageDown,
            Key::LShift => egui::Key::ShiftLeft,
            _ => egui::Key::Escape,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn prepare_inputs(&mut self, delta: f32) -> RawInput {
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Default::default(),
                [self.width, self.height].into(),
            )),
            predicted_dt: delta,
            ..Default::default()
        };

        self.input.drain(..).for_each(|e| match e {
            Input::KeyPressed(key) => {
                input.events.push(Event::Key {
                    key: Self::get_key(key),
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                    physical_key: None,
                });
            }
            Input::KeyReleased(key) => {
                input.events.push(Event::Key {
                    key: Self::get_key(key),
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                    physical_key: None,
                });
            }
            Input::Char(c) => {
                input.events.push(Event::Text(c.to_string()));
            }
            Input::MousePressed(button) => match button {
                MouseButton::Left => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    });
                }
                MouseButton::Right => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Secondary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    });
                }
                MouseButton::Middle => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Middle,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    });
                }
                _ => {}
            },
            Input::MouseReleased(button) => match button {
                MouseButton::Left => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    });
                }
                MouseButton::Right => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Secondary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    });
                }
                MouseButton::Middle => {
                    input.events.push(Event::PointerButton {
                        pos: self.mouse_position.into(),
                        button: PointerButton::Middle,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    });
                }
                _ => {}
            },
            Input::MouseWheel(delta) => input.events.push(Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: (0., delta).into(),
                phase: TouchPhase::Move,
                modifiers: Modifiers::NONE,
            }),
            Input::CursorMoved(x, y) => {
                self.mouse_position = (x as f32, y as f32);
                input
                    .events
                    .push(Event::PointerMoved(self.mouse_position.into()));
            }
            _ => (),
        });

        input
    }

    pub fn input(&mut self, input: Input) {
        self.input.push(input);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn update_textures(&mut self, resource_manager: &mut ResourceManager) {
        if let Some(output) = self.output.write().as_ref() {
            for (id, img) in &output.textures_delta.set {
                tracing::debug!(target: logger::UI, "New texture {:?}", id);
                if img.pos.is_some() {
                    tracing::debug!(target: logger::UI, "Patching texture");
                    let texture = self.patch_texture(
                        resource_manager,
                        self.font_texture
                            .get(id)
                            .cloned()
                            .expect("Cannot update unallocated texture"),
                        img,
                    );

                    let material_properties =
                        MaterialInstanceProperties::new("font", self.material).textures(&[texture]);

                    let material_instance = resource_manager.add::<MaterialInstance>(
                        material_properties,
                        ResourceLifetime::Static,
                        false,
                    );

                    *self.font_texture.get_mut(id).unwrap() = material_instance;
                } else if self.font_texture.contains_key(id) {
                    tracing::warn!(target: logger::UI, "Replacing Font texture {:?}", id);

                    let old_material_handle = self.font_texture.remove(id).unwrap();

                    if let Ok(old_material) = resource_manager.get(&old_material_handle) {
                        let old_textures = old_material.properties.textures.clone();
                        for old_texture in old_textures {
                            resource_manager.schedule_removal(&old_texture);
                        }
                        resource_manager.schedule_removal(&old_material_handle);

                        let texture = self.decode_texture(resource_manager, img);

                        self.font_texture.insert(*id, texture);
                        tracing::trace!(target: logger::UI, "Texture reloaded");
                    };
                } else {
                    tracing::debug!(target: logger::UI, "Allocate new texture");
                    let texture = self.decode_texture(resource_manager, img);
                    self.font_texture.insert(*id, texture);
                    tracing::trace!(target: logger::UI, "Texture loaded");
                }
            }
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn cleanup_textures(&mut self, ctx: &mut GfxContext, resource_manager: &mut ResourceManager) {
        if let Some(output) = self.output.write().as_ref() {
            for id in &output.textures_delta.free {
                tracing::debug!(target: logger::UI, "Remove texture {:?}", id);

                let material_handle = self.font_texture.remove(id);
                let mut to_remove = Vec::new();

                if let Some(material_handle) = material_handle {
                    let material = resource_manager.get_data(ctx, &material_handle);
                    if let Ok(material) = material {
                        for texture in &material.properties.textures {
                            to_remove.push(*texture);
                        }
                    }
                    resource_manager.schedule_removal(&material_handle);
                }

                for texture in to_remove {
                    resource_manager.schedule_removal(&texture);
                }
            }
        }
    }

    fn import_texture(
        &self,
        resource_manager: &mut ResourceManager,
        texture_handle: ResourceHandle<Texture>,
    ) -> ResourceHandle<MaterialInstance> {
        let material_properties =
            MaterialInstanceProperties::new("font", self.material).textures(&[texture_handle]);

        resource_manager.add::<MaterialInstance>(
            material_properties,
            ResourceLifetime::Static,
            false,
        )
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn decode_texture(
        &self,
        resource_manager: &mut ResourceManager,
        img: &ImageDelta,
    ) -> ResourceHandle<MaterialInstance> {
        match &img.image {
            egui::ImageData::Color(color) => {
                let texture_handle = self.decode_image(resource_manager, color);

                self.import_texture(resource_manager, texture_handle)
            }
        }
    }

    pub fn decode_image(
        &self,
        resource_manager: &mut ResourceManager,
        color: &ColorImage,
    ) -> ResourceHandle<Texture> {
        let bytes: Vec<u8> = bytemuck::cast_slice(color.pixels.as_ref()).to_vec();

        let texture_properties = TextureProperties::with_data(
            "Font texture",
            ImageFormat::R8g8b8a8Srgb,
            bytes,
            ImageExtent2D::new(color.width() as u32, color.height() as u32),
        );

        resource_manager.add(texture_properties, ResourceLifetime::Static, false)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn patch_texture(
        &self,
        resource_manager: &mut ResourceManager,
        material: ResourceHandle<MaterialInstance>,
        img: &ImageDelta,
    ) -> ResourceHandle<Texture> {
        match &img.image {
            egui::ImageData::Color(color) => {
                let bytes: &[u8] = bytemuck::cast_slice(color.pixels.as_ref());

                let pos = img.pos.expect("Can only patch texture with start position");

                tracing::trace!(target: logger::UI,
                    "Patching texture origin: {}/{}, size: {}/{} len={}",
                    color.width(),
                    color.height(),
                    pos[0],
                    pos[1],
                    bytes.len()
                );

                let texture = resource_manager
                    .get(&material)
                    .expect("UI material not registered")
                    .properties
                    .textures[0];
                let handle = resource_manager
                    .replace(&texture)
                    .expect("UI texture not registered");
                let texture = resource_manager
                    .get_mut(&handle)
                    .expect("UI texture image not registered");
                tracing::trace!(target: logger::UI,
                    "Patching texture original size: {:?}",
                    texture.properties.format.extent
                );

                texture.patch(
                    pos[0] as u32,
                    pos[1] as u32,
                    color.width() as u32,
                    color.height() as u32,
                    bytes,
                )
            }
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load_model(
        &self,
        resource_manager: &mut ResourceManager,
        output: FullOutput,
    ) -> Option<Arc<Model>> {
        tracing::debug!(target: logger::UI, "Loading model");

        let span = tracing::span!(target: logger::PROFILE, Level::TRACE, "Tesselate").entered();

        let primitives = self.ectx.tessellate(output.shapes, PIXEL_PER_POINT);

        span.exit();

        tracing::debug!(target: logger::UI, "Load {} primitives", primitives.len());

        if primitives.is_empty() {
            return None;
        }

        let mut layer = 1;

        let mut meshes = Vec::new();

        for primitive in &primitives {
            let span =
                tracing::span!(target: logger::PROFILE, Level::TRACE, "Primitive", "{}", layer)
                    .entered();

            if let Primitive::Mesh(m) = &primitive.primitive {
                tracing::trace!(target: logger::UI,
                    "Primitive: {} vertices, {} indices, texture id: {:?}",
                    m.vertices.len(),
                    m.indices.len(),
                    m.texture_id
                );

                if !self.font_texture.contains_key(&m.texture_id) {
                    tracing::debug!(target: logger::UI, "Missing texture: {:?}", m.texture_id);
                    continue;
                }

                let mut vertices = Vec::with_capacity(m.vertices.len());

                for vertex in &m.vertices {
                    vertices.push(UIVertex {
                        position: [
                            vertex.pos.x.min(self.width),
                            (self.height - vertex.pos.y).min(self.height),
                            0.,
                        ],
                        color: [
                            vertex.color.r() as f32 / 255.,
                            vertex.color.g() as f32 / 255.,
                            vertex.color.b() as f32 / 255.,
                            vertex.color.a() as f32 / 255.,
                        ],
                        uv: [vertex.uv.x, vertex.uv.y],
                    });
                }

                let vertices = bytemuck::cast_slice(&vertices).to_vec();

                let material = self.font_texture.get(&m.texture_id).cloned().unwrap();
                let mesh = RenderMeshBuilder::new(resource_manager, "ui")
                    .with_bytes(vertices, m.indices.clone())
                    .for_material(material)
                    .with_layer(layer)
                    .transient(true)
                    .build();

                meshes.push((mesh, material));

                layer += 1;
            } else {
                tracing::error!(target: logger::UI, "Primitive unknown");
            }

            span.exit();
        }

        let mut model = RenderModelBuilder::new(resource_manager, "ui");

        for (mesh, material) in meshes {
            model = model.with_mesh(mesh).with_material(material);
        }
        Some(model.build())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }
}

impl Renderable for UIRenderer {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
        bounding_box: Option<BoundingBox>,
        render_flags: RenderFlags,
    ) -> Result<(), ResourceError> {
        if let Some(output) = self.output.write().take() {
            let transform = match transform {
                Some(transform) => transform,
                None => Transform::IDENTITY,
            };

            if let Some(model) = self.load_model(resource_manager, output) {
                batch.add_model(
                    ctx,
                    resource_manager,
                    model,
                    transform,
                    bounding_box,
                    render_flags,
                )?;
            }

            batch.add_extent_data(ImageExtent2D::new(self.width as u32, self.height as u32));
        }

        Ok(())
    }
}
