use std::{collections::HashMap, sync::Arc};

use egui::{
    epaint::{ImageDelta, Primitive},
    Event, FullOutput, Modifiers, PointerButton, RawInput, Rect, TextureId,
};
use glam::{Vec2, Vec3};

use gobs_core::Transform;
use gobs_core::Color;
use gobs_game::input::{Input, Key};
use gobs_render::{
    batch::RenderBatch,
    context::Context,
    geometry::{Mesh, Model, ModelId, VertexData, VertexFlag},
    material::{Material, MaterialInstance, MaterialProperty, Texture, TextureType},
    pass::RenderPass,
    renderable::Renderable,
    BlendMode, ImageExtent2D, SamplerFilter,
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
}

impl UIRenderer {
    pub fn new(ctx: &Context, pass: Arc<dyn RenderPass>) -> Self {
        let ectx = egui::Context::default();

        let (width, height): (f32, f32) = ctx.surface.get_extent(ctx.device.clone()).into();

        ectx.set_pixels_per_point(PIXEL_PER_POINT);

        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE;

        let material = Material::builder("ui.vert.spv", "ui.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .no_culling()
            .blend_mode(BlendMode::Premultiplied)
            .build(ctx, pass);

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
        }
    }

    pub fn update<F>(&mut self, ctx: &Context, _pass: Arc<dyn RenderPass>, delta: f32, callback: F)
    where
        F: FnMut(&egui::Context),
    {
        let input = self.prepare_inputs(delta);

        let output = self.ectx.run(input, callback);

        pollster::block_on(self.update_textures(ctx, &output));

        let to_remove = output.textures_delta.free.clone();

        let model_id = match &self.frame_data[ctx.frame_id()].model {
            Some(model) => Some(model.id),
            None => None,
        };

        let model = self.load_model(output, model_id);

        self.frame_data[ctx.frame_id()].model = Some(model);

        self.cleanup_textures(to_remove);
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
            Input::MousePressed => {
                input.events.push(Event::PointerButton {
                    pos: self.mouse_position.into(),
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                });
            }
            Input::MouseReleased => {
                input.events.push(Event::PointerButton {
                    pos: self.mouse_position.into(),
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                });
            }
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

    async fn update_textures(&mut self, ctx: &Context, output: &FullOutput) {
        for (id, img) in &output.textures_delta.set {
            log::info!("New texture {:?}", id);
            if img.pos.is_some() {
                log::info!("Patching texture");
                self.patch_texture(
                    ctx,
                    self.font_texture
                        .get(id)
                        .cloned()
                        .expect("Cannot update unallocated texture"),
                    img,
                )
                .await;
            } else {
                log::info!("Allocate new texture");
                let texture = self.decode_texture(ctx, img).await;
                self.font_texture.insert(*id, texture);
            }
        }
    }

    fn cleanup_textures(&mut self, to_remove: Vec<TextureId>) {
        for id in &to_remove {
            log::info!("Remove texture {:?}", id);

            self.font_texture.remove(id);
        }
    }

    async fn decode_texture(&self, ctx: &Context, img: &ImageDelta) -> Arc<MaterialInstance> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: Vec<u8> = bytemuck::cast_slice(pixels.as_slice()).to_vec();

                let texture = Texture::new(
                    ctx,
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

    async fn patch_texture(
        &self,
        ctx: &Context,
        material: Arc<MaterialInstance>,
        img: &ImageDelta,
    ) {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let pos = img.pos.expect("Can only patch texture with start position");

                log::info!(
                    "Patching texture origin: {}/{}, size: {}/{}, len={}",
                    pos[0],
                    pos[1],
                    font.width(),
                    font.height(),
                    bytes.len()
                );
                log::info!(
                    "Patching texture original size: {:?}",
                    material.textures[0].read().image.extent
                );

                material.textures[0].patch(
                    ctx,
                    pos[0] as u32,
                    pos[1] as u32,
                    font.width() as u32,
                    font.height() as u32,
                    bytes,
                );
            }
        }
    }

    fn load_model(&self, output: FullOutput, model_id: Option<ModelId>) -> Arc<Model> {
        let primitives = self.ectx.tessellate(output.shapes, PIXEL_PER_POINT);

        let mut model = Model::builder("ui");

        if let Some(model_id) = model_id {
            model = model.id(model_id);
        }

        log::debug!("Load {} primitives", primitives.len());

        for primitive in &primitives {
            if let Primitive::Mesh(m) = &primitive.primitive {
                log::debug!("Primitive: {} vertices, {} indices", m.vertices.len(), m.indices.len());

                let mut mesh = Mesh::builder("egui").indices(&m.indices);

                for vertex in &m.vertices {
                    let color = Color::from_rgba8(
                        vertex.color.r(),
                        vertex.color.g(),
                        vertex.color.b(),
                        vertex.color.a(),
                    );
                    let vertex_data = VertexData::builder()
                        .position(Vec3::new(vertex.pos.x.min(self.width), (self.height - vertex.pos.y).min(self.height), 0.))
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
                log::error!("Primitive unknown");
            }
        }

        model.build()
    }
}

impl Renderable for UIRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch) {
        let frame_id = ctx.frame_id();
        let model = self.frame_data[frame_id].model.clone();

        if let Some(model) = model {
            batch.add_model(ctx, model, Transform::IDENTITY, pass.clone(), true);
        }

        batch.add_extent_data(
            ImageExtent2D::new(self.width as u32, self.height as u32),
            pass,
        );
    }
}
