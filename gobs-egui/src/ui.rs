use std::{collections::HashMap, sync::Arc};

use egui::{
    Event, FullOutput, Modifiers, MouseWheelUnit, PointerButton, RawInput, Rect, TextureId,
    epaint::{ImageDelta, Primitive},
};
use glam::{Vec2, Vec3};
use parking_lot::RwLock;

use gobs_core::{Color, ImageExtent2D, Input, Key, MouseButton, Transform};
use gobs_render::{
    Material, MaterialInstance, MaterialProperties, MaterialProperty, Model, RenderBatch,
    Renderable, Texture, TextureProperties, TextureUpdate,
};
use gobs_render_graph::{BlendMode, GfxContext, PassType, RenderError, RenderPass};
use gobs_resource::{
    geometry::{MeshGeometry, VertexAttribute, VertexData},
    manager::ResourceManager,
    resource::{ResourceHandle, ResourceLifetime},
};

use crate::UIError;

const PIXEL_PER_POINT: f32 = 1.;

pub struct UIRenderer {
    ectx: egui::Context,
    width: f32,
    height: f32,
    material: ResourceHandle<Material>,
    font_texture: HashMap<TextureId, Arc<MaterialInstance>>,
    input: Vec<Input>,
    mouse_position: (f32, f32),
    output: RwLock<Option<FullOutput>>,
    vertex_padding: bool,
}

impl UIRenderer {
    pub fn new(
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        transparent: bool,
    ) -> Result<Self, UIError> {
        let ectx = egui::Context::default();

        let (width, height): (f32, f32) = ctx.extent().into();

        ectx.set_pixels_per_point(PIXEL_PER_POINT);

        let vertex_attributes =
            VertexAttribute::POSITION | VertexAttribute::COLOR | VertexAttribute::TEXTURE;

        let material_properties = MaterialProperties::new(
            ctx,
            "ui",
            "ui.vert.spv",
            "main",
            "ui.frag.spv",
            "main",
            vertex_attributes,
            pass,
        )
        .prop("diffuse", MaterialProperty::Texture)
        .no_culling()
        .blend_mode(if transparent {
            BlendMode::Premultiplied
        } else {
            BlendMode::None
        });

        let material = resource_manager.add(material_properties, ResourceLifetime::Static);

        Ok(UIRenderer {
            ectx,
            width,
            height,
            material,
            font_texture: HashMap::new(),
            input: Vec::new(),
            mouse_position: (0., 0.),
            output: RwLock::new(None),
            vertex_padding: ctx.vertex_padding,
        })
    }

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    pub fn update<F>(&mut self, resource_manager: &mut ResourceManager, delta: f32, callback: F)
    where
        F: FnMut(&egui::Context),
    {
        let input = self.prepare_inputs(delta);
        let output = self.ectx.run(input, callback);

        self.update_textures(resource_manager, &output);
        self.cleanup_textures(&output);

        self.output.write().replace(output);
    }

