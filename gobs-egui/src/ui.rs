use std::{collections::HashMap, sync::Arc};

use egui::{
    epaint::{ImageDelta, Primitive},
    Context, Event, FullOutput, Modifiers, PointerButton, RawInput, Rect, Rgba, TextureId,
};
use glam::{Vec2, Vec3, Vec4};

use gobs_game::input::Input;
use gobs_wgpu as render;

use log::info;
use render::{
    model::{Material, MaterialBuilder, MeshBuilder, Model, ModelBuilder, Texture, TextureType},
    render::Gfx,
    shader::Shader,
};

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

        ctx.set_pixels_per_point(1.);

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

    pub fn update<F>(&mut self, gfx: &Gfx, callback: F) -> Vec<Arc<Model>>
    where
        F: Fn(&Context),
    {
        let input = self.prepare_inputs();

        let output = self.ctx.run(input, callback);

        pollster::block_on(self.update_textures(gfx, &output));

        let to_remove = output.textures_delta.free.clone();

        let models = self.load_models(gfx, &self.ctx, self.shader.clone(), output);

        self.cleanup_textures(to_remove);

        models
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    fn prepare_inputs(&mut self) -> RawInput {
        let mut input = RawInput::default();
        input.screen_rect = Some(Rect::from_min_size(
            Default::default(),
            [self.width, self.height].into(),
        ));

        self.input.drain(..).for_each(|e| match e {
            Input::KeyPressed(_) => (),
            Input::KeyReleased(_) => (),
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

    async fn update_textures(&mut self, gfx: &Gfx, output: &FullOutput) {
        for (id, img) in &output.textures_delta.set {
            info!("New texture {:?}", id);
            if let Some(_) = img.pos {
                info!("Patching texture");
                self.patch_texture(
                    gfx,
                    self.font_texture
                        .get(id)
                        .cloned()
                        .expect("Cannot update unallocated texture"),
                    img,
                )
                .await;
            } else {
                info!("Allocate new texture");
                let texture = self.decode_texture(gfx, img).await;
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

    async fn decode_texture(&self, gfx: &Gfx, img: &ImageDelta) -> Arc<Material> {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let texture = Texture::new(
                    &gfx,
                    "egui",
                    TextureType::IMAGE,
                    img.image.width() as u32,
                    img.image.height() as u32,
                    bytes,
                );

                MaterialBuilder::new("diffuse")
                    .diffuse_texture_t(texture)
                    .await
                    .build(gfx)
            }
        }
    }

    async fn patch_texture(&self, gfx: &Gfx, material: Arc<Material>, img: &ImageDelta) {
        match &img.image {
            egui::ImageData::Color(_) => todo!(),
            egui::ImageData::Font(font) => {
                let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                let pos = img.pos.expect("Can only patch texture with start position");

                material.diffuse_texture.patch_texture(
                    gfx,
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
        gfx: &Gfx,
        ctx: &Context,
        shader: Arc<Shader>,
        output: FullOutput,
    ) -> Vec<Arc<Model>> {
        let mut models = Vec::new();

        let primitives = ctx.tessellate(output.shapes);

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
                    .build(gfx, shader.clone());

                models.push(model);
            }
        });

        models
    }
}
