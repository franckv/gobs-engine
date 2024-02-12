use std::{collections::HashMap, sync::Arc};

use egui::{
    epaint::{ImageDelta, Primitive},
    Event, FullOutput, Modifiers, PointerButton, RawInput, Rect, Rgba, TextureId,
};
use glam::{Mat3, Mat4, Vec2, Vec3};
use gobs_core::entity::{
    camera::Camera,
    uniform::{UniformData, UniformLayout, UniformProp, UniformPropData},
};
use gobs_scene::resources::{ModelResource, UniformBuffer};
use gobs_vulkan::{
    descriptor::{
        DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
    },
    pipeline::PipelineId,
};
use log::info;

use gobs_game::input::{Input, Key};
use gobs_render::{
    context::Context,
    geometry::{Mesh, Model, VertexData, VertexFlag},
    material::{
        Material, MaterialInstance, MaterialInstanceId, MaterialProperty, Texture, TextureType,
    },
    pass::RenderPass,
    CommandBuffer, ImageExtent2D, SamplerFilter,
};

const PIXEL_PER_POINT: f32 = 1.;

struct SceneFrameData {
    pub uniform_ds: DescriptorSet,
    pub uniform_buffer: UniformBuffer,
    ui_model: Option<Arc<ModelResource>>,
}

impl SceneFrameData {
    pub fn new(
        ctx: &Context,
        uniform_layout: Arc<UniformLayout>,
        uniform_ds: DescriptorSet,
    ) -> Self {
        let uniform_buffer = UniformBuffer::new(
            ctx,
            uniform_ds.layout.clone(),
            uniform_layout.size(),
            ctx.allocator.clone(),
        );

        uniform_ds
            .update()
            .bind_buffer(&uniform_buffer.buffer, 0, uniform_buffer.buffer.size)
            .end();

        SceneFrameData {
            uniform_ds,
            uniform_buffer,
            ui_model: None,
        }
    }
}

pub struct UIRenderer {
    ectx: egui::Context,
    width: f32,
    height: f32,
    material: Arc<Material>,
    camera: Camera,
    pub scene_data_layout: Arc<UniformLayout>,
    frame_number: usize,
    _scene_ds_pool: DescriptorSetPool,
    scene_frame_data: Vec<SceneFrameData>,
    font_texture: HashMap<TextureId, Arc<MaterialInstance>>,
    input: Vec<Input>,
    mouse_position: (f32, f32),
}

