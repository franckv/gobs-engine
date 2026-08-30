use std::marker::PhantomData;

use gobs::{
    core::{Color, ConfigWriter as _, ImageFormat, Input, logger},
    game::{AppError, Application, GameContext, GobsContext, GobsGame},
    graphics::{AlignMode, AttributeData, Shapes, VertexAttribute, VertexData},
    render::{
        BufferType, CommandBuffer, CommandQueueType, CullMode, DynamicStateElem, FrontFace,
        GfxContext, Handle, ImageLayout, ObjectDataLayout, ObjectDataProp, Rect2D, RenderConfig,
        RenderError, RenderHalConfig, UniformData, Viewport,
    },
};

struct App<Context: GobsContext> {
    frame_number: usize,
    cmd: Box<dyn CommandBuffer>,
    pipeline: Handle,
    vertex_buffer: Handle,
    index_buffer: Handle,
    context: PhantomData<Context>,
}

/// Minimalist exemple showcasing the Hardware abstraction layer (HAL)
impl<Context: GobsContext> GobsGame for App<Context> {
    type Context = Context;

    async fn create(ctx: &mut Context) -> Result<Self, AppError> {
        let gfx = ctx.gfx_mut();
        let mut cmd = gfx.create_command_buffer("cmd", CommandQueueType::Graphics);

        let (vertex_buffer, index_buffer) = Self::load_mesh(gfx, cmd.as_mut());

        let pipeline = Self::create_pipeline(gfx);

        Ok(App {
            frame_number: 0,
            cmd,
            pipeline,
            vertex_buffer,
            index_buffer,
            context: PhantomData,
        })
    }

    fn update(&mut self, _ctx: &mut Context, _delta: f32) {}

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        let gfx = ctx.gfx_mut();

        self.frame_number += 1;

        let frame_id = gfx.frame_id(self.frame_number);

        self.cmd.wait();

        if gfx.acquire(frame_id).is_err() {
            return Err(RenderError::Outdated);
        }

        self.cmd.reset();

        let color = gfx.get_render_target();
        let extent = gfx.get_extent();

        self.cmd.begin(self.frame_number);
        self.cmd
            .begin_label(&format!("Begin frame {}", self.frame_number));

        if let Some(color) = color {
            self.cmd
                .transition_image_layout(gfx, color, ImageLayout::Color);
        }

        self.cmd
            .begin_rendering(gfx, color, extent, None, true, false, [0., 0., 0., 0.], 0.);

        self.cmd.set_viewport(extent.width, extent.height);
        self.cmd.bind_pipeline(gfx, self.pipeline);
        self.cmd.bind_index_buffer(gfx, self.index_buffer);

        let mut constants = vec![];
        let push_layout = gfx.get_pipeline_push_layout(self.pipeline);
        push_layout.copy_data(&mut constants, |p| match p {
            ObjectDataProp::VertexBufferAddress => {
                let vertex_buffer_address = gfx.get_buffer_address(self.vertex_buffer);
                AttributeData::U64(vertex_buffer_address)
            }
            _ => unreachable!(),
        });

        self.cmd.push_constants(gfx, self.pipeline, &constants);

        self.cmd.draw_indexed(3, 1);

        self.cmd.end_rendering();

        if let Some(color) = color {
            self.cmd
                .transition_image_layout(gfx, color, ImageLayout::Present);
        } else {
            tracing::info!("no image");
        }

        self.cmd.end_label();
        self.cmd.end();

        self.cmd.submit_graphics(gfx, frame_id);

        let Ok(_) = gfx.present() else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    fn input(&mut self, _ctx: &mut Context, _input: Input) {}

    fn resize(&mut self, _ctx: &mut Context, _width: u32, _height: u32) {}

    async fn start(&mut self, _ctx: &mut Context) {}

    fn should_update(&mut self, _ctx: &mut Context) -> bool {
        true
    }

    fn close(&mut self, _ctx: &mut Context) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl<Context: GobsContext> App<Context> {
    fn load_mesh(gfx: &mut GfxContext, cmd: &mut dyn CommandBuffer) -> (Handle, Handle) {
        let mesh = Shapes::triangle(&[Color::RED, Color::GREEN, Color::BLUE], 0.5);
        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;

        let mut vertices = vec![];

        VertexData::copy_data(
            &mesh.vertices,
            vertex_attributes,
            &mut vertices,
            AlignMode::Scalar,
        );

        let indices = &mesh.indices;

        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();
        let staging_size = indices_size + vertices_size;

        let vertex_buffer = gfx.create_buffer("vertex", vertices_size, BufferType::Vertex);
        let index_buffer = gfx.create_buffer("index", indices_size, BufferType::Index);
        let staging = gfx.create_buffer("staging", staging_size, BufferType::Staging);

        gfx.upload_buffer(staging, &vertices, 0);
        gfx.upload_buffer(staging, bytemuck::cast_slice(indices), vertices_size as u64);

        cmd.run_immediate_mut("Upload buffer", &mut |cmd| {
            cmd.copy_buffer_to_buffer(gfx, staging, vertex_buffer, vertices_size, 0, 0);
            cmd.copy_buffer_to_buffer(
                gfx,
                staging,
                index_buffer,
                indices_size,
                vertices_size as u64,
                0,
            );
        });

        gfx.destroy_buffer(staging);

        (vertex_buffer, index_buffer)
    }

    fn create_pipeline(gfx: &mut GfxContext) -> Handle {
        gfx.create_graphics_pipeline("color")
            .vertex_shader("color_buffer_reference.spv", "vertex_main")
            .fragment_shader("color_buffer_reference.spv", "fragment_main")
            .push_constants(ObjectDataLayout::new(false).prop(ObjectDataProp::VertexBufferAddress))
            .attachments(Some(ImageFormat::B8g8r8a8Unorm), None)
            .depth_test_disable()
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .front_face(FrontFace::CCW)
            .cull_mode(CullMode::Back)
            .build(gfx)
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    Application::<App<GameContext>>::new("Triangle", examples::WIDTH, examples::HEIGHT)
        .with_config(|config| {
            config.set_string(RenderConfig::GraphName, "none");
            config.set_bool(RenderConfig::LoadGraph, false);
            config.set_int(RenderHalConfig::FramesInFlight, 1);
        })
        .run();
}
