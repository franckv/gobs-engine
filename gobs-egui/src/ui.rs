use std::sync::Arc;

use egui::{
    epaint::Primitive, Context, Event, FullOutput, Modifiers, PointerButton, RawInput, Rect, Rgba,
};
use glam::{Vec2, Vec3, Vec4};

use gobs_game::input::Input;
use gobs_wgpu as render;

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
    font_texture: Option<Arc<Material>>,
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
            font_texture: None,
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

        if self.font_texture.is_none() {
            self.font_texture = Some(pollster::block_on(Self::load_texture(gfx, &output)));
        }

        self.load_models(gfx, &self.ctx, self.shader.clone(), output)
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

    async fn load_texture(gfx: &Gfx, output: &FullOutput) -> Arc<Material> {
        let mut textures: Vec<Texture> = output
            .textures_delta
            .set
            .iter()
            .map(|(_, img)| match &img.image {
                egui::ImageData::Color(_) => todo!(),
                egui::ImageData::Font(font) => {
                    let pixels = font.srgba_pixels(None).collect::<Vec<_>>();
                    let bytes: &[u8] = bytemuck::cast_slice(pixels.as_slice());

                    Texture::new(
                        &gfx,
                        "egui",
                        TextureType::IMAGE,
                        img.image.width() as u32,
                        img.image.height() as u32,
                        bytes,
                    )
                }
            })
            .collect();

        MaterialBuilder::new("diffuse")
            .diffuse_texture_t(textures.remove(0))
            .await
            .build(gfx)
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
                    .add_mesh(mesh, self.font_texture.clone())
                    .build(gfx, shader.clone());

                models.push(model);
            }
        });

        models
    }
}