impl UIRenderer {
    pub fn new(ctx: &Context) -> Self {
        let ectx = egui::Context::default();

        let (width, height): (f32, f32) = ctx.surface.get_extent(ctx.device.clone()).into();

        ectx.set_pixels_per_point(PIXEL_PER_POINT);

        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE;

        let material = Material::builder("ui.vert.spv", "ui.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .no_culling()
            .blending_enabled()
            .build(ctx);

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let scene_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .build();

        let mut _scene_ds_pool = DescriptorSetPool::new(
            ctx.device.clone(),
            scene_descriptor_layout.clone(),
            ctx.frames_in_flight as u32,
        );

        let scene_frame_data = (0..ctx.frames_in_flight)
            .map(|_| SceneFrameData::new(ctx, scene_data_layout.clone(), _scene_ds_pool.allocate()))
            .collect();

        let camera = Camera::ortho(
            (width / 2., height / 2., 1.),
            width,
            height,
            0.1,
            10.,
            (0. as f32).to_radians(),
            (0. as f32).to_radians(),
            Vec3::Y,
        );

        UIRenderer {
            ectx,
            width,
            height,
            material,
            camera,
            scene_data_layout,
            frame_number: 0,
            _scene_ds_pool,
            scene_frame_data,
            font_texture: HashMap::new(),
            input: Vec::new(),
            mouse_position: (0., 0.),
        }
    }

    fn frame_id(&self, ctx: &Context) -> usize {
        (self.frame_number - 1) % ctx.frames_in_flight
    }

    pub fn update<F>(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, callback: F)
    where
        F: FnMut(&egui::Context),
    {
        self.frame_number += 1;
        let frame_id = self.frame_id(ctx);

        let scene_data = UniformData::new(
            &self.scene_data_layout,
            &[
                UniformPropData::Vec3F(self.camera.position.into()),
                UniformPropData::Mat4F(self.camera.view_proj().to_cols_array_2d()),
            ],
        );

        self.scene_frame_data[frame_id]
            .uniform_buffer
            .update(&scene_data);

        let input = self.prepare_inputs();

        let output = self.ectx.run(input, callback);

        pollster::block_on(self.update_textures(ctx, &output));

        let to_remove = output.textures_delta.free.clone();

        let model = self.load_models(output);

        self.scene_frame_data[frame_id].ui_model = Some(ModelResource::new(ctx, model, pass));

        self.cleanup_textures(to_remove);
    }

    pub fn draw(&self, ctx: &Context, _pass: Arc<dyn RenderPass>, cmd: &CommandBuffer) {
        let frame_id = self.frame_id(ctx);

        let world_matrix = Mat4::IDENTITY;
        let normal_matrix = Mat3::IDENTITY;
        let model = self.scene_frame_data[frame_id].ui_model.clone().unwrap();

        let mut last_material = MaterialInstanceId::nil();
        let mut last_pipeline = PipelineId::nil();

        let scene_data_ds = &self.scene_frame_data[frame_id].uniform_ds;

        for primitive in &model.primitives {
            let material = &model.model.materials[&primitive.material];
            let pipeline = material.pipeline();

            if last_material != material.id {
                if last_pipeline != pipeline.id {
                    cmd.bind_pipeline(&pipeline);
                    last_pipeline = pipeline.id;
                }
                cmd.bind_descriptor_set(scene_data_ds, 0, &pipeline);
                if let Some(material_ds) = &material.material_ds {
                    cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                }

                last_material = material.id;
            }

            let model_data = UniformData::new(
                &ctx.push_layout,
                &[
                    UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                    UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
                    UniformPropData::U64(model.vertex_buffer.address(ctx.device.clone())),
                ],
            );
            cmd.push_constants(pipeline.layout.clone(), &model_data.raw());

            cmd.bind_index_buffer::<u32>(&model.index_buffer, primitive.offset);
            cmd.draw_indexed(primitive.len, 1);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
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
                let texture = self.decode_texture(ctx, img).await;
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
                    SamplerFilter::FilterLinear,
                );

                self.material.instantiate(vec![texture])
            }
        }
    }

    async fn patch_texture(&self, material: Arc<MaterialInstance>, img: &ImageDelta) {
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
                    material.textures[0].read().image.extent
                );

                todo!();

                /*material.textures[0].write().patch_texture(
                    pos[0] as u32,
                    pos[1] as u32,
                    font.width() as u32,
                    font.height() as u32,
                    bytes,
                );*/
            }
        }
    }

    fn load_models(&self, output: FullOutput) -> Arc<Model> {
        let primitives = self.ectx.tessellate(output.shapes, PIXEL_PER_POINT);

        let mut model = Model::builder("ui");

        for primitive in &primitives {
            if let Primitive::Mesh(m) = &primitive.primitive {
                let mut mesh = Mesh::builder("egui").indices(&m.indices);

                for vertex in &m.vertices {
                    let color = Rgba::from_srgba_premultiplied(
                        vertex.color.r(),
                        vertex.color.g(),
                        vertex.color.b(),
                        vertex.color.a(),
                    );
                    let vertex_data = VertexData::builder()
                        .position(Vec3::new(vertex.pos.x, self.height - vertex.pos.y, 0.))
                        .color(color.to_array().into())
                        .texture(Vec2::new(vertex.uv.x, vertex.uv.y))
                        .normal(Vec3::new(0., 0., 1.))
                        .padding(true)
                        .build();

                    mesh = mesh.vertex(vertex_data);
                }

                model = model.mesh(
                    mesh.build(),
                    self.font_texture.get(&m.texture_id).cloned().unwrap(),
                );
            }
        }

        model.build()
    }
}
