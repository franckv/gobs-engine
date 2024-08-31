use std::{collections::HashMap, sync::Arc};

use egui::{
    epaint::{ImageDelta, Primitive},
    Event, FullOutput, Modifiers, MouseWheelUnit, PointerButton, RawInput, Rect, TextureId,
};
use glam::{Vec2, Vec3};

use gobs_core::{Color, ImageExtent2D, SamplerFilter, Transform};
use gobs_game::input::{Input, Key, MouseButton};
use gobs_render::{
    BlendMode, Context, Material, MaterialInstance, MaterialProperty, Model, ModelId, RenderBatch,
    RenderPass, Renderable, RenderableLifetime,
};
use gobs_resource::{
    geometry::{Mesh, VertexData, VertexFlag},
    material::{Texture, TextureType},
};

const PIXEL_PER_POINT: f32 = 1.;

struct FrameData {
    model: Option<Arc<Model>>,
}

pub struct UIRenderer {
    ectx: egui::Context,
    width: f32,
    height: f32,
    material: Arc<Material>,
    font_texture: HashMap<TextureId, Arc<MaterialInstance>>,
    input: Vec<Input>,
    mouse_position: (f32, f32),
    frame_data: Vec<FrameData>,
    output: Option<FullOutput>,
}

impl UIRenderer {
    pub fn new(ctx: &Context, pass: RenderPass) -> Self {
        let ectx = egui::Context::default();

        let (width, height): (f32, f32) = ctx.extent().into();

        ectx.set_pixels_per_point(PIXEL_PER_POINT);

        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE;

        let material = Material::builder(ctx, "ui.vert.spv", "ui.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .no_culling()
            .blend_mode(BlendMode::Premultiplied)
            .build(pass);

        let frame_data = (0..ctx.frames_in_flight)
            .map(|_| FrameData { model: None })
            .collect();

        UIRenderer {
            ectx,
            width,
            height,
            material,
            font_texture: HashMap::new(),
            input: Vec::new(),
            mouse_position: (0., 0.),
            frame_data,
            output: None,
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    pub fn update<F>(&mut self, _ctx: &Context, _pass: RenderPass, delta: f32, callback: F)
    where
        F: FnMut(&egui::Context),
    {
        let input = self.prepare_inputs(delta);

        self.output = Some(self.ectx.run(input, callback));
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn upload_ui_data(&mut self, ctx: &Context, output: FullOutput) {
        self.update_textures(&output);

        self.cleanup_textures(&output);

        let model_id = match &self.frame_data[ctx.frame_id()].model {
            Some(model) => Some(model.id),
            None => None,
        };

        let model = self.load_model(output, model_id);

        self.frame_data[ctx.frame_id()].model = model;
    }

    pub fn dump_model(&self, ctx: &Context) {
        if let Some(model) = self.frame_data[ctx.frame_id()].model.clone() {
            tracing::warn!("Dump model: {:?}", model);
        }
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

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn update_textures(&mut self, output: &FullOutput) {
        for (id, img) in &output.textures_delta.set {
            tracing::debug!("New texture {:?}", id);
            if img.pos.is_some() {
                tracing::info!("Patching texture");
                let texture = self.patch_texture(
                    self.font_texture
                        .get(id)
                        .cloned()
                        .expect("Cannot update unallocated texture"),
                    img,
                );

                let material = self.material.instantiate(vec![texture]);

                *self.font_texture.get_mut(id).unwrap() = material;
            } else {
                tracing::debug!("Allocate new texture");
                let texture = self.decode_texture(img);
                self.font_texture.insert(*id, texture);
                tracing::debug!("Texture loaded");
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn cleanup_textures(&mut self, output: &FullOutput) {
        for id in &output.textures_delta.free {
            tracing::debug!("Remove texture {:?}", id);

            self.font_texture.remove(id);
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn decode_texture(&self, img: &ImageDelta) -> Arc<MaterialInstance> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: Vec<u8> = bytemuck::cast_slice(pixels.as_slice()).to_vec();

                let texture = Texture::new(
                    "egui",
                    &bytes,
                    ImageExtent2D::new(img.image.width() as u32, img.image.height() as u32),
                    TextureType::Diffuse,
                    TextureType::Diffuse.into(),
                    SamplerFilter::FilterLinear,
                    SamplerFilter::FilterLinear,
                );

                self.material.instantiate(vec![texture])
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn patch_texture(&self, material: Arc<MaterialInstance>, img: &ImageDelta) -> Arc<Texture> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let pos = img.pos.expect("Can only patch texture with start position");

                tracing::debug!(
                    "Patching texture origin: {}/{}, size: {}/{}, len={}",
                    pos[0],
                    pos[1],
                    font.width(),
                    font.height(),
                    bytes.len()
                );
                tracing::debug!(
                    "Patching texture original size: {:?}",
                    material.textures[0].extent
                );

                material.textures[0].patch(
                    pos[0] as u32,
                    pos[1] as u32,
                    font.width() as u32,
                    font.height() as u32,
                    bytes,
                )
            }
        }
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn load_model(&mut self, output: FullOutput, model_id: Option<ModelId>) -> Option<Arc<Model>> {
        tracing::debug!("Loading model");

        let primitives = self.ectx.tessellate(output.shapes, PIXEL_PER_POINT);

        tracing::debug!("Load {} primitives", primitives.len());

        if primitives.len() == 0 {
            return None;
        }

        let mut model = Model::builder("ui");

        if let Some(model_id) = model_id {
            model = model.id(model_id);
        }

        for primitive in &primitives {
            if let Primitive::Mesh(m) = &primitive.primitive {
                tracing::debug!(
                    "Primitive: {} vertices, {} indices",
                    m.vertices.len(),
                    m.indices.len()
                );

                let mut mesh = Mesh::builder("egui")
                    .indices(&m.indices)
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
                        .padding(true)
                        .build();

                    mesh = mesh.vertex(vertex_data);
                }

                model = model.mesh(
                    mesh.build(),
                    Some(self.font_texture.get(&m.texture_id).cloned().unwrap()),
                );
            } else {
                tracing::error!("Primitive unknown");
            }
        }

        Some(model.build())
    }
}

impl Renderable for UIRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    #[tracing::instrument(target = "ui", skip_all, level = "debug")]
    fn draw(
        &mut self,
        ctx: &Context,
        pass: RenderPass,
        batch: &mut RenderBatch,
        _transform: Transform,
        lifetime: RenderableLifetime,
    ) {
        let frame_id = ctx.frame_id();

        let output = self.output.take().unwrap();

        self.upload_ui_data(ctx, output);

        let model = self.frame_data[frame_id].model.clone();

        if let Some(model) = model {
            batch.add_model(ctx, model, Transform::IDENTITY, pass.clone(), lifetime);
        }

        batch.add_extent_data(
            ImageExtent2D::new(self.width as u32, self.height as u32),
            pass,
        );
    }
}
