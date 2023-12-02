use std::{collections::HashMap, sync::Arc};

use egui::{
    epaint::{ImageDelta, Primitive},
    Context, Event, FullOutput, Modifiers, PointerButton, RawInput, Rect, Rgba, TextureId,
};
use glam::{Vec2, Vec3, Vec4};
use log::info;

use gobs_core as core;
use gobs_game::input::{Input, Key};
use gobs_render as render;

use core::{
    geometry::mesh::MeshBuilder,
    material::texture::{Texture, TextureType},
};
use render::{
    model::{Material, MaterialBuilder, Model, ModelBuilder},
    shader::Shader,
};

const PIXEL_PER_POINT: f32 = 1.;

pub struct UIRenderer {
    ctx: Context,
    width: f32,
    height: f32,
    shader: Arc<Shader>,
    font_texture: HashMap<TextureId, Arc<Material>>,
    input: Vec<Input>,
    mouse_position: (f32, f32),
}

impl UIRenderer {
    pub fn new(width: f32, height: f32, shader: Arc<Shader>) -> Self {
        let ctx = egui::Context::default();

        ctx.set_pixels_per_point(PIXEL_PER_POINT);

        UIRenderer {
            ctx,
            width,
            height,
            shader,
            font_texture: HashMap::new(),
            input: Vec::new(),
            mouse_position: (0., 0.),
        }
    }

    pub fn update<F>(&mut self, callback: F) -> Vec<Arc<Model>>
    where
        F: FnMut(&Context),
    {
        let input = self.prepare_inputs();

        let output = self.ctx.run(input, callback);

        pollster::block_on(self.update_textures(&output));

        let to_remove = output.textures_delta.free.clone();

        let models = self.load_models(&self.ctx, self.shader.clone(), output);

        self.cleanup_textures(to_remove);

        models
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
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

    fn prepare_inputs(&mut self) -> RawInput {
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Default::default(),
                [self.width, self.height].into(),
            )),
            ..Default::default()
        };

        self.input.drain(..).for_each(|e| match e {
            Input::KeyPressed(key) => {
                input.events.push(Event::Key {
                    key: Self::get_key(key),
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                });
            }
            Input::KeyReleased(key) => {
                input.events.push(Event::Key {
                    key: Self::get_key(key),
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
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

    async fn update_textures(&mut self, output: &FullOutput) {
        for (id, img) in &output.textures_delta.set {
            info!("New texture {:?}", id);
            if img.pos.is_some() {
                info!("Patching texture");
                self.patch_texture(
                    self.font_texture
                        .get(id)
                        .cloned()
                        .expect("Cannot update unallocated texture"),
                    img,
                )
                .await;
            } else {
                info!("Allocate new texture");
                let texture = self.decode_texture(img).await;
                self.font_texture.insert(*id, texture);
            }
        }
    }

    fn cleanup_textures(&mut self, to_remove: Vec<TextureId>) {
        for id in &to_remove {
            info!("Remove texture {:?}", id);

            self.font_texture.remove(id);
        }
    }

    async fn decode_texture(&self, img: &ImageDelta) -> Arc<Material> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: Vec<u8> = bytemuck::cast_slice(pixels.as_slice()).to_vec();

                let texture = Texture::new(
                    "egui",
                    TextureType::IMAGE,
                    bytes,
                    img.image.width() as u32,
                    img.image.height() as u32,
                );

                MaterialBuilder::new("diffuse")
                    .diffuse_texture_t(texture)
                    .await
                    .build()
            }
        }
    }

    async fn patch_texture(&self, material: Arc<Material>, img: &ImageDelta) {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let pos = img.pos.expect("Can only patch texture with start position");

                info!(
                    "Patching texture origin: {}/{}, size: {}/{}, len={}",
                    pos[0],
                    pos[1],
                    font.width(),
                    font.height(),
                    bytes.len()
                );
                info!(
                    "Patching texture original size: {:?}",
                    material.diffuse_texture.read().unwrap().size()
                );

                material.diffuse_texture.write().unwrap().patch_texture(
                    pos[0] as u32,
                    pos[1] as u32,
                    font.width() as u32,
                    font.height() as u32,
                    bytes,
                );
            }
        }
    }

    fn load_models(
        &self,
        ctx: &Context,
        shader: Arc<Shader>,
        output: FullOutput,
    ) -> Vec<Arc<Model>> {
        let mut models = Vec::new();

        let primitives = ctx.tessellate(output.shapes, PIXEL_PER_POINT);

        //println!("{:#?}", primitives);
        primitives.iter().for_each(|s| {
            if let Primitive::Mesh(m) = &s.primitive {
                let mut mesh = MeshBuilder::new("egui").add_indices(&m.indices);

                for vertex in &m.vertices {
                    let color = Rgba::from_srgba_premultiplied(
                        vertex.color.r(),
                        vertex.color.g(),
                        vertex.color.b(),
                        vertex.color.a(),
                    );
                    mesh = mesh.add_vertex(
                        Vec3::new(vertex.pos.x, vertex.pos.y, 0.),
                        Vec4::new(color[0], color[1], color[2], color[3]),
                        Vec2::new(vertex.uv.x, vertex.uv.y),
                        Vec3::new(0., 0., 1.),
                        Vec2::new(vertex.uv.x, vertex.uv.y),
                    );
                }

                let mesh = mesh.build();
                let model = ModelBuilder::new()
                    .add_mesh(mesh, self.font_texture.get(&m.texture_id).cloned())
                    .build(shader.clone());

                models.push(model);
            }
        });

        models
    }
}
