use gobs::{
    core::{Color, ImageFormat, Input, logger},
    game::{AppError, Application, GameContext, GameOptions, GobsGame},
    render::{
        BufferType, CommandBuffer, CommandQueueType, CullMode, DynamicStateElem, FrontFace, Handle,
        ImageLayout, ObjectDataLayout, ObjectDataProp, Rect2D, RenderError, RenderHAL, Shapes,
        UniformData, UniformPropData, VertexAttribute, VertexData, Viewport,
    },
};

struct App {
    frame_number: usize,
    cmd: Box<dyn CommandBuffer>,
    pipeline: Handle,
    vertex_buffer: Handle,
    index_buffer: Handle,
}

/// Minimalist exemple showcasing the Hardware abstraction layer (HAL)
impl GobsGame for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let hal = ctx.renderer.gfx.hal_mut();
        let mut cmd = hal.create_command_buffer("cmd", CommandQueueType::Graphics);

        let (vertex_buffer, index_buffer) = Self::load_mesh(hal, cmd.as_mut());

        let pipeline = Self::create_pipeline(hal);

        Ok(App {
            frame_number: 0,
            cmd,
            pipeline,
            vertex_buffer,
            index_buffer,
        })
    }

    fn update(&mut self, _ctx: &mut GameContext, _delta: f32) {}

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        let hal = ctx.renderer.gfx.hal_mut();

        self.frame_number += 1;

        let frame_id = hal.frame_id(self.frame_number);

        self.cmd.wait();

        if hal.acquire(frame_id).is_err() {
            return Err(RenderError::Outdated);
        }

        self.cmd.reset();

        let color = hal.get_render_target();
        let extent = hal.get_extent();

        self.cmd.begin(self.frame_number);
        self.cmd
            .begin_label(&format!("Begin frame {}", self.frame_number));

        if let Some(color) = color {
            self.cmd
                .transition_image_layout(hal, color, ImageLayout::Color);
        }

        self.cmd
            .begin_rendering(hal, color, extent, None, true, false, [0., 0., 0., 0.], 0.);

        self.cmd.set_viewport(extent.width, extent.height);
        self.cmd.bind_pipeline(hal, self.pipeline);
        self.cmd.bind_index_buffer(hal, self.index_buffer);

        let mut constants = vec![];
        let object_layout = hal.get_pipeline_object_layout(self.pipeline);
        object_layout.copy_data(&mut constants, |p| match p {
            ObjectDataProp::VertexBufferAddress => {
                let vertex_buffer_address = hal.get_buffer_address(self.vertex_buffer);
                UniformPropData::U64(vertex_buffer_address)
            }
            _ => unreachable!(),
        });

        self.cmd.push_constants(hal, self.pipeline, &constants);

        self.cmd.draw_indexed(3, 1);

        self.cmd.end_rendering();

        if let Some(color) = color {
            self.cmd
                .transition_image_layout(hal, color, ImageLayout::Present);
        } else {
            tracing::info!("no image");
        }

        self.cmd.end_label();
        self.cmd.end();

        self.cmd.submit2(hal, frame_id);

        let Ok(_) = hal.present() else {
            return Err(RenderError::Outdated);
        };

        Ok(())
    }

    fn input(&mut self, _ctx: &mut GameContext, _input: Input) {}

    fn resize(&mut self, _ctx: &mut GameContext, _width: u32, _height: u32) {}

    async fn start(&mut self, _ctx: &mut GameContext) {}

    fn should_update(&mut self, _ctx: &mut GameContext) -> bool {
        true
    }

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: logger::APP, "Closed");
    }
}

impl App {
    fn load_mesh(hal: &mut dyn RenderHAL, cmd: &mut dyn CommandBuffer) -> (Handle, Handle) {
        let mesh = Shapes::triangle(&[Color::RED, Color::GREEN, Color::BLUE], 0.5);
        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;

        let mut vertices = vec![];

        VertexData::copy_data(&mesh.vertices, vertex_attributes, &mut vertices);

        let indices = &mesh.indices;

        let vertices_size = vertices.len();
        let indices_size = indices.len() * std::mem::size_of::<u32>();
        let staging_size = indices_size + vertices_size;

        let vertex_buffer = hal.create_buffer("vertex", vertices_size, BufferType::Vertex);
        let index_buffer = hal.create_buffer("index", indices_size, BufferType::Index);
        let staging = hal.create_buffer("staging", staging_size, BufferType::Staging);

        hal.upload_buffer(staging, &vertices, 0);
        hal.upload_buffer(staging, bytemuck::cast_slice(indices), vertices_size as u64);

        cmd.run_immediate_mut("Upload buffer", &mut |cmd| {
            cmd.copy_buffer_to_buffer(hal, staging, vertex_buffer, vertices_size, 0, 0);
            cmd.copy_buffer_to_buffer(
                hal,
                staging,
                index_buffer,
                indices_size,
                vertices_size as u64,
                0,
            );
        });

        hal.destroy_buffer(staging);

        (vertex_buffer, index_buffer)
    }

    fn create_pipeline(hal: &mut dyn RenderHAL) -> Handle {
        hal.create_graphics_pipeline("color")
            .vertex_shader("color_buffer_reference.spv", "vertex_main")
            .fragment_shader("color_buffer_reference.spv", "fragment_main")
            .push_constants(ObjectDataLayout::default().prop(ObjectDataProp::VertexBufferAddress))
            .attachments(Some(ImageFormat::B8g8r8a8Unorm), None)
            .depth_test_disable()
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .front_face(FrontFace::CCW)
            .cull_mode(CullMode::Back)
            .build(hal)
    }
}

fn main() {
    examples::init_logger();

    tracing::info!(target: logger::APP, "Engine start");

    let mut options = GameOptions::default();
    options.renderer.graph = "none".to_string();
    options.renderer.frames_in_flight = 1;
    options.renderer.load_graph = false;

    Application::<App>::new("Triangle", options, examples::WIDTH, examples::HEIGHT).run();
}