    fn get_key(key: Key) -> egui::Key {
        match key {
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
            _ => egui::Key::Escape,
        }
    }

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

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn update_textures(&mut self, resource_manager: &mut ResourceManager, output: &FullOutput) {
        for (id, material) in self.font_texture.iter() {
            tracing::debug!(target: "ui", "Font texture id={:?}, material={}", id, material.id);
            for texture in &material.textures {
                tracing::debug!(target: "ui", "  Using texture={:?}", texture.id);
            }
        }

        for (id, img) in &output.textures_delta.set {
            tracing::debug!(target: "ui", "New texture {:?}", id);
            if img.pos.is_some() {
                tracing::debug!(target: "ui", "Patching texture");
                let texture = self.patch_texture(
                    resource_manager,
                    self.font_texture
                        .get(id)
                        .cloned()
                        .expect("Cannot update unallocated texture"),
                    img,
                );

                let material = MaterialInstance::new(self.material, vec![texture]);

                *self.font_texture.get_mut(id).unwrap() = material;
            } else {
                tracing::debug!(target: "ui", "Allocate new texture");
                let texture = self.decode_texture(resource_manager, img);
                self.font_texture.insert(*id, texture);
                tracing::trace!(target: "ui", "Texture loaded");
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn cleanup_textures(&mut self, output: &FullOutput) {
        for id in &output.textures_delta.free {
            tracing::debug!(target: "ui", "Remove texture {:?}", id);

            self.font_texture.remove(id);
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn decode_texture(
        &self,
        resource_manager: &mut ResourceManager,
        img: &ImageDelta,
    ) -> Arc<MaterialInstance> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: Vec<u8> = bytemuck::cast_slice(pixels.as_slice()).to_vec();

                let properties = TextureProperties::with_data(
                    "Font texture",
                    bytes,
                    ImageExtent2D::new(img.image.width() as u32, img.image.height() as u32),
                );

                let handle = resource_manager.add(properties, ResourceLifetime::Static);

                MaterialInstance::new(self.material, vec![handle])
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn patch_texture(
        &self,
        resource_manager: &mut ResourceManager,
        material: Arc<MaterialInstance>,
        img: &ImageDelta,
    ) -> ResourceHandle<Texture> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let pos = img.pos.expect("Can only patch texture with start position");

                tracing::trace!(target: "ui",
                    "Patching texture origin: {}/{}, size: {}/{}, len={}",
                    pos[0],
                    pos[1],
                    font.width(),
                    font.height(),
                    bytes.len()
                );

                let handle = resource_manager.replace(&material.textures[0]);
                let texture = resource_manager.get_mut(&handle);
                tracing::trace!(target: "ui",
                    "Patching texture original size: {:?}",
                    texture.properties.format.extent
                );

                texture.patch(
                    pos[0] as u32,
                    pos[1] as u32,
                    font.width() as u32,
                    font.height() as u32,
                    bytes,
                )
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn load_model(
        &self,
        resource_manager: &mut ResourceManager,
        output: FullOutput,
    ) -> Option<Arc<Model>> {
        tracing::debug!(target: "ui", "Loading model");

        let primitives = self.ectx.tessellate(output.shapes, PIXEL_PER_POINT);

        tracing::debug!(target: "ui", "Load {} primitives", primitives.len());

        if primitives.is_empty() {
            return None;
        }

        let mut model = Model::builder("ui");

        for primitive in &primitives {
            if let Primitive::Mesh(m) = &primitive.primitive {
                tracing::trace!(target: "ui",
                    "Primitive: {} vertices, {} indices",
                    m.vertices.len(),
                    m.indices.len()
                );

                let mut mesh = MeshGeometry::builder("egui")
                    .indices(&m.indices, false)
                    .generate_tangents(false);

                for vertex in &m.vertices {
                    let color = Color::from_rgba8(
                        vertex.color.r(),
                        vertex.color.g(),
                        vertex.color.b(),
                        vertex.color.a(),
                    );
                    let vertex_data = VertexData::builder()
                        .position(Vec3::new(
                            vertex.pos.x.min(self.width),
                            (self.height - vertex.pos.y).min(self.height),
                            0.,
                        ))
                        .color(color)
                        .texture(Vec2::new(vertex.uv.x, vertex.uv.y))
                        .normal(Vec3::new(0., 0., 1.))
                        .padding(self.vertex_padding)
                        .build();

                    mesh = mesh.vertex(vertex_data);
                }

                model = model.mesh(
                    mesh.build(),
                    Some(self.font_texture.get(&m.texture_id).cloned().unwrap()),
                    resource_manager,
                    ResourceLifetime::Transient,
                );
            } else {
                tracing::error!(target: "ui", "Primitive unknown");
            }
        }

        Some(model.build(resource_manager))
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }
}

impl Renderable for UIRenderer {
    #[tracing::instrument(target = "ui", skip_all, level = "trace")]
    fn draw(
        &self,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
    ) -> Result<(), RenderError> {
        if pass.ty() == PassType::Ui {
            let output = self.output.write().take().unwrap();

            let transform = match transform {
                Some(transform) => transform,
                None => Transform::IDENTITY,
            };

            if let Some(model) = self.load_model(resource_manager, output) {
                batch.add_model(resource_manager, model, transform, pass.clone())?;
            }

            batch.add_extent_data(
                ImageExtent2D::new(self.width as u32, self.height as u32),
                pass,
            );
        }
        Ok(())
    }
}
